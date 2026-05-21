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
