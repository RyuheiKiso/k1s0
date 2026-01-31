//! コレオグラフィ Saga（イベント駆動型）。
//!
//! `k1s0-domain-event` と連携し、イベント駆動で Saga ステップを進行する。

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use k1s0_domain_event::{DomainEvent, EventEnvelope, EventHandler, EventPublisher};
use serde::Serialize;
use tokio::sync::Mutex;

use crate::error::{ConsensusError, ConsensusResult};
use crate::saga::SagaStatus;

/// イベント駆動型ステップハンドラ。
#[async_trait]
pub trait EventStepHandler: Send + Sync {
    /// 処理対象のイベント型を返す。
    fn event_type(&self) -> &str;

    /// ステップ名を返す。
    fn step_name(&self) -> &str;

    /// イベントを処理し、次のイベントを発行する。
    async fn handle(
        &self,
        envelope: &EventEnvelope,
    ) -> Result<Option<Box<dyn DomainEvent>>, String>;

    /// 補償イベントを発行する。
    async fn compensate(
        &self,
        envelope: &EventEnvelope,
    ) -> Result<Option<Box<dyn DomainEvent>>, String>;
}

/// コレオグラフィ Saga の状態。
struct SagaState {
    status: SagaStatus,
    completed_steps: Vec<String>,
}

/// コレオグラフィ Saga。
pub struct ChoreographySaga {
    name: String,
    handlers: Vec<Arc<dyn EventStepHandler>>,
    publisher: Arc<dyn EventPublisher>,
    timeout: Duration,
    states: Arc<Mutex<HashMap<String, SagaState>>>,
}

impl ChoreographySaga {
    /// Saga のステータスを取得する。
    ///
    /// # Errors
    ///
    /// 該当する Saga が見つからない場合にエラーを返す。
    pub async fn status(&self, saga_id: &str) -> ConsensusResult<SagaStatus> {
        let states = self.states.lock().await;
        states
            .get(saga_id)
            .map(|s| s.status)
            .ok_or_else(|| ConsensusError::Config(format!("saga not found: {saga_id}")))
    }
}

/// コレオグラフィ Saga の完了イベント。
#[derive(Debug, Serialize)]
pub struct SagaCompletedEvent {
    /// Saga ID。
    pub saga_id: String,
    /// Saga 名。
    pub saga_name: String,
}

impl DomainEvent for SagaCompletedEvent {
    fn event_type(&self) -> &str {
        "saga.completed"
    }

    fn aggregate_id(&self) -> Option<&str> {
        Some(&self.saga_id)
    }
}

/// コレオグラフィ Saga のビルダー。
pub struct ChoreographySagaBuilder {
    name: String,
    handlers: Vec<Arc<dyn EventStepHandler>>,
    publisher: Option<Arc<dyn EventPublisher>>,
    timeout: Duration,
}

impl ChoreographySagaBuilder {
    /// 新しいビルダーを作成する。
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            handlers: Vec::new(),
            publisher: None,
            timeout: Duration::from_secs(300),
        }
    }

    /// イベントハンドラを追加する。
    #[must_use]
    pub fn handler(mut self, handler: Arc<dyn EventStepHandler>) -> Self {
        self.handlers.push(handler);
        self
    }

    /// イベントパブリッシャーを設定する。
    #[must_use]
    pub fn publisher(mut self, publisher: Arc<dyn EventPublisher>) -> Self {
        self.publisher = Some(publisher);
        self
    }

    /// タイムアウトを設定する。
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// `ChoreographySaga` を構築する。
    ///
    /// # Errors
    ///
    /// パブリッシャーが設定されていない場合にエラーを返す。
    pub fn build(self) -> ConsensusResult<ChoreographySaga> {
        let publisher = self
            .publisher
            .ok_or_else(|| ConsensusError::Config("publisher is required".into()))?;

        Ok(ChoreographySaga {
            name: self.name,
            handlers: self.handlers,
            publisher,
            timeout: self.timeout,
            states: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}
