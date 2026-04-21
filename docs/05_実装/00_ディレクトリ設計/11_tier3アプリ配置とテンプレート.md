# 11. tier3 アプリ配置とテンプレート

本ファイルは tier3 エンドユーザアプリの**リポジトリ配置と内部構造**を方式として固定する。tier3 は Web App（TypeScript + Next.js / React）・Mobile App（.NET MAUI）・BFF（Backend for Frontend、TypeScript + Node.js）の 3 バリエーションを想定し、いずれも Backstage Software Template から生成される独立 GitHub repo で開発する。上流は概要設計 [DS-SW-DOC-001（Golden Path 全体手順）](../../04_概要設計/20_ソフトウェア方式設計/05_利用者文書_暫定版/01_tier2_tier3開発者向けGoldenPath初版.md)・[ADR-TIER1-003（内部言語不透明性）](../../02_構想設計/adr/ADR-TIER1-003-language-opacity.md)、および Phase 2 の tier3 展開計画である。

## 本ファイルの位置付け

tier3 は tier2 と同じ polyrepo 方針（[09 章 DS-IMPL-DIR-222](09_tier1全体配置とSDK境界.md)）に従うが、3 つの重要な差が tier2 との間にある。(a) 配信経路が Kubernetes ではなく CDN / Object Storage（Web）やアプリストア（Mobile）で、GitOps repo を経由しない。(b) 認証フローが Service Account Token ではなくユーザ OIDC（Keycloak Interactive Login）で、tier2 の「透過的 tenant_id 自動付与」とは別の認証経路を必要とする。(c) 国際化（i18n）・アクセシビリティ（WCAG 2.1）の要件が強く、エンドユーザ視点の UI 骨格が Template の最初から組み込まれている必要がある。本章はこれらの差を吸収したうえで、tier2 と平仄を揃えた配置設計を確定する。

tier3 ディレクトリ設計が曖昧なまま Phase 2 の本格展開に入ると、(a) Web / Mobile / BFF が repo ごとにバラバラのフレームワーク（Vue 派と React 派が混在、MAUI 派と Flutter 派が混在）で実装され、運用チームが習得すべき技術が爆発する。(b) 認証フローが開発者ごとに独自実装になり、Keycloak の仕様変更時の改修が全 tier3 repo で個別発生する。本章はこれを Template の骨格固定と「削除禁止改変」の明示で封じる。

Phase 1a 時点では tier3 本体の開発は行われず、本章の主目的は **Template ソースの Phase 1a 整備**である。Phase 2 での本格展開時に、Template を 1 回でも触ってない状態で開始すると Golden Path 10 分ルール（[DX-GP-001](../../04_概要設計/20_ソフトウェア方式設計/05_利用者文書_暫定版/01_tier2_tier3開発者向けGoldenPath初版.md)）の達成が危うい。Phase 1a で Template 骨格と生成 repo 構造を固定しておき、Phase 1b の tier2 パイロットで Template 運用の練習を積んだ上で、Phase 2 の tier3 本番投入に備える。

## 概要設計との役割分担

概要設計 [DS-SW-DOC-001](../../04_概要設計/20_ソフトウェア方式設計/05_利用者文書_暫定版/01_tier2_tier3開発者向けGoldenPath初版.md) は tier2 / tier3 を区別せず Golden Path を共通で宣言し、SDK 1 行呼び出しサンプル（[DS-SW-DOC-003](../../04_概要設計/20_ソフトウェア方式設計/05_利用者文書_暫定版/01_tier2_tier3開発者向けGoldenPath初版.md)）で TypeScript の例を示した。本章はその抽象を tier3 の **3 バリエーション（Web / Mobile / BFF）** に分解し、言語・フレームワーク・ビルドツール・配信経路まで具体化する。tier2 の配置設計（[10 章](10_tier2サービス配置とテンプレート.md)）との重複部分（Template ソースの基本構造、CODEOWNERS 雛形ロジック、禁止改変の考え方）は本章では最小限の参照に留め、tier3 固有の差分に集中する。

## 設計 ID 一覧と採番方針

本ファイルで採番する設計 ID は `DS-IMPL-DIR-261` 〜 `DS-IMPL-DIR-280` の 20 件である。3 バリエーションを 20 件で扱うため、共通項目（SDK 消費、CODEOWNERS、禁止改変）はバリエーション横断で 1 ID に集約する。

## tier3 3 バリエーションの概観

tier3 の 3 バリエーションを先に整理する。それぞれ Template が異なり、生成 repo の内部構造も異なる。配信経路も別経路を採る。

- **tier3 Web App**（TypeScript + Next.js + React）: 社内ポータル・申請画面などの社内向け Web アプリ。配信は Object Storage（MinIO）上に静的ビルド成果物を配置し、Ingress 経由で CDN キャッシュ配信。BFF を伴う構成を想定し、ブラウザから直接 tier1 gRPC は叩かず、BFF 経由にする。
- **tier3 Mobile App**（C# + .NET MAUI）: iOS / Android 両対応のクロスプラットフォームアプリ。配信は App Store / Play Store（Phase 2 後半で MDM 配信の選択肢検討）。MAUI から直接 tier1 を叩くか BFF 経由かは、Mobile 特有の認証フローとオフライン対応の要件で決まる（Phase 2 初頭に ADR で判断）。
- **tier3 BFF**（TypeScript + Node.js + Express/Fastify）: Web / Mobile からの呼び出しを受け、tier1 API への呼び出しに集約する薄いレイヤ。認証セッション管理（OIDC + refresh）・レスポンスキャッシュ・複数 API の aggregation を担う。配信は Kubernetes 上で tier2 と同じ GitOps 経路を使う（BFF は技術的には tier2 と同じサーバサイドだが、役割的には tier3 の一部）。

これらは同じ Backstage organization 配下に並立する別 repo として存在し、Template は `tier3/webapp-typescript/` / `tier3/mobile-maui/` / `tier3/bff-typescript/` の 3 つに分かれる（07 章 DS-IMPL-DIR-185 の 2 階層分割に準拠）。

## DS-IMPL-DIR-261 本章の位置付け（tier3 ディレクトリ設計の範囲）

本章が対象とする物理境界は次の 2 つ。(a) k1s0 repo 内の `tools/backstage-templates/tier3/*/` 配下の Template ソース、(b) Backstage から生成される tier3 repo の骨格（3 バリエーション分）。(b) の tier3 repo は本 repo の外に存在するが、生成直後の骨格は本章で規定する。

