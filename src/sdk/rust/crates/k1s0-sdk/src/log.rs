// 本ファイルは k1s0-sdk の Log 動詞統一 facade。
use crate::client::Client;
use crate::proto::k1s0::tier1::log::v1::{
    BulkSendLogRequest, LogEntry, SendLogRequest, Severity,
    log_service_client::LogServiceClient,
};
use prost_types::Timestamp;
use std::collections::HashMap;
use std::time::SystemTime;
use tonic::{Status, transport::Channel};

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
        let ts = Timestamp {
            seconds: now.as_secs() as i64,
            nanos: now.subsec_nanos() as i32,
        };

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

    /// bulk_send は LogEntry の一括送信（FR-T1-LOG-* 共通、Send の高スループット版）。
    /// 各 entry の timestamp が None なら呼出時刻を自動設定する。戻り値は (accepted, rejected)。
    pub async fn bulk_send(&mut self, entries: Vec<LogEntryInput>) -> Result<(i32, i32), Status> {
        // 共通の現在時刻（unset entry の補完用）。
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let now_ts = Timestamp {
            seconds: now.as_secs() as i64,
            nanos: now.subsec_nanos() as i32,
        };
        // SDK の LogEntryInput を proto LogEntry に詰め替える。
        let pe: Vec<LogEntry> = entries
            .into_iter()
            .map(|e| LogEntry {
                timestamp: Some(e.timestamp.unwrap_or_else(|| now_ts.clone())),
                severity: e.severity as i32,
                body: e.body,
                attributes: e.attributes,
                stack_trace: String::new(),
            })
            .collect();
        let req = BulkSendLogRequest {
            entries: pe,
            context: Some(self.client.tenant_context()),
        };
        let resp = self.raw.bulk_send(req).await?.into_inner();
        Ok((resp.accepted, resp.rejected))
    }
}

/// LogEntryInput は bulk_send の 1 件分の入力。
#[derive(Debug, Clone)]
pub struct LogEntryInput {
    /// 重大度（OTel SeverityNumber）。
    pub severity: Severity,
    /// 発生時刻。None なら呼出時刻が自動設定される。
    pub timestamp: Option<Timestamp>,
    /// 本文。
    pub body: String,
    /// 構造化属性（OTel attributes）。
    pub attributes: HashMap<String, String>,
}
