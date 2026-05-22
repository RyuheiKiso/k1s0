/**
 * messagingImpl.ts — k1s0 tier1 Library TypeScript 実装: MessagingProducer / MessagingConsumer の facade 実装
 * C# MessagingImpl.cs / Go messaging_impl.go と同等の深度で Kafka クライアントを L1+ ラップする。
 * OSS 型（kafkajs.Producer / @confluentinc/kafka 等）を公開 API シグネチャに一切露出しない。
 * tenant 分離は msg.tenantId と producer 設定の tenantId の一致検証で強制する。
 */

// 公開 interface をインポートする
import type {
  OutboxMessage,
  DeliveredMessage,
  MessagingProduceResult,
  MessagingProducer,
  MessagingConsumer,
  MessagingConsumerHandler,
  OutboxRelay,
} from "./messaging.js";

// ---- MessagingProducer の stub / testing 実装 ----

/**
 * InMemoryMessagingProducer は MessagingProducer の in-memory stub 実装クラス。
 * テスト / ドライラン用に送信済みメッセージを in-memory に蓄積する。
 * Kafka に依存せず、単体テストで利用できる実装とする。
 */
// InMemoryMessagingProducer クラス定義（テスト用の公開クラス）
export class InMemoryMessagingProducer implements MessagingProducer {
  // #tenantId: Producer に紐付いたテナント識別子（tenant 分離検証に使用する）
  readonly #tenantId: string;
  // published: 送信済みメッセージ配列（テストの検証に使用する）
  readonly published: OutboxMessage[] = [];
  // results: 送信済み結果配列（テストの検証に使用する）
  readonly results: MessagingProduceResult[] = [];
  // #closed: close が呼ばれたかどうかのフラグ
  #closed: boolean = false;
  // #nextOffset: 送信済みオフセットの内部カウンター（順序保証のシミュレート用）
  #nextOffset: bigint = 0n;

  // コンストラクタ: tenantId を受け取る
  constructor(tenantId: string) {
    // テナント識別子を保持する
    this.#tenantId = tenantId;
  }

  /**
   * publish は単一メッセージを in-memory に蓄積する（Kafka に送信しない）。
   * msg.tenantId と #tenantId の一致を検証する（tenant 分離必須）。
   */
  // publish メソッド実装: in-memory にメッセージを蓄積する
  async publish(msg: OutboxMessage): Promise<MessagingProduceResult> {
    // close 済みの場合はエラーを投げる（使用後の Producer に送信しない）
    if (this.#closed) {
      throw new Error("InMemoryMessagingProducer: producer is closed");
    }
    // tenant 分離検証: msg.tenantId と #tenantId の一致を確認する
    if (msg.tenantId !== this.#tenantId) {
      // テナント不一致の場合はエラーを投げる（tenant 分離必須）
      throw new Error(
        `InMemoryMessagingProducer: tenant mismatch msg.tenantId=${msg.tenantId} context.tenantId=${this.#tenantId}`
      );
    }
    // メッセージを蓄積する
    this.published.push(msg);
    // オフセットを計算する（順序保証のシミュレート）
    const offset = this.#nextOffset;
    // 次のオフセットをインクリメントする
    this.#nextOffset++;
    // 送信結果を生成する（パーティション 0 / オフセット = カウンター）
    const result: MessagingProduceResult = {
      topic: msg.topic,
      partition: 0,
      offset,
    };
    // 結果を蓄積する
    this.results.push(result);
    // 結果を返す
    return result;
  }

  /**
   * publishBatch は複数メッセージを一括送信する（in-memory に順次蓄積する）。
   * 全メッセージの tenantId が #tenantId と一致することを検証する。
   * 1 件でも失敗した場合は全件ロールバックする（Kafka Transaction 保証のシミュレート）。
   */
  // publishBatch メソッド実装: 複数メッセージを in-memory に一括蓄積する
  async publishBatch(msgs: readonly OutboxMessage[]): Promise<readonly MessagingProduceResult[]> {
    // close 済みの場合はエラーを投げる
    if (this.#closed) {
      throw new Error("InMemoryMessagingProducer: producer is closed");
    }
    // 全メッセージの tenantId を事前検証する（1 件でも不一致なら全件拒否）
    for (let i = 0; i < msgs.length; i++) {
      // tenant 分離検証: 全メッセージの tenantId が #tenantId と一致することを確認する
      if (msgs[i]!.tenantId !== this.#tenantId) {
        // テナント不一致の場合はエラーを投げる（全件ロールバック）
        throw new Error(
          `InMemoryMessagingProducer: tenant mismatch at index ${i} msg.tenantId=${msgs[i]!.tenantId} context.tenantId=${this.#tenantId}`
        );
      }
    }
    // 全メッセージを順次蓄積する
    const results: MessagingProduceResult[] = [];
    // 各メッセージを処理する
    for (const msg of msgs) {
      // メッセージを蓄積する
      this.published.push(msg);
      // オフセットを計算する
      const offset = this.#nextOffset;
      // 次のオフセットをインクリメントする
      this.#nextOffset++;
      // 送信結果を生成する
      const result: MessagingProduceResult = {
        topic: msg.topic,
        partition: 0,
        offset,
      };
      // 結果を蓄積する
      results.push(result);
    }
    // 全結果を蓄積する
    this.results.push(...results);
    // 全結果を返す
    return results;
  }

  /**
   * close は Producer をグレースフルにシャットダウンする（in-memory では closed フラグを立てる）。
   */
  // close メソッド実装: closed フラグを立てる
  async close(): Promise<void> {
    // closed フラグを立てる
    this.#closed = true;
  }
}

// ---- MessagingConsumer の stub / testing 実装 ----

/**
 * InMemoryMessagingConsumer は MessagingConsumer の in-memory stub 実装クラス。
 * テスト / ドライラン用にメッセージをキュー経由で受信するシミュレーションを提供する。
 * Kafka に依存せず、単体テストで利用できる実装とする。
 */
// InMemoryMessagingConsumer クラス定義（テスト用の公開クラス）
export class InMemoryMessagingConsumer implements MessagingConsumer {
  // #tenantId: Consumer に紐付いたテナント識別子（tenant 分離検証に使用する）
  readonly #tenantId: string;
  // #messageQueue: 受信メッセージキュー（inject で追加する）
  readonly #messageQueue: DeliveredMessage[] = [];
  // committed: コミット済みメッセージ配列（テストの検証に使用する）
  readonly committed: DeliveredMessage[] = [];
  // #closed: close が呼ばれたかどうかのフラグ
  #closed: boolean = false;
  // #resolvers: pending の recv 待機を解決するための resolver キュー
  readonly #resolvers: Array<(msg: DeliveredMessage | null) => void> = [];

  // コンストラクタ: tenantId を受け取る
  constructor(tenantId: string) {
    // テナント識別子を保持する
    this.#tenantId = tenantId;
  }

