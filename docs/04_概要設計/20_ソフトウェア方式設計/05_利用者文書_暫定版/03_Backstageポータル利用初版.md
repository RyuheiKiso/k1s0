# 03. Backstage ポータル利用（暫定版）

本ファイルは IPA 共通フレーム 2013 の **7.1.2.4 利用者文書（暫定版）の作成** に対応する利用者文書のうち、開発者・運用者・アーキテクトが k1s0 の開発者ポータル（Backstage）を利用する際の手引き初版である。Backstage は k1s0 の「開発者と運用者の共通玄関」として リリース時点 で導入し、採用後の運用拡大時 で本格統合する。

## 本ファイルの位置付け

プラットフォーム製品において、サービス一覧・所有者情報・依存関係・技術文書・デプロイ状況・ダッシュボードが個別ツールにバラバラに存在すると、開発者は「どのサービスの何を調べたいか」の質問に対して毎回複数ツールを横断することになる。これは既存 採用側組織の情シス現場の「属人的ナレッジへの依存」を助長し、企画で約束した「バス係数 2」「採用側の小規模運用」を破綻させる典型経路である。

Backstage は、これらを一つのポータルに統合する仕組みである。Spotify 社発の OSS であり、Linux Foundation の CNCF Incubating プロジェクトとして運営されている。Software Catalog（サービス一覧とメタデータ）、Software Templates（Golden Path の入口）、TechDocs（リポジトリに併存するドキュメントを集約検索）、プラグイン体系（Argo CD / GitHub / Grafana / Kafka Topics など外部ツール表示）の 4 機能を核とする。

本章は リリース時点 導入時点での「最初の 1 時間」で読者が Backstage を使いこなせるようになることを目的とする。画面操作の詳細は リリース時点 デモで動画化し、本章は文字情報として 段階を横断できる粒度で記述する。

## 対象読者とペルソナ別利用像

Backstage の利用者は 3 つのペルソナに分かれる。各ペルソナで優先的に利用する機能が異なるため、本章は共通基礎 → ペルソナ別優先機能 の順で記述する。

**開発者ペルソナ**（tier2 / tier3 開発者）: Software Templates で新規サービスを立ち上げ、Software Catalog で依存サービスの API 仕様を確認し、TechDocs で過去の設計意図を参照する。最頻利用は Templates と TechDocs 検索。

**運用者ペルソナ**（採用側組織の情シス採用側の小規模運用）: Catalog から障害サービスの Owner・SLA・Runbook リンクを引き出し、Argo CD プラグインでデプロイ状況を確認し、Grafana プラグインでダッシュボードに遷移する。最頻利用は Catalog 詳細と運用プラグイン群。

**アーキテクトペルソナ**（基盤チーム・Product Council）: Catalog の依存関係可視化で tier 間の暗黙依存を検出し、TechDocs 横断検索で設計変更の影響範囲を把握し、System Model で全体アーキテクチャを俯瞰する。最頻利用は Catalog Graph と TechDocs 全文検索。

## 設計項目 DS-SW-DOC-080 Backstage とは — 4 機能の全体像

Backstage の 4 機能は互いに補強し合う構造で、単独利用では価値の半分以下しか発揮されない。本節では 4 機能の相互関係を整理する。

**Software Catalog** はすべての k1s0 コンポーネント（tier1 の 11 API 実装 / tier2 サンプルサービス / tier3 配信ポータル / infra OSS / data 層）を単一のグラフとして保持する。各コンポーネントは Kind（Component / System / API / Resource）、Type（service / library / website 等）、Owner（GitHub Team）、Lifecycle（production / experimental / deprecated）の 4 属性をメタデータとして持つ。

**Software Templates** は新規コンポーネント作成の入口で、tier2 Microservice / tier3 Web App / tier3 MAUI App / 定型インフラリソース の 4 テンプレートを提供する。テンプレート実行は [01_tier2_tier3開発者向けGoldenPath初版.md](01_tier2_tier3開発者向けGoldenPath初版.md) 参照。

**TechDocs** は各リポジトリの `docs/` 配下 Markdown を自動ビルドし、Backstage 内で検索可能にする。リポジトリ内にドキュメントを保持するため、コードと文書の乖離を構造的に抑止する。

**プラグイン体系** は Argo CD / GitHub Actions / Harbor / Grafana / Kafka Topics / Keycloak / Flagd など外部ツールを Catalog の各コンポーネントページに埋め込む。プラグインは Backstage Marketplace から追加するか、採用側組織の固有プラグインを自作する。

この 4 機能が相互に参照することで、例えば「tier1 State API のダッシュボード」「tier1 State API の TechDocs」「tier1 State API の CI 状態」のすべてが Catalog の State API コンポーネントページから 1 クリックで辿れる状態を作る。

## 設計項目 DS-SW-DOC-081 ログインと認証 — Keycloak SSO 連携

Backstage への認証は Keycloak OIDC 経由の SSO で行う。採用側組織の社内 AD アカウントが Keycloak の Identity Provider として連携済みであり、開発者は AD アカウントで直接 Backstage にログインできる。

