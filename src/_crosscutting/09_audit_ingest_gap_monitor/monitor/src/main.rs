// k1s0-impl: IMPL-cross_pii-0002 realizes=FR-cross_pii-002
// k1s0 監査取り込みギャップモニターのメインファイル
// audit_event の hash chain 改竄検知と ingest gap heartbeat (5 分間隔) を実装する
use std::{
    // HashMap でイベントを管理する
    collections::HashMap,
    // Arc でグローバル状態を共有する
    sync::Arc,
    // Duration で時間を管理する
    time::Duration,
};
// tokio の非同期 Mutex をインポートする
use tokio::sync::Mutex;
// serde のシリアライズ/デシリアライズトレイトをインポートする
use serde::{Deserialize, Serialize};
// SHA-256 ハッシュ関数をインポートする
use sha2::{Sha256, Digest};
// 16 進数エンコードをインポートする
use hex;
// tracing でログを記録する
use tracing::{info, warn, error};
// anyhow エラーハンドリングをインポートする
use anyhow::{Result, anyhow};
// chrono で日時を管理する
use chrono::{DateTime, Utc};
// prometheus メトリクスをインポートする
use prometheus::{
    // Counter: 単調増加カウンターをインポートする
    Counter,
    // Gauge: 現在値を保持するゲージをインポートする
    Gauge,
    // Registry: メトリクスレジストリをインポートする
    Registry,
    // opts マクロ: メトリクスオプション定義に使用する
    opts,
};
// axum ルーターをインポートする
use axum::{Router, routing::get, extract::State, response::IntoResponse};

// 監査イベントを表す構造体: hash chain の各ノードを定義する
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuditEvent {
    // イベント ID: 監査イベントの一意識別子
    event_id: String,
    // タイムスタンプ: イベント発生時刻 (UTC)
    timestamp: DateTime<Utc>,
    // イベント種別: ログイン/ログアウト/データアクセス等のイベント種別
    event_type: String,
    // アクター ID: イベントを起こしたユーザー/サービスの識別子
    actor_id: String,
    // リソース ID: 操作対象のリソース識別子
    resource_id: String,
    // 前イベントハッシュ: hash chain の前ノードのハッシュ値
    previous_hash: String,
    // 現イベントハッシュ: このイベントのハッシュ値 (前ハッシュを含む)
    current_hash: String,
    // シーケンス番号: hash chain 内での順序番号
    sequence_number: u64,
}

// 監査ギャップ情報を表す構造体: 検知されたギャップの詳細を定義する
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuditGap {
    // ギャップ開始シーケンス番号: ギャップが始まる直前のシーケンス番号
    from_sequence: u64,
    // ギャップ終了シーケンス番号: ギャップが終わる直後のシーケンス番号
    to_sequence: u64,
    // 検知時刻: ギャップが検知された時刻
    detected_at: DateTime<Utc>,
    // ギャップ種別: sequence_gap (欠番) / hash_mismatch (改竄) / timeout (タイムアウト)
    gap_type: String,
}

// モニターの共有状態を保持する構造体
struct MonitorState {
    // 最後に受信した監査イベントのシーケンス番号
    last_sequence: u64,
    // 最後に受信した監査イベントのハッシュ値
    last_hash: String,
    // 最後にハートビートを記録した時刻
    last_heartbeat_at: DateTime<Utc>,
    // 検知されたギャップのリスト
    detected_gaps: Vec<AuditGap>,
    // Prometheus カウンター: 受信イベント総数
    events_received_total: Counter,
    // Prometheus カウンター: 改竄検知総数
    tampering_detected_total: Counter,
    // Prometheus ゲージ: 最後のハートビートからの経過秒数
    seconds_since_last_heartbeat: Gauge,
    // Prometheus ゲージ: 検知されたギャップ総数
    gaps_detected_total: Gauge,
}

