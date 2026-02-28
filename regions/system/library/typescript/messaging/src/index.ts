import { v4 as uuidv4 } from 'uuid';

/** イベントのメタデータ。 */
export interface EventMetadata {
  eventId: string;
  eventType: string;
  correlationId: string;
  traceId: string;
  timestamp: string;
  source: string;
  schemaVersion: number;
}

/** 新しい EventMetadata を生成する。 */
export function createEventMetadata(
  eventType: string,
  source: string,
  correlationId?: string,
  traceId?: string,
): EventMetadata {
  return {
    eventId: uuidv4(),
    eventType,
    correlationId: correlationId ?? uuidv4(),
    traceId: traceId ?? uuidv4(),
    timestamp: new Date().toISOString(),
    source,
    schemaVersion: 1,
  };
}

/** イベントのエンベロープ（メタデータ + ペイロード）。 */
export interface EventEnvelope {
  topic: string;
  key: string;
  payload: unknown;
  metadata: EventMetadata;
}

/** イベントを処理するハンドラー関数型。 */
export type EventHandler = (event: EventEnvelope) => Promise<void>;

/** イベントを送信するインターフェース。 */
export interface EventProducer {
  publish(event: EventEnvelope): Promise<void>;
  close(): Promise<void>;
}

/** イベントを受信するインターフェース。 */
export interface EventConsumer {
  subscribe(topic: string, handler: EventHandler): Promise<void>;
  close(): Promise<void>;
}

/** テスト用の no-op EventProducer 実装。 */
export class NoOpEventProducer implements EventProducer {
  published: EventEnvelope[] = [];

  async publish(event: EventEnvelope): Promise<void> {
    this.published.push(event);
  }

  async close(): Promise<void> {
    // no-op
  }
}

/** メッセージング操作のエラー。 */
export class MessagingError extends Error {
  constructor(
    public readonly op: string,
    public readonly cause?: Error,
  ) {
    super(cause ? `${op}: ${cause.message}` : op);
    this.name = 'MessagingError';
  }
}
