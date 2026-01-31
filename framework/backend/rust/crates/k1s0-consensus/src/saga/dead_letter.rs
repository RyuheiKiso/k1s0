//! デッドレターキュー。
//!
//! 補償処理にも失敗した Saga をデッドレターとして管理する。

use crate::saga::{SagaInstance, SagaStatus};

/// デッドレターインスタンスを作成する（インメモリ用）。
#[must_use]
pub fn to_dead_letter(
    saga_id: String,
    saga_name: String,
    context: serde_json::Value,
    error_message: String,
) -> SagaInstance {
    let now = chrono::Utc::now();
    SagaInstance {
        saga_id,
        saga_name,
        status: SagaStatus::DeadLetter,
        current_step: -1,
        context,
        error_message: Some(error_message),
        created_at: now,
        updated_at: now,
    }
}

/// デッドレターのフィルタリング。
#[must_use]
pub fn filter_dead_letters(instances: &[SagaInstance]) -> Vec<&SagaInstance> {
    instances
        .iter()
        .filter(|i| i.status == SagaStatus::DeadLetter)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_dead_letter() {
        let dl = to_dead_letter(
            "saga-1".into(),
            "test-saga".into(),
            serde_json::json!({"key": "value"}),
            "compensation failed".into(),
        );
        assert_eq!(dl.status, SagaStatus::DeadLetter);
        assert_eq!(dl.saga_id, "saga-1");
        assert!(dl.error_message.is_some());
    }

    #[test]
    fn test_filter_dead_letters() {
        let now = chrono::Utc::now();
        let instances = vec![
            SagaInstance {
                saga_id: "1".into(),
                saga_name: "s".into(),
                status: SagaStatus::DeadLetter,
                current_step: 0,
                context: serde_json::json!({}),
                error_message: None,
                created_at: now,
                updated_at: now,
            },
            SagaInstance {
                saga_id: "2".into(),
                saga_name: "s".into(),
                status: SagaStatus::Completed,
                current_step: 0,
                context: serde_json::json!({}),
                error_message: None,
                created_at: now,
                updated_at: now,
            },
        ];
        let dead = filter_dead_letters(&instances);
        assert_eq!(dead.len(), 1);
        assert_eq!(dead[0].saga_id, "1");
    }
}