// MonitorState の初期化実装
impl MonitorState {
    // 新しい MonitorState を作成する
    fn new(registry: &Registry) -> Result<Self> {
        // 受信イベント総数カウンターを作成する
        let events_received_total = Counter::with_opts(opts!(
            "k1s0_audit_events_received_total",
            "受信した監査イベントの総数"
        ))?;
        // 改竄検知総数カウンターを作成する
        let tampering_detected_total = Counter::with_opts(opts!(
            "k1s0_audit_tampering_detected_total",
            "検知された hash chain 改竄の総数"
        ))?;
        // 最後のハートビートからの経過秒数ゲージを作成する
        let seconds_since_last_heartbeat = Gauge::with_opts(opts!(
            "k1s0_audit_seconds_since_last_heartbeat",
            "最後のハートビートから経過した秒数"
        ))?;
        // 検知されたギャップ総数ゲージを作成する
        let gaps_detected_total = Gauge::with_opts(opts!(
            "k1s0_audit_gaps_detected_total",
            "検知された監査ギャップの総数"
        ))?;
        // メトリクスをレジストリに登録する
        registry.register(Box::new(events_received_total.clone()))?;
        // 改竄検知カウンターをレジストリに登録する
        registry.register(Box::new(tampering_detected_total.clone()))?;
        // ハートビートゲージをレジストリに登録する
        registry.register(Box::new(seconds_since_last_heartbeat.clone()))?;
        // ギャップ総数ゲージをレジストリに登録する
        registry.register(Box::new(gaps_detected_total.clone()))?;
        // MonitorState を返す
        Ok(Self {
            // 初期シーケンス番号を 0 に設定する
            last_sequence: 0,
            // 初期ハッシュを genesis ハッシュに設定する
            last_hash: "genesis".to_string(),
            // 初期ハートビート時刻を現在時刻に設定する
            last_heartbeat_at: Utc::now(),
            // 検知されたギャップリストを空で初期化する
            detected_gaps: Vec::new(),
            // カウンターとゲージを設定する
            events_received_total,
            tampering_detected_total,
            seconds_since_last_heartbeat,
            gaps_detected_total,
        })
    }
}

// 監査イベントのハッシュ計算関数: イベント内容と前ハッシュを連結して SHA-256 を計算する
fn compute_event_hash(event: &AuditEvent, previous_hash: &str) -> String {
    // SHA-256 ハッシャーを初期化する
    let mut hasher = Sha256::new();
    // イベント ID をハッシュ入力に追加する
    hasher.update(event.event_id.as_bytes());
    // タイムスタンプをハッシュ入力に追加する
    hasher.update(event.timestamp.to_rfc3339().as_bytes());
    // イベント種別をハッシュ入力に追加する
    hasher.update(event.event_type.as_bytes());
    // アクター ID をハッシュ入力に追加する
    hasher.update(event.actor_id.as_bytes());
    // リソース ID をハッシュ入力に追加する
    hasher.update(event.resource_id.as_bytes());
    // 前イベントハッシュをハッシュ入力に追加する
    hasher.update(previous_hash.as_bytes());
    // シーケンス番号をハッシュ入力に追加する
    hasher.update(event.sequence_number.to_le_bytes());
    // ハッシュを計算して 16 進数文字列として返す
    hex::encode(hasher.finalize())
}

// hash chain 検証関数: 受信イベントの hash chain を検証する
fn verify_hash_chain(
    // 受信したイベント
    event: &AuditEvent,
    // 前イベントの期待されるハッシュ値
    expected_previous_hash: &str,
) -> bool {
    // 期待されるハッシュを計算する
    let computed_hash = compute_event_hash(event, expected_previous_hash);
    // 計算したハッシュが current_hash と一致するかを確認する
    if computed_hash != event.current_hash {
        // hash chain 不一致ログを出力する
        warn!(
            "hash chain 不一致検知: event_id={}, computed={}, received={}",
            event.event_id,
            &computed_hash[..16],
            &event.current_hash[..16.min(event.current_hash.len())],
        );
        // 検証失敗を返す
        return false;
    }
    // 前イベントハッシュが期待値と一致するかを確認する
    if event.previous_hash != expected_previous_hash {
        // 前ハッシュ不一致ログを出力する
        warn!(
            "前ハッシュ不一致検知: event_id={}, expected={}, received={}",
            event.event_id,
            &expected_previous_hash[..16.min(expected_previous_hash.len())],
            &event.previous_hash[..16.min(event.previous_hash.len())],
        );
        // 検証失敗を返す
        return false;
    }
    // 検証成功を返す
    true
}

