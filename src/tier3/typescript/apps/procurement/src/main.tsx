// k1s0 発注 tier3 SPA エントリポイント（React 19 + Vite）
// 調達ドメイン UI（発注 / 検収 / 受注 / 出荷指示）のエントリポイント
// cookie scope: __Host-procurement-access-token（deployment 固有の cookie 分離）
// createRoot + StrictMode で React 19 の concurrent モードを有効化する

// React の createRoot API をインポートする
import { createRoot } from "react-dom/client";
// React の StrictMode をインポートする
import { StrictMode } from "react";
// アプリケーションルートコンポーネントをインポートする
import { App } from "./App.js";

// DOM から root 要素を取得する
const rootElement = document.getElementById("root");
// root 要素が存在しない場合は例外を投げる（開発者向けエラーメッセージ）
if (rootElement === null) {
  // 開発者向けエラーメッセージを設定する
  throw new Error(
    'Root element (#root) not found. index.html に <div id="root"></div> が必要です。',
  );
}

// document の lang 属性を日本語に設定する（WCAG 2.1 a11y: スクリーンリーダー向け）
document.documentElement.lang = "ja-JP";

// React 19 の createRoot API で root を生成する
const root = createRoot(rootElement);
// StrictMode で App をレンダリングする（開発時の副作用二重呼び出し検知を有効化する）
root.render(
  // StrictMode: React の開発ヘルパー（本番ビルドでは無効化される）
  <StrictMode>
    {/* 発注 tier3 SPA のルートコンポーネントをマウントする */}
    <App />
  </StrictMode>,
);
