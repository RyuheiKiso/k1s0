use std::collections::HashMap;

use async_trait::async_trait;

use crate::BindingError;

/// BindingData は入力バインディングから受信したデータを表す。
#[derive(Debug, Clone)]
pub struct BindingData {
    pub data: Vec<u8>,
    pub metadata: HashMap<String, String>,
}

/// BindingResponse は出力バインディングの呼び出し結果を表す。
#[derive(Debug, Clone)]
pub struct BindingResponse {
    pub data: Vec<u8>,
    pub metadata: HashMap<String, String>,
}

/// InputBinding は外部ソースからデータを受信する入力バインディングの抽象インターフェース。
/// Component トレイトを拡張する。
#[async_trait]
pub trait InputBinding: k1s0_bb_core::Component {
    /// データを読み取る。
    async fn read(&self) -> Result<BindingData, BindingError>;
}

/// OutputBinding は外部サービスにデータを送信する出力バインディングの抽象インターフェース。
/// Component トレイトを拡張する。
#[async_trait]
pub trait OutputBinding: k1s0_bb_core::Component {
    /// 操作を呼び出す。
    async fn invoke(
        &self,
        operation: &str,
        data: &[u8],
        metadata: Option<HashMap<String, String>>,
    ) -> Result<BindingResponse, BindingError>;
}
