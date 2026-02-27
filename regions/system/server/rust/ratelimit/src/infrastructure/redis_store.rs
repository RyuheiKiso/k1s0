use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::Script;

use crate::domain::entity::RateLimitDecision;
use crate::domain::repository::{RateLimitStateStore, UsageSnapshot};

/// TOKEN_BUCKET_SCRIPT は Redis Lua スクリプトでアトミックなトークンバケットを実装する。
///
/// KEYS[1]: バケットキー
/// ARGV[1]: 最大トークン数 (limit)
/// ARGV[2]: ウィンドウ秒数 (refill interval)
/// ARGV[3]: 現在時刻 (Unix epoch seconds)
///
/// Returns: {allowed (0/1), remaining, reset_at}
const TOKEN_BUCKET_SCRIPT: &str = r#"
local key = KEYS[1]
local limit = tonumber(ARGV[1])
local window = tonumber(ARGV[2])
local now = tonumber(ARGV[3])

local bucket = redis.call('HMGET', key, 'tokens', 'last_refill')
local tokens = tonumber(bucket[1])
local last_refill = tonumber(bucket[2])

if tokens == nil then
    tokens = limit
    last_refill = now
end

local elapsed = now - last_refill
local refill_rate = limit / window
local new_tokens = math.min(limit, tokens + elapsed * refill_rate)
last_refill = now

local allowed = 0
if new_tokens >= 1 then
    new_tokens = new_tokens - 1
    allowed = 1
end

redis.call('HMSET', key, 'tokens', new_tokens, 'last_refill', last_refill)
redis.call('EXPIRE', key, window * 2)

local remaining = math.floor(new_tokens)
local reset_at = now + window

return {allowed, remaining, reset_at}
"#;

/// FIXED_WINDOW_SCRIPT は固定ウィンドウアルゴリズムの Lua スクリプト。
///
/// KEYS[1]: カウンターキー
/// ARGV[1]: 最大リクエスト数 (limit)
/// ARGV[2]: ウィンドウ秒数
/// ARGV[3]: 現在時刻
const FIXED_WINDOW_SCRIPT: &str = r#"
local key = KEYS[1]
local limit = tonumber(ARGV[1])
local window = tonumber(ARGV[2])
local now = tonumber(ARGV[3])

local window_key = key .. ':' .. tostring(math.floor(now / window))
local count = tonumber(redis.call('GET', window_key) or '0')

local allowed = 0
local remaining = limit - count - 1

if count < limit then
    redis.call('INCR', window_key)
    redis.call('EXPIRE', window_key, window)
    allowed = 1
    remaining = limit - count - 1
else
    remaining = 0
end

local reset_at = (math.floor(now / window) + 1) * window

return {allowed, remaining, reset_at}
"#;

/// SLIDING_WINDOW_SCRIPT はスライディングウィンドウアルゴリズムの Lua スクリプト。
///
/// KEYS[1]: ソートセットキー
/// ARGV[1]: 最大リクエスト数 (limit)
/// ARGV[2]: ウィンドウ秒数
/// ARGV[3]: 現在時刻（ミリ秒精度）
const SLIDING_WINDOW_SCRIPT: &str = r#"
local key = KEYS[1]
local limit = tonumber(ARGV[1])
local window = tonumber(ARGV[2])
local now = tonumber(ARGV[3])

local window_start = now - window

redis.call('ZREMRANGEBYSCORE', key, '-inf', window_start)

local count = redis.call('ZCARD', key)

local allowed = 0
local remaining = limit - count - 1

if count < limit then
    redis.call('ZADD', key, now, now .. ':' .. tostring(math.random(1000000)))
    redis.call('EXPIRE', key, window * 2)
    allowed = 1
    remaining = limit - count - 1
else
    remaining = 0
end

local reset_at = now + window

return {allowed, remaining, reset_at}
"#;

/// RedisRateLimitStore は Redis ベースのレートリミット状態ストア。
pub struct RedisRateLimitStore {
    conn: ConnectionManager,
}

