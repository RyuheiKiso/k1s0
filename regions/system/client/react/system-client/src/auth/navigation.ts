// ブラウザナビゲーションを抽象化するヘルパー
// テスト環境では navigateTo をモック差し替え可能にする

// デフォルトのナビゲーション実装（ブラウザ環境）
let navigateImpl = (url: string) => {
  window.location.href = url;
};

// ナビゲーション実装を差し替える（テスト用）
export function setNavigateImpl(fn: (url: string) => void) {
  navigateImpl = fn;
}

// ナビゲーション実装をデフォルトに戻す（テスト用）
export function resetNavigateImpl() {
  navigateImpl = (url: string) => {
    window.location.href = url;
  };
}

// 指定URLへナビゲーションする
export function navigateTo(url: string) {
  navigateImpl(url);
}
