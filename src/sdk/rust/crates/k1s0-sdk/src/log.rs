// 本ファイルは k1s0-sdk の Log 動詞統一 facade。
use crate::client::Client;
use crate::proto::k1s0::tier1::log::v1::{
    log_service_client::LogServiceClient, LogEntry, SendLogRequest, Severity,
};
use prost_types::Timestamp;
use std::collections::HashMap;
use std::time::SystemTime;
use tonic::{transport::Channel, Status};

/// LogFacade は LogService の動詞統一 facade。
pub struct LogFacade {
    client: Client,
    raw: LogServiceClient<Channel>,
}

impl LogFacade {
    pub(crate) fn new(client: Client) -> Self {
        let raw = LogServiceClient::new(client.channel());
        Self { client, raw }
    }

    /// send は単一エントリ送信。
    pub async fn send(
        &mut self,
        severity: Severity,
        body: &str,
        attributes: HashMap<String, String>,
    ) -> Result<(), Status> {
        // 現在時刻を Timestamp に変換する。
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let ts = Timestamp { seconds: now.as_secs() as i64, nanos: now.subsec_nanos() as i32 };

        let req = SendLogRequest {
            entry: Some(LogEntry {
                timestamp: Some(ts),
                severity: severity as i32,
                body: body.to_string(),
                attributes,
                stack_trace: String::new(),
            }),
            context: Some(self.client.tenant_context()),
        };
        self.raw.send(req).await?;
        Ok(())
    }

    /// info は INFO 重大度のショートカット。
    pub async fn info(&mut self, body: &str, attrs: HashMap<String, String>) -> Result<(), Status> {
        self.send(Severity::Info, body, attrs).await
    }

    /// warn は WARN 重大度のショートカット。
    pub async fn warn(&mut self, body: &str, attrs: HashMap<String, String>) -> Result<(), Status> {
        self.send(Severity::Warn, body, attrs).await
    }

    /// error_log は ERROR 重大度のショートカット（error は予約語のため改名）。
    pub async fn error_log(
        &mut self,
        body: &str,
        attrs: HashMap<String, String>,
    ) -> Result<(), Status> {
        self.send(Severity::Error, body, attrs).await
    }

    /// debug は DEBUG 重大度のショートカット。
    pub async fn debug(
        &mut self,
        body: &str,
        attrs: HashMap<String, String>,
    ) -> Result<(), Status> {
        self.send(Severity::Debug, body, attrs).await
    }
}
