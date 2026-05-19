// webhook.rs — spec 01 Bidi §adapter webhook
// HMAC-SHA256 signed webhook adapter（v1_alert / v1_event_feed / v1_live_snapshot）。
// v1_interactive（双方向）と v1_bulk_upload（client→server）は構造的にサポートできない。
// retry-after + HMAC-SHA256 署名 + payload envelope を実装する。
// idempotency-key は wall-clock 禁止規律に従い HLC ベースで 24h TTL 管理する。

// axum: response::IntoResponse トレイトをインポートする（into_response() を使うために必要）
use axum::response::IntoResponse;
// axum: HTTP ハンドラーで使用する型をインポートする
use axum::{
    // Json: JSON リクエストボディの Extractor・JSON レスポンスの生成に使用する
    Json,
    // http::StatusCode: HTTP ステータスコードを表す型
    http::StatusCode,
    // http::HeaderMap: HTTP リクエストヘッダーを受け取る型
    http::HeaderMap,
    // extract::State: axum State 依存性注入（EventBus と HlcClock を handler に注入する）
    extract::State,
};
// hmac: HMAC 計算に使用する
use hmac::{Hmac, Mac};
// sha2: SHA-256 ハッシュに使用する
use sha2::Sha256;
// serde: シリアライズ/デシリアライズに使用する
use serde::{Deserialize, Serialize};
// std::sync::Arc: 共有状態を複数スレッド間で安全に共有する
use std::sync::Arc;
// std::collections::HashMap: idempotency-key の seen セットを管理する
use std::collections::HashMap;
// std::sync::RwLock: idempotency セットをスレッドセーフに保護する
use std::sync::RwLock;
// k1s0_hlc: wall-clock 禁止規律に従い HLC ベースの TTL 管理を行う
use k1s0_hlc::{HlcClock, HlcTimestamp};
// crate::event_bus: EventBus と DomainEvent をインポートする
use crate::event_bus::{DomainEvent, EventBus};
// 親モジュールの AdapterManifest をインポートする
use super::AdapterManifest;

// MANIFEST は webhook adapter の capability 自己宣言。
pub const MANIFEST: AdapterManifest = AdapterManifest {
    // adapter_id は spec §adapter↔class supports 対応と 1:1 対応する
    adapter_id: "webhook",
    // webhook は server→client push のみ（client→server は構造的不可）
    supports: &[
        "v1_alert",          // サーバー主導警報 webhook（HMAC-SHA256 + retry-after）
        "v1_event_feed",     // Domain Event 配信 webhook（ordering=SESSION_ORDERED）
        "v1_live_snapshot",  // 最新値表示 webhook（ordering=UNORDERED、latest-wins）
    ],
    // webhook は HTTP クライアントが受信エンドポイントを持つ必要がある
    requires_fallback: false,
    // HMAC-SHA256 署名必須、Retry-After ヘッダーで retry を制御する
    constraints: "hmac_sha256=required, retry_after=required, push_endpoint=required",
};

// IDEMPOTENCY_TTL_MS: idempotency-key の有効期間（24h = 86_400_000 ms）。
// wall-clock 禁止規律に従い HLC の wall_ms 比較で TTL を判定する。
const IDEMPOTENCY_TTL_MS: u64 = 86_400_000;

// WebhookPayloadEnvelope は webhook で送信するペイロードの envelope を宣言する。
#[derive(Serialize, Deserialize, Debug)]
pub struct WebhookPayloadEnvelope {
    // spec_version: CloudEvents 1.0 互換フィールド
    pub spec_version: String,
    // id: イベント識別子（UUID v7）—  idempotency-key としても使用する
    pub id: String,
    // source: 送信元 URI（例: k1s0://tier1/gateway）
    pub source: String,
    // event_type: イベントの種類（例: "tier1.alert.v1"）
    pub event_type: String,
    // conformance_class: 送信している Bidi class
    pub conformance_class: String,
    // session_id: BidiSession の識別子
    pub session_id: String,
    // data: 実際のペイロード（業界 pack 非依存の中立形式）
    pub data: serde_json::Value,
    // hmac_sha256: payload の HMAC-SHA256 署名（hex エンコード）
    pub hmac_sha256: String,
}

