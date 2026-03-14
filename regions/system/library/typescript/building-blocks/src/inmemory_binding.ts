// インメモリバインディング実装モジュール。
// テストおよびローカル開発環境向けに、外部サービス不要でInputBinding・OutputBindingを提供する。

import { ComponentError } from './errors.js';
import type { ComponentStatus } from './component.js';
import type { BindingData, BindingResponse, InputBinding, OutputBinding } from './binding.js';

/**
 * OutputBindingの呼び出し記録を表すインターフェース。
 * テスト検証時にどの操作がどのデータ・メタデータで呼び出されたかを確認するために使用する。
 */
export interface BindingInvocation {
  // 実行された操作名
  operation: string;
  // 送信されたバイナリデータ
  data: Uint8Array;
  // 操作に付随するメタデータ
  metadata: Record<string, string>;
}

/**
 * インメモリ入力バインディング実装クラス。
 * キューにデータをプッシュしてread()で取得するFIFO構造を持つ。
 * 外部メッセージキュー（Kafka等）の代替として、テスト環境で使用する。
 */
export class InMemoryInputBinding implements InputBinding {
  // コンポーネントの識別名
  readonly name: string;
  // コンポーネント種別（入力バインディングであることを示す）
  readonly componentType = 'binding.input';

  // 現在のコンポーネント状態（未初期化・準備完了・クローズ済み）
  private _status: ComponentStatus = 'uninitialized';
  // 受信データを保持するFIFOキュー
  private _queue: BindingData[] = [];

  // デフォルト名を指定してインスタンスを生成する
  constructor(name = 'inmemory-input-binding') {
    this.name = name;
  }

  // コンポーネントを初期化し、状態をreadyに変更する
  async init(): Promise<void> { this._status = 'ready'; }

  // コンポーネントをクローズし、キューをクリアして状態をclosedに変更する
  async close(): Promise<void> { this._queue = []; this._status = 'closed'; }

  // 現在のコンポーネント状態を返す
  async status(): Promise<ComponentStatus> { return this._status; }

  // バックエンドと方向を示すメタデータを返す
  metadata(): Record<string, string> { return { backend: 'memory', direction: 'input' }; }

  // テスト用: データをキューに追加する。
  push(data: BindingData): void {
    this._queue.push(data);
  }

  /**
   * キューから先頭のデータを取り出して返す。
   * キューが空の場合はComponentErrorをスローする。
   */
  async read(): Promise<BindingData> {
    const item = this._queue.shift();
    if (!item) {
      throw new ComponentError(this.name, 'read', 'queue is empty');
    }
    return item;
  }
}

/**
 * インメモリ出力バインディング実装クラス。
 * invoke()の呼び出し履歴を記録し、モックレスポンスやモックエラーを返せる。
 * 外部サービスへの送信処理（HTTP呼び出し等）の代替として、テスト環境で使用する。
 */
export class InMemoryOutputBinding implements OutputBinding {
  // コンポーネントの識別名
  readonly name: string;
  // コンポーネント種別（出力バインディングであることを示す）
  readonly componentType = 'binding.output';

  // 現在のコンポーネント状態（未初期化・準備完了・クローズ済み）
  private _status: ComponentStatus = 'uninitialized';
  // invoke()の呼び出し履歴を蓄積するリスト
  private _invocations: BindingInvocation[] = [];
  // テスト用モックレスポンス（未設定の場合はデフォルト動作）
  private _mockResponse?: BindingResponse;
  // テスト用モックエラー（設定された場合はinvoke()でスローする）
  private _mockError?: Error;

  // デフォルト名を指定してインスタンスを生成する
  constructor(name = 'inmemory-output-binding') {
    this.name = name;
  }

  // コンポーネントを初期化し、状態をreadyに変更する
  async init(): Promise<void> { this._status = 'ready'; }

  // コンポーネントをクローズし、呼び出し履歴をクリアして状態をclosedに変更する
  async close(): Promise<void> { this._invocations = []; this._status = 'closed'; }

  // 現在のコンポーネント状態を返す
  async status(): Promise<ComponentStatus> { return this._status; }

  // バックエンドと方向を示すメタデータを返す
  metadata(): Record<string, string> { return { backend: 'memory', direction: 'output' }; }

  /**
   * 最後に記録された呼び出し情報を返す。
   * テストで最新のinvoke()呼び出しを検証するために使用する。
   */
  lastInvocation(): BindingInvocation | undefined {
    return this._invocations[this._invocations.length - 1];
  }

  /**
   * 記録された全呼び出し情報のコピーを返す。
   * テストで全invoke()呼び出しを検証するために使用する。
   */
  allInvocations(): BindingInvocation[] {
    return [...this._invocations];
  }

  /**
   * テスト用モックレスポンスまたはモックエラーを設定する。
   * 以降のinvoke()呼び出しで設定した値が使用される。
   */
  setResponse(response?: BindingResponse, error?: Error): void {
    this._mockResponse = response;
    this._mockError = error;
  }

  /**
   * 呼び出し履歴とモック設定をすべてリセットする。
   * テストケース間で状態をクリアするために使用する。
   */
  reset(): void {
    this._invocations = [];
    this._mockResponse = undefined;
    this._mockError = undefined;
  }

  /**
   * 操作を実行し、呼び出し履歴に記録してレスポンスを返す。
   * モックエラーが設定されている場合はそれをスローする。
   * モックレスポンスが設定されている場合はそれを返し、未設定の場合は入力データをそのまま返す。
   */
  async invoke(operation: string, data: Uint8Array, metadata: Record<string, string> = {}): Promise<BindingResponse> {
    // 呼び出し情報を履歴に追記する
    this._invocations.push({ operation, data, metadata });
    // モックエラーが設定されている場合はスローする
    if (this._mockError) throw this._mockError;
    // モックレスポンスが設定されていればそれを返し、未設定の場合は入力をそのまま返す
    return this._mockResponse ?? { data, metadata };
  }
}