impl RedisRateLimitStore {
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl RateLimitStateStore for RedisRateLimitStore {
    async fn check_token_bucket(
        &self,
        key: &str,
        limit: i64,
        window_secs: i64,
    ) -> anyhow::Result<RateLimitDecision> {
        let now = chrono::Utc::now().timestamp();
        let script = Script::new(TOKEN_BUCKET_SCRIPT);
        let result: Vec<i64> = script
            .key(key)
            .arg(limit)
            .arg(window_secs)
            .arg(now)
            .invoke_async(&mut self.conn.clone())
            .await?;

        if result.len() < 3 {
            return Err(anyhow::anyhow!("unexpected Lua script result"));
        }

        let allowed = result[0] == 1;
        let remaining = result[1];
        let reset_at = result[2];

        if allowed {
            Ok(RateLimitDecision::allowed(remaining, reset_at))
        } else {
            Ok(RateLimitDecision::denied(
                remaining,
                reset_at,
                "rate limit exceeded".to_string(),
            ))
        }
    }

    async fn check_fixed_window(
        &self,
        key: &str,
        limit: i64,
        window_secs: i64,
    ) -> anyhow::Result<RateLimitDecision> {
        let now = chrono::Utc::now().timestamp();
        let script = Script::new(FIXED_WINDOW_SCRIPT);
        let result: Vec<i64> = script
            .key(key)
            .arg(limit)
            .arg(window_secs)
            .arg(now)
            .invoke_async(&mut self.conn.clone())
            .await?;

        if result.len() < 3 {
            return Err(anyhow::anyhow!("unexpected Lua script result"));
        }

        let allowed = result[0] == 1;
        let remaining = result[1];
        let reset_at = result[2];

        if allowed {
            Ok(RateLimitDecision::allowed(remaining, reset_at))
        } else {
            Ok(RateLimitDecision::denied(
                remaining,
                reset_at,
                "rate limit exceeded".to_string(),
            ))
        }
    }

    async fn check_sliding_window(
        &self,
        key: &str,
        limit: i64,
        window_secs: i64,
    ) -> anyhow::Result<RateLimitDecision> {
        let now = chrono::Utc::now().timestamp();
        let script = Script::new(SLIDING_WINDOW_SCRIPT);
        let result: Vec<i64> = script
            .key(key)
            .arg(limit)
            .arg(window_secs)
            .arg(now)
            .invoke_async(&mut self.conn.clone())
            .await?;

        if result.len() < 3 {
            return Err(anyhow::anyhow!("unexpected Lua script result"));
        }

        let allowed = result[0] == 1;
        let remaining = result[1];
        let reset_at = result[2];

        if allowed {
            Ok(RateLimitDecision::allowed(remaining, reset_at))
        } else {
            Ok(RateLimitDecision::denied(
                remaining,
                reset_at,
                "rate limit exceeded".to_string(),
            ))
        }
    }

    async fn reset(&self, key: &str) -> anyhow::Result<()> {
        let mut conn = self.conn.clone();
        redis::cmd("DEL")
            .arg(key)
            .query_async::<()>(&mut conn)
            .await?;
        Ok(())
    }

    async fn get_usage(&self, key: &str, limit: i64, window_secs: i64) -> anyhow::Result<Option<UsageSnapshot>> {
        let mut conn = self.conn.clone();
        let result: Vec<Option<String>> = redis::cmd("HMGET")
            .arg(key)
            .arg("tokens")
            .arg("last_refill")
            .query_async(&mut conn)
            .await?;

        let tokens = match result.first().and_then(|v| v.as_ref()) {
            Some(t) => t.parse::<f64>().unwrap_or(limit as f64),
            None => return Ok(None),
        };
        let last_refill = result
            .get(1)
            .and_then(|v| v.as_ref())
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or_else(|| chrono::Utc::now().timestamp());

        let remaining = tokens.floor() as i64;
        let used = limit - remaining;
        let reset_at = last_refill + window_secs;

        Ok(Some(UsageSnapshot {
            used,
            remaining,
            reset_at,
        }))
    }
}
