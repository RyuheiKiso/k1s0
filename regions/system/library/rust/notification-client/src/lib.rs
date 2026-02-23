pub mod client;
pub mod error;
pub mod request;

pub use client::NotificationClient;
pub use error::NotificationClientError;
pub use request::{NotificationChannel, NotificationRequest, NotificationResponse};

#[cfg(feature = "mock")]
pub use client::MockNotificationClient;
