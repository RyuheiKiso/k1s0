// messaging.rs — k1s0 tier1 Library: メッセージング L1+ facade trait
// Kafka 等の OSS 型を公開 API に露出しない（L1+ ラップ規約）。
// wall-clock TTL 禁止規約: タイムスタンプは HLC 形式（文字列）で表現する。
// 全 trait は Send + Sync を要求する（スレッド安全性の強制）。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;

// MessagingProducer は Kafka を L1+ ラップするメッセージ送信 facade trait。
// 公開 API シグネチャに OSS 型（rdkafka::producer::FutureProducer 等）を一切含まない。
#[async_trait]
pub trait MessagingProducer: Send + Sync {
    // publish はメッセージを指定トピックに送信する。
    // key は Kafka パーティションキー（テナント ID や集約 ID を推奨する）。
    // payload は送信するメッセージのバイト列（シリアライズ形式は呼び出し元が決定する）。
    async fn publish(&self, topic: &str, key: &str, payload: Vec<u8>) -> crate::Result<()>;
}

// MessagingConsumer は Kafka を L1+ ラップするメッセージ受信 facade trait。
// 公開 API シグネチャに OSS 型（rdkafka::consumer::StreamConsumer 等）を一切含まない。
#[async_trait]
pub trait MessagingConsumer: Send + Sync {
    // poll は指定トピックからメッセージをポーリングする。
    // timeout_hlc_ms は HLC ミリ秒単位のタイムアウト（wall-clock 禁止規約に準拠する）。
    // メッセージがない場合は None を返す（タイムアウト時も None）。
    async fn poll(&mut self, timeout_hlc_ms: u64) -> crate::Result<Option<MessagingRecord>>;

    // commit は現在のオフセットをコミットする（at-least-once 配信保証）。
    // poll で受け取ったメッセージを処理後に必ず呼ぶこと。
    async fn commit(&mut self) -> crate::Result<()>;
}

// MessagingRecord は受信したメッセージレコードを表す Library 独自型。
// OSS 固有の型（rdkafka::message::OwnedMessage 等）を公開 API に含まない。
pub struct MessagingRecord {
    // key: Kafka パーティションキー（テナント ID や集約 ID 等）
    pub key: String,

    // payload: メッセージペイロードのバイト列（デシリアライズは呼び出し元が行う）
    pub payload: Vec<u8>,

    // hlc_timestamp: メッセージのタイムスタンプ（HLC 形式文字列）
    // wall-clock 禁止規約: std::time::SystemTime を直接使用しない
    pub hlc_timestamp: String,
}
