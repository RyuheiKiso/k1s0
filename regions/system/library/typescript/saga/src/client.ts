import type { SagaState, StartSagaRequest, StartSagaResponse } from './types.js';
import { SagaError } from './error.js';

/** Saga サーバーへの REST クライアント。 */
export class SagaClient {
  private readonly endpoint: string;

  constructor(endpoint: string) {
    this.endpoint = endpoint.replace(/\/$/, '');
  }

  /** Saga を開始する。POST /api/v1/sagas */
  async startSaga(request: StartSagaRequest): Promise<StartSagaResponse> {
    const resp = await fetch(`${this.endpoint}/api/v1/sagas`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        workflow_name: request.workflowName,
        payload: request.payload,
        correlation_id: request.correlationId,
        initiated_by: request.initiatedBy,
      }),
    });

    if (!resp.ok) {
      const text = await resp.text();
      throw new SagaError(`start_saga failed (status ${resp.status}): ${text}`, resp.status);
    }

    // HIGH-002 監査対応: StartSagaResponse の status フィールドを返す。
    // サーバーが status を返さない場合は 'STARTED' をデフォルト値として使用する。
    const data = (await resp.json()) as { saga_id: string; status?: string };
    return {
      sagaId: data.saga_id,
      status: (data.status as import('./types.js').SagaStatus | undefined) ?? 'STARTED',
    };
  }

  /** Saga の状態を取得する。GET /api/v1/sagas/:sagaId */
  async getSaga(sagaId: string): Promise<SagaState> {
    const resp = await fetch(`${this.endpoint}/api/v1/sagas/${sagaId}`);

    if (!resp.ok) {
      const text = await resp.text();
      throw new SagaError(`get_saga failed (status ${resp.status}): ${text}`, resp.status);
    }

    const data = (await resp.json()) as { saga: Record<string, unknown> };
    const saga = data.saga ?? data;
    const stepLogs = ((saga['step_logs'] as Record<string, unknown>[] | undefined) ?? []).map(
      (log) => ({
        id: String(log['id'] ?? ''),
        sagaId: String(log['saga_id'] ?? ''),
        stepIndex: Number(log['step_index'] ?? 0),
        stepName: String(log['step_name'] ?? ''),
        action: String(log['action'] ?? ''),
        status: String(log['status'] ?? ''),
        requestPayload: log['request_payload'],
        responsePayload: log['response_payload'],
        errorMessage:
          log['error_message'] == null ? undefined : String(log['error_message']),
        startedAt: String(log['started_at'] ?? ''),
        completedAt:
          log['completed_at'] == null ? undefined : String(log['completed_at']),
      }),
    );
    return {
      sagaId: saga['saga_id'] as string,
      workflowName: saga['workflow_name'] as string,
      currentStep: Number(saga['current_step'] ?? 0),
      status: saga['status'] as import('./types.js').SagaStatus,
      payload: (saga['payload'] as Record<string, unknown>) ?? {},
      correlationId:
        saga['correlation_id'] == null ? undefined : String(saga['correlation_id']),
      initiatedBy:
        saga['initiated_by'] == null ? undefined : String(saga['initiated_by']),
      errorMessage:
        saga['error_message'] == null ? undefined : String(saga['error_message']),
      stepLogs,
      createdAt: saga['created_at'] as string,
      updatedAt: saga['updated_at'] as string,
    };
  }

  /** Saga をキャンセルする。POST /api/v1/sagas/:sagaId/cancel */
  async cancelSaga(sagaId: string): Promise<void> {
    const resp = await fetch(`${this.endpoint}/api/v1/sagas/${sagaId}/cancel`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: '{}',
    });

    if (!resp.ok) {
      const text = await resp.text();
      throw new SagaError(`cancel_saga failed (status ${resp.status}): ${text}`, resp.status);
    }
  }
}