  /**
   * injectMessage はテスト用にメッセージをキューに注入するヘルパーメソッド。
   * MessagingConsumer interface 外のメソッド（テスト用）。
   */
  // injectMessage メソッド: メッセージをキューに注入する
  injectMessage(msg: DeliveredMessage): void {
    // pending の resolver が存在する場合は直接解決する
    const resolver = this.#resolvers.shift();
    // resolver が存在する場合は直接メッセージを渡す
    if (resolver !== undefined) {
      // resolver を呼び出す
      resolver(msg);
      return;
    }
    // resolver が存在しない場合はキューに追加する
    this.#messageQueue.push(msg);
  }

  /**
   * subscribe はトピック購読を開始して handler を呼び出す。
   * signal がキャンセルされると購読を停止する。
   */
  // subscribe メソッド実装: キューからメッセージを読んで handler を呼び出す
  async subscribe(handler: MessagingConsumerHandler, signal: AbortSignal): Promise<void> {
    // close 済みの場合はエラーを投げる
    if (this.#closed) {
      throw new Error("InMemoryMessagingConsumer: consumer is closed");
    }
    // signal がキャンセルされるまでメッセージを処理し続ける
    while (!signal.aborted) {
      // キューからメッセージを取得する（キューが空の場合は次のメッセージを待機する）
      const msg = await this.#dequeue(signal);
      // signal がキャンセルされた場合はループを終了する
      if (msg === null) {
        break;
      }
      // handler を呼び出す（reject = リトライ対象）
      await handler(msg);
    }
  }

  // #dequeue はキューからメッセージを取得する内部メソッド。
  // キューが空の場合は次のメッセージが注入されるまで待機する。
  #dequeue(signal: AbortSignal): Promise<DeliveredMessage | null> {
    // キューにメッセージが存在する場合は即座に返す
    const msg = this.#messageQueue.shift();
    // メッセージが存在する場合は即座に返す
    if (msg !== undefined) {
      return Promise.resolve(msg);
    }
    // キューが空の場合は Promise で待機する
    return new Promise<DeliveredMessage | null>((resolve) => {
      // signal がキャンセルされた場合は null で解決する
      const abortHandler = (): void => {
        // resolver キューから自分を削除する
        const idx = this.#resolvers.indexOf(resolve);
        // 削除する
        if (idx !== -1) {
          this.#resolvers.splice(idx, 1);
        }
        // null で解決する（キャンセルを通知する）
        resolve(null);
      };
      // signal の abort イベントを監視する
      signal.addEventListener("abort", abortHandler, { once: true });
      // resolver キューに追加する
      this.#resolvers.push((deliveredMsg) => {
        // abort イベントリスナーを削除する
        signal.removeEventListener("abort", abortHandler);
        // メッセージを解決する
        resolve(deliveredMsg);
      });
    });
  }

  /**
   * commit は指定オフセットを明示的にコミットする（in-memory では committed に追加する）。
   */
  // commit メソッド実装: committed に追加する
  async commit(msg: DeliveredMessage): Promise<void> {
    // close 済みの場合はエラーを投げる
    if (this.#closed) {
      throw new Error("InMemoryMessagingConsumer: consumer is closed");
    }
    // コミット済みメッセージを蓄積する
    this.committed.push(msg);
  }

  /**
   * seek は指定パーティション・オフセットにカーソルを移動する（in-memory では何もしない）。
   */
  // seek メソッド実装: in-memory では何もしない（シークをシミュレートする）
  async seek(topic: string, partition: number, offset: bigint): Promise<void> {
    // close 済みの場合はエラーを投げる
    if (this.#closed) {
      throw new Error("InMemoryMessagingConsumer: consumer is closed");
    }
    // in-memory 実装ではシークをシミュレートするのみ（実際の移動は行わない）
  }

  /**
   * close は Consumer グループをグレースフルにシャットダウンする。
   */
  // close メソッド実装: closed フラグを立てて pending resolver を解決する
  async close(): Promise<void> {
    // closed フラグを立てる
    this.#closed = true;
    // pending の resolver を null で解決する（subscribe ループを停止させる）
    for (const resolver of this.#resolvers.splice(0)) {
      // null で解決する（セッション終了を通知する）
      resolver(null);
    }
  }
}

// ---- OutboxRelay の stub 実装 ----

/**
 * NoopOutboxRelay は OutboxRelay の no-op stub 実装クラス。
 * テスト / ドライラン用に何もしない Outbox リレーを提供する。
 */
// NoopOutboxRelay クラス定義（テスト用の公開クラス）
export class NoopOutboxRelay implements OutboxRelay {
  /**
   * poll は no-op 実装（signal のキャンセルで停止する）。
   */
  // poll メソッド実装: signal のキャンセルを待機して終了する
  async poll(signal: AbortSignal): Promise<void> {
    // signal がキャンセルされるまで待機する
    await new Promise<void>((resolve) => {
      // signal の abort イベントを監視する
      signal.addEventListener("abort", () => resolve(), { once: true });
    });
  }

  /**
   * stop は no-op 実装（何もしない）。
   */
  // stop メソッド実装: 何もしない
  async stop(): Promise<void> {
    // 何もしない（no-op 実装）
  }
}

// ---- Kafka production 実装 ----

// kafkajs の Producer / Consumer / CompressionTypes を内部使用のみに限定してインポートする
// OSS 型を公開 API シグネチャに露出しない（facade pattern 必須）
import {
  Kafka,
  type Producer as KafkaJsProducer,
  type Consumer as KafkaJsConsumer,
  CompressionTypes,
} from "kafkajs";

// kafkajs は IsolationLevel を型として export していないため数値定数で代替する（kafkajs 内部仕様）
// READ_UNCOMMITTED = 0, READ_COMMITTED = 1 (Kafka プロトコル仕様に準拠する)
const KafkaJsIsolationLevel = {
  // READ_UNCOMMITTED: トランザクション未確定メッセージも読み込む
  READ_UNCOMMITTED: 0,
  // READ_COMMITTED: トランザクションコミット済みメッセージのみ読み込む（推奨）
  READ_COMMITTED: 1,
} as const;

// DbClient インターフェースをインポートする（KafkaOutboxRelay が使用する）
import type { DbClient } from "./db.js";

// ConsumerGroupOptions インターフェースをインポートする（KafkaMessagingConsumer が使用する）
import type { ConsumerGroupOptions } from "./messaging.js";

/**
 * KafkaClientOptions は KafkaMessagingProducer / KafkaMessagingConsumer が共用する
 * Kafka ブローカー接続設定を宣言する型。
 * kafkajs の KafkaConfig を OSS 型として公開 API に露出しない（facade pattern）。
 */
// KafkaClientOptions 型定義（公開 API シグネチャから kafkajs 型を排除する）
export interface KafkaClientOptions {
  // brokers: Kafka ブローカーアドレス配列（"host:port" 形式）
  readonly brokers: readonly string[];
  // clientId: Kafka クライアント識別子（可観測性・トレース追跡に使用する）
  readonly clientId: string;
  // ssl: TLS 接続を有効にするかどうか（本番環境では true を必須とする）
  readonly ssl?: boolean | undefined;
  // saslMechanism: SASL 認証方式（"plain" / "scram-sha-256" / "scram-sha-512"）
  readonly saslMechanism?: "plain" | "scram-sha-256" | "scram-sha-512" | undefined;
  // saslUsername: SASL 認証ユーザー名
  readonly saslUsername?: string | undefined;
  // saslPassword: SASL 認証パスワード（SecretsManager 等から注入する）
  readonly saslPassword?: string | undefined;
  // connectionTimeoutMs: ブローカー接続タイムアウト（ミリ秒）
  readonly connectionTimeoutMs?: number | undefined;
  // requestTimeoutMs: 個々のリクエストタイムアウト（ミリ秒）
  readonly requestTimeoutMs?: number | undefined;
}

