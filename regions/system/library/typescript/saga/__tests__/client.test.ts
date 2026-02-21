import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';
import { SagaClient, SagaError } from '../src/index.js';

const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

function okResponse(body: unknown) {
  return Promise.resolve({
    ok: true,
    status: 200,
    json: () => Promise.resolve(body),
    text: () => Promise.resolve(JSON.stringify(body)),
  } as Response);
}

function errorResponse(status: number, body: string) {
  return Promise.resolve({
    ok: false,
    status,
    json: () => Promise.reject(new Error('not json')),
    text: () => Promise.resolve(body),
  } as Response);
}

describe('SagaClient', () => {
  let client: SagaClient;

  beforeEach(() => {
    client = new SagaClient('http://localhost:8080');
    mockFetch.mockReset();
  });

  describe('constructor', () => {
    it('末尾スラッシュを除去する', () => {
      const c = new SagaClient('http://localhost:8080/');
      // startSaga でURL検証
      mockFetch.mockReturnValueOnce(okResponse({ saga_id: 'test' }));
      c.startSaga({ workflowName: 'test', payload: {} });
      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:8080/api/v1/sagas',
        expect.any(Object),
      );
    });
  });

  describe('startSaga', () => {
    it('Saga を開始して saga_id を返す', async () => {
      mockFetch.mockReturnValueOnce(okResponse({ saga_id: 'saga-123' }));

      const resp = await client.startSaga({ workflowName: 'order-create', payload: { orderId: '1' } });

      expect(resp.sagaId).toBe('saga-123');
      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:8080/api/v1/sagas',
        expect.objectContaining({ method: 'POST' }),
      );
    });

    it('エラーレスポンスで SagaError を投げる', async () => {
      mockFetch.mockReturnValueOnce(errorResponse(500, 'internal error'));

      await expect(client.startSaga({ workflowName: 'test', payload: {} })).rejects.toThrow(
        SagaError,
      );
    });
  });

  describe('getSaga', () => {
    it('Saga 状態を返す', async () => {
      mockFetch.mockReturnValueOnce(
        okResponse({
          saga: {
            saga_id: 'saga-456',
            workflow_name: 'order-create',
            status: 'RUNNING',
            step_logs: [],
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
        }),
      );

      const state = await client.getSaga('saga-456');
      expect(state.sagaId).toBe('saga-456');
      expect(state.workflowName).toBe('order-create');
      expect(state.status).toBe('RUNNING');
    });

    it('エラーレスポンスで SagaError を投げる', async () => {
      mockFetch.mockReturnValueOnce(errorResponse(404, 'not found'));

      await expect(client.getSaga('unknown')).rejects.toThrow(SagaError);
    });
  });

  describe('cancelSaga', () => {
    it('Saga をキャンセルする', async () => {
      mockFetch.mockReturnValueOnce(okResponse({}));

      await expect(client.cancelSaga('saga-789')).resolves.toBeUndefined();
      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:8080/api/v1/sagas/saga-789/cancel',
        expect.objectContaining({ method: 'POST' }),
      );
    });

    it('エラーレスポンスで SagaError を投げる', async () => {
      mockFetch.mockReturnValueOnce(errorResponse(500, 'error'));

      await expect(client.cancelSaga('saga-789')).rejects.toThrow(SagaError);
    });
  });
});
