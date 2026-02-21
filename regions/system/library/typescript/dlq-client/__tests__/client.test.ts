import { vi, describe, it, expect, beforeEach } from 'vitest';
import { DlqClient, DlqError } from '../src/index.js';

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

describe('DlqClient', () => {
  let client: DlqClient;

  beforeEach(() => {
    client = new DlqClient('http://localhost:8080');
    mockFetch.mockReset();
  });

  describe('listMessages', () => {
    it('DLQ メッセージ一覧を返す', async () => {
      mockFetch.mockReturnValueOnce(
        okResponse({
          messages: [
            {
              id: 'msg-1',
              original_topic: 'orders.v1',
              error_message: 'processing failed',
              retry_count: 1,
              max_retries: 3,
              payload: { order_id: '123' },
              status: 'PENDING',
              created_at: '2024-01-01T00:00:00Z',
              last_retry_at: null,
            },
          ],
          total: 1,
          page: 1,
        }),
      );

      const resp = await client.listMessages('orders.dlq.v1', 1, 20);
      expect(resp.messages).toHaveLength(1);
      expect(resp.messages[0].id).toBe('msg-1');
      expect(resp.messages[0].originalTopic).toBe('orders.v1');
      expect(resp.total).toBe(1);
    });

    it('エラーレスポンスで DlqError を投げる', async () => {
      mockFetch.mockReturnValueOnce(errorResponse(500, 'server error'));
      await expect(client.listMessages('orders.dlq.v1', 1, 20)).rejects.toThrow(DlqError);
    });
  });

  describe('getMessage', () => {
    it('DLQ メッセージ詳細を返す', async () => {
      mockFetch.mockReturnValueOnce(
        okResponse({
          id: 'msg-1',
          original_topic: 'orders.v1',
          error_message: 'error',
          retry_count: 0,
          max_retries: 3,
          payload: {},
          status: 'PENDING',
          created_at: '2024-01-01T00:00:00Z',
          last_retry_at: null,
        }),
      );

      const msg = await client.getMessage('msg-1');
      expect(msg.id).toBe('msg-1');
      expect(msg.status).toBe('PENDING');
    });

    it('エラーレスポンスで DlqError を投げる', async () => {
      mockFetch.mockReturnValueOnce(errorResponse(404, 'not found'));
      await expect(client.getMessage('unknown')).rejects.toThrow(DlqError);
    });
  });

  describe('retryMessage', () => {
    it('再処理レスポンスを返す', async () => {
      mockFetch.mockReturnValueOnce(okResponse({ message_id: 'msg-1', status: 'RETRYING' }));

      const resp = await client.retryMessage('msg-1');
      expect(resp.messageId).toBe('msg-1');
      expect(resp.status).toBe('RETRYING');
    });

    it('エラーレスポンスで DlqError を投げる', async () => {
      mockFetch.mockReturnValueOnce(errorResponse(500, 'error'));
      await expect(client.retryMessage('msg-1')).rejects.toThrow(DlqError);
    });
  });

  describe('deleteMessage', () => {
    it('正常に削除する', async () => {
      mockFetch.mockReturnValueOnce(okResponse({}));
      await expect(client.deleteMessage('msg-1')).resolves.toBeUndefined();
    });

    it('エラーレスポンスで DlqError を投げる', async () => {
      mockFetch.mockReturnValueOnce(errorResponse(404, 'not found'));
      await expect(client.deleteMessage('msg-1')).rejects.toThrow(DlqError);
    });
  });

  describe('retryAll', () => {
    it('一括再処理を実行する', async () => {
      mockFetch.mockReturnValueOnce(okResponse({}));
      await expect(client.retryAll('orders.dlq.v1')).resolves.toBeUndefined();
    });

    it('エラーレスポンスで DlqError を投げる', async () => {
      mockFetch.mockReturnValueOnce(errorResponse(500, 'error'));
      await expect(client.retryAll('orders.dlq.v1')).rejects.toThrow(DlqError);
    });
  });
});
