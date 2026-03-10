use async_trait::async_trait;

/// CosignVerifier はコンテナイメージやバイナリの署名検証を行うトレイト。
#[async_trait]
#[allow(dead_code)]
pub trait CosignVerifier: Send + Sync {
    /// 指定されたアーティファクトの署名を検証する。
    async fn verify(&self, artifact: &str, signature: &str) -> anyhow::Result<bool>;
}

/// StubCosignVerifier は開発用のスタブ実装。常に true を返す。
#[allow(dead_code)]
pub struct StubCosignVerifier;

#[async_trait]
impl CosignVerifier for StubCosignVerifier {
    async fn verify(&self, _artifact: &str, _signature: &str) -> anyhow::Result<bool> {
        Ok(true)
    }
}
