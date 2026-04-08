use anyhow::Context;
use serde_json::Value;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};

/// JWKS (JSON Web Key Set) をキャッシュ付きで取得するプロバイダー。
/// 同時リフレッシュ時のサンダリングハード問題を防ぐため、
/// `fetch_lock` で排他制御を行い、1リクエストだけがフェッチし他は待機する。
#[derive(Clone)]
pub struct JwksProvider {
    inner: Arc<Inner>,
}

struct Inner {
    url: String,
    ttl: Duration,
    client: reqwest::Client,
    /// キャッシュされた JWKS レスポンス
    cache: RwLock<Option<CachedJwks>>,
    /// リフレッシュの排他制御用ロック。
    /// キャッシュ期限切れ時に複数リクエストが同時にフェッチすることを防ぐ。
    fetch_lock: Mutex<()>,
}

struct CachedJwks {
    fetched_at: Instant,
    value: Value,
}

impl JwksProvider {
    #[must_use] 
    pub fn new(url: String, ttl: Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap_or_default();
        Self {
            inner: Arc::new(Inner {
                url,
                ttl,
                client,
                cache: RwLock::new(None),
                fetch_lock: Mutex::new(()),
            }),
        }
    }

    /// キャッシュが有効ならそのまま返す。
    /// 期限切れの場合は `fetch_lock` を取得し、1リクエストだけが JWKS を再取得する。
    /// ロック待ちの間に他のリクエストがキャッシュを更新した場合はそれを返す（ダブルチェック）。
    pub async fn get(&self) -> anyhow::Result<Value> {
        // 高速パス: キャッシュが有効ならロック不要で返す
        {
            let cache = self.inner.cache.read().await;
            if let Some(cached) = cache.as_ref() {
                if cached.fetched_at.elapsed() < self.inner.ttl {
                    return Ok(cached.value.clone());
                }
            }
        }

        // キャッシュ期限切れ: fetch_lock を取得して排他的にリフレッシュする
        let _fetch_guard = self.inner.fetch_lock.lock().await;

        // ダブルチェック: ロック待機中に別リクエストが既にキャッシュを更新した可能性がある
        {
            let cache = self.inner.cache.read().await;
            if let Some(cached) = cache.as_ref() {
                if cached.fetched_at.elapsed() < self.inner.ttl {
                    return Ok(cached.value.clone());
                }
            }
        }

        // このリクエストだけが JWKS エンドポイントからフェッチする
        let value = self.fetch().await?;
        let mut cache = self.inner.cache.write().await;
        *cache = Some(CachedJwks {
            fetched_at: Instant::now(),
            value: value.clone(),
        });
        Ok(value)
    }

    async fn fetch(&self) -> anyhow::Result<Value> {
        let response = self
            .inner
            .client
            .get(&self.inner.url)
            .send()
            .await
            .with_context(|| format!("failed to fetch JWKS from {}", self.inner.url))?;

        let status = response.status();
        if !status.is_success() {
            anyhow::bail!("JWKS endpoint returned non-success status: {status}");
        }

        response.json::<Value>().await.with_context(|| {
            format!(
                "failed to parse JWKS response as JSON from {}",
                self.inner.url
            )
        })
    }
}