生成後に tier3 開発者が追加する画面デザイン・ビジネスロジック・UI コンポーネントは本章の対象外で、各 tier3 アプリ開発チームの判断に委ねる。ただし Template が生成した認証フロー・i18n 骨格・アクセシビリティの基本構成を**削除・無効化**することは禁止する（DS-IMPL-DIR-278）。

**確定フェーズ**: Phase 0。**対応要件**: DX-GP-001、NFR-C-NOP-001、ADR-TIER1-003。**上流**: DS-SW-DOC-001、DS-IMPL-DIR-222、DS-IMPL-DIR-241。

## DS-IMPL-DIR-262 tier3 repo も polyrepo（各アプリ 1 repo）

tier3 の 3 バリエーション（Web / Mobile / BFF）はそれぞれ独立 GitHub repo で開発する。命名は [09 章 DS-IMPL-DIR-235](09_tier1全体配置とSDK境界.md) で `k1s0-tier3-<app>` と確定済みで、バリエーションを示すサフィックスは任意で付けられる（例: `k1s0-tier3-employee-portal`、`k1s0-tier3-employee-portal-mobile`、`k1s0-tier3-employee-portal-bff`）。

Web と BFF をペアで開発する場合は 2 repo に分ける（同一 repo で monorepo 化しない）。理由は (a) Web は CDN 配信・ビルド成果物のみ publish、BFF は Kubernetes 配信・container image publish で、CI パイプラインとリリース経路が全く異なる。(b) Web 開発者（フロントエンド専任チーム）と BFF 開発者（バックエンド寄り）でオーナーチームが異なる可能性が高い。1 repo で混ぜると CODEOWNERS が複雑化し、`k1s0-tier3-<app>-web` の CI が毎回 BFF もビルドする非効率が発生する。

**確定フェーズ**: Phase 0（ルール）、Phase 2 以降（適用）。**対応要件**: DX-GP-001、NFR-C-NOP-001。**上流**: DS-IMPL-DIR-222、DS-IMPL-DIR-235、DS-IMPL-DIR-242。

## DS-IMPL-DIR-263 Backstage Template ソースの配置

tier3 用の Backstage Software Template のソースは、k1s0 repo 内の `tools/backstage-templates/tier3/` 配下に 3 サブディレクトリで配置する（07 章 DS-IMPL-DIR-185 で確定した `tier2/` / `tier3/` の 2 階層分割に準拠）。

```
tools/backstage-templates/
└── tier3/                           # tier3 エンドユーザーアプリ雛形
    ├── webapp-typescript/           # Web App Template
    │   ├── template.yaml
    │   ├── skeleton/
    │   │   ├── .github/workflows/ci.yml
    │   │   ├── .github/CODEOWNERS
    │   │   ├── .devcontainer/
    │   │   ├── src/                 # Next.js app router 構造
    │   │   ├── package.json
    │   │   ├── tsconfig.json
    │   │   ├── next.config.mjs
    │   │   ├── Dockerfile           # Standalone build（nginx 配信）
    │   │   ├── docs/
    │   │   └── README.md
    │   └── README.md                # Template 利用ガイド
    ├── mobile-maui/                 # Mobile App Template
    │   ├── template.yaml
    │   ├── skeleton/
    │   │   ├── .github/workflows/ci.yml
    │   │   ├── src/                 # MAUI XAML + C# コード
    │   │   ├── <AppName>.sln
    │   │   ├── docs/
    │   │   └── README.md
    │   └── README.md
    └── bff-typescript/              # BFF Template
        ├── template.yaml
        ├── skeleton/
        │   ├── .github/workflows/ci.yml
        │   ├── src/                 # Fastify + TypeScript
        │   ├── package.json
        │   ├── tsconfig.json
        │   ├── Dockerfile           # distroless node
        │   ├── docs/
        │   └── README.md
        └── README.md
```

3 サブディレクトリを分ける理由は (a) 3 バリエーションで言語・ランタイム・ビルドツール・配信経路が全て異なり、Template の `parameters` セクション・skeleton の構造・生成物の CI 設定が共通化できない、(b) Template 更新のリリースサイクルをバリエーション別に独立させたい（Web は React 更新追従、Mobile は MAUI 更新追従、BFF は Node.js LTS 更新追従）、の 2 点。

Phase 1b では `tier3/webapp-typescript/` と `tier3/bff-typescript/` の骨格版（最小 skeleton）を配置し、`tier3/mobile-maui/` は README のみ（Phase 2 で正式化）。Phase 1c で Web / BFF Template を完全版化。Phase 2 で Mobile Template を完全版化し、3 バリエーションすべてを GA する。

**確定フェーズ**: Phase 1a（Web/BFF の骨格）、Phase 1b（Web/BFF 完全版）、Phase 2（Mobile 完全版）。**対応要件**: DX-GP-001、ADR-TIER1-003。**上流**: DS-SW-DOC-001、DS-IMPL-DIR-243。

## DS-IMPL-DIR-264 tier3 Template の parameters と共通骨格

3 バリエーションの `template.yaml` の `parameters` セクションは以下を共通項目として持つ。

- `app_name`: アプリ名（kebab-case、GitHub repo 名の一部に展開）
- `team_name`: 所属チーム名（canonical 8 チームに限定、[01 章 DS-IMPL-DIR-018](01_リポジトリルート構成.md) と整合）
- `contact_email`: 一次連絡先メール
- `tenant_id`: テナント ID（デフォルトテナント、UI 上でユーザが切替可能にするかは別設定）
- `i18n_languages`: 対応言語一覧（カンマ区切り、例: `ja,en`、少なくとも `ja` を含む）
- `needs_pii_screen`: PII を扱う画面があるか（`boolean`）

バリエーション固有の追加項目として、Web は `uses_bff`（BFF 経由かどうか）、Mobile は `target_platforms`（iOS / Android / Windows / macOS の複数選択）、BFF は `aggregates_apis`（集約対象の tier1 API リスト）を持つ。

**確定フェーズ**: Phase 1a。**対応要件**: DX-GP-001、DX-GP-002、NFR-C-NOP-001。**上流**: DS-SW-DOC-001、DS-IMPL-DIR-244。

## DS-IMPL-DIR-265 生成される tier3 Web App repo のルート構造

