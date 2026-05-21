// k1s0 発注 tier3 SPA ルートコンポーネント
// 調達ドメイン UI（発注 / 検収 / 受注 / 出荷指示）のルーティングを定義する
// react-router-dom v6 の Routes / Route で宣言的にルートを管理する
// cookie scope: __Host-procurement-access-token（deployment 固有の cookie 分離）

// React をインポートする
import React from "react";
// BrowserRouter / Routes / Route / Navigate を react-router-dom からインポートする
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
// 発注一覧・発注詳細画面をインポートする（調達 scenario 6: 発注 / 検収）
import { PurchaseOrderScreen } from "./screens/PurchaseOrderScreen.js";
// 検収画面をインポートする（調達 scenario 6: 検収）
import { InspectionAcceptanceScreen } from "./screens/InspectionAcceptanceScreen.js";
// 受注 sub 画面をインポートする（調達 scenario 7: 受注 sub）
import { SalesOrderScreen } from "./screens/SalesOrderScreen.js";
// 出荷指示双方向確認画面をインポートする（調達 scenario 8: 出荷指示双方向確認）
import { ShippingScreen } from "./screens/ShippingScreen.js";

// ホーム画面コンポーネント（最小実装）
function HomePage(): React.JSX.Element {
  // ホーム画面の最小 JSX を返す
  return (
    // main 要素: WCAG 2.1 a11y ランドマーク
    <main id="main-content" role="main" aria-label="発注 tier3 ホーム">
      {/* アプリ名見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件を満たす） */}
      <h1>発注 tier3 — 調達ドメイン UI</h1>
      {/* 利用可能な機能の説明 */}
      <p>発注 / 検収 / 受注 / 出荷指示の業務 UI です。</p>
    </main>
  );
}

// ログイン画面コンポーネント（BFF 経由で認証する）
function LoginPage(): React.JSX.Element {
  // ログイン画面の最小 JSX を返す
  return (
    // main 要素: WCAG 2.1 a11y ランドマーク
    <main id="main-content" role="main" aria-label="ログイン">
      {/* ログイン見出し */}
      <h1>ログイン</h1>
      {/* BFF 経由で認証を開始するメッセージ */}
      <p>BFF auth-edge を経由して認証してください。</p>
      {/* BFF 認証開始リンク（__Host-procurement-access-token cookie を設定するエンドポイントへ遷移する） */}
      <a href="/auth/login" aria-label="BFF 経由でログインする">
        ログイン
      </a>
    </main>
  );
}

// 404 Not Found ページコンポーネント
function NotFoundPage(): React.JSX.Element {
  // 404 ページの最小 JSX を返す
  return (
    // main 要素: WCAG 2.1 a11y ランドマーク
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

// 発注 tier3 SPA のルートコンポーネント
export function App(): React.JSX.Element {
  // ルーター設定を返す（Routes で宣言的にルートを定義する）
  return (
    // BrowserRouter: HTML5 History API を使ったルーティングを有効化する
    <BrowserRouter>
      {/* Routes: 宣言的なルートマッチングコンテナ */}
      <Routes>
        {/* ルートパス: ホーム画面へリダイレクトする */}
        <Route path="/" element={<Navigate to="/purchase-orders" replace />} />
        {/* 発注一覧・詳細画面（調達 scenario 6: 発注 / 検収） */}
        <Route path="/purchase-orders" element={<PurchaseOrderScreen />} />
        {/* 検収画面（調達 scenario 6: 検収サブフロー） */}
        <Route path="/inspection-acceptance" element={<InspectionAcceptanceScreen />} />
        {/* 受注 sub 画面（調達 scenario 7: 受注 sub） */}
        <Route path="/sales-orders" element={<SalesOrderScreen />} />
        {/* 出荷指示双方向確認画面（調達 scenario 8） */}
        <Route path="/shipping" element={<ShippingScreen />} />
        {/* ログイン画面（auth guard の外側に置く） */}
        <Route path="/login" element={<LoginPage />} />
        {/* ホーム画面（index）*/}
        <Route path="/home" element={<HomePage />} />
        {/* 404 フォールバック */}
        <Route path="*" element={<NotFoundPage />} />
      </Routes>
    </BrowserRouter>
  );
}