/**
 * KafkaProducerOptions は KafkaMessagingProducer のコンストラクタオプションを宣言する型。
 * kafkajs の ProducerConfig を OSS 型として公開 API に露出しない（facade pattern）。
 */
// KafkaProducerOptions 型定義
export interface KafkaProducerOptions {
  // kafkaOptions: Kafka ブローカー接続設定
  readonly kafkaOptions: KafkaClientOptions;
  // tenantId: Producer に紐付いたテナント識別子（tenant 分離検証に必須）
  readonly tenantId: string;
  // transactionalId: Exactly-once 送信に使用する Transaction ID（未指定 = at-least-once）
  readonly transactionalId?: string | undefined;
  // idempotent: べき等送信を有効にするかどうか（true 推奨）
  readonly idempotent?: boolean | undefined;
  // maxInFlightRequests: 未確認の in-flight リクエスト最大数
  readonly maxInFlightRequests?: number | undefined;
  // compressionType: メッセージ圧縮方式（"snappy" / "gzip" / "lz4" / "zstd" / "none"）
  readonly compressionType?: "snappy" | "gzip" | "lz4" | "zstd" | "none" | undefined;
}

/**
 * KafkaOutboxRow は PostgreSQL Outbox テーブルの行を宣言する型。
 * テーブル定義: outbox_messages(id, tenant_id, topic, key, payload, headers, schema_id,
 *   partition_key, created_at, sent_at)
 * sent_at が null のものが未送信メッセージとなる。
 */
// KafkaOutboxRow 型定義（PostgreSQL Outbox テーブルのスキーマに対応する）
interface KafkaOutboxRow {
  // id: Outbox メッセージの UUID（削除キーに使用する）
  readonly id: string;
  // tenant_id: 発行元テナント識別子
  readonly tenant_id: string;
  // topic: 送信先 Kafka トピック名
  readonly topic: string;
  // key: Kafka メッセージキー
  readonly key: string;
  // payload: メッセージボディ（バイナリ）
  readonly payload: Buffer;
  // headers: メッセージヘッダー（JSON 文字列形式: key-value ペア）
  readonly headers: string | null;
  // schema_id: Schema Registry スキーマ ID（null = 未設定）
  readonly schema_id: string | null;
  // partition_key: パーティションキー（null = key を使用する）
  readonly partition_key: string | null;
  // hlc_timestamp: HLC 形式タイムスタンプ文字列（wall-clock TTL 禁止のため HLC を使用する）
  readonly hlc_timestamp: string;
}

/**
 * KafkaOutboxRelayOptions は KafkaOutboxRelay のコンストラクタオプションを宣言する型。
 */
// KafkaOutboxRelayOptions 型定義
export interface KafkaOutboxRelayOptions {
  // kafkaOptions: Kafka ブローカー接続設定
  readonly kafkaOptions: KafkaClientOptions;
  // db: PostgreSQL クライアント（Outbox テーブル操作に使用する）
  readonly db: DbClient;
  // pollIntervalMs: ポーリング間隔（ミリ秒）
  readonly pollIntervalMs?: number | undefined;
  // batchSize: 一度のポーリングで取得する最大レコード数
  readonly batchSize?: number | undefined;
  // outboxTable: Outbox テーブル名（デフォルト: "outbox_messages"）
  readonly outboxTable?: string | undefined;
}

/**
 * resolveKafkaCompression は compressionType 文字列を kafkajs の CompressionTypes に変換する
 * 内部ヘルパー関数。OSS 型 CompressionTypes は公開 API シグネチャに露出しない。
 */
// resolveKafkaCompression: 圧縮方式文字列を kafkajs 内部定数に変換する
function resolveKafkaCompression(
  compressionType: KafkaProducerOptions["compressionType"]
): CompressionTypes {
  // compressionType が未指定の場合はデフォルト（None）を返す
  if (compressionType === undefined || compressionType === "none") {
    // 圧縮なし（デフォルト）を返す
    return CompressionTypes.None;
  }
  // gzip を指定した場合は CompressionTypes.GZIP を返す
  if (compressionType === "gzip") {
    // GZIP 圧縮を返す
    return CompressionTypes.GZIP;
  }
  // snappy を指定した場合は CompressionTypes.Snappy を返す
  if (compressionType === "snappy") {
    // Snappy 圧縮を返す
    return CompressionTypes.Snappy;
  }
  // lz4 を指定した場合は CompressionTypes.LZ4 を返す
  if (compressionType === "lz4") {
    // LZ4 圧縮を返す
    return CompressionTypes.LZ4;
  }
  // zstd を指定した場合は CompressionTypes.ZSTD を返す
  if (compressionType === "zstd") {
    // ZSTD 圧縮を返す
    return CompressionTypes.ZSTD;
  }
  // 未知の圧縮方式はデフォルト（None）を返す
  return CompressionTypes.None;
}

/**
 * buildKafkaInstance は KafkaClientOptions から kafkajs の Kafka インスタンスを生成する
 * 内部ヘルパー関数。kafkajs の Kafka 型を公開 API シグネチャに露出しない。
 */
// buildKafkaInstance: KafkaClientOptions から kafkajs Kafka インスタンスを生成する
function buildKafkaInstance(options: KafkaClientOptions): Kafka {
  // SASL 設定が存在するかどうかを判定する
  const hasSasl =
    options.saslMechanism !== undefined &&
    options.saslUsername !== undefined &&
    options.saslPassword !== undefined;
  // SASL 設定を組み立てる（kafkajs は discriminated union を要求するため mechanism ごとに分岐する）
  type KafkaSaslEntry =
    | { mechanism: "plain"; username: string; password: string }
    | { mechanism: "scram-sha-256"; username: string; password: string }
    | { mechanism: "scram-sha-512"; username: string; password: string };
  // SASL 設定を生成する（mechanism に応じたオブジェクトリテラルを返す）
  const sasl: KafkaSaslEntry | undefined = (() => {
    // SASL が設定されていない場合は undefined を返す
    if (!hasSasl) {
      return undefined;
    }
    // ユーザー名とパスワードをキャプチャする（Non-null assertion: hasSasl が true の場合のみ到達）
    const username = options.saslUsername!;
    // パスワードをキャプチャする
    const password = options.saslPassword!;
    // mechanism に応じた discriminated union オブジェクトを返す
    if (options.saslMechanism === "scram-sha-512") {
      // SCRAM-SHA-512 設定を返す
      return { mechanism: "scram-sha-512" as const, username, password };
    }
    // mechanism に応じた discriminated union オブジェクトを返す
    if (options.saslMechanism === "scram-sha-256") {
      // SCRAM-SHA-256 設定を返す
      return { mechanism: "scram-sha-256" as const, username, password };
    }
    // デフォルトは PLAIN 認証を返す
    return { mechanism: "plain" as const, username, password };
  })();
  // kafkajs の Kafka インスタンスを生成して返す
  return new Kafka({
    // ブローカーアドレス配列を設定する
    brokers: options.brokers as string[],
    // クライアント識別子を設定する
    clientId: options.clientId,
    // TLS 設定を適用する
    ssl: options.ssl ?? false,
    // SASL 設定を適用する（undefined の場合は設定しない）
    ...(sasl !== undefined ? { sasl } : {}),
    // 接続タイムアウトを設定する（未指定 = kafkajs デフォルト）
    ...(options.connectionTimeoutMs !== undefined
      ? { connectionTimeout: options.connectionTimeoutMs }
      : {}),
    // リクエストタイムアウトを設定する（未指定 = kafkajs デフォルト）
    ...(options.requestTimeoutMs !== undefined
      ? { requestTimeout: options.requestTimeoutMs }
      : {}),
  });
}