// 監査イベント処理関数: 受信イベントを検証して状態を更新する
async fn process_audit_event(
    // モニター共有状態への Arc<Mutex> 参照
    state: &Arc<Mutex<MonitorState>>,
    // 処理するイベント
    event: AuditEvent,
) -> Result<()> {
    // 共有状態のロックを取得する
    let mut state_guard = state.lock().await;
    // 受信イベント数カウンターを増加させる
    state_guard.events_received_total.inc();
    // ハートビート時刻を更新する
    state_guard.last_heartbeat_at = Utc::now();

    // シーケンス番号のギャップを確認する
    let expected_sequence = state_guard.last_sequence + 1;
    // シーケンス番号が期待値と異なる場合はギャップを記録する
    if event.sequence_number != expected_sequence {
        // シーケンスギャップを検知したことをログに記録する
        warn!(
            "シーケンスギャップ検知: expected={}, received={}",
            expected_sequence, event.sequence_number
        );
        // ギャップを検知リストに追加する
        let gap = AuditGap {
            // ギャップ開始シーケンス番号を設定する
            from_sequence: state_guard.last_sequence,
            // ギャップ終了シーケンス番号を設定する
            to_sequence: event.sequence_number,
            // 検知時刻を現在時刻に設定する
            detected_at: Utc::now(),
            // ギャップ種別: sequence_gap
            gap_type: "sequence_gap".to_string(),
        };
        // ギャップをリストに追加する
        state_guard.detected_gaps.push(gap);
        // ギャップ総数ゲージを更新する
        state_guard.gaps_detected_total.set(state_guard.detected_gaps.len() as f64);
    }

    // hash chain の整合性を検証する
    let previous_hash = state_guard.last_hash.clone();
    // hash chain を検証する
    if !verify_hash_chain(&event, &previous_hash) {
        // 改竄検知カウンターを増加させる
        state_guard.tampering_detected_total.inc();
        // 改竄を示すギャップを記録する
        let tampering_gap = AuditGap {
            // 改竄が検知されたシーケンス番号を設定する
            from_sequence: state_guard.last_sequence,
            // 改竄イベントのシーケンス番号を設定する
            to_sequence: event.sequence_number,
            // 検知時刻を現在時刻に設定する
            detected_at: Utc::now(),
            // ギャップ種別: hash_mismatch (改竄)
            gap_type: "hash_mismatch".to_string(),
        };
        // 改竄ギャップをリストに追加する
        state_guard.detected_gaps.push(tampering_gap);
        // ギャップ総数ゲージを更新する
        state_guard.gaps_detected_total.set(state_guard.detected_gaps.len() as f64);
        // 改竄検知エラーを返す
        return Err(anyhow!(
            "hash chain 改竄検知: event_id={}, sequence={}",
            event.event_id,
            event.sequence_number
        ));
    }

    // 状態を更新する (シーケンス番号とハッシュ値を更新する)
    state_guard.last_sequence = event.sequence_number;
    // 最後のハッシュ値を現イベントのハッシュで更新する
    state_guard.last_hash = event.current_hash.clone();
    // イベント処理成功ログを出力する
    info!(
        "監査イベント処理成功: event_id={}, sequence={}",
        event.event_id, event.sequence_number
    );
    // 正常終了を返す
    Ok(())
}

