import { v4 as uuidv4 } from 'uuid';

/** アウトボックスメッセージのステータス。 */
export type OutboxStatus = 'PENDING' | 'PROCESSING' | 'DELIVERED' | 'FAILED' | 'DEAD_LETTER';

/** アウトボックス操作のエラーコード。 */
export type OutboxErrorCode = 'STORE_ERROR' | 'PUBLISH_ERROR' | 'SERIALIZATION_ERROR' | 'NOT_FOUND';

/** アウトボックスパターンで管理するメッセージ。 */
export interface OutboxMessage {
  id: string;
  topic: string;
  partitionKey: string;
  payload: string;
  status: OutboxStatus;
  retryCount: number;
  maxRetries: number;
  lastError: string | null;
  createdAt: Date;
  processAfter: Date;
}

/** 新しい OutboxMessage を生成する。 */
export function createOutboxMessage(
  topic: string,
  partitionKey: string,
  payload: string,
): OutboxMessage {
  const now = new Date();
  return {
    id: uuidv4(),
    topic,
    partitionKey,
    payload,
    status: 'PENDING',
    retryCount: 0,
    maxRetries: 3,
    lastError: null,
    createdAt: now,
    processAfter: now,
  };
}

/** メッセージを処理中状態に遷移する。 */
export function markProcessing(msg: OutboxMessage): void {
  msg.status = 'PROCESSING';
}

/** メッセージを配信完了状態に遷移する。 */
export function markDelivered(msg: OutboxMessage): void {
  msg.status = 'DELIVERED';
}

/** メッセージを失敗状態に遷移し、リトライ回数をインクリメントする。 */
export function markFailed(msg: OutboxMessage, error: string): void {
  msg.retryCount += 1;
  msg.lastError = error;
  if (msg.retryCount >= msg.maxRetries) {
    msg.status = 'DEAD_LETTER';
  } else {
    msg.status = 'FAILED';
    // Exponential backoff: 2^retryCount 秒後に再処理
    const delaySecs = Math.pow(2, msg.retryCount);
    msg.processAfter = new Date(Date.now() + delaySecs * 1000);
  }
}

/** メッセージが処理可能かどうか判定する。 */
export function isProcessable(msg: OutboxMessage): boolean {
  return (msg.status === 'PENDING' || msg.status === 'FAILED')
    && msg.processAfter <= new Date();
}

/** 現在のステータスから目的のステータスへ遷移可能かを返す。 */
export function canTransitionTo(from: OutboxStatus, to: OutboxStatus): boolean {
  switch (from) {
    case 'PENDING':
      return to === 'PROCESSING';
    case 'PROCESSING':
      return to === 'DELIVERED' || to === 'FAILED' || to === 'DEAD_LETTER';
    case 'FAILED':
      return to === 'PROCESSING';
    case 'DELIVERED':
      return false;
    case 'DEAD_LETTER':
      return false;
  }
}

/** アウトボックスメッセージの永続化インターフェース。 */
export interface OutboxStore {
  save(msg: OutboxMessage): Promise<void>;
  fetchPending(limit: number): Promise<OutboxMessage[]>;
  update(msg: OutboxMessage): Promise<void>;
  deleteDelivered(olderThanDays: number): Promise<number>;
}

/** メッセージを外部に送信するインターフェース。 */
export interface OutboxPublisher {
  publish(msg: OutboxMessage): Promise<void>;
}

/** アウトボックスメッセージを定期的に処理する。 */
export class OutboxProcessor {
  private readonly batchSize: number;

  constructor(
    private readonly store: OutboxStore,
    private readonly publisher: OutboxPublisher,
    batchSize?: number,
  ) {
    this.batchSize = batchSize !== undefined && batchSize > 0 ? batchSize : 100;
  }

  /** 保留中のメッセージを一括処理する。 */
  async processBatch(): Promise<number> {
    const messages = await this.store.fetchPending(this.batchSize);

    let processed = 0;
    for (const msg of messages) {
      markProcessing(msg);
      await this.store.update(msg);

      try {
        await this.publisher.publish(msg);
        markDelivered(msg);
        await this.store.update(msg);
        processed++;
      } catch (e) {
        const errorMessage = e instanceof Error ? e.message : String(e);
        markFailed(msg, errorMessage);
        await this.store.update(msg);
      }
    }
    return processed;
  }

  /** intervalMs 間隔でバッチ処理を継続実行する。signal でキャンセル可能。 */
  async run(intervalMs: number, signal?: AbortSignal): Promise<void> {
    while (!signal?.aborted) {
      await this.processBatch();
      await new Promise<void>((resolve) => {
        const timer = setTimeout(resolve, intervalMs);
        signal?.addEventListener('abort', () => {
          clearTimeout(timer);
          resolve();
        }, { once: true });
      });
    }
  }
}

/** アウトボックス操作のエラー。 */
export class OutboxError extends Error {
  constructor(
    public readonly code: OutboxErrorCode,
    message?: string,
  ) {
    super(message ? `outbox ${code}: ${message}` : `outbox ${code}`);
    this.name = 'OutboxError';
  }
}