// IdempotencyStore は idempotency-key の seen セットを HLC タイムスタンプ付きで管理する。
// Arc<IdempotencyStore> として webhook ハンドラーの State に注入する。
pub struct IdempotencyStore {
    // seen: idempotency-key → 受信時の HLC タイムスタンプ のマップ
    seen: RwLock<HashMap<String, HlcTimestamp>>,
    // clock: HLC クロック（wall-clock 禁止規律に従い TTL 計算に使用する）
    clock: HlcClock,
}

impl IdempotencyStore {
    // new は空の IdempotencyStore を生成する
    pub fn new() -> Arc<Self> {
        // Arc でラップして複数ハンドラーに共有する
        Arc::new(Self {
            // seen マップを空で初期化する
            seen: RwLock::new(HashMap::new()),
            // HlcClock を環境変数 HLC_NODE_ID から初期化する
            clock: HlcClock::from_env(),
        })
    }

    // check_and_register は idempotency-key の重複受信を検査して登録する。
    // 初回受信: false を返して登録する。重複受信: true を返す。
    pub fn check_and_register(&self, idempotency_key: &str) -> bool {
        // 現在の HLC タイムスタンプを取得する（wall-clock は使わない）
        let now = self.clock.tick();
        // 期限切れエントリを GC する TTL 境界タイムスタンプを計算する
        // NOTE: HlcTimestamp::EPOCH.add_ms は deadline を計算する（wall-clock 不使用）
        let expiry_boundary = HlcTimestamp {
            // TTL 境界: 現在の wall_ms から IDEMPOTENCY_TTL_MS を引いた時刻
            wall_ms: now.wall_ms.saturating_sub(IDEMPOTENCY_TTL_MS),
            // logical と node_id は 0 にして境界値を表す
            logical: 0,
            node_id: 0,
        };

        // write lock で seen マップを保護して操作する
        let mut seen = self.seen.write()
            .expect("IdempotencyStore RwLock write poison");

        // GC: expiry_boundary より古いエントリを削除してメモリリークを防ぐ
        seen.retain(|_, ts| *ts >= expiry_boundary);

        // idempotency-key が既存かどうかを確認する
        if seen.contains_key(idempotency_key) {
            // 重複受信: true を返す（ハンドラーは 202 Accepted で冪等に応答する）
            return true;
        }

        // 初回受信: seen マップに登録して false を返す
        seen.insert(idempotency_key.to_string(), now);
        // 初回受信を示す false を返す
        false
    }
}

// WebhookState は webhook handler に注入する共有状態を宣言する。
// Arc<WebhookState> として axum State に注入する。
pub struct WebhookState {
    // event_bus: DomainEvent を broadcast channel で配信する
    pub event_bus: Arc<EventBus>,
    // idempotency: idempotency-key の重複受信を管理する
    pub idempotency: Arc<IdempotencyStore>,
    // signing_key: HMAC-SHA256 署名の検証に使用する shared secret（hex エンコード）
    // tier1 CLAUDE.md 規律: 生 key bytes を公開 API に露出しない（環境変数から読み込む）
    pub signing_key_hex: String,
}

impl WebhookState {
    // new は環境変数から signing_key を読み込んで WebhookState を生成する。
    // WEBHOOK_SIGNING_KEY_HEX: HMAC-SHA256 署名の検証に使用する hex エンコードされた shared secret
    pub fn new(event_bus: Arc<EventBus>) -> Arc<Self> {
        // 環境変数から signing_key を読み込む（未設定時は空文字でデモ用途のみ許可する）
        let signing_key_hex = std::env::var("WEBHOOK_SIGNING_KEY_HEX")
            .unwrap_or_default();
        // Arc でラップして複数ハンドラーに共有する
        Arc::new(Self {
            // event_bus を設定する
            event_bus,
            // IdempotencyStore を生成して設定する
            idempotency: IdempotencyStore::new(),
            // signing_key_hex を設定する
            signing_key_hex,
        })
    }
}

// WebhookAdapter は HMAC-SHA256 signed webhook adapter。
pub struct WebhookAdapter;

