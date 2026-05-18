---
id: arch.tier3.tier3_index
axis: tier3
phase: architecture
kind: index
status: published
version: 1.0.0
depends_on:
  - req.overview.provided_scope
  - req.overview.non_scope
  - arch.tier2.tier2_index
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
---

# tier3 設計方針 index

## 一文方針
- tier3 は個別業務に特化した Web SPA / デスクトップ exe / レガシー .NET Framework 形態の業務 UI 層。tier2 を一次消費し、tier1 / OSS の直接利用を 13 層強制機構で物理拒否、業務不変条件 / 整合性境界 / 認可規則は tier2 が所有して tier3 は「同じ業務処理を異なる操作シナリオで表現する場」として機能する。

## 1.0.0 出荷スコープ
- 1.0.0 で出荷する tier3 群: 製造業 pack を消費する代表 tier3 群（発注 / 検査 / FA など）
- 業界 pack の追加は tier2 が業界拡張モデルに従って行う
- 1.0.0 段階で「業界横断専用の tier3」「業界 pack を跨ぐ汎用 tier3」は出荷しない。tier3 は業界 pack を一つ消費する形態を既定とする

## 位置づけ
- プラットフォーム構成における最上位の業務アプリケーション層
- tier2 を一次消費し、業界横断 / 業界 pack の業務 API / 業務資産（Domain Event / Workflow / 決定表 / 業務マスタ / DB schema）をユーザに対し業務語彙で表現
- tier1 Library / Companion / OSS クライアントを直接利用しない
- 業務不変条件・整合性境界・認可規則を所有しない（これらは tier2 が所有、tier3 は tier2 の override 拡張点を通じて UI 層の業務固有差分のみを表現）

## tier3 が所有するもの
- 画面遷移シナリオ、フォームレイアウト、業務エラーの UI 表現、帳票レイアウト、業務イベントへのユーザ通知 UI、UI 上のキーボード操作 / ショートカット / IME 補助 / アクセシビリティ補助、UI 認可結果の反映

## tier3 が所有しないもの
- 業務資産（Domain Event schema / Workflow / 決定表 / 業務マスタ / DB schema / API 振る舞い）
- infra への直接アクセス
- tier1 Library / Companion / Gateway の直接消費（横断的共通要素 runtime に乗ることだけは許容）
- 業界横断概念 / 業界 pack 概念の独自定義

## 設計原則
- **3 アプリケーション形態**: Web SPA / デスクトップ exe（クロスプラットフォーム）/ レガシー .NET Framework 4.6.2+
- **言語スタック**: TypeScript（Web SPA） / C# (.NET 8+) または Rust + Tauri（exe） / .NET Framework 4.6.2+ + WinForms / WPF（レガシー）
- **4 layer client state**: Server Truth / Optimistic Local / Pending Queue / Draft（[クライアント状態適合仕様](../../04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md)）
- **業務管理機能との分離**: マスタ管理 / 監査検索 / 緊急対応 等の業務管理機能は Backstage プラグインに分離、tier3 業務 UI に載せない
- **defense-in-depth は 13 層**: tier1 / OSS 直接 import 禁止、業務管理 API import 禁止、tenant_id 受付禁止、認可 cache 禁止、PII 保管禁止、監査 emit 抑制経路禁止、翻訳キー未定義検出、公開 type 独自生成禁止、contract test 必須実行、a11y 自動検査、CSP / セキュリティヘッダ宣言、内部レジストリ唯一化、サプライチェーン

