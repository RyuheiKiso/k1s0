//! 補償エンジン。
//!
//! Saga の失敗時に、完了済みステップを逆順で補償する。

use crate::error::{ConsensusError, ConsensusResult};
use crate::saga::{SagaDefinition, SagaStepError};

/// 完了済みステップを逆順で補償する。
///
/// `completed_outputs` は各ステップの出力（インデックスはステップインデックスに対応）。
/// `failed_step_index` は失敗したステップのインデックス（このステップ自体は補償しない）。
///
/// # Errors
///
/// 補償処理が失敗した場合、`ConsensusError::CompensationFailed` を返す。
pub async fn compensate(
    definition: &SagaDefinition,
    saga_id: &str,
    completed_outputs: &[serde_json::Value],
    failed_step_index: usize,
) -> ConsensusResult<()> {
    // 失敗したステップの手前から逆順で補償
    let steps_to_compensate = failed_step_index.min(completed_outputs.len());

    for i in (0..steps_to_compensate).rev() {
        let step = &definition.steps[i].step;
        let output = &completed_outputs[i];

        tracing::info!(
            saga_id,
            step_name = step.name(),
            step_index = i,
            "compensating step"
        );

        match step.compensate(output).await {
            Ok(()) => {
                tracing::info!(
                    saga_id,
                    step_name = step.name(),
                    step_index = i,
                    "step compensated successfully"
                );
            }
            Err(SagaStepError::Retryable(reason) | SagaStepError::NonRetryable(reason)) => {
                tracing::error!(
                    saga_id,
                    step_name = step.name(),
                    step_index = i,
                    error = %reason,
                    "compensation failed"
                );
                return Err(ConsensusError::CompensationFailed {
                    saga_id: saga_id.to_owned(),
                    step_name: step.name().to_owned(),
                    reason,
                });
            }
        }
    }

    Ok(())
}
