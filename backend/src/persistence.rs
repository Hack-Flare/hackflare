use std::io;

use postgres::{Client, NoTls};
use serde::{Deserialize, Serialize};

use crate::domain::auth::AuthSnapshot;
use crate::domain::dns::DnsSnapshot;

const SNAPSHOT_ID: i16 = 1;
const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS backend_state (
    id SMALLINT PRIMARY KEY,
    auth_snapshot TEXT NOT NULL,
    dns_snapshot TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
"#;

#[derive(Clone)]
pub struct BackendStore {
    database_url: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct BackendSnapshot {
    pub auth: AuthSnapshot,
    pub dns: DnsSnapshot,
}

impl BackendStore {
    pub fn new(database_url: String) -> Self {
        Self { database_url }
    }

    pub fn load(&self) -> io::Result<Option<BackendSnapshot>> {
        let mut client = self.connect()?;
        self.ensure_schema(&mut client)?;

        let row = client
            .query_opt(
                "SELECT auth_snapshot, dns_snapshot FROM backend_state WHERE id = $1",
                &[&SNAPSHOT_ID],
            )
            .map_err(Self::map_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let auth_snapshot = row.get::<_, String>(0);
        let dns_snapshot = row.get::<_, String>(1);

        let snapshot = BackendSnapshot {
            auth: serde_json::from_str(&auth_snapshot)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
            dns: serde_json::from_str(&dns_snapshot)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
        };

        Ok(Some(snapshot))
    }

    pub fn save(&self, snapshot: &BackendSnapshot) -> io::Result<()> {
        let mut client = self.connect()?;
        self.ensure_schema(&mut client)?;

        let auth_snapshot = serde_json::to_string(&snapshot.auth)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let dns_snapshot = serde_json::to_string(&snapshot.dns)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

        client
            .execute(
                r#"
                INSERT INTO backend_state (id, auth_snapshot, dns_snapshot, updated_at)
                VALUES ($1, $2, $3, NOW())
                ON CONFLICT (id)
                DO UPDATE SET
                    auth_snapshot = EXCLUDED.auth_snapshot,
                    dns_snapshot = EXCLUDED.dns_snapshot,
                    updated_at = NOW()
                "#,
                &[&SNAPSHOT_ID, &auth_snapshot, &dns_snapshot],
            )
            .map_err(Self::map_error)?;

        Ok(())
    }

    fn connect(&self) -> io::Result<Client> {
        Client::connect(&self.database_url, NoTls).map_err(Self::map_error)
    }

    fn ensure_schema(&self, client: &mut Client) -> io::Result<()> {
        client.batch_execute(SCHEMA).map_err(Self::map_error)
    }

    fn map_error(error: postgres::Error) -> io::Error {
        io::Error::new(io::ErrorKind::Other, error)
    }
}