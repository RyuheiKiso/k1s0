// policy.rs — k1s0 tier1 Library core: retry / timeout / circuit breaker ポリシー
// tier1 Library 全カテゴリの L1+ サービスに適用する横断的ポリシーを定義する。
// wall-clock TTL 禁止（HLC 単位での duration のみ使用する）。
// 公開 API に OSS 型（tower::Policy 等）を露出しない。

// serde: ポリシーの設定値をシリアライズ/デシリアライズする
use serde::{Deserialize, Serialize};

// RetryPolicy は再試行戦略を定義する struct。
// exponential backoff + jitter を標準とし、wall-clock sleep は呼び出し元が担う。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    // max_attempts: 最大試行回数（初回を含む、0 は即失敗）
    pub max_attempts: u32,
    // base_delay_ms: 初回再試行の待機時間（ミリ秒）
    pub base_delay_ms: u64,
    // max_delay_ms: 再試行待機の上限（ミリ秒）
    pub max_delay_ms: u64,
    // multiplier: 指数バックオフの倍率（1.0 は固定待機）
    pub multiplier: f64,
    // jitter: jitter 係数（0.0〜1.0; 0.0 は jitter なし）
    pub jitter: f64,
}

// RetryPolicy のデフォルト値（L1+ 推奨: 3 回 / 100ms 初回 / 5000ms 上限 / 2.0x / 0.2 jitter）
impl Default for RetryPolicy {
    fn default() -> Self {
        // L1+ カテゴリ向け実用的なデフォルトを設定する
        Self {
            // max_attempts: 3 回（初回 + 最大 2 回再試行）
            max_attempts: 3,
            // base_delay_ms: 100ms（初回再試行の待機時間）
            base_delay_ms: 100,
            // max_delay_ms: 5000ms（再試行の上限）
            max_delay_ms: 5000,
            // multiplier: 2.0（指数バックオフ）
            multiplier: 2.0,
            // jitter: 0.2（20% のランダムジッター）
            jitter: 0.2,
        }
    }
}

// RetryPolicy のファクトリメソッド群
impl RetryPolicy {
    // no_retry は再試行なしのポリシーを返す（idempotent でない操作向け）
    pub fn no_retry() -> Self {
        // max_attempts=1 で再試行なし
        Self {
            max_attempts: 1,
            base_delay_ms: 0,
            max_delay_ms: 0,
            multiplier: 1.0,
            jitter: 0.0,
        }
    }

    // compute_delay_ms は試行番号（0 始まり）から次の待機時間（ミリ秒）を計算する。
    // wall-clock 依存なし（duration 計算のみ）。
    pub fn compute_delay_ms(&self, attempt: u32) -> u64 {
        // attempt=0 は初回試行なので 0ms を返す
        if attempt == 0 {
            return 0;
        }
        // 指数バックオフ計算（base_delay × multiplier^(attempt-1)）
        let raw = self.base_delay_ms as f64
            * self.multiplier.powi((attempt - 1) as i32);
        // 上限を適用する
        let capped = raw.min(self.max_delay_ms as f64);
        // jitter を適用する（固定シードなし — 呼び出し元が乱数源を持つ）
        // ここでは jitter 係数を掛けた最大 jitter 幅を返す（実際の加算は呼び出し元）
        (capped * (1.0 + self.jitter)) as u64
    }
}

// TimeoutPolicy は操作全体のタイムアウトを定義する struct。
// wall-clock ではなく HLC tick 数または ms 単位を使う（上位層で解釈する）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutPolicy {
    // connect_timeout_ms: 接続タイムアウト（ミリ秒; 0 は無限待機）
    pub connect_timeout_ms: u64,
    // request_timeout_ms: リクエスト全体のタイムアウト（ミリ秒; 0 は無限待機）
    pub request_timeout_ms: u64,
}

// TimeoutPolicy のデフォルト値（接続 2s / リクエスト 10s）
impl Default for TimeoutPolicy {
    fn default() -> Self {
        // 汎用的なデフォルトタイムアウトを設定する
        Self {
            // connect_timeout_ms: 2000ms（接続タイムアウト）
            connect_timeout_ms: 2000,
            // request_timeout_ms: 10000ms（リクエスト全体タイムアウト）
            request_timeout_ms: 10000,
        }
    }
}

// CircuitBreakerState は circuit breaker の状態を表す enum。
// CLOSED（正常）→ OPEN（遮断）→ HALF_OPEN（試験）の 3 状態 FSM。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    // Closed: 正常状態（全リクエストを通過させる）
    Closed,
    // Open: 遮断状態（全リクエストを即 fail させる）
    Open,
    // HalfOpen: 試験状態（一部リクエストのみ通過させて回復を判定する）
    HalfOpen,
}

// CircuitBreakerPolicy は circuit breaker のパラメータを定義する struct。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerPolicy {
    // failure_threshold: CLOSED → OPEN に遷移する連続失敗回数
    pub failure_threshold: u32,
    // success_threshold: HALF_OPEN → CLOSED に遷移する連続成功回数
    pub success_threshold: u32,
    // half_open_probe_count: HALF_OPEN 状態で通過させる最大リクエスト数
    pub half_open_probe_count: u32,
    // reset_timeout_ms: OPEN 状態から HALF_OPEN に自動遷移するまでの待機（ミリ秒）
    pub reset_timeout_ms: u64,
}

// CircuitBreakerPolicy のデフォルト値
impl Default for CircuitBreakerPolicy {
    fn default() -> Self {
        // L1+ カテゴリ向け実用的なデフォルトを設定する
        Self {
            // failure_threshold: 5 回連続失敗で OPEN に遷移
            failure_threshold: 5,
            // success_threshold: 2 回連続成功で CLOSED に戻る
            success_threshold: 2,
            // half_open_probe_count: HALF_OPEN で最大 1 リクエスト通過
            half_open_probe_count: 1,
            // reset_timeout_ms: 30 秒後に HALF_OPEN へ自動遷移
            reset_timeout_ms: 30_000,
        }
    }
}

// ServicePolicy はサービス呼び出し全体のポリシーを集約する struct。
// RetryPolicy + TimeoutPolicy + CircuitBreakerPolicy を 1 単位で渡す。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePolicy {
    // retry: 再試行ポリシー（L1+ では必須; L3 では no_retry を推奨）
    pub retry: RetryPolicy,
    // timeout: タイムアウトポリシー
    pub timeout: TimeoutPolicy,
    // circuit_breaker: circuit breaker ポリシー（None は circuit breaker なし）
    pub circuit_breaker: Option<CircuitBreakerPolicy>,
}

// ServicePolicy のデフォルト値（L1+ 向け: retry + timeout + circuit breaker あり）
impl Default for ServicePolicy {
    fn default() -> Self {
        // L1+ 向けデフォルトポリシーを設定する
        Self {
            // retry: L1+ 推奨デフォルト（3 回 / 100ms / 5000ms）
            retry: RetryPolicy::default(),
            // timeout: 接続 2s / リクエスト 10s
            timeout: TimeoutPolicy::default(),
            // circuit_breaker: デフォルト有効（5 失敗 / 2 成功 / 30s reset）
            circuit_breaker: Some(CircuitBreakerPolicy::default()),
        }
    }
}