Web App repo（`k1s0-tier3-<app>` または `k1s0-tier3-<app>-web`）のルート構造は Next.js App Router 構成をベースにする。

```
k1s0-tier3-<app>/
├── .github/
│   ├── workflows/ci.yml             # build / unit test / e2e test / a11y check / bundle size check / SBOM
│   ├── CODEOWNERS
│   └── PULL_REQUEST_TEMPLATE.md
├── .devcontainer/
├── src/
│   ├── app/                         # Next.js app router（layout.tsx / page.tsx / loading.tsx / error.tsx）
│   ├── components/                  # 再利用可能 UI コンポーネント
│   ├── lib/
│   │   ├── k1s0-client.ts           # k1s0 SDK ラッパ（BFF 経由 or 直呼び、uses_bff による分岐）
│   │   └── auth.ts                  # OIDC Keycloak 認証
│   ├── locales/                     # i18n 辞書（app_name/i18n_languages から展開）
│   │   ├── ja.json
│   │   └── en.json
│   └── styles/
├── public/                          # 静的資産（画像、favicon）
├── package.json                     # @k1s0/sdk 依存（Phase 2）
├── tsconfig.json
├── next.config.mjs                  # standalone output、security headers
├── Dockerfile                       # multi-stage、nginx 配信 or Node runtime
├── playwright.config.ts             # E2E テスト設定
├── vitest.config.ts                 # unit テスト設定
├── docs/
│   ├── index.md
│   ├── ui-guide.md                  # UI 設計ガイド（デザイントークン、a11y 方針）
│   └── runbook.md
└── mkdocs.yml
```

`src/app/` 配下は初期テンプレートで「ログイン画面」「ランディング画面」「1 つのサンプル CRUD 画面」を組み込み、開発者はサンプル画面を参考に自アプリの画面を追加する。サンプル画面は削除可能だが、認証・i18n・アクセシビリティの基本実装は `src/lib/` の `auth.ts` と `src/locales/` として残り、削除禁止（DS-IMPL-DIR-278）。

**確定フェーズ**: Phase 1b（Template 完全版）、Phase 2（本格展開）。**対応要件**: DX-GP-004、DX-GP-005、NFR-E-AC-\*（認証）、NFR-F-ENV-\*（アクセシビリティ）、NFR-UI-I18N-\*。**上流**: DS-SW-DOC-002、DS-IMPL-DIR-245。

## DS-IMPL-DIR-266 生成される tier3 Mobile（MAUI）repo のルート構造

Mobile App repo（`k1s0-tier3-<app>-mobile`）は .NET MAUI プロジェクト構造を採る。

```
k1s0-tier3-<app>-mobile/
├── .github/workflows/ci.yml         # build / unit test / iOS build / Android build / SBOM
├── .devcontainer/
├── src/
│   ├── <AppName>/                   # MAUI メインプロジェクト
│   │   ├── <AppName>.csproj         # TargetFramework: net8.0-ios;net8.0-android 他
│   │   ├── App.xaml / App.xaml.cs
│   │   ├── Views/                   # XAML 画面
│   │   ├── ViewModels/              # MVVM パターンの ViewModel
│   │   ├── Services/                # k1s0 SDK 呼び出しラッパ、認証
│   │   ├── Resources/
│   │   │   ├── Strings/             # i18n 辞書（.resx）
│   │   │   ├── Images/
│   │   │   └── Styles/
│   │   └── Platforms/               # iOS / Android / Windows / macOS の OS 固有コード
│   └── <AppName>.Tests/             # 単体テスト
├── <AppName>.sln
├── docs/
└── mkdocs.yml
```

Mobile では配信が App Store / Play Store 経由のため、CI でのビルドは署名付き artifact（.ipa / .apk）を生成する。iOS 署名の certificate / provisioning profile は GitHub Actions の secrets（Apple Developer の App Store Connect API 経由）で供給し、Template 側の `ci.yml` に設定済みにする。この前提で、Phase 2 時点で Apple Developer Program と Google Play Developer アカウントを社内で保有しているかを事前確認する必要がある（Phase 1b 後半に Product Council で確認予定）。

**確定フェーズ**: Phase 2。**対応要件**: DX-GP-005、NFR-E-AC-\*、NFR-UI-I18N-\*。**上流**: DS-SW-DOC-002、DS-IMPL-DIR-245。

## DS-IMPL-DIR-267 生成される tier3 BFF repo のルート構造

BFF repo（`k1s0-tier3-<app>-bff`）は TypeScript + Fastify 構成を採る。

```
k1s0-tier3-<app>-bff/
├── .github/workflows/ci.yml         # build / unit / integration / SAST / SBOM / image scan
├── .devcontainer/
├── src/
│   ├── index.ts                     # Fastify エントリポイント
│   ├── routes/                      # REST ルート定義
│   ├── handlers/                    # ルートハンドラ
│   ├── services/                    # ビジネスロジック（tier1 API 集約）
│   ├── adapters/
│   │   └── k1s0-sdk-adapter.ts      # @k1s0/sdk ラッパ
│   └── middleware/
│       ├── auth.ts                  # OIDC セッション検証
│       ├── tenant.ts                # tenant_id 伝搬
│       └── error.ts
├── test/                            # 単体テスト + integration テスト
├── package.json                     # @k1s0/sdk 依存
├── tsconfig.json
├── Dockerfile                       # distroless node、non-root
└── docs/
```

BFF は tier2 と同じく Kubernetes で配信されるため、10 章 DS-IMPL-DIR-257 と同じ GitOps 経路（k1s0-gitops への PR 自動作成 → Argo CD 同期）を使う。Dockerfile も distroless + non-root の制約が tier2 と同じく適用される。BFF が「tier3 の一部」なのは配信経路ではなく**役割**で分類されているためで、実装運用面は tier2 と共通点が多い。

**BFF 責務境界の allowlist / denylist**: 「ビジネスロジックを持たない」という抽象的なルールだけでは境界が運用で融解し、BFF が数年で隠れ tier2 になる。境界を保つため、許容される責務と禁止される責務を具体的に列挙し、迷いを技術判断に閉じる。