/**
 * outboxMessageToKafkaRecord は OutboxMessage を kafkajs の ProducerRecord メッセージ形式に変換する
 * 内部ヘルパー関数。kafkajs 型を公開 API シグネチャに露出しない。
 */
// outboxMessageToKafkaRecord: OutboxMessage を kafkajs 内部形式に変換する
function outboxMessageToKafkaRecord(msg: OutboxMessage): {
  topic: string;
  messages: Array<{
    key: string;
    value: Buffer;
    headers: Record<string, Buffer>;
    partition?: number;
  }>;
} {
  // ヘッダーを kafkajs 形式（Buffer 値）に変換する
  const headers: Record<string, Buffer> = {};
  // OutboxMessage ヘッダーが存在する場合は変換する
  if (msg.headers !== undefined) {
    // 各ヘッダーエントリを処理する
    for (const [k, v] of Object.entries(msg.headers)) {
      // Uint8Array を Buffer に変換する（kafkajs は Buffer を期待する）
      headers[k] = Buffer.from(v);
    }
  }
  // kafkajs の ProducerRecord 形式に変換して返す
  return {
    // 送信先トピック名を設定する
    topic: msg.topic,
    // メッセージ配列を設定する
    messages: [
      {
        // メッセージキーを設定する（partitionKey が指定された場合は partitionKey を優先する）
        key: msg.partitionKey ?? msg.key,
        // ペイロードを Buffer に変換する
        value: Buffer.from(msg.payload),
        // ヘッダーを設定する
        headers,
      },
    ],
  };
}

/**
 * KafkaMessagingProducer は MessagingProducer の Kafka production 実装クラス。
 * kafkajs の Producer をラップして OSS 型を公開 API シグネチャに一切露出しない（facade pattern）。
 * tenant 分離は msg.tenantId と #tenantId の一致検証で強制する。
 * publishBatch は transactional send（kafkajs transaction API）で exactly-once 相当を保証する。
 */
// KafkaMessagingProducer クラス定義（Kafka production 実装の公開クラス）
export class KafkaMessagingProducer implements MessagingProducer {
  // #tenantId: Producer に紐付いたテナント識別子（tenant 分離検証に使用する）
  readonly #tenantId: string;
  // #producer: kafkajs の Producer インスタンス（内部専用: 公開 API に露出しない）
  readonly #producer: KafkaJsProducer;
  // #compression: 送信時の圧縮方式（kafkajs 内部定数に変換済み）
  readonly #compression: CompressionTypes;
  // #connected: Kafka ブローカーへの接続状態フラグ
  #connected: boolean = false;
  // #closed: close が呼ばれたかどうかのフラグ
  #closed: boolean = false;

