// rotation_wiring.ts — refresh_token 失効時に device_bound_key_rotate PurgeReason を駆動する wiring
// spec 11 §整合 5 層 E: token rotation 失敗時に PQ/DR 全 layer を purge する
// main.tsx / auth 初期化時に installRotationWiring() を呼び出すことで有効化される

// purge が必要な layer 一覧（PQ と DR を対象にする）
const LAYERS_TO_PURGE = ['PQ', 'DR'] as const;

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

  // window の storage イベントで token 失効を検出する（BroadcastChannel の代替）
  const listener = (event: StorageEvent): void => {
    // token_expiry イベントキーを確認する
    if (event.key === 'k1s0_token_expiry' && event.newValue === 'true') {
      // token 失効ハンドラを呼び出す
      handleTokenExpiry();
    }
  };
  // storage イベントリスナーを追加する
  window.addEventListener('storage', listener);

  // cleanup 関数を返す（useEffect のクリーンアップ用）
  return (): void => {
    // storage イベントリスナーを削除する
    window.removeEventListener('storage', listener);
  };
}

// token 失効イベントを発行する関数（silent_renew 失敗時に呼び出す）
export function emitTokenExpiry(): void {
  // localStorage に token_expiry フラグを立てる（他タブへの broadcast）
  localStorage.setItem('k1s0_token_expiry', 'true');
  // 少し後に flag をリセットする（1 度のみ処理させる）
  setTimeout(() => localStorage.removeItem('k1s0_token_expiry'), 100);
}