- **BFF に置くことが許容される責務（allowlist）**:
  1. **認証セッション管理**: OIDC Authorization Code Flow のコード交換、refresh token 保管（HttpOnly Cookie）、session 検証。
  2. **tier1 gRPC 呼び出しの HTTP 化**: フロントからの REST / GraphQL 呼び出しを tier1 の gRPC に変換。エラーマッピング（gRPC Status → HTTP Status）を含む。
  3. **複数 tier1 API の aggregation**: 1 画面が複数 tier1 API を必要とする際の並列呼び出しとレスポンス結合（N+1 相当の削減）。
  4. **レスポンスキャッシュ**: tenant / user を key にした短時間（~60s）キャッシュ。冪等な GET 系のみ。
  5. **表示用の整形**: 金額の 3 桁カンマ、日時のタイムゾーン変換、i18n 用の locale 注入など「UI 表示のための無害な変換」。状態遷移を伴わないこと。
  6. **Web / Mobile 固有の request shaping**: CSRF token 注入、Mobile 向けの gzip 強制、ブラウザ向けの CSP ヘッダ付与。
- **BFF に置くことを禁止する責務（denylist）**:
  1. **業務ルール・ドメイン判定**: 「この tenant は上限 N を超えたら reject」などの業務判定は tier2 / tier1 に置く。BFF でやると同じ判定が複数 BFF にコピーされ、結果が divergence する。
  2. **永続化を伴う計算**: 集計・ランキング計算等、結果を persist する処理は tier2 に置く。BFF で計算したら結果を DB に書くという経路は全面禁止。
  3. **Secret の保持**: tier1 Secret API の値を BFF メモリに keep することは禁止。必要な都度 tier1 から取得する。
  4. **tenant 越境の aggregation**: BFF が複数 tenant のデータを 1 レスポンスで返すことは禁止（Policy Enforcer が tier1 で効くが、BFF 層でも tenant 1 境界を物理的に強制する）。
  5. **ビジネスイベントの発行**: Kafka / PubSub へのイベント発行は tier2 経由のみ。BFF から直接 tier1 PubSub API を叩くことも禁止。
  6. **ML モデル推論・重い計算**: p95 レイテンシを損なう処理全般は tier2 に隔離する。

この境界は `tools/bff-boundary-lint/`（Phase 1b）で機械的に強制する。lint は `src/services/` 配下で (a) tier1 SDK の `PubSub` / `Binding` / `Workflow` / `Secrets.Set` 系 API 呼び出し、(b) 永続ストレージクライアント（`pg` / `redis` / `mongodb` 等）の import、(c) 業務ルール判定を想起させる識別子（`calculate*`、`validateBusiness*`、`applyPolicy*`）の定義を検出したら PR を fail させる。allowlist 側の HTTP → gRPC 変換や aggregation は `src/handlers/` / `src/adapters/` 配下に閉じる前提で、`src/services/` は薄く保つ。

迷うケースは ADR で判断するが、BFF 側に寄せる argument は「tier1 / tier2 で提供するより明らかに UI 要件主導で変化する」という一点のみを認める。それ以外は tier2 に寄せる。

**確定フェーズ**: Phase 1b（Template 完全版、bff-boundary-lint 導入）、Phase 2（本格展開）。**対応要件**: DX-GP-005、NFR-E-AC-\*、NFR-B-PERF-\*（BFF のキャッシュで tier1 負荷軽減）、NFR-C-NOP-002。**上流**: DS-SW-DOC-002、DS-IMPL-DIR-245、DS-IMPL-DIR-272。

## DS-IMPL-DIR-268 tier3 CI 設定

`.github/workflows/ci.yml` は 3 バリエーションで異なる段構成を持つ。共通部分（build / unit test / SBOM / image scan）と固有部分の組み合わせは以下。

Web App の 7 段:

1. `install`: `npm ci`
2. `build`: `next build`
3. `unit-test`: `vitest run`
4. `e2e-test`: `playwright test`（ヘッドレス Chrome、社内モック API）
5. `a11y-check`: `axe-core` によるアクセシビリティ自動テスト
6. `bundle-size-check`: ビルド後の JS bundle サイズ閾値検査（社内ポータル向け 500 KB 以下など）
7. `sbom`: `cyclonedx-bom` による SBOM 生成

Mobile の 5 段:

1. `build-ios`: xcodebuild による iOS アーカイブ生成
2. `build-android`: Gradle による Android APK 生成
3. `unit-test`: xUnit / NUnit 単体テスト
4. `ui-test`: Appium または MAUI UITest（Phase 2 後半で追加検討）
5. `sbom`

BFF の 6 段（tier2 Go 版と同じ構成に近い）:

1. `build`: `tsc`
2. `unit-test`: `vitest run`
3. `integration-test`: tier1 mock-server と連携
4. `sast`: `semgrep` による静的解析
5. `sbom`: `cyclonedx-bom`
6. `image-scan`: trivy による container image スキャン

全バリエーションで段の削除は禁止、追加は許容（DS-IMPL-DIR-278）。

**確定フェーズ**: Phase 1b（Web / BFF）、Phase 2（Mobile）。**対応要件**: DX-CICD-\*、NFR-E-AC-\*、NFR-H-INT-\*、NFR-UI-A11Y-\*。**上流**: DS-SW-DOC-002、DS-IMPL-DIR-248。

## DS-IMPL-DIR-269 tier3 CODEOWNERS 雛形

tier3 repo の CODEOWNERS は 10 章 DS-IMPL-DIR-249 と同じパターンで、アプリチームがデフォルトオーナー、tier1 基盤チームが SDK ラッパ + CI + Dockerfile 部分のセカンダリオーナーになる。Web 特有の追加として、`src/locales/` の i18n 辞書は UX / ドキュメントチーム（該当する場合）がセカンダリオーナーに加わる。

```
*                                    @k1s0/${{values.team_name}}
/src/lib/k1s0-client.ts              @k1s0/${{values.team_name}} @k1s0/tier1-architects
/src/locales/                        @k1s0/${{values.team_name}} @k1s0/product-owners
/.github/workflows/                  @k1s0/devex-team
/Dockerfile                          @k1s0/${{values.team_name}} @k1s0/security-team
```

Mobile の場合は `src/<AppName>/Platforms/` のような OS 固有コード部分が「iOS 担当」「Android 担当」で分かれる可能性があるが、Phase 2 初期は単一チーム扱いで進め、担当分離は必要になってから追加する。

**確定フェーズ**: Phase 1b（Web / BFF）、Phase 2（Mobile）。**対応要件**: NFR-C-NOP-001、DX-CICD-\*。**上流**: DS-IMPL-DIR-018、DS-IMPL-DIR-249。

