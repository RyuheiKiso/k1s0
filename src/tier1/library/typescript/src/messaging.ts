/**
 * messaging.ts — k1s0 tier1 Library TypeScript 実装: Messaging / EventBus の L1+ interface
 * 11_メッセージング適合仕様.md §MessagingProducer / §MessagingConsumer（Kafka L1+ 深耕）に準拠する。
 * Kafka の full API を Library 独自語彙で表現しつつ、AuthContext 伝播を強制する。
 * OSS 型（kafkajs / @confluentinc/kafka 等）を公開シグネチャに一切含まない。
 */

/**
 * OutboxMessage は Messaging L1+ (Kafka) の Outbox メッセージを宣言する型。
 * 11_メッセージング適合仕様.md §Outbox Pattern の必須フィールドに準拠する。
 * tenant 分離を保証するために tenantId を必須フィールドとして持つ。
 */
// OutboxMessage 型定義
export interface OutboxMessage {
  // tenantId: メッセージの発行元テナント識別子（必須: partition routing に使用する）
  readonly tenantId: string;
  // topic: Kafka トピック名（"{tenantId}.{事業ドメイン}" 形式を推奨する）
  readonly topic: string;
  // key: Kafka メッセージキー（同一エンティティのメッセージ順序保証に使用する）
  readonly key: string;
  // payload: メッセージボディ（protobuf / JSON バイト列）
  readonly payload: Uint8Array;
  // headers: Kafka メッセージヘッダー（trace_id / auth_class 等の横断属性）
  readonly headers?: Readonly<Record<string, Uint8Array>> | undefined;
  // partitionKey: Kafka パーティションキー（未指定 = key を使用する）
  readonly partitionKey?: string | undefined;
  // schemaId: Schema Registry で登録されたスキーマ ID（0 = スキーマ検証なし）
  readonly schemaId?: bigint | undefined;
}

/**
 * DeliveredMessage は Messaging L1+ (Kafka) で受信したメッセージを宣言する型。
 * Consumer が Kafka から受け取ったメッセージを Library 独自語彙で表現する。
 */
// DeliveredMessage 型定義
export interface DeliveredMessage {
  // tenantId: メッセージの発行元テナント識別子
  readonly tenantId: string;
  // topic: 受信したトピック名
  readonly topic: string;
  // key: メッセージキー
  readonly key: string;
  // payload: メッセージボディ
  readonly payload: Uint8Array;
  // headers: メッセージヘッダー
  readonly headers: Readonly<Record<string, Uint8Array>>;
  // partition: 受信したパーティション番号
  readonly partition: number;
  // offset: 受信したオフセット（bigint: Kafka offset は 64bit）
  readonly offset: bigint;
  // schemaId: Schema Registry のスキーマ ID（0 = 未設定）
  readonly schemaId: bigint;
}

/**
 * MessagingProduceResult は publish の結果を宣言する型。
 * 送信成功したメッセージのパーティションとオフセットを返す（監査ログに使用する）。
 */
// MessagingProduceResult 型定義
export interface MessagingProduceResult {
  // topic: 送信先トピック名
  readonly topic: string;
  // partition: 送信されたパーティション番号
  readonly partition: number;
  // offset: 送信されたオフセット（bigint: Kafka offset は 64bit）
  readonly offset: bigint;
}

/**
 * MessagingProducer は Messaging L1+ (Kafka) の Producer interface を宣言する。
 * Kafka の full API を Library 独自語彙で表現する。
 * ctx に AuthContext が含まれることを強制する（tenant 分離必須）。
 * OSS 型（kafkajs.Producer 等）を一切含まない。
 */
// MessagingProducer インターフェース定義
export interface MessagingProducer {
  /**
   * publish は単一メッセージを Kafka トピックに送信する。
   * msg.tenantId と AuthContext.tenantId の一致を実装側で検証する。
   */
  // publish メソッド: メッセージを送信する
  publish(msg: OutboxMessage): Promise<MessagingProduceResult>;

