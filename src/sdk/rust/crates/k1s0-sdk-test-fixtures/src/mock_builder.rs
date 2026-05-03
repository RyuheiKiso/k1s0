//! Mock builder fluent API（領域 3、ADR-TEST-010 §3）。
//!
//! リリース時点で 3 service（State / Audit / PubSub）を提供。
//! 採用初期で +3 (Workflow / Decision / Secret)、運用拡大時で残 6 service を追加。

use crate::{FixtureError, Result};

/// 12 service の mock builder への entry point。
pub struct MockBuilderRoot {
    /// 既定 tenant（builder の WithTenant 未指定時に使う）
    default_tenant: String,
}

impl MockBuilderRoot {
    /// crate 内から呼ばれる constructor
    pub(crate) fn new(default_tenant: String) -> Self {
        Self { default_tenant }
    }

    /// State service の mock builder
    pub fn state(&self) -> StateMockBuilder {
        StateMockBuilder {
            tenant: self.default_tenant.clone(),
            key: String::new(),
            value: Vec::new(),
            ttl: 0,
        }
    }

    /// Audit service の mock builder
    pub fn audit(&self) -> AuditMockBuilder {
        AuditMockBuilder {
            tenant: self.default_tenant.clone(),
            entry_count: 0,
            start_seq: 0,
        }
    }

    /// PubSub service の mock builder
    pub fn pubsub(&self) -> PubSubMockBuilder {
        PubSubMockBuilder {
            tenant: self.default_tenant.clone(),
            topic: String::new(),
            messages: 0,
            delay_ms: 0,
        }
    }

    /// Workflow は採用初期で実装
    pub fn workflow(&self) -> Result<()> {
        Err(FixtureError::Unimplemented {
            service: "Workflow".to_string(),
            phase: "採用初期".to_string(),
        })
    }
}

// ----- State service mock builder ---------------------------------

/// State service mock data の fluent builder
pub struct StateMockBuilder {
    tenant: String,
    key: String,
    value: Vec<u8>,
    ttl: u32,
}

impl StateMockBuilder {
    /// tenant 指定（fluent chain）
    pub fn with_tenant(mut self, tenant: impl Into<String>) -> Self {
        self.tenant = tenant.into();
        self
    }
    /// key 指定
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = key.into();
        self
    }
    /// value 指定
    pub fn with_value(mut self, value: impl Into<Vec<u8>>) -> Self {
        self.value = value.into();
        self
    }
    /// TTL 秒指定
    pub fn with_ttl(mut self, seconds: u32) -> Self {
        self.ttl = seconds;
        self
    }
    /// build: 最終的な StateEntry を返す（採用初期で contracts/proto 型に置換）
    pub fn build(self) -> Result<StateEntry> {
        Ok(StateEntry {
            tenant: self.tenant,
            key: self.key,
            value: self.value,
            ttl: self.ttl,
        })
    }
}

/// State service の wire 形式（採用初期で contracts/proto 型に置換）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StateEntry {
    pub tenant: String,
    pub key: String,
    pub value: Vec<u8>,
    pub ttl: u32,
}

// ----- Audit service mock builder ---------------------------------

/// Audit service mock data の fluent builder
pub struct AuditMockBuilder {
    tenant: String,
    entry_count: u32,
    start_seq: u64,
}

impl AuditMockBuilder {
    pub fn with_tenant(mut self, tenant: impl Into<String>) -> Self {
        self.tenant = tenant.into();
        self
    }
    /// entry 件数指定（hash chain で連結された N 件を生成）
    pub fn with_entries(mut self, n: u32) -> Self {
        self.entry_count = n;
        self
    }
    pub fn with_sequence(mut self, seq: u64) -> Self {
        self.start_seq = seq;
        self
    }
    /// build: AuditEntry の Vec を返す（採用初期で hash chain 計算 + proto 型に置換）
    pub fn build(self) -> Result<Vec<AuditEntry>> {
        let mut entries = Vec::with_capacity(self.entry_count as usize);
        for i in 0..self.entry_count {
            entries.push(AuditEntry {
                tenant: self.tenant.clone(),
                sequence: self.start_seq + i as u64,
                // 採用初期で prev_id chain を SHA-256 で計算
                prev_id: String::new(),
            });
        }
        Ok(entries)
    }
}

/// Audit service の wire 形式
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub tenant: String,
    pub sequence: u64,
    pub prev_id: String,
}

// ----- PubSub service mock builder --------------------------------

/// PubSub service mock data の fluent builder
pub struct PubSubMockBuilder {
    tenant: String,
    topic: String,
    messages: u32,
    delay_ms: u32,
}

impl PubSubMockBuilder {
    pub fn with_tenant(mut self, tenant: impl Into<String>) -> Self {
        self.tenant = tenant.into();
        self
    }
    pub fn with_topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = topic.into();
        self
    }
    pub fn with_messages(mut self, n: u32) -> Self {
        self.messages = n;
        self
    }
    pub fn with_delay_ms(mut self, ms: u32) -> Self {
        self.delay_ms = ms;
        self
    }
    pub fn build(self) -> Result<Vec<PubSubMessage>> {
        let mut msgs = Vec::with_capacity(self.messages as usize);
        for i in 0..self.messages {
            msgs.push(PubSubMessage {
                tenant: self.tenant.clone(),
                topic: self.topic.clone(),
                seq_id: i as u64,
            });
        }
        Ok(msgs)
    }
}

/// PubSub service の wire 形式
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PubSubMessage {
    pub tenant: String,
    pub topic: String,
    pub seq_id: u64,
}
