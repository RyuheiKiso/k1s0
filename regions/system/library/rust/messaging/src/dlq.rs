//! DLQ（Dead Letter Queue）ヘルパー。
//!
//! Consumer のリトライ失敗後に DLQ トピックへ転送するユーティリティを提供する。
//! DLQ トピック名は元トピック名 + `.dlq` サフィックスのパターンに従う。

use std::collections::HashMap;

use crate::error::MessagingError;
use crate::event::EventEnvelope;
use crate::producer::EventProducer;

/// DLQ トピック名のサフィックス。
const DLQ_SUFFIX: &str = ".dlq";

/// デフォルトの最大リトライ回数。
const DEFAULT_MAX_RETRIES: u32 = 3;

/// 元トピック名から DLQ トピック名を生成する。
///
/// # Examples
///
/// ```
/// use k1s0_messaging::dlq::dlq_topic_name;
/// assert_eq!(dlq_topic_name("k1s0.service.order.created.v1"), "k1s0.service.order.created.v1.dlq");
/// ```
pub fn dlq_topic_name(original_topic: &str) -> String {
    format!("{}{}", original_topic, DLQ_SUFFIX)
}

/// DLQ トピック名から元トピック名を復元する。
/// `.dlq` サフィックスがない場合は None を返す。
pub fn original_topic_name(dlq_topic: &str) -> Option<&str> {
    dlq_topic.strip_suffix(DLQ_SUFFIX)
}

/// 処理失敗したメッセージを DLQ トピックに転送する。
///
/// `producer` を使い、元トピック名 + `.dlq` の DLQ トピックにメッセージを発行する。
/// ヘッダーに元トピック名とエラーメッセージを付与する。
pub async fn forward_to_dlq(
    producer: &dyn EventProducer,
    original_topic: &str,
    key: &str,
    payload: Vec<u8>,
    error_message: &str,
) -> Result<(), MessagingError> {
    let dlq_topic = dlq_topic_name(original_topic);

    let mut headers = Vec::new();
    headers.push((
        "original_topic".to_string(),
        original_topic.as_bytes().to_vec(),
    ));
    headers.push(("error".to_string(), error_message.as_bytes().to_vec()));

    let envelope = EventEnvelope {
        topic: dlq_topic,
        key: key.to_string(),
        payload,
        headers,
        metadata: HashMap::new(),
    };

    producer.publish(envelope).await
}

/// リトライ付きメッセージ処理ヘルパー。
///
/// `handler` を最大 `max_retries` 回試行し、全て失敗した場合に
/// DLQ トピックへ転送する。成功した場合は Ok(()) を返す。
pub async fn process_with_dlq_fallback<F, Fut>(
    producer: &dyn EventProducer,
    original_topic: &str,
    key: &str,
    payload: Vec<u8>,
    max_retries: Option<u32>,
    handler: F,
) -> Result<(), MessagingError>
where
    F: Fn(&[u8]) -> Fut,
    Fut: std::future::Future<Output = Result<(), MessagingError>>,
{
    let retries = max_retries.unwrap_or(DEFAULT_MAX_RETRIES);
    let mut last_error = None;

    for _ in 0..retries {
        match handler(&payload).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_error = Some(e);
            }
        }
    }

    let error_message = last_error
        .as_ref()
        .map(|e| e.to_string())
        .unwrap_or_else(|| "unknown error".to_string());

    forward_to_dlq(producer, original_topic, key, payload, &error_message).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::producer::NoOpEventProducer;

    // dlq_topic_name が元のトピック名に .dlq サフィックスを付与することを確認する。
    #[test]
    fn test_dlq_topic_name() {
        assert_eq!(
            dlq_topic_name("k1s0.service.order.created.v1"),
            "k1s0.service.order.created.v1.dlq"
        );
    }

    // system tier トピックに対しても dlq_topic_name が正しく .dlq サフィックスを付与することを確認する。
    #[test]
    fn test_dlq_topic_name_system() {
        assert_eq!(
            dlq_topic_name("k1s0.system.auth.login.v1"),
            "k1s0.system.auth.login.v1.dlq"
        );
    }

    // original_topic_name が DLQ トピック名から .dlq サフィックスを除去した元のトピック名を返すことを確認する。
    #[test]
    fn test_original_topic_name() {
        assert_eq!(
            original_topic_name("k1s0.service.order.created.v1.dlq"),
            Some("k1s0.service.order.created.v1")
        );
    }

    // .dlq サフィックスのないトピック名に original_topic_name を適用すると None が返ることを確認する。
    #[test]
    fn test_original_topic_name_no_suffix() {
        assert_eq!(
            original_topic_name("k1s0.service.order.created.v1"),
            None
        );
    }

    // forward_to_dlq がメッセージを DLQ トピックへ正常に転送できることを確認する。
    #[tokio::test]
    async fn test_forward_to_dlq() {
        let producer = NoOpEventProducer;
        let result = forward_to_dlq(
            &producer,
            "k1s0.service.order.created.v1",
            "order-123",
            b"payload".to_vec(),
            "processing failed",
        )
        .await;
        assert!(result.is_ok());
    }

    // ハンドラーが成功した場合に process_with_dlq_fallback が Ok を返すことを確認する。
    #[tokio::test]
    async fn test_process_with_dlq_fallback_success() {
        let producer = NoOpEventProducer;
        let result = process_with_dlq_fallback(
            &producer,
            "k1s0.service.order.created.v1",
            "order-123",
            b"payload".to_vec(),
            Some(3),
            |_payload| async { Ok(()) },
        )
        .await;
        assert!(result.is_ok());
    }

    // リトライが全て失敗した場合に process_with_dlq_fallback が DLQ へフォールバックし Ok を返すことを確認する。
    #[tokio::test]
    async fn test_process_with_dlq_fallback_all_retries_fail() {
        let producer = NoOpEventProducer;
        let result = process_with_dlq_fallback(
            &producer,
            "k1s0.service.order.created.v1",
            "order-123",
            b"payload".to_vec(),
            Some(3),
            |_payload| async {
                Err(MessagingError::ConsumerError(
                    "processing failed".to_string(),
                ))
            },
        )
        .await;
        // forward_to_dlq は NoOpEventProducer なので成功する
        assert!(result.is_ok());
    }

    // max_retries を None にした場合にデフォルト（3回）のリトライが実行されることを確認する。
    #[tokio::test]
    async fn test_process_with_dlq_fallback_default_retries() {
        let producer = NoOpEventProducer;
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let counter = call_count.clone();

        let result = process_with_dlq_fallback(
            &producer,
            "k1s0.service.order.created.v1",
            "order-123",
            b"payload".to_vec(),
            None, // デフォルト: 3回
            |_payload| {
                let c = counter.clone();
                async move {
                    c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Err(MessagingError::ConsumerError("fail".to_string()))
                }
            },
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(
            call_count.load(std::sync::atomic::Ordering::SeqCst),
            DEFAULT_MAX_RETRIES
        );
    }
}