初回ログイン時は Backstage が自動的に `User` エンティティを Catalog に登録し、GitHub Team マッピングから所属組織を推定する。2 回目以降は 8 時間のセッション有効期限でログイン状態を保持する。

ログイン後のアクセス制御は Backstage 側の Role-Based Access Control（RBAC）と、各プラグイン側の認可を組み合わせる。Catalog の閲覧は全社員可能、Template 実行は開発者ロール以上、運用プラグインは運用者ロール以上、といった粒度で制御する。

## 設計項目 DS-SW-DOC-082 Software Catalog — サービス一覧と依存関係

Catalog の中核は、各コンポーネントが保持する `catalog-info.yaml` ファイルである。このファイルはリポジトリに併存し、コンポーネントのメタデータ・Owner・依存関係・API 参照を YAML で宣言する。Backstage は定期的にリポジトリをクロールし、最新の `catalog-info.yaml` を自動反映する。

`catalog-info.yaml` の記述例（tier2 サンプルサービス）を以下に示す。リリース時点では骨子のみを示し、採用初期（デモ）で実サンプルに差し替える。

```yaml
apiVersion: backstage.io/v1alpha1
kind: Component
metadata:
  name: sample-order-service
  description: tier2 サンプル発注サービス
  annotations:
    github.com/project-slug: k1s0-org/sample-order-service
    backstage.io/techdocs-ref: dir:.
spec:
  type: service
  lifecycle: experimental
  owner: team-order-tier2
  system: order-domain
  providesApis:
    - sample-order-api
  consumesApis:
    - k1s0-state-api
    - k1s0-pubsub-api
    - k1s0-decision-api
```

`consumesApis` フィールドの記述により、Backstage は「このサービスがどの tier1 API に依存するか」を自動で依存関係グラフに反映する。グラフ上で tier1 API ノードをクリックすると、全 tier2 依存元が一覧表示される。これにより、tier1 API の破壊的変更時の影響範囲分析が機械的に可能となる。

## 設計項目 DS-SW-DOC-083 Software Templates — Golden Path の入口

Software Templates は Backstage 上での「新規コンポーネント作成」ウィザードである。リリース時点 時点で以下 4 テンプレートを提供する。

