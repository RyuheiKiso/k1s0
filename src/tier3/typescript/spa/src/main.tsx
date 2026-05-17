// k1s0 tier3 SPA エントリポイント
// 4 layer client state の初期化と i18n 初期化を行う

import { createInitialState } from "@k1s0/tier3-state";
import { getLocaleConfig, isSupportedLocale } from "@k1s0/tier3-i18n";
import type { ClientState } from "@k1s0/tier3-state";
import type { SupportedLocale } from "@k1s0/tier3-i18n";

// SPA の初期化エントリポイント
function initializeApp(): void {
  // 4 layer client state を初期化する
  const initialState: ClientState<unknown> = createInitialState();
  // ブラウザの locale 設定を取得する
  const browserLocale = navigator.language;
  // サポート locale かどうかを確認し、サポート外の場合は ja-JP にフォールバックする
  const locale: SupportedLocale = isSupportedLocale(browserLocale) ? browserLocale : "ja-JP";
  // locale 設定を取得する
  const localeConfig = getLocaleConfig(locale);

  // root element を取得する
  const rootElement = document.getElementById("root");
  if (rootElement === null) {
    // root element が存在しない場合はエラーを表示する
    throw new Error("Root element not found");
  }

  // 初期化完了を root element の data 属性に記録する
  rootElement.dataset["initialized"] = "true";
  rootElement.dataset["locale"] = locale;

  // 初期化ログ（本番では削除する）
  console.info("k1s0 tier3 SPA initialized", {
    locale: localeConfig.locale,
    stateLayerCount: Object.keys(initialState).length,
  });
}

// DOM 読み込み完了後に初期化する
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initializeApp);
} else {
  initializeApp();
}