## 主要方針構成（34 方針）
- [01_責務](01_責務.md) — 個別業務 UI と業務シナリオ所有、tier3 が所有する / 所有しない
- [02_アプリケーション形態](02_アプリケーション形態.md) — Web SPA / exe / レガシーの 3 形態、形態選定基準、ハイブリッド構成
- [03_内部アーキテクチャ](03_内部アーキテクチャ.md) — レイヤ構成、状態管理、モジュール分割
- [04_状態管理](04_状態管理.md) — 4 layer の概要、適合仕様への投影
- [05_リアルタイム更新 UX](05_リアルタイム更新UX.md) — per-tab 16 subscription、HTTP/2 multiplex
- [06_端末オフラインデバイス](06_端末オフラインデバイス.md) — PWA、IndexedDB encrypted、Tauri sidecar
- [07_認証認可マルチテナント](07_認証認可マルチテナント.md) — Keycloak OIDC、BFF、tenant 識別子取得
- [08_観測可能性](08_観測可能性.md) — Web Vitals、RUM、tier3_ext SemConv
- [09_レガシー資産統合](09_レガシー資産統合.md) — .NET Framework + Companion、HTTP/1.1 + SSE
- [10_テスト方針](10_テスト方針.md) — Playwright E2E、a11y、Lighthouse、contract test
- [11_画面遷移ルーティング](11_画面遷移ルーティング.md) — SPA routing、認証 / 認可ガード、ディープリンク、未保存変更破棄警告
- [12_フォーム設計規約](12_フォーム設計規約.md) — Section / Step / Wizard、validation、IME 対応、自動保存 Draft
- [13_業務エラー UX](13_業務エラーUX.md) — BusinessConflict subtype 1:1 UI 分岐、recovery_hints、PII redaction
- [14_アクセシビリティ国際化](14_アクセシビリティ国際化.md) — WCAG 2.1 AA、axe-core、i18n、pseudolocalization
- [15_override 実装規約](15_override実装規約.md) — 拡張点 contract、4 言語 DI / 注入経路、invariants 抵触検出
- [16_互換性ポリシー](16_互換性ポリシー.md) — tier2 minor / major 追従、ブラウザ更新、bundle versioning
- [17_セキュリティアーキテクチャ](17_セキュリティアーキテクチャ.md) — クライアント側脅威モデル、CSP / SameSite / HttpOnly cookie
- [18_UI 技術スタック](18_UI技術スタック.md) — TypeScript + React / C# WPF / Tauri / .NET Framework + WinForms
- [19_デザインシステム](19_デザインシステム.md) — 共通 component package + design token + a11y
- [20_UX 原則](20_UX原則.md) — 業務語彙のみ / エラー復旧 action / オフライン透過 / a11y 標準
- [21_検索一覧 UX](21_検索一覧UX.md) — read model 経由 + cursor pagination + virtual scroll
- [22_状態遷移 UI](22_状態遷移UI.md) — 許可遷移のみ activate、4 言語等価強度
- [23_非同期処理 UX](23_非同期処理UX.md) — Long-running operation + progress indicator + cancel
- [24_通知 UI](24_通知UI.md) — toast / banner / inline / browser notification の 4 種、aria-live
- [25_権限 UI 制御](25_権限UI制御.md) — activate / deactivate / 表示 / 非表示、認可 cache 禁止
- [26_添付帳票 UX](26_添付帳票UX.md) — Object Storage signed URL + sandbox iframe + virus scan
- [27_tier2 消費方針](27_tier2消費方針.md) — tier2 generated stub のみ、独自 type 禁止
- [28_業務管理 UI 境界](28_業務管理UI境界.md) — admin 機能を Backstage プラグインに分離
- [29_複数 tier3 連携](29_複数tier3連携.md) — 統一 SSO + cookie scope 分離 + back-channel logout 単一受信器
- [30_運用 UI](30_運用UI.md) — Backstage TechDocs + Perses + Apache Superset
- [31_配布デプロイ運用](31_配布デプロイ運用.md) — CDN 静的 / MDM exe / MSI / Argo Rollouts canary
- [32_開発者体験](32_開発者体験.md) — Backstage Software Template + Tilt + Companion local mock
- [33_業務カスタマイズ粒度](33_業務カスタマイズ粒度.md) — 4 抽象化レベルでのカスタマイズ粒度
- [34_製造業 pack 適用例](34_製造業pack適用例.md) — 9 業務シナリオ + オフライン + レガシー統合

全 34 方針を完備。個別非提供スコープ（03_非提供スコープ）は要件定義 [02_非提供スコープ](../../02_要件定義/01_スコープ/02_非提供スコープ.md) に統合済。

## 詳細設計への参照
- [クライアント状態適合仕様](../../04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md)
- [tier3 強制機構](../../04_詳細設計/02_強制機構/03_tier3強制機構.md)
- [BFF auth-edge](../../04_詳細設計/03_クロスカッティング適合仕様/06_BFF_auth_edge.md)
- [Tauri Companion sidecar](../../04_詳細設計/03_クロスカッティング適合仕様/07_Tauri_companion_sidecar.md)

## 採用しない選択肢
- tier1 Library / Companion / OSS の直接 import / wrap
- 業務管理 API の業務 UI 取込
- tenant_id の form / URL 受付
- 認可結果の client-side cache
- localStorage / sessionStorage / cookie への PII 平文保管
- raw HTTP / raw gRPC で tier1 backend を直接呼出
- tier3 リポジトリ内 .proto 配置
- 業務 entity / 業務 event の独自再宣言
- contract test の skip / 改変
- WCAG 2.1 AA 違反 / Web Vitals 閾値違反の merge
- CSP の `unsafe-inline` / `unsafe-eval`
- iOS / Android ネイティブ（v1.0.0 出荷外、Swift / Kotlin が tier1 / tier2 言語スタック未対応のため）
- AGPL / SSPL 系 OSS の業務 UI 配信

## 至高路線における立ち位置
- 「ジュニア級 tier3 担当が業務不変条件を踏み抜く経路」を 13 層強制機構で物理拒否
- 「同じ業務処理を別 tier3 が別実装で扱う」経路を tier2 集約で構造的に不可能化
- 「a11y / Web Vitals は努力目標」を採らない（CI で自動検査 + merge 阻止）

## 関連参照
- [tier1 設計方針 index](../02_tier1設計方針/README.md)
- [tier2 設計方針 index](../03_tier2設計方針/README.md)
- [client 設計方針 index](../09_client設計方針/README.md)
