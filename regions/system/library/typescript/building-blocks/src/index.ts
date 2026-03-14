// ビルディングブロックライブラリの公開APIエントリーポイント。
// 各コンポーネントのインターフェース・クラス・ユーティリティを一元的にエクスポートする。

// コンポーネントの状態と基底インターフェースを公開する
export type { ComponentStatus, Component } from './component.js';

// PubSub（メッセージ送受信）に関する型を公開する
export type { Message, MessageHandler, PubSub } from './pubsub.js';

// ステートストア（状態管理）に関する型を公開する
export type { StateEntry, StateStore } from './statestore.js';

// シークレットストア（機密情報管理）に関する型を公開する
export type { SecretValue, SecretStore } from './secretstore.js';

// バインディング（入出力接続）に関する型を公開する
export type { BindingData, BindingResponse, InputBinding, OutputBinding } from './binding.js';

// エラークラスを公開する（ComponentError: 汎用コンポーネントエラー、ETagMismatchError: ETag不一致エラー）
export { ComponentError, ETagMismatchError } from './errors.js';

// コンポーネント設定に関する型を公開する
export type { ComponentConfig, ComponentsConfig } from './config.js';

// 設定のロードとパースユーティリティを公開する
export { loadComponentsConfig, parseComponentsConfig } from './config.js';

// インメモリPubSub実装を公開する（テスト・ローカル開発用）
export { InMemoryPubSub } from './inmemory_pubsub.js';

// インメモリステートストア実装を公開する（テスト・ローカル開発用）
export { InMemoryStateStore } from './inmemory_statestore.js';

// インメモリシークレットストア実装を公開する（テスト・ローカル開発用）
export { InMemorySecretStore } from './inmemory_secretstore.js';

// インメモリバインディング実装を公開する（テスト・ローカル開発用）
export { InMemoryInputBinding, InMemoryOutputBinding } from './inmemory_binding.js';

// バインディング呼び出し記録の型を公開する（テスト検証用）
export type { BindingInvocation } from './inmemory_binding.js';
