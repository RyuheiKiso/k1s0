use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::ComponentError;

/// ComponentStatus はコンポーネントの現在の状態を表す。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentStatus {
    Uninitialized,
    Ready,
    Degraded,
    Closed,
    Error(String),
}

/// Component はビルディングブロックの基本インターフェース。
/// すべてのビルディングブロック（PubSub, StateStore 等）はこのトレイトを実装する。
#[async_trait]
pub trait Component: Send + Sync {
    /// コンポーネント名を返す。
    fn name(&self) -> &str;

    /// コンポーネント種別を返す（例: "pubsub", "statestore"）。
    fn component_type(&self) -> &str;

    /// コンポーネントを初期化する。
    async fn init(&self) -> Result<(), ComponentError>;

    /// コンポーネントをクローズする。
    async fn close(&self) -> Result<(), ComponentError>;

    /// コンポーネントの現在のステータスを返す。
    async fn status(&self) -> ComponentStatus;

    /// コンポーネントのメタデータを返す。
    fn metadata(&self) -> HashMap<String, String>;
}
