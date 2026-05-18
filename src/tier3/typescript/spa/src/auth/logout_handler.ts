// logout_handler.ts — performGlobalLogout 時に PQ/DR purgeAllLayers を駆動する hook
// spec 11 §整合 5 層 E: global logout 時に offline layer を全 purge する
// rotation_wiring.ts と協調して device_bound_key_rotate の purge wiring を完成させる

// purge 対象の layer 一覧
const LAYERS_TO_PURGE = ['ST', 'OL', 'PQ', 'DR'] as const;

// global logout 処理を実行する関数
// purgeLayer: 各 layer を purge するコールバック（IndexedDB store の clearLayer を渡す）
export async function performGlobalLogout(
  // layer を purge するコールバック関数を受け取る
  purgeLayer: (layer: string) => Promise<void>
): Promise<void> {
  // すべての layer を purge する
  for (const layer of LAYERS_TO_PURGE) {
    // 各 layer の purge を実行する
    await purgeLayer(layer);
    // purge 完了ログを出力する
    console.warn(`[logout_handler] layer ${layer} purged`);
  }
  // token 失効イベントを発行して他タブにも通知する
  localStorage.setItem('k1s0_token_expiry', 'true');
  // 少し後に flag をリセットする
  setTimeout(() => localStorage.removeItem('k1s0_token_expiry'), 100);
}
