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

    const data = (await resp.json()) as { saga_id: string };
    return { sagaId: data.saga_id };
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
    return {
      sagaId: saga['saga_id'] as string,
      workflowName: saga['workflow_name'] as string,
      status: saga['status'] as import('./types.js').SagaStatus,
      stepLogs: (saga['step_logs'] as import('./types.js').SagaStepLog[]) ?? [],
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