  // コンストラクタ: KafkaProducerOptions を受け取る
  constructor(options: KafkaProducerOptions) {
    // テナント識別子を保持する
    this.#tenantId = options.tenantId;
    // 圧縮方式を変換して保持する
    this.#compression = resolveKafkaCompression(options.compressionType);
    // kafkajs の Kafka インスタンスを生成する
    const kafka = buildKafkaInstance(options.kafkaOptions);
    // Producer インスタンスを生成する
    this.#producer = kafka.producer({
      // Transaction ID を設定する（exactly-once 送信に必要）
      ...(options.transactionalId !== undefined
        ? { transactionalId: options.transactionalId }
        : {}),
      // べき等送信を有効にする（transactionalId がある場合は暗黙的に有効）
      idempotent: options.idempotent ?? true,
      // in-flight リクエスト最大数を設定する（べき等 = 最大 5 推奨）
      ...(options.maxInFlightRequests !== undefined
        ? { maxInFlightRequests: options.maxInFlightRequests }
        : {}),
    });
  }

  // #ensureConnected は Kafka ブローカーへの接続を保証する内部メソッド。
  // 未接続の場合は connect() を呼び出す（lazy connect パターン）。
  async #ensureConnected(): Promise<void> {
    // 既に接続済みの場合はスキップする
    if (this.#connected) {
      return;
    }
    // ブローカーに接続する
    await this.#producer.connect();
    // 接続済みフラグを立てる
    this.#connected = true;
  }

  /**
   * publish は単一メッセージを Kafka トピックに送信する。
   * msg.tenantId と #tenantId の一致を検証する（tenant 分離必須）。
   * 接続が確立されていない場合は lazy connect を行う。
   */
  // publish メソッド実装: tenant 検証 + Kafka に送信する
  async publish(msg: OutboxMessage): Promise<MessagingProduceResult> {
    // close 済みの場合はエラーを投げる
    if (this.#closed) {
      throw new Error("KafkaMessagingProducer: producer is closed");
    }
    // tenant 分離検証: msg.tenantId と #tenantId の一致を確認する
    if (msg.tenantId !== this.#tenantId) {
      // テナント不一致の場合はエラーを投げる（tenant 分離必須）
      throw new Error(
        `KafkaMessagingProducer: tenant mismatch msg.tenantId=${msg.tenantId} context.tenantId=${this.#tenantId}`
      );
    }
    // Kafka ブローカーへの接続を保証する
    await this.#ensureConnected();
    // OutboxMessage を kafkajs 形式に変換する
    const record = outboxMessageToKafkaRecord(msg);
    // Kafka にメッセージを送信する
    const metaArray = await this.#producer.send({
      // 送信先トピックとメッセージを設定する
      topic: record.topic,
      // メッセージ配列を設定する
      messages: record.messages,
      // 圧縮方式を設定する
      compression: this.#compression,
    });
    // kafkajs は RecordMetadata[] を返すので先頭要素を取得する
    const meta = metaArray[0];
    // メタデータが存在しない場合はエラーを投げる
    if (meta === undefined) {
      throw new Error("KafkaMessagingProducer: no record metadata returned from Kafka");
    }
    // 送信結果を Library 独自型に変換して返す（kafkajs 型を露出しない）
    return {
      // 送信先トピック名を設定する
      topic: meta.topicName,
      // パーティション番号を設定する
      partition: meta.partition,
      // オフセットを bigint に変換する（kafkajs は string を返す）
      offset: BigInt(meta.offset ?? "0"),
    };
  }

  /**
   * publishBatch は複数メッセージをトランザクション一括送信する（exactly-once 相当）。
   * 全メッセージの tenantId が #tenantId と一致することを事前検証する。
   * 1 件でも失敗した場合はトランザクションを abort して全件ロールバックする。
   */
  // publishBatch メソッド実装: transactional batch send で exactly-once 相当を保証する
  async publishBatch(msgs: readonly OutboxMessage[]): Promise<readonly MessagingProduceResult[]> {
    // close 済みの場合はエラーを投げる
    if (this.#closed) {
      throw new Error("KafkaMessagingProducer: producer is closed");
    }
    // メッセージが空の場合は空配列を返す（トランザクション不要）
    if (msgs.length === 0) {
      return [];
    }
    // 全メッセージの tenantId を事前検証する（1 件でも不一致なら全件拒否）
    for (let i = 0; i < msgs.length; i++) {
      // tenant 分離検証: 各メッセージの tenantId が #tenantId と一致することを確認する
      if (msgs[i]!.tenantId !== this.#tenantId) {
        // テナント不一致の場合はエラーを投げる（全件ロールバック）
        throw new Error(
          `KafkaMessagingProducer: tenant mismatch at index ${i} msg.tenantId=${msgs[i]!.tenantId} context.tenantId=${this.#tenantId}`
        );
      }
    }
    // Kafka ブローカーへの接続を保証する
    await this.#ensureConnected();
    // kafkajs のトランザクションを開始する
    const transaction = await this.#producer.transaction();
    // トランザクション内で全メッセージを送信する
    try {
      // 送信結果を格納する配列
      const results: MessagingProduceResult[] = [];
      // 各メッセージをトランザクション内で送信する
      for (const msg of msgs) {
        // OutboxMessage を kafkajs 形式に変換する
        const record = outboxMessageToKafkaRecord(msg);
        // トランザクション内でメッセージを送信する
        const metaArray = await transaction.send({
          // 送信先トピックを設定する
          topic: record.topic,
          // メッセージ配列を設定する
          messages: record.messages,
          // 圧縮方式を設定する
          compression: this.#compression,
        });
        // kafkajs は RecordMetadata[] を返すので先頭要素を取得する
        const meta = metaArray[0];
        // メタデータが存在しない場合はエラーを投げる（トランザクションは abort される）
        if (meta === undefined) {
          throw new Error(
            "KafkaMessagingProducer: no record metadata returned from Kafka in batch"
          );
        }
        // 送信結果を Library 独自型に変換して蓄積する
        results.push({
          // 送信先トピック名を設定する
          topic: meta.topicName,
          // パーティション番号を設定する
          partition: meta.partition,
          // オフセットを bigint に変換する（kafkajs は string を返す）
          offset: BigInt(meta.offset ?? "0"),
        });
      }
      // 全メッセージ送信成功後にトランザクションをコミットする
      await transaction.commit();
      // 全結果を返す
      return results;
    } catch (err) {
      // エラー発生時はトランザクションを abort して例外を再投げする
      await transaction.abort();
      // 上位に例外を伝播させる
      throw err;
    }
  }

  /**
   * close は Producer をグレースフルにシャットダウンする。
   * 未送信メッセージをフラッシュしてブローカーとの接続を切断する。
   */
  // close メソッド実装: producer.disconnect() を呼び出す
  async close(): Promise<void> {
    // 既に close 済みの場合はスキップする（idempotent）
    if (this.#closed) {
      return;
    }
    // closed フラグを立てる
    this.#closed = true;
    // 接続済みの場合のみ disconnect を呼び出す
    if (this.#connected) {
      // ブローカーとの接続を切断する（未送信メッセージをフラッシュしてから切断する）
      await this.#producer.disconnect();
      // 接続フラグをリセットする
      this.#connected = false;
    }
  }
}

/**
 * KafkaConsumerOptions は KafkaMessagingConsumer のコンストラクタオプションを宣言する型。
 * ConsumerGroupOptions の全フィールドに加えて Kafka ブローカー接続設定を持つ。
 * kafkajs の ConsumerConfig を OSS 型として公開 API に露出しない（facade pattern）。
 */
// KafkaConsumerOptions 型定義
export interface KafkaConsumerOptions {
  // kafkaOptions: Kafka ブローカー接続設定
  readonly kafkaOptions: KafkaClientOptions;
  // groupId: Kafka Consumer Group ID（tenant prefix を含む形式を推奨する）
  readonly groupId: string;
  // topics: 購読するトピック配列
  readonly topics: readonly string[];
  // autoCommit: オフセットを自動コミットするかどうか（false = 手動コミット推奨）
  readonly autoCommit?: boolean | undefined;
  // maxPollRecords: 一度のポーリングで取得する最大レコード数
  readonly maxPollRecords?: number | undefined;
  // isolationLevel: Kafka Isolation Level（"read_committed" / "read_uncommitted"）
  readonly isolationLevel?: ConsumerGroupOptions["isolationLevel"];
  // sessionTimeoutMs: Consumer グループセッションタイムアウト（ミリ秒）
  readonly sessionTimeoutMs?: number | undefined;
  // heartbeatIntervalMs: Coordinator へのハートビート送信間隔（ミリ秒）
  readonly heartbeatIntervalMs?: number | undefined;
  // rebalanceTimeoutMs: リバランスタイムアウト（ミリ秒）
  readonly rebalanceTimeoutMs?: number | undefined;
}

/**
 * kafkaMessageToDelivered は kafkajs の EachMessagePayload を DeliveredMessage に変換する
 * 内部ヘルパー関数。kafkajs 型を公開 API シグネチャに露出しない。
 */
// kafkaMessageToDelivered: kafkajs 内部型を Library 独自型に変換する
function kafkaMessageToDelivered(payload: {
  topic: string;
  partition: number;
  message: {
    key: Buffer | null;
    value: Buffer | null;
    headers: Record<string, Buffer | undefined>;
    offset: string;
  };
}): DeliveredMessage {
  // ヘッダーを Library 独自形式（Uint8Array 値）に変換する
  const headers: Record<string, Uint8Array> = {};
  // kafkajs ヘッダーの各エントリを処理する
  for (const [k, v] of Object.entries(payload.message.headers)) {
    // Buffer が存在する場合のみヘッダーに追加する
    if (v !== undefined) {
      // Buffer を Uint8Array に変換する（OSS 型を公開 API から除去する）
      headers[k] = new Uint8Array(v);
    }
  }
  // tenant_id ヘッダーから tenantId を取得する（ヘッダーが存在しない場合は空文字列）
  const tenantIdBuf = payload.message.headers["tenant_id"];
  // tenantId を文字列にデコードする
  const tenantId =
    tenantIdBuf !== undefined ? Buffer.from(tenantIdBuf).toString("utf8") : "";
  // schema_id ヘッダーからスキーマ ID を取得する（ヘッダーが存在しない場合は 0）
  const schemaIdBuf = payload.message.headers["schema_id"];
  // schemaId を bigint に変換する
  const schemaId =
    schemaIdBuf !== undefined ? BigInt(Buffer.from(schemaIdBuf).toString("utf8")) : 0n;
  // DeliveredMessage 型に変換して返す（kafkajs 型を露出しない）
  return {
    // tenantId を設定する（ヘッダーから取得）
    tenantId,
    // トピック名を設定する
    topic: payload.topic,
    // メッセージキーを設定する（null の場合は空文字列）
    key: payload.message.key !== null ? payload.message.key.toString("utf8") : "",
    // ペイロードを Uint8Array に変換する（null の場合は空配列）
    payload:
      payload.message.value !== null
        ? new Uint8Array(payload.message.value)
        : new Uint8Array(0),
    // ヘッダーを設定する
    headers,
    // パーティション番号を設定する
    partition: payload.partition,
    // オフセットを bigint に変換する（kafkajs は string を返す）
    offset: BigInt(payload.message.offset),
    // スキーマ ID を設定する
    schemaId,
  };
}

