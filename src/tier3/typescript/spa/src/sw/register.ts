// register.ts — Service Worker の登録と storage.persisted() health probe
// spec 11 §PWA: SW を登録して IndexedDB の persistent storage を要求する

// Service Worker が登録可能かチェックして登録する関数
export async function registerServiceWorker(): Promise<void> {
  // Service Worker API がサポートされていない場合は早期リターンする
  if (!('serviceWorker' in navigator)) {
    // SW 未サポートのブラウザでは何もしない
    return;
  }
  // SW 登録を試みる
  try {
    // /sw.js の Service Worker を登録する
    const registration = await navigator.serviceWorker.register('/sw.js', {
      // SW のスコープを root に設定する
      scope: '/',
    });
    // 登録成功のデバッグログを出力する
    console.debug('[SW] registered:', registration.scope);
  } catch (error) {
    // 登録失敗時は警告ログを出力する
    console.warn('[SW] registration failed:', error);
  }
}

// IndexedDB の persistent storage を要求する health probe 関数
export async function checkStoragePersistence(): Promise<boolean> {
  // storage API がサポートされていない場合は false を返す
  if (!navigator.storage?.persist) {
    // 非対応ブラウザは false として扱う
    return false;
  }
  // persistent storage を要求する（ブラウザが許可するか確認する）
  const isPersisted = await navigator.storage.persisted();
  // まだ persistent でない場合は許可を要求する
  if (!isPersisted) {
    // ブラウザに persistent storage の許可を要求する
    return navigator.storage.persist();
  }
  // 既に persistent な場合は true を返す
  return isPersisted;
}
