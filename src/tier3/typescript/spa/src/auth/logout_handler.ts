// logout_handler.ts — performGlobalLogout 時に PQ/DR purgeAllLayers を駆動する hook
// spec 11 §整合 5 層 E: global logout 時に offline layer を全 purge する
// rotation_wiring.ts と協調して device_bound_key_rotate の purge wiring を完成させる
// localStorage 禁止規約: token 失効フラグは in-memory のみで管理する（localStorage への書き込み禁止）

// packages/auth の canonical performGlobalLogout をインポートして使用する
// このファイルは logout_broadcast.ts の performGlobalLogout の wrapper として機能する
import { performGlobalLogout as broadcastPerformGlobalLogout } from '../../../packages/auth/src/logout_broadcast.js';

// state/src/layers.ts の ALL_LAYERS を import して LAYERS_TO_PURGE の単一定義を使用する
// LAYERS_TO_PURGE の重複宣言を排除し、ALL_LAYERS を正規の定義源（SoT）にする
import { ALL_LAYERS } from '../../../packages/state/src/layers.js';

// purge 対象の layer 一覧: state/src/layers.ts の ALL_LAYERS 定数を使用する（重複定義禁止）
const LAYERS_TO_PURGE = ALL_LAYERS;

// in-memory logout 実行済みフラグ（localStorage の代わりに使用する）
// 同一タブ内での二重 logout 処理を防ぐ
let _logoutInProgress = false;

// global logout 処理を実行する関数
// purgeLayer: 各 layer を purge するコールバック（IndexedDB store の clearLayer を渡す）
// bffLogout: BFF ログアウト API を呼び出す関数
// sessionHash: セッション hash（PII を含まない）
export async function performGlobalLogout(
  // layer を purge するコールバック関数を受け取る
  purgeLayer: (layer: string) => Promise<void>,
  // BFF ログアウト関数を受け取る
  bffLogout: () => Promise<void>,
  // セッション hash を受け取る（PII を含まない）
  sessionHash: string,
): Promise<void> {
  // 二重 logout を防ぐ in-memory フラグをチェックする
  if (_logoutInProgress) {
    // 既に logout 処理中の場合は何もしない
    return;
  }
  // in-memory フラグを立てて二重処理を防ぐ
  _logoutInProgress = true;
  try {
    // すべての layer を purge する
    for (const layer of LAYERS_TO_PURGE) {
      // 各 layer の purge を実行する
      await purgeLayer(layer);
      // purge 完了ログを出力する
      console.warn(`[logout_handler] layer ${layer} purged`);
    }
    // canonical の performGlobalLogout を呼び出して BFF logout + BroadcastChannel 通知を実行する
    // localStorage への書き込みは一切行わない（localStorage 禁止規約）
    await broadcastPerformGlobalLogout(bffLogout, sessionHash);
  } finally {
    // 処理完了後にフラグをリセットする
    _logoutInProgress = false;
  }
}