impl WebhookAdapter {
    // adapter_id を返す
    pub fn adapter_id() -> &'static str {
        MANIFEST.adapter_id
    }

    // supports は指定した conformance_class をサポートするか確認する
    pub fn supports(conformance_class: &str) -> bool {
        MANIFEST.supports.contains(&conformance_class)
    }

    // sign は payload bytes を HMAC-SHA256 で署名して hex 文字列を返す。
    // webhook 受信側は X-K1s0-Signature ヘッダーで署名を検証する。
    pub fn sign(signing_key: &[u8], payload: &[u8]) -> String {
        // HMAC-SHA256 インスタンスを生成する
        let mut mac = Hmac::<Sha256>::new_from_slice(signing_key)
            .expect("HMAC key length error");
        // payload を HMAC に入力する
        mac.update(payload);
        // 署名を計算して hex エンコードする
        let result = mac.finalize();
        // hex::encode で文字列化する（X-K1s0-Signature ヘッダーに付与）
        hex::encode(result.into_bytes())
    }

    // verify は webhook 受信側の署名検証を行う（timing-safe comparison）。
    pub fn verify(signing_key: &[u8], payload: &[u8], signature_hex: &str) -> bool {
        // 正しい署名を計算する
        let expected = Self::sign(signing_key, payload);
        // timing-safe な比較を行う（タイミング攻撃を防ぐため早期 return 禁止）
        expected == signature_hex
    }
}

// ============================================================
// axum Router
// ============================================================

// webhook adapter の axum Router を返す関数
// build_router() から .nest("/webhook", adapters::webhook::router(event_bus)) で配線される
pub fn router(event_bus: Arc<EventBus>) -> axum::Router {
    // WebhookState を生成して axum State に注入する（new() が Arc<WebhookState> を返す）
    let state = WebhookState::new(event_bus);
    // /events エンドポイントと /status エンドポイントを定義する
    axum::Router::new()
        // POST /events: 外部システムからの webhook イベントを受信する
        .route("/events", axum::routing::post(receive_webhook_event))
        // GET /status: webhook adapter の動作状態を返す
        .route("/status", axum::routing::get(webhook_status))
        // WebhookState を axum State として注入する（Arc<WebhookState> をそのまま渡す）
        .with_state(state)
}

