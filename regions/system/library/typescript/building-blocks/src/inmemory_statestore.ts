// インメモリステートストア実装モジュール。
// テストおよびローカル開発環境向けに、Redis等の外部ストア不要でStateStoreを提供する。

import { ETagMismatchError } from './errors.js';
import type { ComponentStatus } from './component.js';
import type { StateEntry, StateStore } from './statestore.js';

/**
 * 内部ストアエントリーの構造を定義するインターフェース。
 * バイナリ値とETagをセットで管理することで楽観的並行制御を実現する。
 */
interface _StoreEntry {
  // 保存されたバイナリ値
  value: Uint8Array;
  // 楽観的並行制御に使用するETag文字列
  etag: string;
}

// ETagの一意性を保証するためのモジュールスコープカウンター
let _etagCounter = 0;

/**
 * インメモリステートストア実装クラス。
 * キーとバイナリ値のMapで状態を管理し、ETagによる楽観的並行制御をサポートする。
 * 外部ストアへの接続なしにステート管理ロジックをテストできる。
 */
export class InMemoryStateStore implements StateStore {
  // コンポーネントの識別名
  readonly name: string;
  // コンポーネント種別（ステートストアであることを示す）
  readonly componentType = 'statestore';

  // 現在のコンポーネント状態（未初期化・準備完了・クローズ済み）
  private _status: ComponentStatus = 'uninitialized';
  // キー → エントリー（値+ETag）のマッピングを保持するMap
  private readonly _entries = new Map<string, _StoreEntry>();

  // デフォルト名を指定してインスタンスを生成する
  constructor(name = 'inmemory-statestore') {
    this.name = name;
  }

  // コンポーネントを初期化し、状態をreadyに変更する
  async init(): Promise<void> { this._status = 'ready'; }

  // コンポーネントをクローズし、全エントリーをクリアして状態をclosedに変更する
  async close(): Promise<void> { this._entries.clear(); this._status = 'closed'; }

  // 現在のコンポーネント状態を返す
  async status(): Promise<ComponentStatus> { return this._status; }

  // バックエンドを示すメタデータを返す
  metadata(): Record<string, string> { return { backend: 'memory' }; }

  /**
   * 指定キーのステートエントリーを取得して返す。
   * キーが存在しない場合はnullを返す。
   */
  async get(key: string): Promise<StateEntry | null> {
    const entry = this._entries.get(key);
    // エントリーが存在しない場合はnullを返す
    if (!entry) return null;
    // キー・値・ETagを含むStateEntryオブジェクトを返す
    return { key, value: entry.value, etag: entry.etag };
  }

  /**
   * 指定キーに値を保存し、新しいETagを返す。
   * etagが指定された場合は楽観的並行制御を行い、不一致の場合はETagMismatchErrorをスローする。
   * etagが未指定の場合は無条件で上書きする。
   */
  async set(key: string, value: Uint8Array, etag?: string): Promise<string> {
    const existing = this._entries.get(key);
    // ETagが指定されている場合は楽観的並行制御チェックを実行する
    if (etag !== undefined) {
      // エントリーが存在しない場合はETag不一致エラーをスローする
      if (!existing) throw new ETagMismatchError(key, etag, '');
      // 現在のETagと異なる場合は競合エラーをスローする
      if (existing.etag !== etag) throw new ETagMismatchError(key, etag, existing.etag);
    }
    // カウンターをインクリメントして新しいETagを生成する
    const newETag = String(++_etagCounter);
    this._entries.set(key, { value, etag: newETag });
    return newETag;
  }

  /**
   * 指定キーのエントリーを削除する。
   * キーが存在しない場合は何もしない。
   * etagが指定された場合は楽観的並行制御を行い、不一致の場合はETagMismatchErrorをスローする。
   */
  async delete(key: string, etag?: string): Promise<void> {
    const existing = this._entries.get(key);
    // エントリーが存在しない場合は何もしない
    if (!existing) return;
    // ETagが指定されており、現在のETagと異なる場合は競合エラーをスローする
    if (etag !== undefined && existing.etag !== etag) {
      throw new ETagMismatchError(key, etag, existing.etag);
    }
    this._entries.delete(key);
  }

  /**
   * 複数キーのステートエントリーをまとめて取得して返す。
   * 存在するエントリーのみを結果に含め、存在しないキーはスキップする。
   */
  async bulkGet(keys: string[]): Promise<StateEntry[]> {
    const results: StateEntry[] = [];
    // 各キーのエントリーを順次取得し、存在するものだけをresultsに追加する
    for (const key of keys) {
      const entry = await this.get(key);
      if (entry) results.push(entry);
    }
    return results;
  }

  /**
   * 複数のキーと値のペアをまとめて保存し、それぞれの新しいETagを返す。
   * ETagを指定しない無条件上書きで保存するため、競合チェックは行わない。
   */
  async bulkSet(entries: Array<{ key: string; value: Uint8Array }>): Promise<string[]> {
    const etags: string[] = [];
    // 各エントリーを順次保存し、生成されたETagをetags配列に追加する
    for (const { key, value } of entries) {
      etags.push(await this.set(key, value));
    }
    return etags;
  }
}
