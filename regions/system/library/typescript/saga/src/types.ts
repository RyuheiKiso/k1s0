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
  id: string;
  sagaId: string;
  stepIndex: number;
  stepName: string;
  action: string;
  status: string;
  requestPayload: unknown;
  responsePayload: unknown;
  errorMessage?: string;
  startedAt: string;
  completedAt?: string;
}

/** Saga の現在状態。 */
export interface SagaState {
  sagaId: string;
  workflowName: string;
  currentStep: number;
  status: SagaStatus;
  payload: Record<string, unknown>;
  correlationId?: string;
  initiatedBy?: string;
  errorMessage?: string;
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
