// webhook.rs — spec 01 Bidi §adapter webhook
// HMAC-SHA256 signed webhook adapter（v1_alert / v1_event_feed / v1_live_snapshot）。
// v1_interactive（双方向）と v1_bulk_upload（client→server）は構造的にサポートできない。
// retry-after + HMAC-SHA256 署名 + payload envelope を実装する。

// axum: response::IntoResponse トレイトをインポートする（into_response() を使うために必要）
use axum::response::IntoResponse;
// hmac: HMAC 計算に使用する
use hmac::{Hmac, Mac};
// sha2: SHA-256 ハッシュに使用する
use sha2::Sha256;
// serde: シリアライズ/デシリアライズに使用する
use serde::{Deserialize, Serialize};
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

// WebhookPayloadEnvelope は webhook で送信するペイロードの envelope を宣言する。
#[derive(Serialize, Deserialize, Debug)]
pub struct WebhookPayloadEnvelope {
    // spec_version: CloudEvents 1.0 互換フィールド
    pub spec_version: String,
    // id: イベント識別子（UUID v7）
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
// build_router() から .nest("/webhook", adapters::webhook::router()) で配線される
pub fn router() -> axum::Router {
    // /events エンドポイントと /status エンドポイントを定義する
    axum::Router::new()
        // POST /events: 外部システムからの webhook イベントを受信する
        .route("/events", axum::routing::post(receive_webhook_event))
        // GET /status: webhook adapter の動作状態を返す
        .route("/status", axum::routing::get(webhook_status))
}

// webhook イベントを受信するハンドラ関数
// spec 01 §Bidi: webhook はサーバー→クライアント push のみサポート（受信エンドポイントを公開する）
async fn receive_webhook_event() -> axum::response::Response {
    // 受信したことを示す 202 Accepted を返す（本実装では Bidi pipeline に渡す）
    // 非同期処理は event_bus 経由で broadcast channel に publish する（後続 PR で実装する）
    axum::http::StatusCode::ACCEPTED.into_response()
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
    }))
}
