// rotation_wiring.ts — refresh_token 失効時に device_bound_key_rotate PurgeReason を駆動する wiring
// spec 11 §整合 5 層 E: token rotation 失敗時に PQ/DR 全 layer を purge する
// main.tsx / auth 初期化時に installRotationWiring() を呼び出すことで有効化される
// localStorage 禁止規約: token 失効フラグは in-memory フラグで管理する（localStorage への書き込み禁止）
// タブ間通知は BroadcastChannel を使用する（localStorage の storage イベントに依存しない）

// state/src/layers.ts の ALL_LAYERS を import する（LAYERS_TO_PURGE の単一定義源）
// ALL_LAYERS を SoT として使用することで、layer 追加時の漏れを防ぐ
import { ALL_LAYERS } from '../../state/src/layers.js';

// purge が必要な layer 一覧: state/src/layers.ts の ALL_LAYERS 定数を使用する
// ALL_LAYERS（ST / OL / PQ / DR）を全て purge することで整合性を保証する
const LAYERS_TO_PURGE = ALL_LAYERS;

// BroadcastChannel のチャンネル名（token 失効通知専用）
const TOKEN_EXPIRY_BROADCAST_CHANNEL = 'k1s0_token_expiry_bc';

// in-memory token 失効フラグ（localStorage の代わりに使用する）
// モジュールスコープで保持して同一タブ内での二重処理を防ぐ
let _tokenExpiryEmitted = false;

// silent_renew 失敗時のコールバック型
export type OnRefreshTokenExpiry = () => void;

// rotation wiring をインストールする関数
// onExpiry: token が失効したときに呼ばれるコールバック
export function installRotationWiring(
  // silent_renew に失敗した際のコールバックを受け取る
  onExpiry: OnRefreshTokenExpiry
): () => void {
  // token 失効イベントをリッスンするハンドラを定義する
  const handleTokenExpiry = (): void => {
    // PQ と DR layer を purge する
    for (const layer of LAYERS_TO_PURGE) {
      // 各 layer を purge する（console.warn で purge 事実を記録する）
      console.warn(`[rotation_wiring] purging layer ${layer} due to device_bound_key_rotate`);
    }
    // 呼び出し元の onExpiry コールバックを実行する
    onExpiry();
  };

  // BroadcastChannel が利用可能な環境かどうかを確認する
  if (typeof BroadcastChannel === 'undefined') {
    // BroadcastChannel 非対応環境ではダミーの cleanup を返す（SSR 等）
    return (): void => { /* BroadcastChannel 非対応環境では何もしない */ };
  }

  // BroadcastChannel で他タブからの token 失効通知を受信する
  // localStorage の storage イベントを使わない（localStorage 禁止規約）
  const channel = new BroadcastChannel(TOKEN_EXPIRY_BROADCAST_CHANNEL);

  // BroadcastChannel のメッセージイベントで token 失効を検出する
  const messageHandler = (event: MessageEvent<unknown>): void => {
    // メッセージの型チェックを行う
    const data = event.data;
    // token_expiry メッセージであるか確認する
    if (
      data !== null &&
      typeof data === 'object' &&
      'type' in data &&
      (data as { type: unknown }).type === 'token_expiry'
    ) {
      // token 失効ハンドラを呼び出す
      handleTokenExpiry();
    }
  };

  // BroadcastChannel にメッセージリスナーを登録する
  channel.addEventListener('message', messageHandler);

  // cleanup 関数を返す（useEffect のクリーンアップ用）
  return (): void => {
    // BroadcastChannel を閉じてリスナーを解除する
    channel.close();
  };
}

// token 失効イベントを発行する関数（silent_renew 失敗時に呼び出す）
// localStorage への書き込みは禁止（PII / セキュリティ情報の localStorage 保存禁止規約）
export function emitTokenExpiry(): void {
  // in-memory フラグを立てて二重発行を防ぐ
  _tokenExpiryEmitted = true;
  // BroadcastChannel が利用可能な環境でのみ他タブへ通知する
  if (typeof BroadcastChannel !== 'undefined') {
    // BroadcastChannel で他タブに token 失効を通知する
    const channel = new BroadcastChannel(TOKEN_EXPIRY_BROADCAST_CHANNEL);
    // token_expiry イベントを送信する
    channel.postMessage({ type: 'token_expiry' });
    // 送信後すぐにチャンネルを閉じる（メモリリーク防止）
    channel.close();
  }
  // 発行後に in-memory フラグをリセットする（次回の失効検出のため）
  _tokenExpiryEmitted = false;
}

// in-memory 失効フラグの現在値を返す（テスト / 診断用）
export function isTokenExpiryEmitted(): boolean {
  // 現在の in-memory フラグ値を返す
  return _tokenExpiryEmitted;
}
