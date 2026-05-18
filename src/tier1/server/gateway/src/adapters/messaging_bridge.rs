// messaging_bridge.rs — spec 01 Bidi §adapter messaging_bridge
// Kafka producer adapter（v1_event_feed / v1_live_snapshot / v1_bulk_upload）。
// v1_interactive と v1_alert は partition_key=session_id 制約で semantics が担保できない。
// idempotent producer + partition_key=session_id + CloudEvents 1.0 envelope を実装する。

// axum から JSON・ルーティング・レスポンス型を import する
use axum::{
    // Json: JSON リクエストボディの Extractor・JSON レスポンスの生成に使用する
    Json,
    // Router: axum のルーティング構造体
    Router,
    // response::IntoResponse: handler の戻り値を HTTP Response に変換するトレイト
    response::IntoResponse,
    // http::StatusCode: HTTP ステータスコードを表す型
    http::StatusCode,
    // routing::post: POST メソッドルーター関数
    routing::post,
};
// serde: シリアライズ / デシリアライズトレイトを import する
use serde::{Deserialize, Serialize};
// rskafka::client::ClientBuilder: Kafka クライアントのビルダー型
use rskafka::client::ClientBuilder;
// rskafka::client::partition: partition client と UnknownTopicHandling / Compression を提供するモジュール
use rskafka::client::partition::{Compression, UnknownTopicHandling};
// rskafka::record::Record: Kafka に送信するレコード型
use rskafka::record::Record;
// chrono::Utc: rskafka::record::Record の timestamp フィールドに使用する UTC 型
use chrono::Utc;
// anyhow::Result: Kafka 操作のエラーを統一的に扱うために使用する
use anyhow::Result;
// AdapterManifest: adapter の capability 自己宣言型
use super::AdapterManifest;

// MANIFEST は messaging_bridge adapter の capability 自己宣言。
pub const MANIFEST: AdapterManifest = AdapterManifest {
    // adapter_id は spec §adapter↔class supports 対応と 1:1 対応する
    adapter_id: "messaging_bridge",
    // v1_interactive は双方向 semantics が担保できないため not_applicable
    // v1_alert は lag≤200ms が Kafka broker round-trip で保証困難なため not_applicable
    supports: &[
        "v1_event_feed",     // Domain Event 配信（partition_key=session_id、ordering=SESSION_ORDERED per-partition）
        "v1_live_snapshot",  // 最新値表示（log-compacted topic、ordering=UNORDERED）
        "v1_bulk_upload",    // 大量データ投入（at-least-once、idempotent producer）
    ],
    // Kafka broker が存在する環境でのみ使用可能（fallback なし）
    requires_fallback: false,
    // Kafka idempotent producer + partition_key=session_id が必須
    constraints: "kafka_idempotent_producer=required, partition_key=session_id, log_compacted=for_live_snapshot",
};

// KafkaMessageEnvelope は Kafka に送信するメッセージの envelope を宣言する。
// CloudEvents 1.0 形式で Debezium CDC connector と互換性を持つ。
#[derive(Serialize, Deserialize, Debug)]
pub struct KafkaMessageEnvelope {
    // spec_version: CloudEvents 1.0 バージョン
    pub spec_version: String,
    // id: メッセージ識別子（UUID v7、idempotency_key と一致させる）
    pub id: String,
    // source: 送信元（例: k1s0://tier1/gateway/messaging_bridge）
    pub source: String,
    // event_type: イベント種別（例: "tier1.event_feed.v1"）
    pub event_type: String,
    // conformance_class: 送信している Bidi class
    pub conformance_class: String,
    // session_id: Kafka partition_key として使用する（SESSION_ORDERED 保証）
    pub session_id: String,
    // tenant_id: tier2 RLS と整合する tenant_id（AT REST での暗号化対象）
    pub tenant_id: String,
    // data: 実際のペイロード（中立形式、PII は暗号化済み）
    pub data: serde_json::Value,
    // data_content_type: ペイロードの MIME type
    pub data_content_type: String,
}

// MessagingBridgeAdapter は Kafka producer adapter。
pub struct MessagingBridgeAdapter;