- **tier2 Microservice (Go)**: Go + Dapr SDK（tier1 SDK 経由） + Gin フレームワーク
- **tier2 Microservice (C#)**: .NET 9 + tier1 C# SDK + ASP.NET Core Minimal API
- **tier3 Web App (TypeScript + React)**: React + tier1 TypeScript SDK + Vite ビルド
- **tier3 MAUI App (C#)**: .NET MAUI + tier1 C# SDK + MSIX 配布

テンプレート実行画面では、コンポーネント名・所属チーム・一次連絡先・テナント ID・PII 取扱の有無 を入力する。入力完了後、Backstage は GitHub 上で新規リポジトリを作成し、テンプレートファイルを展開し、Argo CD Application リソースを作成し、Catalog に新規 Component を登録する。

テンプレートのメンテナは k1s0 基盤チームである。破壊的変更を避けるため、既存テンプレートの必須項目削除は 段階移行時のみ許可する。SDK バージョン更新は Renovate が自動 PR を作成し、基盤チームがレビューして main マージする。

## 設計項目 DS-SW-DOC-084 TechDocs — リポジトリ内ドキュメントの集約検索

TechDocs はリポジトリの `docs/` 配下 Markdown を MkDocs でビルドし、Backstage 内で検索・閲覧可能にする。全リポジトリの `docs/` が横断検索されるため、例えば「Saga」で検索すると tier1・tier2・基盤の全設計書で Saga に言及している箇所が一覧表示される。

ドキュメント記述規約は本リポジトリ（k1s0 本体）でも同じ規約を適用する。各ページは「目的 → 前提 → 手順 → 参考」の 4 段構成とし、冒頭に「いつ読むか」「読後に何が分かるか」を明示する。

TechDocs ビルドは GitHub Actions で PR ごとに実行し、Markdown リンク切れ・メタデータ欠落を CI で検出する。ビルド成果物は MinIO（S3 互換）に格納し、Backstage が表示時にオブジェクトを取得する。

## 設計項目 DS-SW-DOC-085 プラグイン体系 — 外部ツール統合

リリース時点 時点で以下 7 プラグインを有効化する。各プラグインは Catalog の各コンポーネントページに専用タブを追加する形で統合する。

- **Argo CD プラグイン**: コンポーネントごとのデプロイ状態（Synced / Healthy / Degraded）を表示
- **GitHub Actions プラグイン**: CI の実行状態、直近 20 回の実行履歴を表示
- **Harbor プラグイン**: コンテナイメージのレジストリ情報、Trivy スキャン結果を表示
- **Grafana プラグイン**: 当該コンポーネントのダッシュボードを Catalog ページに埋め込み
- **Kafka Topics プラグイン**: 当該コンポーネントが produce / consume するトピックの lag と throughput を表示
- **Keycloak プラグイン**: 当該コンポーネントの OIDC クライアント設定、発行済みトークンの統計
- **Flagd プラグイン**: 当該コンポーネントに関連する Feature Flag 一覧、現在の有効状態

採用後の運用拡大時、Temporal Web・Trivy Operator・Istio ダッシュボード の 3 プラグインを追加する。採用側組織の固有プラグインとして「採用検討フロー連動」「既存 AD グループからの Ownership 自動算出」の 2 プラグインを自作する検討を進める。

## 設計項目 DS-SW-DOC-086 ペルソナ別主要画面遷移

開発者ペルソナの典型遷移は以下である。朝の着手時に Home → 自チーム担当 Component 一覧 → 対象 Component の TechDocs → 対象 Component の GitHub Actions タブ でビルド状態確認、の経路を 30 秒以内に辿れることを設計目標とする。

運用者ペルソナの典型遷移は、アラート受信時に Home の「My On-Call」ブロック → 対象 Component の Argo CD タブ → Grafana タブ → Runbook リンク（TechDocs 内）、の経路である。アラート受信から Runbook 到達まで 60 秒以内を設計目標とする。

アーキテクトペルソナの典型遷移は、System エンティティ一覧 → 対象 System の依存関係グラフ → 依存関係の破壊的変更影響分析、の経路である。グラフ表示から影響 Component 一覧出力まで 3 分以内を設計目標とする。

## 設計項目 DS-SW-DOC-087 段階別導入ロードマップ

Backstage 導入は リリース時点 着手で 採用後の運用拡大時 本格統合とする。採用検討時点では導入は始まっていない。

- **リリース時点（採用検討時点）**: 本章の骨子確定、Backstage の採用決定は既に構想設計 ADR 相当で確定済み。
- **採用初期 (デモ段階)**: 未導入。Catalog 相当情報は GitHub README で代替。
- **採用初期 (パイロット段階)**: 未導入。Catalog 相当情報を Confluence 代替で集約するかは検討中。
- **採用初期 (運用品質段階)**: 導入開始。Catalog・Templates・TechDocs・Argo CD プラグインの 4 機能を導入条件とする。
- **採用後の運用拡大時**: 全 7 プラグイン統合、採用側組織の固有プラグイン 2 種を自作、横断検索のチューニングを実施。

リリース時点 導入後の運用負荷増は、運用 採用側の小規模運用の範囲内に収めるために Backstage 自体の運用を基盤チームが担当し、各 tier の開発者・運用者は Backstage を「使うだけ」の分担とする。

## 設計項目 DS-SW-DOC-088 導入後の典型的つまずきと対処

リリース時点 導入時に他社事例で頻発するつまずきを事前に整理する。初期 3 か月で直面するであろう問題と対処を以下に示す。

**Catalog が実態と乖離する**: `catalog-info.yaml` の更新を怠ると、Catalog 上の情報が古くなり信頼性が失われる。対処として、`catalog-info.yaml` の必須項目を CI で検証し、欠落や不整合がある PR をマージさせない。

**Template が陳腐化する**: SDK 更新や CI 設定変更に Template が追従しないと、Template 生成物を受け取った開発者が古い仕様で開発を始めてしまう。対処として、Template 自体も Renovate で依存更新し、四半期ごとに基盤チームがメンテナンスレビューを行う。

**TechDocs ビルドが不安定**: Markdown の書式バリエーションが多様になるとビルド失敗が頻発する。対処として、Markdown Linter を PR 必須チェックに組み込み、書式違反を早期検出する。

**プラグインの権限設計が破綻**: プラグインごとに権限モデルが異なるため、例えば Grafana は閲覧可だが Argo CD は操作不可、といった整合性のある RBAC 設計が難しい。対処として、Backstage RBAC を「閲覧 / 開発 / 運用 / 基盤」の 4 ロール階層で固定し、プラグイン権限をこの階層にマップする。

## 対応要件一覧

本ファイルで採番した設計 ID（`DS-SW-DOC-080` 〜 `DS-SW-DOC-088`）と、充足する要件 ID を以下に列挙する。

- `DS-SW-DOC-080`（Backstage 4 機能）: `ADR-BS-001` / `DX-GP-001` / `BR-PLATUSE-005`
- `DS-SW-DOC-081`（Keycloak SSO）: `NFR-E-AC-001` / `NFR-E-RSK-001`
- `DS-SW-DOC-082`（Software Catalog）: `ADR-BS-001` / `DX-GP-002` / `BR-PLATUSE-006`
- `DS-SW-DOC-083`（Software Templates）: `DX-GP-003` / `ADR-BS-001`
- `DS-SW-DOC-084`（TechDocs）: `ADR-BS-001` / `DX-LD-002`
- `DS-SW-DOC-085`（プラグイン体系）: `ADR-BS-001` / `NFR-I-SLI-001` / `OR-SUP-007`
- `DS-SW-DOC-086`（ペルソナ別画面遷移）: `ADR-BS-001` / `BR-PLATOPS-006`
- `DS-SW-DOC-087`（段階別ロードマップ）: 間接対応（段階進行管理）
- `DS-SW-DOC-088`（つまずきと対処）: `ADR-BS-001` / `OR-SUP-007`