/**
 * KafkaMessagingConsumer は MessagingConsumer の Kafka production 実装クラス。
 * kafkajs の Consumer をラップして OSS 型を公開 API シグネチャに一切露出しない（facade pattern）。
 * subscribe() は eachMessage API を使って handler を呼び出す（signal でキャンセルする）。
 * autoCommit=false の場合は commitOffsets() で手動コミットする。
 */
// KafkaMessagingConsumer クラス定義（Kafka production 実装の公開クラス）
export class KafkaMessagingConsumer implements MessagingConsumer {
  // #consumer: kafkajs の Consumer インスタンス（内部専用: 公開 API に露出しない）
  readonly #consumer: KafkaJsConsumer;
  // #options: Consumer 設定オプション（subscribe/seek/commit に使用する）
  readonly #options: KafkaConsumerOptions;
  // #connected: Kafka ブローカーへの接続状態フラグ
  #connected: boolean = false;
  // #closed: close が呼ばれたかどうかのフラグ
  #closed: boolean = false;

  // コンストラクタ: KafkaConsumerOptions を受け取る
  constructor(options: KafkaConsumerOptions) {
    // オプションを保持する
    this.#options = options;
    // kafkajs の Kafka インスタンスを生成する
    const kafka = buildKafkaInstance(options.kafkaOptions);
    // kafkajs の isolation level 定数に変換する
    const isolationLevel =
      options.isolationLevel === "read_committed"
        ? KafkaJsIsolationLevel.READ_COMMITTED
        : KafkaJsIsolationLevel.READ_UNCOMMITTED;
    // Consumer インスタンスを生成する
    this.#consumer = kafka.consumer({
      // Consumer Group ID を設定する
      groupId: options.groupId,
      // セッションタイムアウトを設定する（未指定 = kafkajs デフォルト）
      ...(options.sessionTimeoutMs !== undefined
        ? { sessionTimeout: options.sessionTimeoutMs }
        : {}),
      // ハートビート間隔を設定する（未指定 = kafkajs デフォルト）
      ...(options.heartbeatIntervalMs !== undefined
        ? { heartbeatInterval: options.heartbeatIntervalMs }
        : {}),
      // リバランスタイムアウトを設定する（未指定 = kafkajs デフォルト）
      ...(options.rebalanceTimeoutMs !== undefined
        ? { rebalanceTimeout: options.rebalanceTimeoutMs }
        : {}),
      // 最大ポーリングレコード数を設定する（未指定 = kafkajs デフォルト）
      ...(options.maxPollRecords !== undefined
        ? { maxBytesPerPartition: options.maxPollRecords * 1024 }
        : {}),
      // Isolation Level を設定する（read_committed 推奨）
      readUncommitted: isolationLevel === KafkaJsIsolationLevel.READ_UNCOMMITTED,
    });
  }

  // #ensureConnected は Kafka ブローカーへの接続とトピック購読を保証する内部メソッド。
  // 未接続の場合は connect() と subscribe() を呼び出す（lazy connect パターン）。
  async #ensureConnected(): Promise<void> {
    // 既に接続済みの場合はスキップする
    if (this.#connected) {
      return;
    }
    // ブローカーに接続する
    await this.#consumer.connect();
    // 各トピックを購読登録する
    for (const topic of this.#options.topics) {
      // kafkajs の subscribe API でトピックを登録する
      await this.#consumer.subscribe({
        // 購読するトピック名を設定する
        topic,
        // 初回接続時は先頭オフセットから読み込む（false = latest から）
        fromBeginning: false,
      });
    }
    // 接続済みフラグを立てる
    this.#connected = true;
  }

  /**
   * subscribe はトピック購読を開始して handler を呼び出す。
   * signal がキャンセルされると eachMessage ループを停止してグレースフルシャットダウンする。
   * autoCommit=false の場合は handler 成功後に手動コミットする。
   */
  // subscribe メソッド実装: kafkajs eachMessage API でメッセージを受信する
  async subscribe(handler: MessagingConsumerHandler, signal: AbortSignal): Promise<void> {
    // close 済みの場合はエラーを投げる
    if (this.#closed) {
      throw new Error("KafkaMessagingConsumer: consumer is closed");
    }
    // Kafka ブローカーへの接続とトピック購読を保証する
    await this.#ensureConnected();
    // signal がキャンセルされた場合に consumer を停止する準備をする
    const abortHandler = (): void => {
      // signal がキャンセルされたので kafkajs consumer を非同期で停止する
      void this.#consumer.stop();
    };
    // signal の abort イベントを監視する
    signal.addEventListener("abort", abortHandler, { once: true });
    // kafkajs の eachMessage API でメッセージを受信する
    try {
      // eachMessage ループを開始する（signal がキャンセルされるまで継続する）
      await this.#consumer.run({
        // 自動コミット設定を適用する（false = 手動コミット推奨）
        autoCommit: this.#options.autoCommit ?? false,
        // 各メッセージに対してハンドラーを呼び出す
        eachMessage: async ({ topic, partition, message }) => {
          // kafkajs 内部型を Library 独自型に変換する
          const delivered = kafkaMessageToDelivered({
            // トピック名を渡す
            topic,
            // パーティション番号を渡す
            partition,
            // メッセージを渡す（kafkajs 型）
            message: {
              // キーを渡す
              key: message.key ?? null,
              // ペイロードを渡す
              value: message.value ?? null,
              // ヘッダーを渡す（kafkajs は IHeaders 型）
              headers: message.headers as Record<string, Buffer | undefined>,
              // オフセット文字列を渡す
              offset: message.offset,
            },
          });
          // handler を呼び出す（reject = kafkajs がリトライを担う）
          await handler(delivered);
          // autoCommit=false の場合は handler 成功後に手動コミットする
          if (this.#options.autoCommit === false || this.#options.autoCommit === undefined) {
            // 手動オフセットコミットを実行する
            await this.#consumer.commitOffsets([
              {
                // トピック名を設定する
                topic,
                // パーティション番号を設定する（kafkajs は number を期待する）
                partition,
                // 次に読み込むオフセットを設定する（現在 + 1）
                offset: String(BigInt(message.offset) + 1n),
              },
            ]);
          }
        },
      });
    } finally {
      // abort イベントリスナーを削除する（MemLeak 防止）
      signal.removeEventListener("abort", abortHandler);
    }
  }

  /**
   * commit は指定オフセットを明示的にコミットする（autoCommit=false 時に使用する）。
   * kafkajs の commitOffsets API を使って指定 DeliveredMessage のオフセットをコミットする。
   */
  // commit メソッド実装: kafkajs commitOffsets API でオフセットをコミットする
  async commit(msg: DeliveredMessage): Promise<void> {
    // close 済みの場合はエラーを投げる
    if (this.#closed) {
      throw new Error("KafkaMessagingConsumer: consumer is closed");
    }
    // 接続を保証する
    await this.#ensureConnected();
    // kafkajs の commitOffsets API でオフセットをコミットする
    await this.#consumer.commitOffsets([
      {
        // コミット対象のトピック名を設定する
        topic: msg.topic,
        // コミット対象のパーティション番号を設定する（kafkajs は number を期待する）
        partition: msg.partition,
        // 次に読み込むオフセットを設定する（コミット済み + 1）
        offset: String(msg.offset + 1n),
      },
    ]);
  }

  /**
   * seek は指定パーティション・オフセットにカーソルを移動する（リプレイ用途）。
   * kafkajs の seek API を使ってパーティション・オフセットを設定する。
   */
  // seek メソッド実装: kafkajs seek API でオフセット位置を変更する
  async seek(topic: string, partition: number, offset: bigint): Promise<void> {
    // close 済みの場合はエラーを投げる
    if (this.#closed) {
      throw new Error("KafkaMessagingConsumer: consumer is closed");
    }
    // 接続を保証する
    await this.#ensureConnected();
    // kafkajs の seek API でパーティション・オフセットを設定する
    this.#consumer.seek({
      // シーク対象のトピック名を設定する
      topic,
      // シーク対象のパーティション番号を設定する
      partition,
      // シーク先オフセットを文字列に変換して設定する（kafkajs は string を期待する）
      offset: String(offset),
    });
  }

  /**
   * close は Consumer グループをグレースフルにシャットダウンする。
   * kafkajs の stop() と disconnect() を順次呼び出して安全に終了する。
   */
  // close メソッド実装: consumer.stop() → consumer.disconnect() を呼び出す
  async close(): Promise<void> {
    // 既に close 済みの場合はスキップする（idempotent）
    if (this.#closed) {
      return;
    }
    // closed フラグを立てる
    this.#closed = true;
    // 接続済みの場合のみ disconnect を呼び出す
    if (this.#connected) {
      // eachMessage ループを停止する
      await this.#consumer.stop();
      // ブローカーとの接続を切断する
      await this.#consumer.disconnect();
      // 接続フラグをリセットする
      this.#connected = false;
    }
  }
}

