// k1s0 FA tier3 SPA ルートコンポーネント
// FA ドメイン UI（設備リモート操作 / ライン監視 / 生産指示 / SCADA / 計量）のルーティングを定義する
// react-router-dom v6 の Routes / Route で宣言的にルートを管理する
// cookie scope: __Host-fa-access-token（deployment 固有の cookie 分離）
// 既存 spa の PlantScreen（ライン監視）と FloorScreen（計量）を活用する

// React をインポートする
import React from "react";
// BrowserRouter / Routes / Route / Navigate を react-router-dom からインポートする
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
// 設備リモート操作画面をインポートする（FA scenario 1: 設備リモート操作）
import { EquipmentRemoteScreen } from "./screens/EquipmentRemoteScreen.js";
// ライン稼働監視 live tile 画面をインポートする（FA scenario 2: ライン稼働監視）
import { LineMonitoringScreen } from "./screens/LineMonitoringScreen.js";
// 生産指示・進捗実績画面をインポートする（FA scenario 3: 生産指示 / 進捗実績）
import { ProductionScheduleScreen } from "./screens/ProductionScheduleScreen.js";
// SCADA テレメトリ収集画面をインポートする（FA scenario 4: SCADA テレメトリ収集）
import { SCADAScreen } from "./screens/SCADAScreen.js";
// 計量装置連続データ画面をインポートする（FA scenario 5: 計量装置連続データ）
import { GaugeScreen } from "./screens/GaugeScreen.js";

// ホーム画面コンポーネント（最小実装）
function HomePage(): React.JSX.Element {
  // ホーム画面の最小 JSX を返す
  return (
    // main 要素: WCAG 2.1 a11y ランドマーク
    <main id="main-content" role="main" aria-label="FA tier3 ホーム">
      {/* アプリ名見出し（h1 は 1 ページに 1 つの WCAG 2.1 要件を満たす） */}
      <h1>FA tier3 — FA ドメイン UI</h1>
      {/* 利用可能な機能の説明 */}
      <p>設備リモート操作 / ライン監視 / 生産指示 / SCADA / 計量の業務 UI です。</p>
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
      {/* BFF 認証開始リンク（__Host-fa-access-token cookie を設定するエンドポイントへ遷移する） */}
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

// FA tier3 SPA のルートコンポーネント
export function App(): React.JSX.Element {
  // ルーター設定を返す（Routes で宣言的にルートを定義する）
  return (
    // BrowserRouter: HTML5 History API を使ったルーティングを有効化する
    <BrowserRouter>
      {/* Routes: 宣言的なルートマッチングコンテナ */}
      <Routes>
        {/* ルートパス: ライン監視画面へリダイレクトする */}
        <Route path="/" element={<Navigate to="/line-monitoring" replace />} />
        {/* 設備リモート操作画面（FA scenario 1） */}
        <Route path="/equipment-remote" element={<EquipmentRemoteScreen />} />
        {/* ライン稼働監視 live tile 画面（FA scenario 2） */}
        <Route path="/line-monitoring" element={<LineMonitoringScreen />} />
        {/* 生産指示・進捗実績画面（FA scenario 3） */}
        <Route path="/production-schedule" element={<ProductionScheduleScreen />} />
        {/* SCADA テレメトリ収集画面（FA scenario 4） */}
        <Route path="/scada" element={<SCADAScreen />} />
        {/* 計量装置連続データ画面（FA scenario 5） */}
        <Route path="/gauge" element={<GaugeScreen />} />
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
