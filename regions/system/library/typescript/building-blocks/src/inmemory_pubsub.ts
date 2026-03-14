// インメモリPubSub実装モジュール。
// テストおよびローカル開発環境向けに、外部ブローカー（Kafka等）不要でPubSubを提供する。

import type { ComponentStatus } from './component.js';
import type { Message, MessageHandler, PubSub } from './pubsub.js';

// メッセージIDの重複を防ぐためのモジュールスコープカウンター
let _idCounter = 0;

/**
 * ユニークなメッセージIDを生成する。
 * タイムスタンプとインクリメントカウンターを組み合わせることで、
 * 同一ミリ秒内の複数メッセージでも衝突しないIDを保証する。
 */
function generateId(): string {
  return `${Date.now()}-${++_idCounter}`;
}

/**
 * インメモリPubSub実装クラス。
 * トピックごとにサブスクライバーをMapで管理し、publishで即時に全ハンドラーへ配信する。
 * 外部ブローカーへの接続なしにイベント駆動アーキテクチャをテストできる。
 */
export class InMemoryPubSub implements PubSub {
  // コンポーネントの識別名
  readonly name: string;
  // コンポーネント種別（PubSubであることを示す）
  readonly componentType = 'pubsub';

  // 現在のコンポーネント状態（未初期化・準備完了・クローズ済み）
  private _status: ComponentStatus = 'uninitialized';
  // トピック名 → (サブスクリプションID → ハンドラー) のネストされたMap
  private readonly _subs = new Map<string, Map<string, MessageHandler>>();

  // デフォルト名を指定してインスタンスを生成する
  constructor(name = 'inmemory-pubsub') {
    this.name = name;
  }

  // コンポーネントを初期化し、状態をreadyに変更する
  async init(): Promise<void> {
    this._status = 'ready';
  }

  // コンポーネントをクローズし、全サブスクリプションをクリアして状態をclosedに変更する
  async close(): Promise<void> {
    this._subs.clear();
    this._status = 'closed';
  }

  // 現在のコンポーネント状態を返す
  async status(): Promise<ComponentStatus> {
    return this._status;
  }

  // バックエンドを示すメタデータを返す
  metadata(): Record<string, string> {
    return { backend: 'memory' };
  }

  /**
   * 指定トピックにメッセージを発行し、購読中の全ハンドラーへ並列配信する。
   * 該当トピックのサブスクライバーが存在しない場合は何もしない。
   */
  async publish(topic: string, data: Uint8Array, metadata: Record<string, string> = {}): Promise<void> {
    const handlers = this._subs.get(topic);
    // 購読者がいない場合は即座に返る
    if (!handlers) return;
    // ユニークIDを付与したメッセージオブジェクトを生成する
    const msg: Message = { topic, data, metadata, id: generateId() };
    // 全ハンドラーを並列実行して配信完了を待つ
    await Promise.all([...handlers.values()].map(h => h.handle(msg)));
  }

  /**
   * 指定トピックにハンドラーを登録し、サブスクリプションIDを返す。
   * トピックが未登録の場合は新規にMapを作成してから登録する。
   * 返却されたIDはunsubscribe()で購読解除に使用する。
   */
  async subscribe(topic: string, handler: MessageHandler): Promise<string> {
    // トピックが未登録の場合は新規Mapを作成する
    if (!this._subs.has(topic)) {
      this._subs.set(topic, new Map());
    }
    // ユニークなサブスクリプションIDを生成して登録する
    const id = generateId();
    this._subs.get(topic)!.set(id, handler);
    return id;
  }

  /**
   * 指定のサブスクリプションIDに対応するハンドラーを全トピックから削除する。
   * 対象IDが存在しない場合は何もしない。
   */
  async unsubscribe(subscriptionId: string): Promise<void> {
    // 全トピックを走査してIDが一致するハンドラーを削除する
    for (const handlers of this._subs.values()) {
      handlers.delete(subscriptionId);
    }
  }
}
