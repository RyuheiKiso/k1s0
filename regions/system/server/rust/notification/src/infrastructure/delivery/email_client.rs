use async_trait::async_trait;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::domain::service::{DeliveryClient, DeliveryError};

pub struct EmailDeliveryClient {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from_address: String,
}

impl EmailDeliveryClient {
    pub fn new(
        smtp_host: &str,
        smtp_port: u16,
        username: &str,
        password: &str,
        from_address: &str,
    ) -> Result<Self, DeliveryError> {
        let creds = Credentials::new(username.to_string(), password.to_string());

        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::relay(smtp_host)
                .map_err(|e: lettre::transport::smtp::Error| {
                    DeliveryError::ConnectionFailed(e.to_string())
                })?
                .port(smtp_port)
                .credentials(creds)
                .build();

        Ok(Self {
            mailer,
            from_address: from_address.to_string(),
        })
    }
}

#[async_trait]
impl DeliveryClient for EmailDeliveryClient {
    async fn send(&self, recipient: &str, subject: &str, body: &str) -> Result<(), DeliveryError> {
        let email = Message::builder()
            .from(
                self.from_address
                    .parse()
                    .map_err(|e: lettre::address::AddressError| {
                        DeliveryError::Other(format!("invalid from address: {}", e))
                    })?,
            )
            .to(recipient.parse().map_err(|e: lettre::address::AddressError| {
                DeliveryError::Other(format!("invalid recipient address: {}", e))
            })?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())
            .map_err(|e| DeliveryError::Other(format!("failed to build email: {}", e)))?;

        self.mailer
            .send(email)
            .await
            .map_err(|e: lettre::transport::smtp::Error| {
                DeliveryError::ConnectionFailed(e.to_string())
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_with_valid_params() {
        let result = EmailDeliveryClient::new("localhost", 587, "user", "pass", "test@example.com");
        assert!(result.is_ok());
    }
}
