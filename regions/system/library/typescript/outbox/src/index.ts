import { v4 as uuidv4 } from 'uuid';

/** アウトボックスメッセージのステータス。 */
export type OutboxStatus = 'PENDING' | 'PROCESSING' | 'DELIVERED' | 'FAILED';

/** アウトボックスパターンで管理するメッセージ。 */
export interface OutboxMessage {
  id: string;
  topic: string;
  eventType: string;
  payload: string;
  status: OutboxStatus;
  retryCount: number;
  scheduledAt: Date;
  createdAt: Date;
  updatedAt: Date;
  correlationId: string;
}

/** 新しい OutboxMessage を生成する。 */
export function createOutboxMessage(
  topic: string,
  eventType: string,
  payload: string,
  correlationId: string,
): OutboxMessage {
  const now = new Date();
  return {
    id: uuidv4(),
    topic,
    eventType,
    payload,
    status: 'PENDING',
    retryCount: 0,
    scheduledAt: now,
    createdAt: now,
    updatedAt: now,
    correlationId,
  };
}

/** 次回処理予定時刻を指数バックオフで計算する。min(2^retryCount, 60) 分後。 */
export function nextScheduledAt(retryCount: number): Date {
  let delayMinutes = 1 << retryCount; // 2^retryCount
  if (delayMinutes > 60) {
    delayMinutes = 60;
  }
  return new Date(Date.now() + delayMinutes * 60 * 1000);
}

/** 現在のステータスから目的のステータスへ遷移可能かを返す。 */
export function canTransitionTo(from: OutboxStatus, to: OutboxStatus): boolean {
  switch (from) {
    case 'PENDING':
      return to === 'PROCESSING';
    case 'PROCESSING':
      return to === 'DELIVERED' || to === 'FAILED';
    case 'FAILED':
      return to === 'PENDING';
    case 'DELIVERED':
      return false;
  }
}

/** アウトボックスメッセージの永続化インターフェース。 */
export interface OutboxStore {
  saveMessage(msg: OutboxMessage): Promise<void>;
  getPendingMessages(limit: number): Promise<OutboxMessage[]>;
  updateStatus(id: string, status: OutboxStatus): Promise<void>;
  updateStatusWithRetry(id: string, status: OutboxStatus, retryCount: number, scheduledAt: Date): Promise<void>;
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
    const messages = await this.store.getPendingMessages(this.batchSize);

    let processed = 0;
    for (const msg of messages) {
      await this.store.updateStatus(msg.id, 'PROCESSING');

      try {
        await this.publisher.publish(msg);
        await this.store.updateStatus(msg.id, 'DELIVERED');
        processed++;
      } catch {
        const retryCount = msg.retryCount + 1;
        const scheduledAt = nextScheduledAt(retryCount);
        await this.store.updateStatusWithRetry(msg.id, 'FAILED', retryCount, scheduledAt);
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
    public readonly op: string,
    cause?: Error,
  ) {
    super(cause ? `outbox ${op}: ${cause.message}` : `outbox ${op}`);
    this.name = 'OutboxError';
  }
}
