# tier3 開発者体験設計

## 目的

tier3 開発者 (初級〜中級) が k1s0 で開発を開始する際の学習負荷を、tier1 が構造的に引き受けるための施策を定義する。既存のオンボーディング戦略 ([`07_tier2オンボーディング戦略.md`](07_tier2オンボーディング戦略.md) と開発者アダプション戦略 ([`12_開発者アダプション戦略.md`](12_開発者アダプション戦略.md) は tier2 向けに設計されており、tier3 向けの導線が未整備である。本資料はこのギャップを埋める。

---

## 1. 背景となる課題

### 1.1 設計上の意図と現実のギャップ

概念アーキテクチャ ([`../../01_アーキテクチャ/01_基礎/00_概念アーキテクチャ.md`](../../01_アーキテクチャ/01_基礎/00_概念アーキテクチャ.md)) では「tier の番号が上がるほど、必要な開発者レベルが下がる」と定義している。tier3 は「初級〜中級」であり、参入障壁が最も低いことで開発リソースをスケールさせやすくなるという設計意図がある。

しかし現状の設計では、tier3 開発者にも以下の学習が暗黙的に要求される。

| 学習項目 | tier3 開発者のバックグラウンドとの差分 |
|---|---|
| gRPC / Protobuf | REST + JSON からの移行。`.proto` ファイルの記法・`buf` CLI の操作が必要 |
| tier1 公開 API 11 種 | 各 API の使い方・エラーハンドリング方針が異なる |
| グレースフルデグラデーション 3 分類 | 観測系 / 参照系 / 状態系のエラー方針の理解 |
| 構造化ログの書き方 | `k1s0.Log.Info(...)` の引数規約 |
| OTel 計装 | `k1s0.Telemetry.*` による span 生成 |
| 監査ログ | `k1s0.Audit.Record(...)` の呼び出し |
| イベント駆動 / PubSub | 同期 API 呼び出しからの概念移行 |
| 結果整合性 / Saga | ACID トランザクションからの概念移行 |

これらは tier2 のオンボーディング (L1-L5) で段階的に解消される設計だが、tier3 は tier2 とは異なるバックグラウンド・責務・利用パターンを持つため、**tier2 向け教材の流用では不十分**である。

### 1.2 tier3 開発者の典型的なバックグラウンド

| 項目 | 想定 |
|---|---|
| 言語経験 | C# (ASP.NET) / HTML + CSS + JavaScript (React は一部) |
| 通信プロトコル | REST + JSON。gRPC は未経験 |
| データベース | SQL Server / Entity Framework |
| インフラ知識 | IIS / Windows Server。Docker / k8s は未経験 |
| 主な関心事 | 画面設計、ユーザー体験、業務要件の実現 |

### 1.3 解決の方向性

tier3 開発者の学習負荷を tier1 が吸収する方法は 1 つ: **tier3 から見える接触面を減らし、既知の概念の延長で使えるようにする**。具体的には以下の 7 施策で構成する。

| # | 施策 | 削減される学習項目 |
|---|---|---|
| 1 | tier3 専用 REST ゲートウェイ | gRPC / Protobuf / buf |
| 2 | フレームワーク統合 SDK | tier1 API の生の呼び出し方 |
| 3 | tier3 専用オンボーディングパス | tier2 向け L1-L5 の全体 |
| 4 | tier2 モックサーバー自動生成 | tier2 完成待ちの待機 |
| 5 | 業務パターン別テンプレート | ゼロからの画面構築 |
| 6 | 自動計装による暗黙の観測性 | Log / Telemetry / Audit |
| 7 | エラー体験のデフォルト設計 | グレースフルデグラデーション 3 分類 |

---

## 2. tier3 専用 REST ゲートウェイ

### 2.1 課題

tier1 公開 API は gRPC + Protobuf を正の契約としている ([`06_クライアントライブラリ配布戦略.md`](../02_API契約/06_クライアントライブラリ配布戦略.md)。tier2 開発者にとっては型安全性と後方互換検証 (`buf breaking`) の恩恵があるが、tier3 開発者には以下の問題がある。

| 問題 | 影響 |
|---|---|
| `.proto` ファイルの記法を覚える必要がある | REST + JSON のバックグラウンドとの断絶 |
| `buf` CLI のインストールと操作 | ツールチェーンの学習コスト |
| Protobuf のバイナリシリアライゼーション | `curl` でのデバッグが困難 |
| gRPC クライアントの DI 登録 | フレームワーク固有のボイラープレート |

### 2.2 解決策

tier1 が tier3 専用の REST + JSON ゲートウェイを提供する。

```
tier3 (REST + JSON)
  → tier1 REST ゲートウェイ (自動変換)
    → tier1 内部 gRPC サービス群
```

| 要素 | tier2 向け (既存) | tier3 向け (新設) |
|---|---|---|
| 通信プロトコル | gRPC + Protobuf | REST + JSON |
| API 定義の正 | `.proto` ファイル | `.proto` から自動生成された OpenAPI スキーマ |
| クライアント生成 | `buf generate` | OpenAPI Generator で各言語クライアントを生成 |
| ドキュメント | Backstage + protoc-gen-doc | Swagger UI |
| デバッグ | gRPC リフレクション / grpcurl | `curl` / ブラウザ |

### 2.3 実装方針

REST ゲートウェイは tier1 Go ファサードに併設する。`.proto` から REST エンドポイントを自動生成する仕組みとして `grpc-gateway` を使用する。

| コンポーネント | 言語 | 担当 |
|---|---|---|
| REST → gRPC 変換プロキシ | Go | tier1 Go ファサードに組み込み |
| OpenAPI スキーマ生成 | — | `protoc-gen-openapiv2` で `.proto` から自動生成 |
| Swagger UI | — | 静的ファイルとして tier1 namespace に配置 |

`.proto` を Single Source of Truth とする原則は変わらない。REST ゲートウェイは `.proto` の派生物であり、手書きの REST 定義は作らない。

### 2.4 tier2 / tier3 の使い分け

| 利用者 | 使用するインターフェース | 理由 |
|---|---|---|
| tier2 | gRPC クライアントライブラリ (既存) | 型安全性・パフォーマンス・後方互換検証が必要 |
| tier3 (サーバーサイド) | REST + JSON | 学習コスト削減が最優先。パフォーマンス差は tier3 のユースケースで問題にならない |
| tier3 (フロントエンド) | フレームワーク統合 SDK (後述 3 章) が内部で REST を呼ぶ | tier3 開発者は REST の存在すら意識しない |

REST ゲートウェイは tier3 の参入障壁を下げるための手段であり、gRPC を否定するものではない。tier3 開発者がスキルアップして gRPC を直接使いたくなった場合、その移行を妨げない。

---

## 3. フレームワーク統合 SDK

### 3.1 課題

現在の tier1 公開 API (`k1s0.Log.Info(...)`, `k1s0.State.GetAsync(...)`) は言語レベルの汎用ラッパーである。tier3 開発者が日常的に使う React や ASP.NET のイディオムとは乖離しており、「新しい API の使い方を覚える」学習コストが発生する。

### 3.2 解決策

tier1 がフレームワーク固有の統合パッケージを提供する。tier3 開発者は **既に知っている概念の延長** で tier1 を使える。

### 3.3 React (TSX) 向け: `@k1s0/react`

| 提供物 | 機能 | 対応する tier1 API |
|---|---|---|
| `<K1s0Provider>` | 全 Hook の初期化を 1 箇所で完結 | DI 統合層 |
| `useK1s0Auth()` | ログイン状態・ユーザー情報・ロール確認 | `k1s0.Auth` |
| `useK1s0State(store, key)` | State の読み書き (SWR 風キャッシュ付き) | `k1s0.State` |
| `useK1s0Feature(flagName)` | Feature Flag の評価 | `k1s0.Feature` |
| `useK1s0Settings(key)` | 設定値の取得 | `k1s0.Settings` |
| `useK1s0Log()` | 任意の追加ログ出力 (自動計装で十分な場合は不要) | `k1s0.Log` |
| `<K1s0TrackedButton>` | ボタンクリックの操作ログ自動記録 | `k1s0.Audit` (暗黙) |
| `<K1s0TrackedForm>` | フォーム送信の操作ログ自動記録 | `k1s0.Audit` (暗黙) |
| `<K1s0ErrorBoundary>` | エラー発生時のフォールバック UI + 自動通知 | `k1s0.Log` / `k1s0.Telemetry` (暗黙) |

使用例:

```tsx
// tier3 開発者が書くコード — tier1 API を直接意識しない
import { useK1s0Auth, useK1s0State } from '@k1s0/react';

function OrderDetail({ orderId }) {
  const { user, hasRole } = useK1s0Auth();
  const { data: order, isLoading, error } = useK1s0State('orders', orderId);

  if (isLoading) return <Spinner />;
  return (
    <div>
      <h1>{order.title}</h1>
      {hasRole('admin') && <AdminPanel order={order} />}
    </div>
  );
}
```

### 3.4 C# ASP.NET 向け: `K1s0.AspNet`

| 提供物 | 機能 | 対応する tier1 API |
|---|---|---|
| `AddK1s0AspNet()` | 1 行で全 middleware・フィルタ・DI を登録 | DI 統合層 |
| `[K1s0Authorize("role")]` 属性 | ロール認証を属性で宣言 | `k1s0.Auth` |
| `K1s0LogActionFilter` | API リクエスト / レスポンスの自動ログ | `k1s0.Log` (暗黙) |
| `K1s0ExceptionFilter` | エラーハンドリングとエラーレスポンス生成の統一 | `k1s0.Log` / `k1s0.Telemetry` (暗黙) |
| `K1s0HealthCheck` | ASP.NET HealthCheck に tier1 診断を統合 | `k1s0.Diag` |
| `IK1s0StateService<T>` | Entity Framework 風のインターフェースで State を操作 | `k1s0.State` |
| `ILogger<T>` 互換プロバイダ | 既存の `ILogger` をそのまま使える | `k1s0.Log` |

使用例:

```csharp
// tier3 開発者が書くコード — ASP.NET の世界観で完結
[K1s0Authorize("admin")]
public class OrderController(IK1s0StateService<Order> orders) : ControllerBase
{
    [HttpGet("{orderId}")]
    public async Task<Order> Get(string orderId)
        => await orders.GetAsync(orderId);
}
// ログ / トレース / 監査は ActionFilter が自動処理。明示的な呼び出し不要
```

### 3.5 MAUI 向け: `K1s0.Maui`

MAUI アプリケーションは PWA と異なりオフライン動作が求められる場面がある。`K1s0.Maui` は以下を追加で提供する。

| 提供物 | 機能 |
|---|---|
| `K1s0OfflineState` | オフライン時にローカル SQLite にキャッシュし、オンライン復帰時に同期 |
| `K1s0ConnectivityService` | ネットワーク状態の監視とオフライン/オンラインの自動切替 |

### 3.6 フレームワーク SDK の保守コスト

| 懸念 | 対処 |
|---|---|
| 3 フレームワーク分の保守が tier1 に集中する | 各 SDK は「薄いラッパー + REST 呼び出し」で構成する。ビジネスロジックを持たないため変更頻度は低い |
| フレームワークのメジャーバージョン更新 | React / ASP.NET / MAUI の LTS に追従する方針とし、追従責任は tier1 チームが持つ |
| 自動計装のカバレッジ不足 | 初期リリースでは最小構成 (認証 / ログ / エラー) に絞り、利用フィードバックで拡張する |

---

## 4. tier3 専用オンボーディングパス

### 4.1 課題

既存のオンボーディング戦略 (L1-L5) は tier2 開発者向けに設計されている。tier3 開発者がこのパスを辿ると、以下の問題が起きる。

| 問題 | 影響 |
|---|---|
| L1 で gRPC / Protobuf が視界に入る | tier2 向けクライアントライブラリを使う前提で説明されている |
| L3 で PubSub / Service.Invoke が登場する | tier3 の責務 (UI 実装) には不要 |
| L4 で Workflow / Decision / Audit が登場する | tier3 が直接使う場面がない |
| サンプルサービスが tier2 の業務ロジック | tier3 は画面を作りたいのに、バックエンド処理のサンプルを読まされる |

### 4.2 tier3 専用レベル定義

tier3 向けに独立した 3 段階のオンボーディングを定義する。tier2 の L1-L5 とは別の学習パスとする。

| レベル | 学ぶこと | 使用するフレームワーク SDK | 目安期間 |
|---|---|---|---|
| **T1: 画面を作る** | SSO ログイン、データの取得と表示、ローカル環境の起動 | `useK1s0Auth` / `useK1s0State` (React) or `[K1s0Authorize]` / `IK1s0StateService` (C#) | 半日 |
| **T2: 業務画面を作る** | tier2 API の呼び出し (REST)、tier2 仕様への準拠、Feature Flag による段階公開 | + tier2 REST API 呼び出し + `useK1s0Feature` | 1-2 日 |
| **T3: 応用** | 設定同期、エラー発生時のユーザー体験設計、Grafana の基本操作 | + `useK1s0Settings` + `onError` / `onDegraded` コールバック | 随時 |

### 4.3 tier3 開発者に見せない API

tier3 オンボーディングでは、以下の tier1 API を**意図的に導線から除外**する。

| API | 除外理由 | tier3 が同等機能を利用する手段 |
|---|---|---|
| `k1s0.PubSub` | tier3 はイベントを発行しない (データアーキテクチャの制約)。購読が必要な場合も tier2 経由 | tier2 が提供する REST API を呼ぶ |
| `k1s0.Workflow` | Saga パターンは tier2 の責務 | tier2 が提供するワークフロー起動 API を呼ぶ |
| `k1s0.Decision` | ビジネスルール評価は tier2 の責務 | tier2 が評価結果を API で返す |
| `k1s0.Audit` | 監査ログはフレームワーク SDK の自動計装が処理 | 明示的な呼び出し不要 |
| `k1s0.Telemetry` | OTel 計装はフレームワーク SDK の自動計装が処理 | 明示的な span 生成不要 |
| `k1s0.Log` | 構造化ログはフレームワーク SDK の自動計装が処理 | 追加ログが必要な場合のみ `useK1s0Log` / `ILogger<T>` を使う |
| `k1s0.Secrets` | tier3 がシークレットに直接アクセスする場面はない | tier2 がシークレットを利用し、結果を API で返す |

tier3 が実質的に意識する API は **Auth / State / Feature / Settings の 4 種**。しかもフレームワーク SDK 経由で使うため、tier1 API の生の呼び出し方を学ぶ必要がない。

### 4.4 サンプルアプリケーションの配置

```
examples/
├── tier2-reference-service/      # 既存: tier2 向け全 API 網羅
├── onboarding-L1/                # 既存: tier2 向け L1
├── ...
├── tier3-onboarding-T1/          # 新設: React + useK1s0Auth + useK1s0State
├── tier3-onboarding-T2/          # 新設: T1 + tier2 REST API 呼び出し + Feature Flag
├── tier3-onboarding-T3/          # 新設: T2 + Settings + エラーハンドリング
└── tier3-reference-app/          # 新設: 全パターン網羅の模範 tier3 アプリ
```

各サンプルは画面を持つ Web アプリケーションとして実装する。tier2 のサンプル (バックエンドサービス) とは異なり、ブラウザで動作を確認できる形式とする。

### 4.5 ファーストコンタクト体験の tier3 版

[`12_開発者アダプション戦略.md`](12_開発者アダプション戦略.md) のファーストコンタクト体験 (60 分) を tier3 向けに再構成する。

| 時間 | 操作 | ゴール |
|---|---|---|
| 0-5 分 | `k1s0 scaffold --template crud --service-name hello --language react` | 画面付きプロジェクトが生成されている |
| 5-10 分 | `k1s0 dev start` | ブラウザでログイン画面が表示される |
| 10-15 分 | テストユーザーでログイン | SSO が動作し、一覧画面が表示される |
| 15-30 分 | 一覧画面のカラムを 1 つ追加する | ファイル保存 → 数秒で自動反映 → ブラウザで確認 |
| 30-45 分 | 新しいデータ項目を追加し、フォームから登録する | `useK1s0State` でデータが保存・表示される |
| 45-55 分 | Feature Flag で新カラムの表示/非表示を切り替える | `useK1s0Feature` の効果を体感する |
| 55-60 分 | `k1s0 dev status` で全体を確認 | 「自分が書いた画面が動いている」達成感 |

設計上の制約: この 60 分で gRPC、Protobuf、Kafka、Saga の単語を**一度も見せない**。

---

## 5. tier2 モックサーバー自動生成

### 5.1 課題

tier3 は tier2 の API に依存する ([`../../01_アーキテクチャ/01_基礎/01_レイヤ構成と責務.md`](../../01_アーキテクチャ/01_基礎/01_レイヤ構成と責務.md))。tier2 の実装が先行しないと tier3 は開発を開始できない。これは tier3 の開発リードタイムに直接影響する。

### 5.2 解決策

tier2 が定義した API 仕様 (`.proto` or OpenAPI) から、tier1 の雛形 CLI がモックサーバーを自動生成する。

```bash
k1s0 scaffold --mock-from src/tier2/order-service/api.proto \
              --output mocks/order-service
```

### 5.3 モックサーバーの機能

| 機能 | 内容 |
|---|---|
| 静的レスポンス | `.proto` / OpenAPI の型情報からダミーデータを自動生成 |
| シナリオ切替 | `mocks/scenarios/` に複数シナリオの JSON を配置し、`k1s0 dev` で切替可能 |
| 遅延シミュレーション | `--latency 200ms` で tier2 API の応答遅延を再現 |
| エラーシミュレーション | `--error-rate 0.1` で 10% の確率でエラーを返す |
| リクエスト記録 | モックサーバーに送られたリクエストをログに記録し、デバッグを支援 |

### 5.4 `k1s0 dev` との統合

```bash
k1s0 dev start --with-mocks
```

`--with-mocks` を指定すると、tier2 サービスの代わりにモックサーバーが起動する。tier2 の実装が完了した後は、`--with-mocks` を外すだけで実サービスに切り替わる。

### 5.5 モック定義の管理

| 管理方針 | 内容 |
|---|---|
| 正の定義 | tier2 が管理する `.proto` / OpenAPI がモックの元となる |
| モック固有データ | `mocks/scenarios/` ディレクトリに tier3 チームが管理 |
| 同期 | tier2 の API 定義が変更された場合、Renovate が tier3 リポジトリにモック再生成の PR を自動作成 |

---

## 6. 業務パターン別テンプレート

### 6.1 課題

現在の雛形生成 CLI はサービスの骨格 (Dockerfile / k8s manifest / DI 初期化) を生成するが、tier3 が多用する画面パターンのテンプレートは提供しない。tier3 開発者はゼロから画面を構築する必要がある。

### 6.2 解決策

tier3 の典型的な業務パターンに対応したテンプレートを雛形生成 CLI に追加する。

```bash
k1s0 scaffold --template list-detail --service-name asset-management --language react
```

### 6.3 テンプレート一覧

| テンプレート名 | 生成される画面 | 組み込み済みの機能 |
|---|---|---|
| `crud` | 一覧 + 登録 + 編集 + 削除 | Auth (ロール制御) / State (データ保持) |
| `list-detail` | 一覧画面 + 詳細画面 + 検索 | Auth / State (検索条件保持) |
| `form-wizard` | 入力 → 確認 → 完了のウィザード | Auth / State (下書き保存) |
| `dashboard` | KPI 集計 + チャート表示 | Auth / Settings (表示設定) / Feature (新ウィジェット公開) |
| `approval-flow` | 申請 → 承認のワークフロー画面 | Auth / tier2 Workflow API 呼び出し (REST) |

### 6.4 テンプレートの構成

各テンプレートは以下を含む。

| 構成物 | 内容 | tier3 が編集するか |
|---|---|---|
| 画面コンポーネント | React コンポーネント or Razor Pages | はい (業務に合わせてカスタマイズ) |
| デザイントークン | tier2 が定義した色 / フォント / スペーシング | いいえ (tier2 仕様に準拠) |
| フレームワーク SDK 統合 | `@k1s0/react` の Hooks が配置済み | いいえ (自動生成のまま) |
| tier2 API 呼び出し | モックサーバー対応の REST クライアント | はい (呼び出し先の変更) |
| ローカル起動設定 | `Tiltfile` / `k1s0.config.yaml` | いいえ |

tier3 開発者の作業は「テンプレートのカスタマイズ」から始まる。ゼロからの構築が不要になる。

---

## 7. 自動計装による暗黙の観測性

### 7.1 課題

`k1s0.Log.Info(...)` / `k1s0.Telemetry.*` / `k1s0.Audit.Record(...)` の呼び出しを tier3 開発者に要求すると、「業務ロジックに集中」([`../../01_アーキテクチャ/01_基礎/00_概念アーキテクチャ.md`](../../01_アーキテクチャ/01_基礎/00_概念アーキテクチャ.md) セクション 2) の約束が破られる。

### 7.2 解決策

フレームワーク SDK が全ての観測性を自動計装する。tier3 開発者がログ / トレース / メトリクスのコードを一行も書かなくても、十分な可観測性が確保される状態を作る。

### 7.3 自動計装の対象

| 観測対象 | 自動計装の方法 | tier3 開発者の作業 |
|---|---|---|
| API リクエスト / レスポンス | ASP.NET `K1s0LogActionFilter` / Express middleware | なし |
| エラー発生 | `K1s0ExceptionFilter` / `<K1s0ErrorBoundary>` が自動記録 + CorrelationId 付与 | なし |
| tier2 API 呼び出し | REST クライアントラッパーが自動計装 (リクエスト先・所要時間・ステータスを記録) | なし |
| ユーザー操作 | `<K1s0TrackedButton>` / `<K1s0TrackedForm>` が操作ログを自動記録 | 既存の HTML 要素を置き換えるだけ |
| フロントエンドパフォーマンス | Web Vitals (LCP / FID / CLS) を `<K1s0Provider>` が自動収集 | なし |
| ページ遷移 | React Router の遷移を `<K1s0Provider>` がフック | なし |

### 7.4 任意の追加ログ

自動計装で十分だが、業務固有のログを追加したい場合はフレームワークの標準ロガーを使える。

**React**:

```tsx
import { useK1s0Log } from '@k1s0/react';
const log = useK1s0Log();
log.info('カートに追加', { productId, quantity });
```

**C# (ILogger 互換)**:

```csharp
public class OrderController(ILogger<OrderController> logger) : ControllerBase
{
    public async Task<IActionResult> AddToCart(string productId, int quantity)
    {
        logger.LogInformation("カートに追加 {ProductId} x{Quantity}", productId, quantity);
        // K1s0LogProvider が構造化ログに自動変換
    }
}
```

C# の場合、`ILogger<T>` はフレームワーク SDK が tier1 `k1s0.Log` にブリッジする。tier3 開発者は `ILogger` の使い方だけ知っていれば良い。

### 7.5 自動計装と手動ログの使い分け

| 場面 | 推奨 |
|---|---|
| API の呼び出しと応答の記録 | 自動計装に任せる (tier3 は何もしない) |
| エラーの記録 | 自動計装に任せる |
| 業務イベントの記録 (「カートに追加した」等) | `useK1s0Log` / `ILogger` で手動追加 |
| パフォーマンス計測 | 自動計装に任せる |

---

## 8. エラー体験のデフォルト設計

### 8.1 課題

グレースフルデグラデーション ([`../../01_アーキテクチャ/02_可用性と信頼性/03_グレースフルデグラデーション.md`](../../01_アーキテクチャ/02_可用性と信頼性/03_グレースフルデグラデーション.md)) では、tier1 API を 3 分類 (観測系 / 参照系 / 状態系) し、それぞれ異なるエラーハンドリング方針を定義している。この分類と方針を tier3 開発者が理解して実装するのは負荷が高い。

### 8.2 解決策

フレームワーク SDK がエラーハンドリングのデフォルト動作を提供する。tier3 開発者はエラーハンドリングコードを一行も書かなくても、合理的なユーザー体験が提供される。

### 8.3 デフォルト動作

| エラー分類 | フレームワーク SDK のデフォルト動作 | UI への表示 |
|---|---|---|
| 観測系 (Log / Telemetry / Audit) エラー | 完全に握りつぶす | tier3 に通知しない。UI への影響なし |
| 参照系 (Auth / Secrets / Decision / Settings / Feature) エラー | キャッシュ値で自動継続 | 控えめなバナー「最新でない可能性があります」 |
| 状態系 (State / PubSub / Workflow / Service.Invoke) エラー | トースト通知 + 自動リトライ提案 | 「保存に失敗しました。もう一度お試しください」 |
| tier2 API エラー (4xx) | ユーザーフィードバック | 「入力内容を確認してください」(バリデーションエラー) |
| tier2 API エラー (5xx) | エラー画面への遷移 | 「システムエラーが発生しました。しばらくしてからお試しください」 |

### 8.4 カスタマイズポイント

デフォルト動作で十分な場合が大半だが、業務要件に応じてカスタマイズしたい場合のコールバックを提供する。

**React**:

```tsx
<K1s0Provider
  onDegraded={(api) => {
    // 参照系 API がキャッシュで継続している場合のカスタム処理
  }}
  onError={(api, error) => {
    // 状態系 API がエラーを返した場合のカスタム処理
  }}
>
  <App />
</K1s0Provider>
```

**C# ASP.NET**:

```csharp
builder.Services.AddK1s0AspNet(options =>
{
    options.OnStateError = (context, error) =>
    {
        // 状態系 API エラー時のカスタム処理
    };
});
```

### 8.5 tier3 向けエラーメッセージの日本語化

フレームワーク SDK が返すデフォルトのエラーメッセージは日本語とする。tier1 のエラーコード (`K1S0_STATE_UNAVAILABLE` 等) を tier3 開発者やエンドユーザーに直接見せない。

| tier1 エラーコード | SDK のデフォルト日本語メッセージ |
|---|---|
| `K1S0_STATE_UNAVAILABLE` | 「データの保存に失敗しました。もう一度お試しください」 |
| `K1S0_AUTH_EXPIRED` | 「ログインの有効期限が切れました。再度ログインしてください」 |
| `K1S0_SERVICE_UNAVAILABLE` | 「システムが一時的に利用できません。しばらくしてからお試しください」 |
| `K1S0_STATE_CONFLICT` | 「他のユーザーが同時に更新しました。画面を再読み込みしてください」 |

---

## 9. 全施策適用後の tier3 開発者の学習マップ

### 9.1 Before / After

| 学習項目 | Before (現状) | After (全施策適用後) |
|---|---|---|
| gRPC / Protobuf | 必須 | 不要 (REST + JSON) |
| tier1 API 11 種の生の呼び出し方 | 必須 | 不要 (フレームワーク SDK 経由で 4 種のみ) |
| 構造化ログの書き方 | 必須 | 不要 (自動計装) |
| OTel 計装 | 必須 | 不要 (自動計装) |
| 監査ログ | 必須 | 不要 (自動計装) |
| エラーハンドリング方針 (3 分類) | 必須 | 不要 (SDK がデフォルト処理) |
| イベント駆動 / PubSub | 必須 | 不要 (tier2 経由) |
| Saga / Workflow | 必須 | 不要 (tier2 経由) |
| ビジネスルール評価 | 必須 | 不要 (tier2 経由) |
| tier2 の API 完成待ち | ブロッキング | 不要 (モックサーバー) |
| ゼロからの画面構築 | 必須 | 不要 (業務パターンテンプレート) |

### 9.2 tier3 開発者が実際に覚えること

1. `k1s0 scaffold --template <pattern>` でプロジェクトを作る
2. `k1s0 dev start` でローカル環境を起動する
3. React の Hooks (`useK1s0Auth` / `useK1s0State` / `useK1s0Feature`) または ASP.NET の属性・DI で認証・データ取得を行う
4. tier2 が公開する REST API を呼ぶ
5. tier2 が定義した画面仕様に従って UI をカスタマイズする

---

## 10. 段階的導入

| フェーズ | 施策 | 備考 |
|---|---|---|
| Phase 2 開始前 | tier3 専用オンボーディングパス (T1-T3) の設計書整備 | tier3 開発者参画の前提条件 |
| Phase 2 前半 | REST ゲートウェイ (grpc-gateway) の実装 | tier1 Go ファサードに併設 |
| Phase 2 前半 | `@k1s0/react` の最小構成 (`K1s0Provider` / `useK1s0Auth` / `useK1s0State`) | T1 オンボーディングの基盤 |
| Phase 2 前半 | `K1s0.AspNet` の最小構成 (`AddK1s0AspNet` / `[K1s0Authorize]` / `K1s0LogActionFilter`) | T1 オンボーディングの基盤 |
| Phase 2 前半 | 自動計装 (ログ / トレース / エラー) の実装 | フレームワーク SDK に組み込み |
| Phase 2 前半 | エラー体験のデフォルト設計実装 | フレームワーク SDK に組み込み |
| Phase 2 前半 | tier3 ファーストコンタクト体験 (60 分) の教材整備 | T1 サンプルアプリケーションを含む |
| Phase 2 後半 | tier2 モックサーバー自動生成 CLI | tier3 の並行開発を可能にする |
| Phase 2 後半 | `useK1s0Feature` / `useK1s0Settings` の追加 | T2-T3 オンボーディングの基盤 |
| Phase 2 後半 | `crud` / `list-detail` テンプレートの提供 | 最小 2 パターンから開始 |
| Phase 3 | `form-wizard` / `dashboard` / `approval-flow` テンプレートの追加 | 利用フィードバックに基づいて拡張 |
| Phase 3 | `K1s0.Maui` の提供 | ネイティブアプリ開発の本格化に合わせる |
| Phase 3 | `<K1s0TrackedButton>` / `<K1s0TrackedForm>` の追加 | 操作ログの自動記録 |

---

## 11. 既存ドキュメントとの関係

本資料は以下の既存設計と**矛盾しない**。

| 既存設計 | 本資料との関係 |
|---|---|
| Dapr 隠蔽方針 | REST ゲートウェイを追加しても、Dapr は引き続き tier1 内部に隠蔽される |
| API 設計原則 (多層防御) | REST ゲートウェイでも CI ガード・Kyverno は有効。REST 経由でも禁止パターンは検出される |
| クライアントライブラリ配布戦略 | `.proto` を正の定義とする原則は変わらない。REST / OpenAPI は `.proto` の派生物 |
| 依存ルール | tier3 → tier2 → tier1 の方向は変わらない。フレームワーク SDK は tier1 公開 API のラッパーであり、新しい依存方向は生じない |
| グレースフルデグラデーション | 3 分類と縮退方針は変わらない。フレームワーク SDK がこれをデフォルト実装として内包する |
| tier2 オンボーディング戦略 | tier2 向け L1-L5 は存続する。本資料は tier3 向けの別パス (T1-T3) を追加する |

---

## 関連ドキュメント

- [`00_tier1のスコープ.md`](../01_設計の核/00_tier1のスコープ.md) — tier1 公開 API の一覧
- [`03_API設計原則.md`](../02_API契約/03_API設計原則.md) — 多層防御と雛形生成 CLI
- [`06_クライアントライブラリ配布戦略.md`](../02_API契約/06_クライアントライブラリ配布戦略.md) — Protobuf 自動生成と REST ゲートウェイの関係
- [`07_tier2オンボーディング戦略.md`](07_tier2オンボーディング戦略.md) — tier2 向け L1-L5 (本資料の T1-T3 とは別パス)
- [`11_ローカル開発CLI設計.md`](11_ローカル開発CLI設計.md) — `k1s0 dev` CLI との統合
- [`12_開発者アダプション戦略.md`](12_開発者アダプション戦略.md) — ファーストコンタクト体験の tier3 版
- [`../../01_アーキテクチャ/01_基礎/00_概念アーキテクチャ.md`](../../01_アーキテクチャ/01_基礎/00_概念アーキテクチャ.md) — 設計信念と tier3 の位置付け
- [`../../01_アーキテクチャ/01_基礎/01_レイヤ構成と責務.md`](../../01_アーキテクチャ/01_基礎/01_レイヤ構成と責務.md) — tier3 の責務と制約
- [`../../01_アーキテクチャ/02_可用性と信頼性/03_グレースフルデグラデーション.md`](../../01_アーキテクチャ/02_可用性と信頼性/03_グレースフルデグラデーション.md) — エラーハンドリング 3 分類
- [`../../01_アーキテクチャ/04_非機能とデータ/02_データアーキテクチャ.md`](../../01_アーキテクチャ/04_非機能とデータ/02_データアーキテクチャ.md) — tier3 のデータ制約
- [`../../04_CICDと配信/04_ローカル開発環境.md`](../../04_CICDと配信/04_ローカル開発環境.md) — Tilt ローカル環境との統合
