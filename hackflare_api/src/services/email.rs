use crate::config::SmtpConfig;

#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct EmailService {
    from: String,
    host: String,
    port: u16,
    username: String,
    password: String,
}

impl EmailService {
    #[allow(dead_code)]
    pub(crate) fn new(config: &SmtpConfig) -> Self {
        Self {
            from: config.from.clone(),
            host: config.host.clone(),
            port: config.port,
            username: config.username.clone(),
            password: config.password.clone(),
        }
    }

    #[allow(dead_code)]
    pub(crate) async fn send(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), anyhow::Error> {
        use lettre::{
            AsyncSmtpTransport, AsyncTransport, Message,
            transport::smtp::authentication::Credentials,
            Tokio1Executor,
        };

        let email = Message::builder()
            .from(self.from.parse()?)
            .to(to.parse()?)
            .subject(subject)
            .body(body.to_string())?;

        let creds = Credentials::new(self.username.clone(), self.password.clone());
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&self.host)?
            .port(self.port)
            .credentials(creds)
            .build();

        mailer.send(email).await?;
        Ok(())
    }
}