// ハートビートギャップ監視タスク: 5 分間隔で ingest gap を検知する
async fn heartbeat_monitor_task(
    // モニター共有状態への Arc<Mutex> 参照
    state: Arc<Mutex<MonitorState>>,
    // ハートビート間隔: 5 分 = 300 秒
    interval_secs: u64,
) {
    // ハートビートギャップ監視タスク開始ログを出力する
    info!("ハートビートギャップ監視タスク開始: interval={}秒", interval_secs);
    // 指定間隔で繰り返しハートビートを確認する
    loop {
        // 指定した間隔を待機する
        tokio::time::sleep(Duration::from_secs(interval_secs)).await;
        // 共有状態のロックを取得する
        let mut state_guard = state.lock().await;
        // 最後のハートビートからの経過秒数を計算する
        let elapsed = Utc::now()
            .signed_duration_since(state_guard.last_heartbeat_at)
            .num_seconds();
        // 経過秒数ゲージを更新する
        state_guard.seconds_since_last_heartbeat.set(elapsed as f64);
        // ハートビートタイムアウトの閾値: 指定間隔の 2 倍
        let timeout_threshold = (interval_secs * 2) as i64;
        // 経過秒数がタイムアウト閾値を超えた場合はギャップを記録する
        if elapsed > timeout_threshold {
            // タイムアウトギャップを警告ログに記録する
            warn!(
                "監査 ingest ギャップ検知 (タイムアウト): elapsed={}秒, threshold={}秒",
                elapsed, timeout_threshold
            );
            // タイムアウトギャップを検知リストに追加する
            let timeout_gap = AuditGap {
                // タイムアウト開始シーケンス番号を設定する
                from_sequence: state_guard.last_sequence,
                // タイムアウト終了シーケンス番号は未確定のため 0 を設定する
                to_sequence: 0,
                // 検知時刻を現在時刻に設定する
                detected_at: Utc::now(),
                // ギャップ種別: timeout
                gap_type: "timeout".to_string(),
            };
            // タイムアウトギャップをリストに追加する
            state_guard.detected_gaps.push(timeout_gap);
            // ギャップ総数ゲージを更新する
            state_guard.gaps_detected_total.set(state_guard.detected_gaps.len() as f64);
        } else {
            // ハートビートが正常であることをログに記録する
            info!("ハートビート正常: elapsed={}秒", elapsed);
        }
    }
}