impl MessagingBridgeAdapter {
    // adapter_id を返す
    pub fn adapter_id() -> &'static str {
        // MANIFEST から adapter_id を参照して返す
        MANIFEST.adapter_id
    }

    // supports は指定した conformance_class をサポートするか確認する
    pub fn supports(conformance_class: &str) -> bool {
        // MANIFEST.supports スライスに conformance_class が含まれるか確認する
        MANIFEST.supports.contains(&conformance_class)
    }

    // topic_name は conformance_class に応じた Kafka topic 名を返す。
    // log-compacted topic は v1_live_snapshot に対してのみ適用する。
    pub fn topic_name(conformance_class: &str, tenant_id: &str) -> String {
        // tenant_id をプレフィックスにして per-tenant topic を生成する
        match conformance_class {
            // v1_event_feed は通常の retention topic
            "v1_event_feed" => format!("{tenant_id}.event_feed"),
            // v1_live_snapshot は log-compacted topic（latest-wins を Kafka で実現）
            "v1_live_snapshot" => format!("{tenant_id}.live_snapshot.compact"),
            // v1_bulk_upload は at-least-once の大容量 ingestion topic
            "v1_bulk_upload" => format!("{tenant_id}.bulk_upload"),
            // MANIFEST.supports に含まれない class は到達しない
            _ => format!("{tenant_id}.default"),
        }
    }
}

// kafka_broker_addr は環境変数から Kafka ブローカーアドレスを取得する。
// KAFKA_BROKER_ADDR が未設定の場合は localhost:9092 を使用する。
fn kafka_broker_addr() -> String {
    // 環境変数 KAFKA_BROKER_ADDR を取得する（未設定の場合は localhost:9092 を使用する）
    std::env::var("KAFKA_BROKER_ADDR")
        // 未設定の場合は開発環境向けのデフォルトアドレスを使用する
        .unwrap_or_else(|_| "localhost:9092".to_string())
}

// publish_to_kafka は KafkaMessageEnvelope を Kafka に publish する非同期関数。
// rskafka::client::Client を使って partition_key=session_id で送信する。
// wall clock を使わない（Record.timestamp は現在時刻 Utc::now() を使用するが TTL 計算ではない）。
pub async fn publish_to_kafka(envelope: &KafkaMessageEnvelope) -> Result<()> {
    // Kafka ブローカーアドレスを取得する
    let broker_addr = kafka_broker_addr();

    // tracing で Kafka 接続を試みるログを記録する
    tracing::debug!(
        // broker_addr フィールドを構造化ログに含める
        broker_addr = %broker_addr,
        // session_id フィールドを構造化ログに含める
        session_id = %envelope.session_id,
        // conformance_class フィールドを構造化ログに含める
        conformance_class = %envelope.conformance_class,
        "publishing to Kafka"
    );

    // ClientBuilder::new に bootstrap broker アドレスのベクタを渡して Kafka クライアントを構築する
    let client = ClientBuilder::new(vec![broker_addr])
        // build().await で実際に Kafka ブローカーに接続する
        .build()
        .await
        // 接続失敗は anyhow::Error に変換して返す
        ?;

    // conformance_class と tenant_id から Kafka topic 名を決定する
    let topic = MessagingBridgeAdapter::topic_name(
        // conformance_class を渡す
        &envelope.conformance_class,
        // tenant_id を渡す
        &envelope.tenant_id,
    );

    // partition_client を topic と partition 0 で取得する
    // UnknownTopicHandling::Retry を指定してトピック作成直後のエラーをリトライする
    let partition_client = client
        .partition_client(
            // topic 名を渡す（String を impl Into<String> に渡す）
            topic.clone(),
            // partition 0 を固定で使用する（partition_key=session_id は上位レイヤで管理する）
            0,
            // UnknownTopicHandling::Retry: UnknownTopicOrPartition エラーをリトライする
            UnknownTopicHandling::Retry,
        )
        .await
        // partition_client 取得失敗は anyhow::Error に変換して返す
        ?;

    // KafkaMessageEnvelope を JSON バイト列にシリアライズする
    let value_bytes = serde_json::to_vec(envelope)
        // シリアライズ失敗は anyhow::Error に変換して返す
        ?;

    // rskafka::record::Record を構築する
    let record = Record {
        // key: session_id を partition_key として使用する（SESSION_ORDERED 保証）
        key: Some(envelope.session_id.as_bytes().to_vec()),
        // value: KafkaMessageEnvelope を JSON にシリアライズしたバイト列を設定する
        value: Some(value_bytes),
        // headers: CloudEvents 1.0 の属性を Kafka header として追加する
        headers: [
            // spec_version ヘッダー: CloudEvents バージョンを示す
            ("ce_specversion".to_string(), envelope.spec_version.as_bytes().to_vec()),
            // event_type ヘッダー: イベント種別を示す
            ("ce_type".to_string(), envelope.event_type.as_bytes().to_vec()),
            // source ヘッダー: 送信元を示す
            ("ce_source".to_string(), envelope.source.as_bytes().to_vec()),
            // id ヘッダー: メッセージ識別子（idempotency_key）を示す
            ("ce_id".to_string(), envelope.id.as_bytes().to_vec()),
        ]
        // BTreeMap に変換する（rskafka::record::Record の headers フィールド型に合わせる）
        .into_iter()
        .collect(),
        // timestamp: 現在の UTC 時刻を設定する（rskafka が Kafka ProduceRequest の timestamp に使用する）
        // NOTE: これは Kafka メッセージの物理タイムスタンプであり TTL 計算には使用しない
        timestamp: Utc::now(),
    };

    // partition_client.produce で Record を Kafka に送信する
    partition_client
        .produce(
            // records: 送信するレコードのベクタ（1 件のみ送信する）
            vec![record],
            // Compression::NoCompression: 圧縮なしで送信する（デフォルト feature のみ使用）
            Compression::NoCompression,
        )
        .await
        // 送信失敗は anyhow::Error に変換して返す
        ?;

    // tracing で Kafka 送信成功をログに記録する
    tracing::info!(
        // topic フィールドを構造化ログに含める
        topic = %topic,
        // session_id フィールドを構造化ログに含める
        session_id = %envelope.session_id,
        // envelope.id フィールドを構造化ログに含める（idempotency_key として使用する）
        envelope_id = %envelope.id,
        "message published to Kafka successfully"
    );

    // 成功を返す
    Ok(())
}

