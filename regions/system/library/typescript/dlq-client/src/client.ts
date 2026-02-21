import type { DlqMessage, ListDlqMessagesResponse, RetryDlqMessageResponse } from './types.js';
import { DlqError } from './error.js';

/** DLQ 管理サーバーへの REST クライアント。 */
export class DlqClient {
  private readonly endpoint: string;

  constructor(endpoint: string) {
    this.endpoint = endpoint.replace(/\/$/, '');
  }

  /** DLQ メッセージ一覧を取得する。GET /api/v1/dlq/:topic */
  async listMessages(
    topic: string,
    page: number,
    pageSize: number,
  ): Promise<ListDlqMessagesResponse> {
    const url = `${this.endpoint}/api/v1/dlq/${topic}?page=${page}&page_size=${pageSize}`;
    const resp = await fetch(url);

    if (!resp.ok) {
      const text = await resp.text();
      throw new DlqError(`list_messages failed (status ${resp.status}): ${text}`, resp.status);
    }

    const data = (await resp.json()) as {
      messages: Array<{
        id: string;
        original_topic: string;
        error_message: string;
        retry_count: number;
        max_retries: number;
        payload: unknown;
        status: import('./types.js').DlqStatus;
        created_at: string;
        last_retry_at: string | null;
      }>;
      total: number;
      page: number;
    };

    return {
      messages: data.messages.map((m) => ({
        id: m.id,
        originalTopic: m.original_topic,
        errorMessage: m.error_message,
        retryCount: m.retry_count,
        maxRetries: m.max_retries,
        payload: m.payload,
        status: m.status,
        createdAt: m.created_at,
        lastRetryAt: m.last_retry_at,
      })),
      total: data.total,
      page: data.page,
    };
  }

  /** DLQ メッセージの詳細を取得する。GET /api/v1/dlq/messages/:id */
  async getMessage(messageId: string): Promise<DlqMessage> {
    const resp = await fetch(`${this.endpoint}/api/v1/dlq/messages/${messageId}`);

    if (!resp.ok) {
      const text = await resp.text();
      throw new DlqError(`get_message failed (status ${resp.status}): ${text}`, resp.status);
    }

    const m = (await resp.json()) as {
      id: string;
      original_topic: string;
      error_message: string;
      retry_count: number;
      max_retries: number;
      payload: unknown;
      status: import('./types.js').DlqStatus;
      created_at: string;
      last_retry_at: string | null;
    };

    return {
      id: m.id,
      originalTopic: m.original_topic,
      errorMessage: m.error_message,
      retryCount: m.retry_count,
      maxRetries: m.max_retries,
      payload: m.payload,
      status: m.status,
      createdAt: m.created_at,
      lastRetryAt: m.last_retry_at,
    };
  }

  /** DLQ メッセージを再処理する。POST /api/v1/dlq/messages/:id/retry */
  async retryMessage(messageId: string): Promise<RetryDlqMessageResponse> {
    const resp = await fetch(`${this.endpoint}/api/v1/dlq/messages/${messageId}/retry`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: '{}',
    });

    if (!resp.ok) {
      const text = await resp.text();
      throw new DlqError(`retry_message failed (status ${resp.status}): ${text}`, resp.status);
    }

    const data = (await resp.json()) as { message_id: string; status: import('./types.js').DlqStatus };
    return { messageId: data.message_id, status: data.status };
  }

  /** DLQ メッセージを削除する。DELETE /api/v1/dlq/messages/:id */
  async deleteMessage(messageId: string): Promise<void> {
    const resp = await fetch(`${this.endpoint}/api/v1/dlq/messages/${messageId}`, {
      method: 'DELETE',
    });

    if (!resp.ok) {
      const text = await resp.text();
      throw new DlqError(`delete_message failed (status ${resp.status}): ${text}`, resp.status);
    }
  }

  /** トピック内全メッセージを一括再処理する。POST /api/v1/dlq/:topic/retry-all */
  async retryAll(topic: string): Promise<void> {
    const resp = await fetch(`${this.endpoint}/api/v1/dlq/${topic}/retry-all`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: '{}',
    });

    if (!resp.ok) {
      const text = await resp.text();
      throw new DlqError(`retry_all failed (status ${resp.status}): ${text}`, resp.status);
    }
  }
}
