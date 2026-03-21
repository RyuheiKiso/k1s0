//! DLQ（Dead Letter Queue）ヘルパー。
//!
//! Consumer のリトライ失敗後に DLQ トピックへ転送するユーティリティを提供する。
//! DLQ トピック名は元トピック名 + `.dlq` サフィックスのパターンに従う。

use std::collections::HashMap;
use std::time::Duration;

use crate::error::MessagingError;
use crate::event::EventEnvelope;
use crate::producer::EventProducer;

/// DLQ トピック名のサフィックス。
const DLQ_SUFFIX: &str = ".dlq";

/// デフォルトの最大リトライ回数。
const DEFAULT_MAX_RETRIES: u32 = 3;

/// リトライ初回待機時間（ミリ秒）。エクスポネンシャルバックオフの起点。
const RETRY_INITIAL_DELAY_MS: u64 = 100;

/// リトライ待機時間の上限（ミリ秒）。バックオフが際限なく伸びないよう上限を設ける。
const RETRY_MAX_DELAY_MS: u64 = 30_000;

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

    let headers = vec![
        (
            "original_topic".to_string(),
            original_topic.as_bytes().to_vec(),
        ),
        ("error".to_string(), error_message.as_bytes().to_vec()),
    ];

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

    for attempt in 0..retries {
        match handler(&payload).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_error = Some(e);
                // 最終リトライ以外はエクスポネンシャルバックオフで待機する。
                // 一時的な障害（DB 瞬断等）からの自動回復率を高めるため。
                if attempt + 1 < retries {
                    let delay_ms = (RETRY_INITIAL_DELAY_MS * 2u64.pow(attempt))
                        .min(RETRY_MAX_DELAY_MS);
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                }
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
        assert_eq!(original_topic_name("k1s0.service.order.created.v1"), None);
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
    // start_paused = true でエクスポネンシャルバックオフの sleep を即時スキップさせることで実行時間を削減する。
    #[tokio::test(start_paused = true)]
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

    // リトライ回数を 1 に設定した場合にハンドラーが 1 回だけ呼ばれることを確認する。
    #[tokio::test]
    async fn test_process_with_dlq_fallback_single_retry() {
        let producer = NoOpEventProducer;
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let counter = call_count.clone();

        let result = process_with_dlq_fallback(
            &producer,
            "k1s0.service.order.created.v1",
            "order-123",
            b"payload".to_vec(),
            Some(1),
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
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    // 2 回目のリトライで成功する場合に DLQ に転送されないことを確認する。
    // start_paused = true でバックオフ sleep を即時スキップする。
    #[tokio::test(start_paused = true)]
    async fn test_process_with_dlq_fallback_succeeds_on_second_retry() {
        let producer = NoOpEventProducer;
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let counter = call_count.clone();

        let result = process_with_dlq_fallback(
            &producer,
            "k1s0.service.order.created.v1",
            "order-123",
            b"payload".to_vec(),
            Some(3),
            |_payload| {
                let c = counter.clone();
                async move {
                    let count = c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if count == 0 {
                        Err(MessagingError::ConsumerError(
                            "first attempt fail".to_string(),
                        ))
                    } else {
                        Ok(())
                    }
                }
            },
        )
        .await;
        assert!(result.is_ok());
        // 2回目で成功するので呼び出し回数は 2
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    // 空文字列のトピック名でも DLQ トピック名が正しく生成されることを確認する。
    #[test]
    fn test_dlq_topic_name_empty_string() {
        assert_eq!(dlq_topic_name(""), ".dlq");
    }

    // 空文字列に .dlq を付けた DLQ トピックから original_topic_name で空文字列が復元されることを確認する。
    #[test]
    fn test_original_topic_name_from_dlq_suffix_only() {
        assert_eq!(original_topic_name(".dlq"), Some(""));
    }

    // ".dlq" を含むが末尾にない場合に original_topic_name が None を返すことを確認する。
    #[test]
    fn test_original_topic_name_dlq_in_middle() {
        assert_eq!(original_topic_name("my.dlq.topic"), None);
    }

    // max_retries を None にした場合にデフォルト（3回）のリトライが実行されることを確認する。
    // start_paused = true でバックオフ sleep を即時スキップする。
    #[tokio::test(start_paused = true)]
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
