use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

pub fn send_email_smtp(
    to: &str,
    from: &str,
    subject: &str,
    body: &str,
    relay_host: Option<&str>,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<(), String> {
    let relay = relay_host.ok_or_else(|| "no relay host provided".to_string())?;

    let builder = Message::builder()
        .from(
            from.parse::<Mailbox>()
                .map_err(|e| format!("invalid from: {}", e))?,
        )
        .to(to
            .parse::<Mailbox>()
            .map_err(|e| format!("invalid to: {}", e))?)
        .subject(subject);

    let message = builder
        .body(body.to_string())
        .map_err(|e| format!("failed build message: {}", e))?;

    let mut mailer_builder =
        SmtpTransport::relay(relay).map_err(|e| format!("invalid relay: {}", e))?;

    if let (Some(user), Some(pass)) = (username, password) {
        let creds = Credentials::new(user.to_string(), pass.to_string());
        mailer_builder = mailer_builder.credentials(creds);
    }

    let mailer = mailer_builder.build();

    mailer
        .send(&message)
        .map_err(|e| format!("send failed: {}", e))?;

    Ok(())
}

#[rustler::nif]
pub fn send_email_nif(
    to: String,
    from: String,
    subject: String,
    body: String,
    relay_host: Option<String>,
    username: Option<String>,
    password: Option<String>,
) -> Result<(), String> {
    send_email_smtp(
        &to,
        &from,
        &subject,
        &body,
        relay_host.as_deref(),
        username.as_deref(),
        password.as_deref(),
    )
}