## DS-IMPL-DIR-270 tier3 配信経路（Web / Mobile / BFF）

3 バリエーションの配信経路は別経路。

- **Web**: CI で `next build --output standalone` により静的ビルド成果物を生成し、MinIO の bucket（`tier3-web-<app>-<env>`）にアップロード。Ingress（Istio Gateway）が CDN キャッシュ配信する。Canary / Rollback は MinIO 上の versioned object で実現し、Argo Rollouts は使わない（静的配信のため）。
- **Mobile**: CI で署名付き iOS `.ipa` と Android `.apk` / `.aab` を artifact として生成。手動で App Store Connect / Play Console にアップロード（Phase 2 初期）、Phase 2 後半で `fastlane` による自動化検討。MDM 経由の社内配信も Phase 2 後半で検討。
- **BFF**: Kubernetes 配信。tier2 と同じ GitOps 経路（k1s0-gitops への PR 自動作成）、Helm Chart も tier2 と同じ umbrella（[05 章 DS-IMPL-DIR-124](05_infra詳細構成.md)）の subchart として組み込む。

Web と Mobile で GitOps を介さない理由は、Kubernetes 上で稼働しないため。Web は静的ビルド成果物の公開、Mobile はアプリストア経由の配信で、Argo CD / Kubernetes Deployment の対象外。tier1 GitOps の対象は Kubernetes リソースのみで一貫させる。

**確定フェーズ**: Phase 1b（BFF）、Phase 2（Web / Mobile）。**対応要件**: DX-CICD-\*、NFR-A-AV-\*（可用性）、NFR-B-PERF-\*、NFR-F-ENV-\*。**上流**: DS-IMPL-DIR-124、DS-IMPL-DIR-192、DS-IMPL-DIR-257。

## DS-IMPL-DIR-271 tier3 SDK 消費方法（TypeScript npm / C# NuGet）

tier3 repo は k1s0 SDK をパッケージレジストリ経由で取得する。言語別の詳細は以下。

- **Web / BFF（TypeScript）**: `package.json` で `@k1s0/sdk` を依存として宣言。Phase 2 では ghcr.io の npm registry を使う予定（Phase 2 途中で Nexus に移行）。`@k1s0/sdk` のブラウザ版 / Node 版は 1 パッケージ内のサブパス export（`@k1s0/sdk/browser` / `@k1s0/sdk/node`）で提供される（[09 章 DS-IMPL-DIR-228](09_tier1全体配置とSDK境界.md)）。Web は `browser` サブパス、BFF は `node` サブパスを使う。
- **Mobile（C# MAUI）**: `<AppName>.csproj` で `K1S0.Sdk` を NuGet 依存として宣言（[09 章 DS-IMPL-DIR-227](09_tier1全体配置とSDK境界.md)）。tier2 と同じ NuGet パッケージを共有する（C# SDK は tier2 / tier3 共通）。Mobile 特有の機能（オフラインキャッシュ対応など）は `K1S0.Sdk.Mobile` という拡張パッケージで Phase 2 以降に提供する検討があるが、Phase 1a 時点では未確定。

Web が BFF 経由でなく直接 tier1 を呼ぶ構成（`uses_bff: false`）の場合、ブラウザから gRPC-Web 経由で tier1 facade Pod を叩く。この場合の CORS 設定・CSP ヘッダ・Keycloak OIDC コード交換フローは Template が初期実装を持ち、開発者が触る必要はない（触ると認証セキュリティ要件を破壊する可能性が高いため禁止改変リストに含める）。

**確定フェーズ**: Phase 2。**対応要件**: ADR-TIER1-003、DX-GP-006、NFR-E-AC-\*。**上流**: DS-IMPL-DIR-224、DS-IMPL-DIR-227、DS-IMPL-DIR-228。

## DS-IMPL-DIR-272 tier3 認証フロー（OIDC + Keycloak）

tier3 はエンドユーザがログインする前提のため、tier2 の Service Account Token とは別の認証経路を持つ。Keycloak の OIDC Authorization Code Flow（PKCE 拡張）を使い、ログイン画面 → Keycloak → コード交換 → session/token 確立の順で認証する。

Template が生成する実装要素:

- **Web**: `src/lib/auth.ts` に NextAuth.js v5 + Keycloak provider の設定を実装済み。`.env.local` 雛形に Keycloak の `issuer`・`clientId`・`clientSecret` の placeholder が入り、開発者は実値を差し込むだけ。session 管理は JWT（サーバサイド署名検証）と HttpOnly Cookie の組み合わせで、XSS 耐性を確保する。
- **Mobile**: `Services/AuthService.cs` に IdentityModel.OidcClient（or MAUI 標準の WebAuthenticator）による Authorization Code Flow を実装済み。refresh token はプラットフォーム標準の secure storage（iOS Keychain / Android Keystore）に保存する。
- **BFF**: `middleware/auth.ts` でフロントから受け取った session Cookie を検証し、tier1 呼び出し時の Service Account Token を使う（OIDC On-Behalf-Of フローまたは service account で代替、Phase 2 初頭に ADR で決定）。

**Auth Flow Matrix — 構成と責務の対応表**: tier3 は Web/Mobile の 2 クライアント × BFF 経由/直接の 2 ネットワーク経路の計 4 組合せを想定する。各組合せで「ユーザ認証の主体」「token 保管場所」「tier1 呼び出し時の identity」「refresh 経路」が異なり、これを散文だけで伝えると実装で取り違える。本マトリクスは章末付録ではなく**本 ID の本文**として、構成選択時の判断材料を一箇所に集約する。

本マトリクスは「1 行読めば当該構成の全責務が分かる」密度を目標にし、セル内はラベルではなく具体的な技術要素で記述する。