// Prometheus メトリクスエンドポイントのハンドラ
async fn metrics_handler(
    // Prometheus レジストリへの Arc<Mutex> 参照を State で受け取る
    State(registry): State<Arc<Registry>>,
) -> impl IntoResponse {
    // Prometheus メトリクスをテキスト形式でエンコードする
    let encoder = prometheus::TextEncoder::new();
    // メトリクスファミリーを収集する
    let metric_families = registry.gather();
    // テキスト形式にエンコードする
    match encoder.encode_to_string(&metric_families) {
        // エンコード成功の場合はテキストを返す
        Ok(text) => (axum::http::StatusCode::OK, text),
        // エンコード失敗の場合はエラーを返す
        Err(e) => {
            // エンコードエラーログを記録する
            error!("メトリクスエンコード失敗: {}", e);
            // 500 Internal Server Error を返す
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    }
}

// ヘルスチェックエンドポイントのハンドラ
async fn health_handler() -> impl IntoResponse {
    // ヘルスチェック応答を返す
    (axum::http::StatusCode::OK, axum::Json(serde_json::json!({"status": "ok"})))
}

// 監査イベントシミュレーションタスク: テスト用にイベントを定期生成する
async fn audit_event_simulation_task(
    // モニター共有状態への Arc<Mutex> 参照
    state: Arc<Mutex<MonitorState>>,
) {
    // テスト用シミュレーションタスク開始ログを出力する
    info!("監査イベントシミュレーションタスク開始 (テスト用)");
    // シーケンス番号のカウンターを初期化する
    let mut sequence: u64 = 1;
    // 前ハッシュを genesis ハッシュに設定する
    let mut prev_hash = "genesis".to_string();
    // 定期的に模擬イベントを生成する
    loop {
        // 30 秒ごとにイベントを生成する
        tokio::time::sleep(Duration::from_secs(30)).await;
        // テスト用の監査イベントを生成する
        let mut event = AuditEvent {
            // イベント ID を生成する
            event_id: format!("sim-event-{}", sequence),
            // タイムスタンプを現在時刻に設定する
            timestamp: Utc::now(),
            // イベント種別: data_access (データアクセス)
            event_type: "data_access".to_string(),
            // アクター ID: simulation-service
            actor_id: "simulation-service".to_string(),
            // リソース ID: resource-001
            resource_id: format!("resource-{:03}", sequence % 10),
            // 前ハッシュを設定する
            previous_hash: prev_hash.clone(),
            // 現ハッシュは後で計算して設定する
            current_hash: String::new(),
            // シーケンス番号を設定する
            sequence_number: sequence,
        };
        // イベントのハッシュを計算する
        event.current_hash = compute_event_hash(&event, &prev_hash);
        // 現ハッシュを次のイベントの前ハッシュとして保存する
        prev_hash = event.current_hash.clone();
        // イベントを処理する
        if let Err(e) = process_audit_event(&state, event).await {
            // イベント処理エラーログを記録する
            error!("シミュレーションイベント処理エラー: {}", e);
        }
        // シーケンス番号をインクリメントする
        sequence += 1;
    }
}

// メイン関数: 監査ギャップモニターを起動する
#[tokio::main]
async fn main() -> Result<()> {
    // tracing サブスクライバーを初期化する
    tracing_subscriber::fmt()
        // 環境変数フィルターを設定する
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("k1s0_audit_gap_monitor=info".parse()?)
        )
        // tracing サブスクライバーを初期化する
        .init();

    // Prometheus レジストリを作成する
    let registry = Registry::new();
    // モニター共有状態を初期化する
    let monitor_state = Arc::new(Mutex::new(MonitorState::new(&registry)?));
    // Prometheus レジストリを Arc でラップする
    let registry = Arc::new(registry);
    // 起動ログを出力する
    info!("k1s0 監査取り込みギャップモニター起動");

    // ハートビートギャップ監視タスクを非同期で起動する (5 分 = 300 秒間隔)
    let state_clone = monitor_state.clone();
    // tokio タスクとして heartbeat_monitor_task を起動する
    tokio::spawn(async move {
        // 5 分 (300 秒) 間隔でハートビートを確認する
        heartbeat_monitor_task(state_clone, 300).await;
    });

    // シミュレーションタスクを非同期で起動する (テスト用)
    let state_clone2 = monitor_state.clone();
    // tokio タスクとしてシミュレーションタスクを起動する
    tokio::spawn(async move {
        // テスト用シミュレーションタスクを起動する
        audit_event_simulation_task(state_clone2).await;
    });

    // HTTP サーバーをメトリクスエンドポイントと共に起動する
    let app = Router::new()
        // ヘルスチェックエンドポイント: GET /health
        .route("/health", get(health_handler))
        // Prometheus メトリクスエンドポイント: GET /metrics
        .route("/metrics", get(metrics_handler))
        // Prometheus レジストリを State として設定する
        .with_state(registry);

    // HTTP サーバーのリスニングアドレスを設定する
    let listen_addr = std::env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:9090".to_string());
    // TCP リスナーを作成する
    let listener = tokio::net::TcpListener::bind(&listen_addr).await?;
    // HTTP サーバー起動ログを出力する
    info!("HTTP サーバー起動: addr={}", listen_addr);
    // axum サーバーを起動する
    axum::serve(listener, app).await?;
    // 正常終了を返す
    Ok(())
}
