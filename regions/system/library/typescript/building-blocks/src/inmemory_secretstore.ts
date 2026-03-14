// インメモリシークレットストア実装モジュール。
// テストおよびローカル開発環境向けに、Vault等の外部シークレット管理サービス不要でSecretStoreを提供する。

import { ComponentError } from './errors.js';
import type { ComponentStatus } from './component.js';
import type { SecretValue, SecretStore } from './secretstore.js';

/**
 * インメモリシークレットストア実装クラス。
 * キーと平文値のMapでシークレットを管理する。
 * 本番環境では実際のVault等を使用し、このクラスはテスト専用として扱う。
 */
export class InMemorySecretStore implements SecretStore {
  // コンポーネントの識別名
  readonly name: string;
  // コンポーネント種別（シークレットストアであることを示す）
  readonly componentType = 'secretstore';

  // 現在のコンポーネント状態（未初期化・準備完了・クローズ済み）
  private _status: ComponentStatus = 'uninitialized';
  // キー → 平文値のマッピングを保持するMap
  private readonly _secrets = new Map<string, string>();

  // デフォルト名を指定してインスタンスを生成する
  constructor(name = 'inmemory-secretstore') {
    this.name = name;
  }

  // コンポーネントを初期化し、状態をreadyに変更する
  async init(): Promise<void> { this._status = 'ready'; }

  // コンポーネントをクローズし、全シークレットをクリアして状態をclosedに変更する
  async close(): Promise<void> { this._secrets.clear(); this._status = 'closed'; }

  // 現在のコンポーネント状態を返す
  async status(): Promise<ComponentStatus> { return this._status; }

  // バックエンドを示すメタデータを返す
  metadata(): Record<string, string> { return { backend: 'memory' }; }

  // テスト用: シークレットを設定する。
  put(key: string, value: string): void {
    this._secrets.set(key, value);
  }

  /**
   * 指定キーのシークレットを取得して返す。
   * キーが存在しない場合はComponentErrorをスローする。
   */
  async getSecret(key: string): Promise<SecretValue> {
    const value = this._secrets.get(key);
    // キーが見つからない場合はエラーをスローする
    if (value === undefined) {
      throw new ComponentError(this.name, 'getSecret', `secret "${key}" not found`);
    }
    // キー・値・空のメタデータを含むSecretValueオブジェクトを返す
    return { key, value, metadata: {} };
  }

  /**
   * 複数キーのシークレットをまとめて取得して返す。
   * 内部でgetSecret()を呼び出すため、いずれかのキーが存在しない場合はエラーをスローする。
   */
  async bulkGet(keys: string[]): Promise<Record<string, SecretValue>> {
    const result: Record<string, SecretValue> = {};
    // 各キーのシークレットを順次取得してresultに集約する
    for (const key of keys) {
      result[key] = await this.getSecret(key);
    }
    return result;
  }
}