  /**
   * publishBatch は複数メッセージを一括送信する（Transaction Producer を使用する）。
   * 全メッセージの tenantId が AuthContext.tenantId と一致することを検証する。
   * 1 件でも失敗した場合は全件ロールバックする（Kafka Transaction 保証）。
   */
  // publishBatch メソッド: 複数メッセージを一括送信する
  publishBatch(msgs: readonly OutboxMessage[]): Promise<readonly MessagingProduceResult[]>;

  /**
   * close は Producer をグレースフルにシャットダウンする（未送信メッセージをフラッシュする）。
   */
  // close メソッド: Producer をシャットダウンする
  close(): Promise<void>;
}

/**
 * ConsumerGroupOptions は ConsumerGroup の設定オプションを宣言する型。
 */
// ConsumerGroupOptions 型定義
export interface ConsumerGroupOptions {
  // groupId: Kafka Consumer Group ID（tenant prefix を含む形式を推奨する）
  readonly groupId: string;
  // topics: 購読するトピック配列
  readonly topics: readonly string[];
  // autoCommit: オフセットを自動コミットするかどうか（false = 手動コミット推奨）
  readonly autoCommit?: boolean | undefined;
  // maxPollRecords: 一度のポーリングで取得する最大レコード数
  readonly maxPollRecords?: number | undefined;
  // isolationLevel: Kafka Isolation Level（"read_committed" / "read_uncommitted"）
  readonly isolationLevel?: "read_committed" | "read_uncommitted" | undefined;
}

/**
 * MessagingConsumerHandler は Consumer が受信したメッセージを処理する関数型。
 * 関数が Promise を reject した場合は実装がリトライ / DLQ 送信を行う。
 */
// MessagingConsumerHandler 型定義
export type MessagingConsumerHandler = (msg: DeliveredMessage) => Promise<void>;

/**
 * MessagingConsumer は Messaging L1+ (Kafka) の Consumer interface を宣言する。
 * Kafka Consumer Group の full API を Library 独自語彙で表現する。
 * OSS 型（kafkajs.Consumer 等）を一切含まない。
 */
// MessagingConsumer インターフェース定義
export interface MessagingConsumer {
  /**
   * subscribe はトピック購読を開始して handler を呼び出す。
   * signal がキャンセルされると購読を停止する（MemLeak 防止のため必ず signal を渡す）。
   * handler にはメッセージを処理するコールバックを渡す（reject = リトライ対象）。
   */
  // subscribe メソッド: トピック購読を開始する
  subscribe(handler: MessagingConsumerHandler, signal: AbortSignal): Promise<void>;

  /**
   * commit は指定オフセットを明示的にコミットする（autoCommit=false 時に使用する）。
   */
  // commit メソッド: オフセットをコミットする
  commit(msg: DeliveredMessage): Promise<void>;

  /**
   * seek は指定パーティション・オフセットにカーソルを移動する（リプレイ用途）。
   */
  // seek メソッド: パーティション・オフセットにシークする
  seek(topic: string, partition: number, offset: bigint): Promise<void>;

  /**
   * close は Consumer グループをグレースフルにシャットダウンする。
   */
  // close メソッド: Consumer をシャットダウンする
  close(): Promise<void>;
}

/**
 * OutboxRelay は Outbox Pattern の中継 interface を宣言する。
 * DB Outbox テーブルから Kafka に at-least-once でメッセージを転送する。
 */
// OutboxRelay インターフェース定義
export interface OutboxRelay {
  /**
   * poll は Outbox テーブルから未送信メッセージを取得して Kafka に転送する。
   * signal がキャンセルされると停止する（永続バックグラウンド処理を想定する）。
   */
  // poll メソッド: Outbox メッセージを転送する
  poll(signal: AbortSignal): Promise<void>;

  /**
   * stop はポーリングを停止してグレースフルシャットダウンする。
   */
  // stop メソッド: ポーリングを停止する
  stop(): Promise<void>;
}
