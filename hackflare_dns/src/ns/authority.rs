use crate::dns::DnsConfig;
use hickory_server::net::runtime::TokioRuntimeProvider;
use hickory_server::proto::rr::{
    rdata::SOA, LowerName, Name, RData, Record, RecordType,
};
use hickory_server::server::{Request, RequestHandler, ResponseHandler, ResponseInfo};
use hickory_server::store::in_memory::InMemoryZoneHandler;
use hickory_server::zone_handler::{AxfrPolicy, Catalog, ZoneHandler, ZoneType};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

fn normalize_zone_name(name: &str) -> String {
    let trimmed = name.trim().trim_end_matches('.').to_ascii_lowercase();
    if trimmed.is_empty() {
        trimmed
    } else {
        format!("{trimmed}.")
    }
}

fn parse_name(value: &str) -> Option<Name> {
    value.parse::<Name>().ok().or_else(|| Name::from_ascii(value).ok())
}

fn parse_u32(value: &str, default: u32) -> u32 {
    value.trim().parse::<u32>().unwrap_or(default)
}

fn as_zone_handler(
    handler: Arc<InMemoryZoneHandler<TokioRuntimeProvider>>,
) -> Arc<dyn ZoneHandler> {
    handler
}

fn build_default_soa(zone_name: &str, config: &DnsConfig) -> Option<Record> {
    let origin = parse_name(zone_name)?;
    let mname = parse_name(&config.soa_mname)?;
    let rname = parse_name(&config.soa_rname)?;

    let soa = SOA::new(
        mname,
        rname,
        parse_u32(&config.soa_serial, 1),
        parse_u32(&config.soa_refresh, 3600) as i32,
        parse_u32(&config.soa_retry, 1800) as i32,
        parse_u32(&config.soa_expire, 604800) as i32,
        parse_u32(&config.soa_minimum, 86400),
    );

    Some(Record::from_rdata(
        origin,
        parse_u32(&config.soa_ttl, 3600),
        RData::SOA(soa),
    ))
}

pub struct AuthorityStore {
    config: DnsConfig,
    catalog: RwLock<Catalog>,
    zones: RwLock<HashMap<String, Arc<InMemoryZoneHandler<TokioRuntimeProvider>>>>,
}

impl AuthorityStore {
    pub fn new(config: DnsConfig) -> Self {
        Self {
            config,
            catalog: RwLock::new(Catalog::new()),
            zones: RwLock::new(HashMap::new()),
        }
    }

    pub async fn create_zone(&self, name: impl Into<String>) -> bool {
        let normalized = normalize_zone_name(&name.into());
        let Some(origin) = parse_name(&normalized) else {
            return false;
        };

        let Some(soa_record) = build_default_soa(&normalized, &self.config) else {
            return false;
        };

        let handler = Arc::new(InMemoryZoneHandler::empty(
            origin.clone(),
            ZoneType::Primary,
            AxfrPolicy::Deny,
        ));

        let serial = parse_u32(&self.config.soa_serial, 1);
        if !handler.upsert(soa_record, serial).await {
            return false;
        }

        self.zones
            .write()
            .await
            .insert(normalized.trim_end_matches('.').to_string(), Arc::clone(&handler));

        let zone_name = LowerName::new(&origin);
        let catalog_handler = as_zone_handler(Arc::clone(&handler));
        self.catalog.write().await.upsert(zone_name, vec![catalog_handler]);

        true
    }

    pub async fn delete_zone(&self, name: &str) -> bool {
        let normalized = normalize_zone_name(name);
        let Some(origin) = parse_name(&normalized) else {
            return false;
        };

        let removed = self
            .zones
            .write()
            .await
            .remove(normalized.trim_end_matches('.'))
            .is_some();
        if removed {
            let _ = self.catalog.write().await.remove(&LowerName::new(&origin));
        }
        removed
    }

    pub async fn add_record(
        &self,
        zone_name: &str,
        name: &str,
        rtype: &str,
        ttl: u32,
        data: &str,
    ) -> bool {
        let normalized_zone = normalize_zone_name(zone_name);
        let Some(handler) = self
            .zones
            .read()
            .await
            .get(normalized_zone.trim_end_matches('.'))
            .cloned()
        else {
            return false;
        };

        let zone_key = normalized_zone.trim_end_matches('.').to_ascii_lowercase();
        let fqdn = if name.trim().is_empty() || name.trim() == "@" {
            normalized_zone.clone()
        } else {
            let normalized_name = name.trim().trim_end_matches('.').to_ascii_lowercase();
            if normalized_name.ends_with(&zone_key) {
                format!("{}.", normalized_name)
            } else {
                format!("{normalized_name}.{zone_key}.")
            }
        };

        let Some(record_name) = parse_name(&fqdn) else {
            return false;
        };

        let Ok(record_type) = RecordType::from_str(rtype.trim()) else {
            return false;
        };

        let Ok(rdata) = RData::try_from_str(record_type, data) else {
            return false;
        };

        handler
            .upsert(Record::from_rdata(record_name, ttl, rdata), parse_u32(&self.config.soa_serial, 1))
            .await
    }

    pub async fn remove_record(&self, zone_name: &str, name: &str, rtype: &str) -> bool {
        let normalized_zone = normalize_zone_name(zone_name);
        let Some(handler) = self
            .zones
            .read()
            .await
            .get(normalized_zone.trim_end_matches('.'))
            .cloned()
        else {
            return false;
        };

        let zone_key = normalized_zone.trim_end_matches('.').to_ascii_lowercase();
        let fqdn = if name.trim().is_empty() || name.trim() == "@" {
            normalized_zone.clone()
        } else {
            let normalized_name = name.trim().trim_end_matches('.').to_ascii_lowercase();
            if normalized_name.ends_with(&zone_key) {
                format!("{}.", normalized_name)
            } else {
                format!("{normalized_name}.{zone_key}.")
            }
        };

        let Some(record_name) = parse_name(&fqdn) else {
            return false;
        };

        let Ok(record_type) = RecordType::from_str(rtype.trim()) else {
            return false;
        };

        let mut records = handler.records_mut().await;
        let before = records.len();
        let target_name = LowerName::new(&record_name);
        records.retain(|key, _| !(key.name() == &target_name && key.record_type == record_type));
        before != records.len()
    }

    pub async fn list_zones(&self) -> Vec<String> {
        self.zones.read().await.keys().cloned().collect()
    }

    pub async fn contains_zone_for(&self, name: &LowerName) -> bool {
        self.catalog.read().await.find(name).is_some()
    }

    pub async fn handle_request<R: ResponseHandler, T: hickory_server::net::runtime::Time>(
        &self,
        request: &Request,
        response_handle: R,
    ) -> ResponseInfo {
        let catalog = self.catalog.read().await;
        RequestHandler::handle_request::<R, T>(&*catalog, request, response_handle).await
    }
}