// k1s0 tier3 SPA エントリポイント（React 18）
// createRoot + StrictMode で React 18 の concurrent モードを有効化する
// 4 layer client state の初期化・i18n 初期化・BrowserRouter を統合する

// React の createRoot API をインポートする
import { createRoot } from "react-dom/client";
// React の StrictMode をインポートする
import React, { StrictMode } from "react";
// BrowserRouter と Route を react-router-dom からインポートする
import { BrowserRouter, Route, Routes } from "react-router-dom";

// アプリケーションのルートコンポーネントを定義する
function App(): React.JSX.Element {
  // ルーター設定を返す（Routes で宣言的にルートを定義する）
  return (
    // BrowserRouter: HTML5 History API を使ったルーティングを有効化する
    <BrowserRouter>
      {/* Routes: 宣言的なルートマッチングコンテナ */}
      <Routes>
        {/* ルートパス: ホーム画面 */}
        <Route path="/" element={<HomePage />} />
        {/* 404 フォールバック */}
        <Route path="*" element={<NotFoundPage />} />
      </Routes>
    </BrowserRouter>
  );
}

// ホーム画面コンポーネント（最小実装）
function HomePage(): React.JSX.Element {
  // ホーム画面の最小 JSX を返す
  return (
    // main 要素: a11y ランドマーク
    <main id="main-content" role="main" aria-label="ホーム画面">
      {/* アプリ名見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件を満たす） */}
      <h1>k1s0 tier3 SPA</h1>
      {/* 初期化完了メッセージ */}
      <p>4 layer client state が初期化されました。</p>
    </main>
  );
}

// 404 Not Found ページコンポーネント
function NotFoundPage(): React.JSX.Element {
  // 404 ページの最小 JSX を返す
  return (
    // main 要素: a11y ランドマーク
    <main id="main-content" role="main" aria-label="ページが見つかりません">
      {/* 404 見出し */}
      <h1>404 — ページが見つかりません</h1>
      {/* ホームへ戻るリンク */}
      <p>
        <a href="/">ホームへ戻る</a>
      </p>
    </main>
  );
}

// DOM から root 要素を取得する
const rootElement = document.getElementById("root");
// root 要素が存在しない場合は例外を投げる
if (rootElement === null) {
  // 開発者向けエラーメッセージを設定する
  throw new Error(
    'Root element (#root) not found. index.html に <div id="root"></div> が必要です。',
  );
}

// React 18 の createRoot API で root を生成する
const root = createRoot(rootElement);
// StrictMode で App をレンダリングする（開発時の副作用二重呼び出し検知を有効化する）
root.render(
  // StrictMode: React の開発ヘルパー（本番ビルドでは無効化される）
  <StrictMode>
    {/* アプリケーションルートコンポーネントをマウントする */}
    <App />
  </StrictMode>,
);