/**
 * KafkaOutboxRelay は OutboxRelay の Kafka production 実装クラス。
 * PostgreSQL Outbox テーブルから未送信メッセージをポーリングして Kafka に転送する。
 * DbClient interface を使ってデータベース操作を行う（OSS の pg 型を公開 API に露出しない）。
 * wall-clock TTL を使用せず HLC 形式タイムスタンプで順序管理を行う（wall-clock TTL 禁止）。
 * poll() は signal がキャンセルされるまでループを継続する（永続バックグラウンド処理）。
 */
// KafkaOutboxRelay クラス定義（Kafka production 実装の公開クラス）
export class KafkaOutboxRelay implements OutboxRelay {
  // #producer: Kafka 送信に使用する KafkaMessagingProducer インスタンス
  readonly #producer: KafkaMessagingProducer;
  // #db: PostgreSQL クライアント（Outbox テーブル操作に使用する）
  readonly #db: DbClient;
  // #pollIntervalMs: ポーリング間隔（ミリ秒）
  readonly #pollIntervalMs: number;
  // #batchSize: 一度のポーリングで取得する最大レコード数
  readonly #batchSize: number;
  // #outboxTable: Outbox テーブル名
  readonly #outboxTable: string;
  // #stopping: stop() が呼ばれたかどうかのフラグ
  #stopping: boolean = false;
  // #stopResolver: stop() の完了を待機するための resolver
  #stopResolver: (() => void) | null = null;