| 構成 | ユーザ認証 | access token 保管 | refresh token 保管 | tier1 呼び出し identity | refresh 経路 | 主な脅威対策 |
|---|---|---|---|---|---|---|
| **Web + BFF**（Phase 1b 推奨） | Keycloak OIDC Auth Code + PKCE。ブラウザが BFF にログイン要求→BFF が Keycloak と code 交換 | BFF プロセスメモリ（tenant/user 単位、TTL 5 min）、ブラウザは session Cookie（HttpOnly + Secure + SameSite=Strict）のみ保持 | BFF プロセスメモリ。Cookie には入れない | Keycloak Token Exchange（RFC 8693）で user token → tier1 向け audience の service account token に交換 | BFF が access 期限 60 sec 前に Keycloak に refresh。ブラウザは気付かない | XSS 耐性（token は JS 不可視）、CSRF token 必須 |
| **Web 直接**（`uses_bff: false`、小規模想定） | Keycloak OIDC Auth Code + PKCE。ブラウザが直接 Keycloak と code 交換 | NextAuth.js の JWT Cookie（HttpOnly + Secure + SameSite=Strict）に id/access をまとめて格納 | NextAuth.js の JWT Cookie（同上） | gRPC-Web で tier1 facade Pod を直接呼ぶ。user JWT を Authorization ヘッダに付与 | ブラウザが NextAuth.js の session API 経由で refresh。Cookie 再発行 | XSS 耐性は JWT Cookie の HttpOnly 属性に依存。CSP ヘッダ厳格化で XSS 発生確率を低減 |
| **Mobile + BFF**（Phase 2 後半検討） | MAUI WebAuthenticator で Keycloak OIDC Auth Code + PKCE | BFF プロセスメモリ（Web + BFF と同じ） | アプリ側: iOS Keychain / Android Keystore。BFF 側: プロセスメモリ | Web + BFF と同じ Token Exchange | アプリが期限前に BFF の `/auth/refresh` を叩く | XSS 非該当（native）、OS 標準の secure storage で token 保護 |
| **Mobile 直接**（Phase 2 標準） | MAUI WebAuthenticator で Keycloak OIDC Auth Code + PKCE | アプリメモリ | iOS Keychain / Android Keystore | gRPC で tier1 facade Pod を直接呼ぶ。user JWT を metadata に付与 | アプリが期限前に Keycloak Token Endpoint を直接叩く | OS 標準 secure storage。プラットフォーム標準の pinning でトランスポート保護 |

このマトリクスの読み方は 2 点。(1) **BFF 経由では token の「実体」を常にサーバ側に置く**ことで、ブラウザ側の脆弱性（XSS で token 奪取）の影響範囲を session Cookie に限定する。(2) **Mobile 直接は native の secure storage がブラウザよりセキュアな保管経路を提供する**ため、BFF 経由の優位性がブラウザ構成ほどは大きくない。この 2 点が各構成の推奨理由の本質。

**tier1 呼び出しの identity 原則**: どの構成でも、tier1 facade Pod に届く gRPC 呼び出しには「ユーザ身元を示す token」が含まれる必要がある。BFF 経由の場合、BFF が自分の service account token を勝手に付けて tier1 を呼ぶと、tier1 側で「誰の操作か」が失われる。したがって BFF は Keycloak Token Exchange で user token を tier1 用 audience に変換した token を常に添付する。Policy Enforcer（tier1、DS-IMPL-DIR-051）が user 主体の RBAC / tenant 境界を評価できるのはこの token のおかげで、この原則を崩すと tier1 の audit / authorization が全滅する。

Keycloak 側の client 設定は tier3 repo ごとに 1 client を発行し、`redirect_uri` や許容 `scope` は Backstage Template 生成時に Keycloak Admin API 経由で自動登録する仕組みを Phase 2 で検討する（Phase 1b 時点では Keycloak 管理者が手動設定）。各 client は構成（Web + BFF / Web 直接 / Mobile 直接）に応じて `confidential` / `public` の種別を分け、`public` client には PKCE を必須化する。

**確定フェーズ**: Phase 1b（Auth Flow Matrix 確立、Web + BFF 実装）、Phase 2（Mobile 構成の実装、Token Exchange ADR 確定）。**対応要件**: NFR-E-AC-\*（認証）、NFR-E-SEC-\*（XSS / CSRF 対策）、BR-PLATUSE-002。**上流**: ADR-SEC-001（Keycloak）、DS-SW-DOC-008、DS-IMPL-DIR-051、DS-IMPL-DIR-267。

## DS-IMPL-DIR-273 tier3 テスト配置

3 バリエーションのテスト配置は以下。

- **Web**: `src/**/*.test.ts(x)` で unit（Vitest）、`tests/e2e/` で Playwright E2E、`tests/a11y/` で axe-core によるアクセシビリティ自動テスト。
- **Mobile**: `src/<AppName>.Tests/` で unit（xUnit）、Phase 2 後半で UI Test（MAUI UITest / Appium）を追加検討。
- **BFF**: `test/unit/` で unit（Vitest）、`test/integration/` で tier1 mock-server との結合テスト、契約テストは pact-js で tier1 契約を検証（契約ファイルは k1s0 repo の `tests/contract/consumers/` にコミット）。

Web の E2E は tier3 repo 側では「ログインからサンプル画面までの一気通貫」を最小シナリオとして実装し、tier3 ↔ BFF ↔ tier1 の横断 E2E は k1s0 repo の `tests/e2e/scenarios/cross_pod/` に追加する（10 章 DS-IMPL-DIR-253 と同じ方針）。

**確定フェーズ**: Phase 1b（Web/BFF 基本）、Phase 2（Mobile 追加、UI Test 追加）。**対応要件**: DX-TEST-\*、DX-CICD-\*、NFR-UI-A11Y-\*。**上流**: DS-IMPL-DIR-166、DS-IMPL-DIR-253。

## DS-IMPL-DIR-274 tier3 docs/（TechDocs）配置

tier3 repo の `docs/` は TechDocs の source として配置し、Web / Mobile / BFF それぞれに応じた必須ファイルを Template が生成する。共通は `index.md` / `runbook.md`、バリエーション固有は以下。

- **Web**: `ui-guide.md`（デザイントークン、コンポーネント一覧、a11y 方針）、`i18n.md`（翻訳追加手順）
- **Mobile**: `platform-guide.md`（iOS / Android 特有の注意事項、OS ごとの既知の制限）、`release-process.md`（ストア配信の手順）
- **BFF**: `api.md`（BFF が Web / Mobile に公開する REST 仕様）、`aggregation-map.md`（BFF のどの endpoint がどの tier1 API を集約しているかのマップ）

これらのファイルは削除禁止、内容は各チームが充填する。TechDocs 生成は `mkdocs build` で MkDocs Material テーマを使い、Backstage 上で検索可能にする（Phase 1c の backstage/ 整備と連動）。

**確定フェーズ**: Phase 1b（Web/BFF）、Phase 2（Mobile）。**対応要件**: DX-DEVEX-\*、NFR-SUP-\*、NFR-UI-I18N-\*。**上流**: DS-SW-DOC-002、DS-IMPL-DIR-254。

