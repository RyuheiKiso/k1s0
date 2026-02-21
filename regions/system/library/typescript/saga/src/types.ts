/** Saga の実行ステータス。 */
export type SagaStatus =
  | 'STARTED'
  | 'RUNNING'
  | 'COMPLETED'
  | 'COMPENSATING'
  | 'FAILED'
  | 'CANCELLED';

/** Saga ステップのログ。 */
export interface SagaStepLog {
  stepName: string;
  status: string;
  message: string;
  createdAt: string;
}

/** Saga の現在状態。 */
export interface SagaState {
  sagaId: string;
  workflowName: string;
  status: SagaStatus;
  stepLogs: SagaStepLog[];
  createdAt: string;
  updatedAt: string;
}

/** Saga 開始リクエスト。 */
export interface StartSagaRequest {
  workflowName: string;
  payload: unknown;
  correlationId?: string;
  initiatedBy?: string;
}

/** Saga 開始レスポンス。 */
export interface StartSagaResponse {
  sagaId: string;
}
