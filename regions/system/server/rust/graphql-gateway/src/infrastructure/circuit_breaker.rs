/// graphql-gateway 用のサーキットブレーカーレジストリ。
/// 各バックエンド gRPC サービスに対して個別のサーキットブレーカーを管理する。
/// 外部サービスの障害が他のサービスへ連鎖的に伝播するのを防止する。
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use k1s0_circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError};
use tokio::sync::RwLock;

/// サーキットブレーカーのデフォルト設定値
#[allow(dead_code)]
const DEFAULT_FAILURE_THRESHOLD: u32 = 5;
#[allow(dead_code)]
const DEFAULT_SUCCESS_THRESHOLD: u32 = 3;
#[allow(dead_code)]
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// バックエンドサービス名ごとにサーキットブレーカーを保持するレジストリ。
/// graphql-gateway が呼び出す各 gRPC バックエンドの障害を個別に管理する。
#[allow(dead_code)]
pub struct CircuitBreakerRegistry {
    /// サービス名 → サーキットブレーカーのマッピング
    breakers: RwLock<HashMap<String, Arc<CircuitBreaker>>>,
    /// 新規サーキットブレーカー作成時に使用するデフォルト設定
    default_config: CircuitBreakerConfig,
}

#[allow(dead_code)]
impl CircuitBreakerRegistry {
    /// デフォルト設定でレジストリを生成する。
    #[must_use] 
    pub fn new() -> Self {
        Self {
            breakers: RwLock::new(HashMap::new()),
            default_config: CircuitBreakerConfig {
                failure_threshold: DEFAULT_FAILURE_THRESHOLD,
                success_threshold: DEFAULT_SUCCESS_THRESHOLD,
                timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            },
        }
    }

    /// カスタム設定でレジストリを生成する。
    #[must_use] 
    pub fn with_config(config: CircuitBreakerConfig) -> Self {
        Self {
            breakers: RwLock::new(HashMap::new()),
            default_config: config,
        }
    }

    /// 指定されたサービス名のサーキットブレーカーを取得する。
    /// 存在しない場合はデフォルト設定で新規作成する。
    pub async fn get(&self, service_name: &str) -> Arc<CircuitBreaker> {
        // 読み取りロックで既存のブレーカーを検索する
        {
            let breakers = self.breakers.read().await;
            if let Some(cb) = breakers.get(service_name) {
                return cb.clone();
            }
        }

        // 書き込みロックで新規ブレーカーを作成する（ダブルチェックパターン）
        let mut breakers = self.breakers.write().await;
        breakers
            .entry(service_name.to_string())
            .or_insert_with(|| Arc::new(CircuitBreaker::new(self.default_config.clone())))
            .clone()
    }

    /// サーキットブレーカーで保護された関数を実行する。
    /// サーキットブレーカーがオープン状態の場合は即座にエラーを返す。
    pub async fn call<F, Fut, T>(
        &self,
        service_name: &str,
        f: F,
    ) -> Result<T, CircuitBreakerError<anyhow::Error>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, anyhow::Error>>,
    {
        let cb = self.get(service_name).await;
        cb.call(f).await
    }
}

impl Default for CircuitBreakerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // レジストリが同じサービス名に対して同一のサーキットブレーカーを返すことを確認する。
    #[tokio::test]
    async fn test_registry_returns_same_breaker() {
        let registry = CircuitBreakerRegistry::new();
        let cb1 = registry.get("tenant-service").await;
        let cb2 = registry.get("tenant-service").await;
        // Arc のポインタが同一であることを確認する
        assert!(Arc::ptr_eq(&cb1, &cb2));
    }

    // 異なるサービス名に対して異なるサーキットブレーカーが返されることを確認する。
    #[tokio::test]
    async fn test_registry_different_services() {
        let registry = CircuitBreakerRegistry::new();
        let cb1 = registry.get("tenant-service").await;
        let cb2 = registry.get("workflow-service").await;
        assert!(!Arc::ptr_eq(&cb1, &cb2));
    }

    // サーキットブレーカーを経由した呼び出しが成功することを確認する。
    #[tokio::test]
    async fn test_call_success() {
        let registry = CircuitBreakerRegistry::new();
        let result: Result<i32, CircuitBreakerError<anyhow::Error>> =
            registry.call("test-service", || async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    // 失敗が閾値に達した後にサーキットブレーカーがオープンになることを確認する。
    #[tokio::test]
    async fn test_call_opens_after_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            timeout: Duration::from_secs(60),
        };
        let registry = CircuitBreakerRegistry::with_config(config);

        // 2 回失敗させてオープン状態にする
        for _ in 0..2 {
            let _: Result<i32, CircuitBreakerError<anyhow::Error>> = registry
                .call("test-service", || async {
                    Err(anyhow::anyhow!("service down"))
                })
                .await;
        }

        // オープン状態では即座にエラーが返る
        let result: Result<i32, CircuitBreakerError<anyhow::Error>> =
            registry.call("test-service", || async { Ok(42) }).await;
        assert!(matches!(result, Err(CircuitBreakerError::Open)));
    }
}