## DS-IMPL-DIR-275 tier3 ローカル開発との統合

tier3 開発者のローカル開発は 2 層構造で支援する。

1. **Web 単独**: `npm run dev` で Next.js dev server を起動。tier1 呼び出しは BFF モック（local に立てる fastify）経由、または tier1 mock-server（07 章 DS-IMPL-DIR-186）に BFF 経由で接続。
2. **Web + BFF 両立ち上げ**: `k1s0 dev up --tier3=<app>` で Web と BFF を同時起動、tier1 mock-server も Testcontainers で並走。この単一コマンドで「ブラウザで Web を開き、BFF を経由し、tier1 モックを叩く」フローが完成する。

Mobile の開発は iOS simulator / Android emulator を手元で立ち上げる前提で、`k1s0 dev up` の対象外とする（Mobile シミュレータの起動は MAUI 開発環境の標準手順で、k1s0 CLI が重複で用意する必要はない）。

BFF 単独のローカル開発は 10 章の tier2 と同じ `k1s0 dev up` 経路を使う。

**確定フェーズ**: Phase 2。**対応要件**: DX-LD-\*、DX-GP-001。**上流**: DS-IMPL-DIR-184、DS-IMPL-DIR-186、DS-IMPL-DIR-255。

## DS-IMPL-DIR-276 tier3 .devcontainer

3 バリエーションの `.devcontainer/devcontainer.json` は以下の言語ランタイムと CLI を固定。

- **Web**: Node 20 LTS、npm、Playwright ブラウザ（Chromium）、k1s0 CLI
- **Mobile**: .NET 8 SDK、MAUI workloads、Xcode（macOS ホスト前提）、Android SDK、k1s0 CLI
- **BFF**: Node 20 LTS、npm、k1s0 CLI

Mobile は macOS 依存の Xcode を Dev Container 内に含められない（ライセンス上の制約）ため、Dev Container はあくまで C# コード編集・ビルド検証用で、iOS ビルドはホスト macOS での手動実行を前提にする。この制約は `docs/platform-guide.md` に明記される。

`postCreateCommand: k1s0 doctor` は 3 バリエーション共通で、ローカル環境の前提チェックを行う。

**確定フェーズ**: Phase 1b（Web / BFF）、Phase 2（Mobile）。**対応要件**: DX-LD-\*、DX-DEVEX-\*。**上流**: DS-IMPL-DIR-184、DS-IMPL-DIR-256。

## DS-IMPL-DIR-277 tier3 Web の配信バージョニングと Rollback

Web の配信は MinIO bucket 上の versioned object で管理する。CI は成功時に `tier3-web-<app>-<env>/v<semver>-phase<N>/` というプレフィックスでビルド成果物を配置し、Ingress は env ごとに current version（`current` シンボリックリンク相当の bucket metadata）を指す。

Rollback は current metadata を前バージョンに戻すだけで、再ビルドは不要。これは Kubernetes Rollout とは別の経路で、Web 特有の高速ロールバックを実現する。Canary は current とは別の weight-routed rule で、Istio Gateway が 5% / 10% / ... の traffic を新バージョンに流す。

この運用は 05 章の Argo Rollouts（Kubernetes Pod 向け）とは別体系で、Web 配信用に独自の Rollout プロセスを Phase 2 で整備する。`docs/02_構想設計/04_CICDと配信/` 配下に Phase 2 で ADR を起票して詳細決定する。

**確定フェーズ**: Phase 2。**対応要件**: NFR-A-AV-\*、NFR-A-REC-\*、DX-CICD-\*。**上流**: DS-IMPL-DIR-124、ADR-CICD-002（Argo Rollouts、tier3 用の別体系として言及）。

## DS-IMPL-DIR-278 tier3 生成後の禁止変更

Template が生成した骨格のうち、以下は削除・無効化禁止とする。

- `src/lib/auth.ts`（Web）/ `Services/AuthService.cs`（Mobile）/ `middleware/auth.ts`（BFF）: 認証実装を自前実装に置き換えるのは禁止。セキュリティ要件を崩す危険が高い。
- `src/locales/`（Web）/ `Resources/Strings/`（Mobile）: i18n 骨格の削除は禁止。対応言語の追加は自由。
- `.github/workflows/ci.yml`: 段の削除は禁止、追加は許容。
- `.github/CODEOWNERS`: tier1 基盤チームのセカンダリオーナー削除は禁止。
- `Dockerfile`（Web / BFF）: distroless / non-root / readOnlyRootFilesystem を崩す改変は禁止。
- `docs/` 必須ファイル: 削除禁止、内容の充実は自由。

Web 特有の追加として、`next.config.mjs` の `headers()` で設定される Security Headers（CSP / X-Frame-Options / Strict-Transport-Security 等）は削除禁止。これらは OWASP 推奨の Web セキュリティ基線で、tier1 セキュリティチームが管理する。

禁止改変の検出は PR レビューと一部は CI の静的チェック（`auth.ts` のシグネチャ検査、i18n 辞書の存在検査）で行う。

**確定フェーズ**: Phase 2。**対応要件**: NFR-E-AC-\*、NFR-E-SEC-\*、NFR-H-INT-\*、NFR-UI-I18N-\*、NFR-UI-A11Y-\*。**上流**: DS-SW-DOC-002、DS-IMPL-DIR-258。

## DS-IMPL-DIR-279 tier3 国際化とアクセシビリティ骨格

tier3 は JTC の社内アプリで、**日本語が第 1 言語**であることが運用要件として確定している。同時に、グループ会社・海外拠点を含む場合に英語対応が必要になるケースがあるため、Template は初期から i18n 骨格を持つ。

- 辞書ファイル: `src/locales/ja.json` は必須、他言語は `template.yaml` の `i18n_languages` パラメータから生成される（最低限 `ja` を含む）。
- 辞書キー命名: `<画面名>.<要素種別>.<識別子>` 形式（例: `login.button.submit`、`dashboard.title.main`）。命名規則違反は CI の静的チェックで検出する。
- フォールバック: 翻訳未定義の場合は `ja` にフォールバックする実装を Template に含む。