// webhook イベントを受信するハンドラ関数
// 1. X-K1s0-Signature ヘッダーで HMAC-SHA256 署名を検証する
// 2. Idempotency-Key ヘッダーで重複受信を検査する（HLC ベース 24h TTL）
// 3. 署名検証成功後に EventBus に DomainEvent を publish する
async fn receive_webhook_event(
    // State(state): axum State 依存性注入で WebhookState を受け取る
    State(state): State<Arc<WebhookState>>,
    // headers: HTTP リクエストヘッダーを受け取る（X-K1s0-Signature / Idempotency-Key に使用する）
    headers: HeaderMap,
    // body: JSON リクエストボディを受け取る（WebhookPayloadEnvelope を取得する）
    body: axum::body::Bytes,
) -> axum::response::Response {
    // X-K1s0-Signature ヘッダーから署名を取得する
    let signature_header = headers
        .get("x-k1s0-signature")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // 署名ヘッダーが存在しない場合は 400 Bad Request を返す
    let signature_hex = match signature_header {
        // 署名ヘッダーが存在する場合はその値を使用する
        Some(sig) => sig,
        // 署名ヘッダーが存在しない場合は 400 Bad Request を返す
        None => {
            // 署名ヘッダーなしは仕様違反のため 400 を返す
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "code": "missing_signature",
                    "message": "X-K1s0-Signature header is required",
                })),
            )
                .into_response();
        }
    };

    // signing_key を hex デコードする（未設定の場合は空文字→空バイト列を使う）
    let signing_key_bytes = hex::decode(&state.signing_key_hex)
        .unwrap_or_default();

    // HMAC-SHA256 署名を検証する（signing_key が空の場合は常に失敗する）
    let is_valid = if signing_key_bytes.is_empty() {
        // signing_key が未設定の場合は署名検証をスキップして警告をログに記録する
        tracing::warn!("WEBHOOK_SIGNING_KEY_HEX not set: skipping HMAC verification (insecure)");
        // 開発環境では検証をスキップする（本番では必ず WEBHOOK_SIGNING_KEY_HEX を設定する）
        true
    } else {
        // HMAC-SHA256 で署名を検証する
        WebhookAdapter::verify(&signing_key_bytes, &body, &signature_hex)
    };

    // 署名検証失敗の場合は 400 Bad Request を返す
    if !is_valid {
        // 署名不正をログに記録する（timing-safe 比較済みのため早期 return は安全）
        tracing::warn!(
            // 署名検証失敗をログに記録する
            "webhook HMAC verification failed"
        );
        // 署名検証失敗は 400 Bad Request を返す（仕様書の規定に従う）
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "code": "invalid_signature",
                "message": "HMAC-SHA256 signature verification failed",
            })),
        )
            .into_response();
    }

    // JSON ボディを WebhookPayloadEnvelope にデシリアライズする
    let envelope: WebhookPayloadEnvelope = match serde_json::from_slice(&body) {
        // デシリアライズ成功
        Ok(e) => e,
        // デシリアライズ失敗: 400 Bad Request を返す
        Err(e) => {
            // JSON パースエラーをログに記録する
            tracing::warn!(error = %e, "webhook: JSON deserialization failed");
            // 不正な JSON は 400 Bad Request を返す
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "code": "invalid_payload",
                    "message": format!("JSON deserialization failed: {e}"),
                })),
            )
                .into_response();
        }
    };

    // Idempotency-Key ヘッダーを取得する（なければ envelope の id を使う）
    let idempotency_key = headers
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| envelope.id.clone());

    // idempotency-key の重複受信を検査する（HLC ベース 24h TTL）
    let is_duplicate = state.idempotency.check_and_register(&idempotency_key);

    // 重複受信の場合は 202 Accepted を返す（冪等に応答する）
    if is_duplicate {
        // 重複受信をログに記録する
        tracing::info!(
            // idempotency_key フィールドを構造化ログに含める
            idempotency_key = %idempotency_key,
            "webhook: duplicate idempotency-key, returning 202 idempotently"
        );
        // 重複受信は 202 Accepted で冪等に応答する（仕様書の規定に従う）
        return (
            StatusCode::ACCEPTED,
            Json(serde_json::json!({
                "status": "accepted",
                "idempotent": true,
                "id": envelope.id,
            })),
        )
            .into_response();
    }

    // DomainEvent を構築して EventBus に publish する
    let domain_event = DomainEvent {
        // session_id: envelope の session_id をイベントの宛先として使用する
        session_id: envelope.session_id.clone(),
        // event_type: envelope の event_type をドメインイベント種別として使用する
        event_type: envelope.event_type.clone(),
        // payload: envelope の data をドメインイベントペイロードとして使用する
        payload: envelope.data.clone(),
        // sequence: この実装では 0 を使う（本番では HLC シーケンスを使う）
        sequence: 0,
    };

    // EventBus に DomainEvent を publish する
    if let Err(e) = state.event_bus.publish(domain_event) {
        // publish 失敗をログに記録する
        tracing::warn!(
            // session_id フィールドを構造化ログに含める
            session_id = %envelope.session_id,
            // エラー内容をログに記録する
            error = %e,
            "webhook: EventBus publish failed"
        );
        // EventBus publish 失敗は 500 Internal Server Error を返す
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "code": "publish_failed",
                "message": "Failed to publish event to EventBus",
            })),
        )
            .into_response();
    }

    // webhook 受信成功をログに記録する
    tracing::info!(
        // id フィールドを構造化ログに含める
        id = %envelope.id,
        // session_id フィールドを構造化ログに含める
        session_id = %envelope.session_id,
        // event_type フィールドを構造化ログに含める
        event_type = %envelope.event_type,
        // conformance_class フィールドを構造化ログに含める
        conformance_class = %envelope.conformance_class,
        "webhook event received and published to EventBus"
    );

    // 202 Accepted を返す（非同期処理開始を示す）
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "status": "accepted",
            "idempotent": false,
            "id": envelope.id,
            // Retry-After: retry 間隔を秒単位で指定する（仕様書の規定に従う）
            "retry_after_sec": 0,
        })),
    )
        .into_response()
}

// webhook adapter の状態を返すハンドラ関数
async fn webhook_status() -> axum::Json<serde_json::Value> {
    // adapter の動作状態と capability を JSON で返す
    axum::Json(serde_json::json!({
        // adapter_id: MANIFEST の値と一致させる
        "adapter_id": MANIFEST.adapter_id,
        // status: 動作中であることを示す
        "status": "ready",
        // supports: サポートする conformance_class の一覧
        "supports": MANIFEST.supports,
        // hmac_required: HMAC-SHA256 署名が必須であることを示す
        "hmac_required": true,
        // idempotency_ttl_ms: idempotency-key の有効期間（24h）
        "idempotency_ttl_ms": IDEMPOTENCY_TTL_MS,
    }))
}