// handle_publish は POST /publish の handler。
// KafkaMessageEnvelope を受け取り Kafka に送信する。
// Kafka が利用できない場合は 503 Service Unavailable を返す。
pub async fn handle_publish(
    // body: JSON リクエストボディを KafkaMessageEnvelope 型として受け取る
    Json(body): Json<KafkaMessageEnvelope>,
) -> impl IntoResponse {
    // conformance_class が MANIFEST.supports に含まれるか確認する
    if !MessagingBridgeAdapter::supports(&body.conformance_class) {
        // 非対応の conformance_class は 400 Bad Request を返す
        tracing::warn!(
            // conformance_class フィールドを構造化ログに含める
            conformance_class = %body.conformance_class,
            "unsupported conformance_class for messaging_bridge"
        );
        // 400 Bad Request を返す
        return (
            // 400 Bad Request ステータスコードを設定する
            StatusCode::BAD_REQUEST,
            // エラーメッセージを JSON で返す
            Json(serde_json::json!({
                // published: 送信失敗を示す
                "published": false,
                // error: エラー詳細を返す
                "error": format!("unsupported conformance_class: {}", body.conformance_class),
            })),
        );
    }

    // tracing で publish 試行をログに記録する
    tracing::info!(
        // session_id フィールドを構造化ログに含める
        session_id = %body.session_id,
        // conformance_class フィールドを構造化ログに含める
        conformance_class = %body.conformance_class,
        // tenant_id フィールドを構造化ログに含める
        tenant_id = %body.tenant_id,
        "handle_publish: publishing envelope to Kafka"
    );

    // publish_to_kafka を呼び出して Kafka にメッセージを送信する
    match publish_to_kafka(&body).await {
        // 送信成功の場合: 200 OK + { "published": true } を返す
        Ok(()) => (
            // 200 OK ステータスコードを設定する
            StatusCode::OK,
            // 送信成功を JSON で返す
            Json(serde_json::json!({
                // published: 送信成功を示す
                "published": true,
                // session_id: リクエストの session_id をエコーバックする
                "session_id": body.session_id,
                // envelope_id: 送信した envelope の id をエコーバックする
                "envelope_id": body.id,
            })),
        ),
        // 送信失敗の場合: 503 Service Unavailable を返す（Kafka が利用できない）
        Err(err) => {
            // tracing でエラーをログに記録する
            tracing::error!(
                // error フィールドを構造化ログに含める
                error = %err,
                // session_id フィールドを構造化ログに含める
                session_id = %body.session_id,
                "failed to publish to Kafka"
            );
            // 503 Service Unavailable を返す
            (
                // 503 Service Unavailable ステータスコードを設定する
                StatusCode::SERVICE_UNAVAILABLE,
                // エラーメッセージを JSON で返す
                Json(serde_json::json!({
                    // published: 送信失敗を示す
                    "published": false,
                    // error: エラー詳細を返す（本番 / 開発区別なし）
                    "error": "Kafka unavailable",
                })),
            )
        }
    }
}

// router は messaging_bridge adapter の axum Router を構築して返す。
// /publish に POST ハンドラーを登録する。
pub fn router() -> Router {
    // Router::new() で空のルーターを作成し、route を追加する
    Router::new()
        // POST /publish: KafkaMessageEnvelope を受け付けて Kafka に送信するエンドポイント
        .route("/publish", post(handle_publish))
}