アクセシビリティは WCAG 2.1 Level AA 準拠を目標とする。Template 生成時の基本実装として、(a) すべてのインタラクティブ要素にキーボード操作可能、(b) 色だけに依存しない情報伝達、(c) 画像の alt 属性必須、(d) フォーム要素に label 必須、を CI の a11y-check（axe-core）で検証する。違反は PR をブロックする。

**確定フェーズ**: Phase 2。**対応要件**: NFR-UI-I18N-\*、NFR-UI-A11Y-\*、NFR-F-ENV-\*。**上流**: DS-SW-DOC-002、要件定義の UI 章（[03_要件定義/60_事業契約/](../../03_要件定義/60_事業契約/)）。

## DS-IMPL-DIR-280 tier3 Template 変更時の ADR 起票条件

`tools/backstage-templates/tier3/*/` 配下の Template 変更のうち、以下は ADR を要する。軽微な変更は ADR 不要で `@k1s0/devex-team` + `@k1s0/tier1-architects` のレビューで通す。

1. 新規 tier3 バリエーション Template の追加（現状: Web / Mobile / BFF 以外の追加、例: Desktop アプリ）
2. フレームワーク変更（Next.js → Remix、MAUI → Flutter、Fastify → Express 等）
3. 認証フロー変更（OIDC → SAML 等）
4. Web 配信経路変更（MinIO → 別 Object Storage）
5. i18n の既定言語変更（`ja` を外す等）
6. CI の段構成変更（7 段 → 6 段化、Mobile 5 段の追加段）
7. CODEOWNERS の tier1 基盤チーム外し
8. 禁止改変リストの緩和（DS-IMPL-DIR-278）

ADR は `docs/02_構想設計/adr/ADR-TIER3-<NNN>-<title>.md` 形式で起票する（`TIER3` カテゴリを新規追加、[08 章 DS-IMPL-DIR-218](08_命名規約と配置ルール.md) のカテゴリ拡張）。

**確定フェーズ**: Phase 0（ルール）、各変更時（適用）。**対応要件**: NFR-C-NOP-001、DX-CICD-\*、ADR-TIER1-003。**上流**: DS-SW-COMP-138、DS-IMPL-DIR-218、DS-IMPL-DIR-260。

## 章末サマリ

### 設計 ID 一覧

| 設計 ID | 内容 | 確定フェーズ |
|---|---|---|
| DS-IMPL-DIR-261 | 本章の位置付け（tier3 配置の範囲） | Phase 0 |
| DS-IMPL-DIR-262 | tier3 repo も polyrepo | Phase 0 |
| DS-IMPL-DIR-263 | Backstage Template ソースの配置 | Phase 1a/1b/2 |
| DS-IMPL-DIR-264 | tier3 Template の parameters と共通骨格 | Phase 1a |
| DS-IMPL-DIR-265 | 生成される tier3 Web App repo のルート構造 | Phase 1b/2 |
| DS-IMPL-DIR-266 | 生成される tier3 Mobile（MAUI）repo のルート構造 | Phase 2 |
| DS-IMPL-DIR-267 | 生成される tier3 BFF repo のルート構造 | Phase 1b/2 |
| DS-IMPL-DIR-268 | tier3 CI 設定 | Phase 1b/2 |
| DS-IMPL-DIR-269 | tier3 CODEOWNERS 雛形 | Phase 1b/2 |
| DS-IMPL-DIR-270 | tier3 配信経路（Web / Mobile / BFF） | Phase 1b/2 |
| DS-IMPL-DIR-271 | tier3 SDK 消費方法 | Phase 2 |
| DS-IMPL-DIR-272 | tier3 認証フロー（OIDC + Keycloak） | Phase 2 |
| DS-IMPL-DIR-273 | tier3 テスト配置 | Phase 1b/2 |
| DS-IMPL-DIR-274 | tier3 docs/（TechDocs）配置 | Phase 1b/2 |
| DS-IMPL-DIR-275 | tier3 ローカル開発との統合 | Phase 2 |
| DS-IMPL-DIR-276 | tier3 .devcontainer | Phase 1b/2 |
| DS-IMPL-DIR-277 | tier3 Web の配信バージョニングと Rollback | Phase 2 |
| DS-IMPL-DIR-278 | tier3 生成後の禁止変更 | Phase 2 |
| DS-IMPL-DIR-279 | tier3 国際化とアクセシビリティ骨格 | Phase 2 |
| DS-IMPL-DIR-280 | tier3 Template 変更時の ADR 起票条件 | Phase 0 |

### 対応要件一覧

- BR-PLATUSE-001（単一プラットフォーム）、BR-PLATUSE-002（透過的動作）
- DX-GP-001（Golden Path 10 分）、DX-GP-002（Template 選択）、DX-GP-004（禁止改変）、DX-GP-005（監査要件）、DX-GP-006（SDK 1 行呼び出し）、DX-TEST-\*、DX-CICD-\*、DX-LD-\*、DX-DEVEX-\*
- NFR-A-AV-\*、NFR-A-REC-\*、NFR-B-PERF-\*、NFR-C-NOP-001、NFR-E-AC-\*、NFR-E-SEC-\*、NFR-F-ENV-\*、NFR-H-INT-\*、NFR-SUP-\*
- NFR-UI-I18N-\*（国際化）、NFR-UI-A11Y-\*（アクセシビリティ）
- ADR-TIER1-003（内部言語不透明性）、ADR-SEC-001（Keycloak）、ADR-CICD-002（Argo Rollouts）

### 上流設計 ID

DS-SW-DOC-001（Golden Path 全体手順）、DS-SW-DOC-002（Template 生成物）、DS-SW-DOC-003（SDK 1 行呼び出し）、DS-SW-DOC-008（SDK 自動付与）、DS-SW-COMP-138（変更手続）、DS-IMPL-DIR-018（CODEOWNERS 母集団）、DS-IMPL-DIR-124（Helm umbrella）、DS-IMPL-DIR-166（contract テスト）、DS-IMPL-DIR-184（k1s0 dev up）、DS-IMPL-DIR-186（mock-server）、DS-IMPL-DIR-192（release.yml）、DS-IMPL-DIR-218（ADR 命名）、DS-IMPL-DIR-222（polyrepo 方針）、DS-IMPL-DIR-224（SDK 配布）、DS-IMPL-DIR-227（C# SDK）、DS-IMPL-DIR-228（TypeScript SDK）、DS-IMPL-DIR-235（tier2/3 repo 命名）、DS-IMPL-DIR-241〜260（tier2 章との整合）。