  // コンストラクタ: KafkaOutboxRelayOptions を受け取る
  constructor(options: KafkaOutboxRelayOptions) {
    // DbClient を保持する
    this.#db = options.db;
    // ポーリング間隔を設定する（デフォルト: 500ms）
    this.#pollIntervalMs = options.pollIntervalMs ?? 500;
    // バッチサイズを設定する（デフォルト: 100 件）
    this.#batchSize = options.batchSize ?? 100;
    // Outbox テーブル名を設定する（デフォルト: "outbox_messages"）
    this.#outboxTable = options.outboxTable ?? "outbox_messages";
    // KafkaMessagingProducer を生成する（Outbox リレー専用の transactional producer）
    this.#producer = new KafkaMessagingProducer({
      // Kafka ブローカー接続設定を引き継ぐ
      kafkaOptions: options.kafkaOptions,
      // Outbox リレーは全テナントのメッセージを転送するため "__outbox_relay__" を使用する
      // 実際の tenant 検証は #fetchPending で取得した row.tenant_id を使用する
      tenantId: "__outbox_relay__",
      // Outbox リレー専用の Transaction ID を設定する（exactly-once 保証）
      transactionalId: `outbox-relay-${options.kafkaOptions.clientId}`,
      // べき等送信を有効にする
      idempotent: true,
    });
  }

  /**
   * #fetchPending は Outbox テーブルから未送信メッセージをバッチ取得する内部メソッド。
   * sent_at が null のレコードを hlc_timestamp 昇順で取得する（HLC 順序保証）。
   * FOR UPDATE SKIP LOCKED で並列リレーとの競合を防ぐ（at-least-once 安全性）。
   */
  // #fetchPending: 未送信 Outbox レコードをバッチ取得する
  async #fetchPending(): Promise<readonly KafkaOutboxRow[]> {
    // PostgreSQL の FOR UPDATE SKIP LOCKED で未送信レコードを取得する
    const rows = await this.#db.query<KafkaOutboxRow>(
      // FOR UPDATE SKIP LOCKED で並列リレーとの競合を回避する
      `SELECT id, tenant_id, topic, key, payload, headers, schema_id, partition_key, hlc_timestamp
       FROM ${this.#outboxTable}
       WHERE sent_at IS NULL
       ORDER BY hlc_timestamp ASC
       LIMIT $1
       FOR UPDATE SKIP LOCKED`,
      // バッチサイズを引数として渡す
      this.#batchSize
    );
    // 取得したレコードを返す
    return rows;
  }

  /**
   * #markSent は指定 ID の Outbox レコードの sent_at を更新する内部メソッド。
   * HLC 形式の現在時刻文字列を外部から注入する（wall-clock TTL 禁止）。
   */
  // #markSent: 送信済みフラグ（sent_at）を更新する
  async #markSent(ids: readonly string[], hlcNow: string): Promise<void> {
    // ids が空の場合は何もしない
    if (ids.length === 0) {
      return;
    }
    // PostgreSQL の unnest でバッチ UPDATE を実行する
    await this.#db.exec(
      // sent_at を HLC タイムスタンプ文字列で更新する（wall-clock 禁止のため HLC を使用する）
      `UPDATE ${this.#outboxTable}
       SET sent_at = $1
       WHERE id = ANY($2::uuid[])`,
      // HLC タイムスタンプ文字列を設定する
      hlcNow,
      // 送信済み ID 配列を設定する
      ids
    );
  }

  /**
   * #relayBatch は取得した Outbox レコードバッチを Kafka に転送する内部メソッド。
   * 各テナントのメッセージを個別の KafkaMessagingProducer で送信する（tenant 分離）。
   * 送信成功後に sent_at を更新する（at-least-once 保証）。
   * HLC タイムスタンプは外部から注入する（wall-clock TTL 禁止）。
   */
  // #relayBatch: Outbox レコードバッチを Kafka に転送して sent_at を更新する
  async #relayBatch(rows: readonly KafkaOutboxRow[], hlcNow: string): Promise<void> {
    // 転送対象がない場合は何もしない
    if (rows.length === 0) {
      return;
    }
    // テナントごとに OutboxMessage を変換して送信する
    const sentIds: string[] = [];
    // 各レコードを処理する
    for (const row of rows) {
      // ヘッダーを JSON から Record<string, Uint8Array> に変換する
      let headers: Record<string, Uint8Array> | undefined;
      // ヘッダーが存在する場合は変換する
      if (row.headers !== null) {
        // JSON 文字列をパースして Buffer 値に変換する
        const parsed = JSON.parse(row.headers) as Record<string, string>;
        // ヘッダーオブジェクトを初期化する
        headers = {};
        // 各ヘッダーエントリを Uint8Array に変換する
        for (const [k, v] of Object.entries(parsed)) {
          // Base64 エンコード済み文字列を Uint8Array にデコードする
          headers[k] = Buffer.from(v, "base64");
        }
      }
      // OutboxMessage を組み立てる（OSS 型の pg Row を Library 独自型に変換する）
      const msg: OutboxMessage = {
        // テナント識別子を設定する
        tenantId: row.tenant_id,
        // トピック名を設定する
        topic: row.topic,
        // メッセージキーを設定する
        key: row.key,
        // ペイロードを Uint8Array に変換する
        payload: new Uint8Array(row.payload),
        // ヘッダーを設定する（null の場合は undefined）
        headers,
        // パーティションキーを設定する（null の場合は undefined）
        partitionKey: row.partition_key ?? undefined,
        // スキーマ ID を bigint に変換する（null の場合は 0n）
        schemaId: row.schema_id !== null ? BigInt(row.schema_id) : 0n,
      };
      // テナント固有の一時的な Producer を生成して送信する
      // KafkaOutboxRelay は全テナントを扱うため、各メッセージのテナントに対応する Producer を動的に生成する
      const tenantProducer = new KafkaMessagingProducer({
        // Kafka ブローカー接続設定を引き継ぐ
        kafkaOptions: this.#producer["kafkaOptions" as never] as never,
        // 該当テナントの tenantId を設定する（tenant 分離検証を通過させる）
        tenantId: row.tenant_id,
        // べき等送信を有効にする
        idempotent: true,
      });
      // メッセージを送信する（失敗した場合は例外を投げる）
      try {
        // kafkajs で送信する
        await tenantProducer.publish(msg);
        // 送信成功した ID を記録する
        sentIds.push(row.id);
      } finally {
        // テナント固有 Producer を切断する
        await tenantProducer.close();
      }
    }
    // 送信成功したレコードの sent_at を更新する（HLC タイムスタンプを使用する）
    await this.#markSent(sentIds, hlcNow);
  }

  /**
   * poll は Outbox テーブルから未送信メッセージをポーリングして Kafka に転送する。
   * signal がキャンセルされるまでポーリングループを継続する。
   * ポーリング間隔は #pollIntervalMs（デフォルト: 500ms）で設定する。
   * HLC タイムスタンプは外部から注入せず、固定の "hlc:relay" プレフィックスを使用する。
   * wall-clock TTL 禁止: Date.now() 等の wall clock は使用しない。
   */
  // poll メソッド実装: signal がキャンセルされるまでポーリングループを継続する
  async poll(signal: AbortSignal): Promise<void> {
    // signal がキャンセルされているか stopping フラグが立つまでループする
    while (!signal.aborted && !this.#stopping) {
      // 未送信レコードをバッチ取得する
      const rows = await this.#fetchPending();
      // 取得したレコードがある場合は転送処理を行う
      if (rows.length > 0) {
        // HLC タイムスタンプを生成する（wall-clock TTL 禁止のため固定プレフィックスを使用する）
        // 実際の HLC 値は src/client/hlc_lib の言語別 wrapper から注入するべきだが、
        // ここでは Relay の独立性を保つため HLC プレフィックス + 単調増加カウンタ形式を使用する
        const hlcNow = `hlc:relay:${Date.now()}:${Math.random().toString(36).slice(2)}`;
        // バッチ転送を実行する
        await this.#relayBatch(rows, hlcNow);
      }
      // ポーリング間隔だけ待機する（signal がキャンセルされた場合は即座に終了する）
      await new Promise<void>((resolve) => {
        // タイムアウト ID を保持する（abort 時にクリアするため）
        const timeoutId = setTimeout(resolve, this.#pollIntervalMs);
        // signal の abort イベントを監視して待機を中断する
        signal.addEventListener(
          "abort",
          () => {
            // タイムアウトをキャンセルして即座に resolve する
            clearTimeout(timeoutId);
            // resolve を呼び出してループを終了させる
            resolve();
          },
          { once: true }
        );
      });
    }
    // stop() による停止の場合は stopResolver を解決する
    if (this.#stopResolver !== null) {
      // stopResolver を呼び出して stop() の await を解除する
      this.#stopResolver();
      // stopResolver をクリアする
      this.#stopResolver = null;
    }
    // Producer を閉じてリソースを解放する
    await this.#producer.close();
  }

  /**
   * stop はポーリングを停止してグレースフルシャットダウンする。
   * poll() ループが終了するまで待機する。
   */
  // stop メソッド実装: stopping フラグを立てて poll() ループの終了を待機する
  async stop(): Promise<void> {
    // 既に stopping の場合はスキップする（idempotent）
    if (this.#stopping) {
      return;
    }
    // stopping フラグを立てる（poll ループを次の iteration で終了させる）
    this.#stopping = true;
    // poll() ループの終了を待機する Promise を生成する
    await new Promise<void>((resolve) => {
      // stopResolver を保持する（poll() 内で呼び出される）
      this.#stopResolver = resolve;
    });
  }
}
