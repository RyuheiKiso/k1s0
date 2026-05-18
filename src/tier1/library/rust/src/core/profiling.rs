// profiling.rs — k1s0 tier1 Library core: Profiling L2* trait
// Observability/Profiling カテゴリを L2* として定義する。
// L2*: 族内で共通の API + OSS 概念は Library 独自語彙に翻訳する。
// backend 専用（frontend には提供しない）。
// 公開 API に OSS 型（pprof::Report 等）を露出しない。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::Result;
// serde: プロファイリングデータのシリアライズに使用する
use serde::{Deserialize, Serialize};

// ProfileKind は Library 独自のプロファイリング種別。
// OSS の pprof profile type に依存しない語彙で定義する。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileKind {
    // Cpu: CPU プロファイリング（サンプリング周期あり）
    Cpu,
    // Memory: メモリ割り当てプロファイリング（ヒープスナップショット）
    Memory,
    // Goroutine: goroutine / thread スタックトレース
    Goroutine,
    // BlockContention: blocking 操作の競合プロファイリング
    BlockContention,
    // MutexContention: mutex 競合プロファイリング
    MutexContention,
}

// ProfileRequest はプロファイリング開始要求のパラメータを保持する struct。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileRequest {
    // kind: プロファイリング種別
    pub kind: ProfileKind,
    // duration_ms: プロファイリング期間（ミリ秒; 0 は手動 stop まで）
    pub duration_ms: u64,
    // sample_rate_hz: CPU プロファイリングのサンプリング周波数（Hz; Cpu 以外は無視する）
    pub sample_rate_hz: u32,
    // label: プロファイルに付与するラベル（識別用）
    pub label: String,
}

// ProfileData は取得したプロファイルデータを表す Library 独自型。
// pprof や perf 等の OSS 固有フォーマットを外部に露出しない。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileData {
    // kind: プロファイリング種別
    pub kind: ProfileKind,
    // label: プロファイルのラベル
    pub label: String,
    // payload_bytes: プロファイルのバイナリペイロード（pprof proto 形式等を内部で使用）
    // 公開 API では不透明なバイト列として扱う（OSS フォーマットは実装詳細）
    pub payload_bytes: Vec<u8>,
    // duration_ms: 実際に計測した期間（ミリ秒）
    pub duration_ms: u64,
    // sample_count: 収集したサンプル数（CPU プロファイリングのみ）
    pub sample_count: u64,
}

// Profiler は Library 独自のプロファイリング抽象 trait（L2*）。
// backend 専用; frontend ビルドには含まない。
// OSS の pprof::ProfilerGuard 等を公開 API に露出しない。
#[async_trait]
pub trait Profiler: Send + Sync {
    // start は ProfileRequest を受け取り、プロファイリングを開始する。
    // 返した profile_id を stop で渡すことでデータを回収する。
    async fn start(&self, request: ProfileRequest) -> Result<String>;

    // stop は profile_id を受け取り、ProfileData を返す。
    // duration_ms=0 で開始した場合はここで終了する。
    async fn stop(&self, profile_id: &str) -> Result<ProfileData>;

    // snapshot は duration_ms を指定して同期的にプロファイルを取得する。
    // start / stop のペアを 1 呼び出しで完結させる便利メソッド。
    async fn snapshot(&self, request: ProfileRequest) -> Result<ProfileData>;

    // upload は ProfileData をバックエンド（Pyroscope / pprof UI 等）に送信する。
    // OSS 固有の URL / API key は実装で保持し、呼び出し元には露出しない。
    async fn upload(&self, data: &ProfileData) -> Result<()>;
}
