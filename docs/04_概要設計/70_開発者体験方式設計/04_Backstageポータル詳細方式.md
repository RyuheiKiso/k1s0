# 04. Backstage ポータル詳細方式

本ファイルは k1s0 の開発者ポータルとして採用する Backstage の詳細構成方式を定める。対象は Software Catalog（全サービス登録）、TechDocs（Markdown 自動ビルド）、Software Templates（雛形生成）、プラグイン構成（GHA / Argo CD / Harbor / Grafana / Kafka / OpenBao / flagd）、SSO（Keycloak 連携）、デプロイ形態（Phase 1c 単 Pod → Phase 2 HA）、k1s0 固有カスタムプラグイン（API ドキュメント自動抽出 / Feature Flag 管理 UI）の 7 領域である。

## 本ファイルの位置付け

Backstage は「開発者が最初に開くポータル」として機能し、サービス一覧・ドキュメント・雛形生成・観測ダッシュボード・Feature Flag 管理などを 1 箇所に集約する。ポータルが弱いと、開発者は各ツールをタブで開いて切り替え、必要な情報が見つからず、Golden Path 10 分ルールが破綻する。

本設計は構想設計 ADR-BS-001（Backstage 採用）を前提として、Phase 1c で単 Pod デプロイ、Phase 2 で HA 化、カスタムプラグインの導入までを段階的に設計する。対応要件は DX-BS-001〜006（Backstage ポータル）、DX-GP-001〜006（Golden Path）、DX-FM-001〜007（Feature Management）である。

## 全体構造

Backstage の役割は以下 5 層で整理する。第 1 層は Service Discovery（Software Catalog）で、全 tier1 / tier2 / tier3 サービスをエンティティとして登録する。第 2 層は Documentation（TechDocs）で、各サービスの技術ドキュメントを Markdown から自動ビルドする。第 3 層は Templates（Software Templates）で、新規サービス作成を雛形化する。第 4 層は Observability（Grafana / Argo CD / Harbor プラグイン）で、運用ダッシュボードを集約する。第 5 層は Governance（Feature Flag 管理 / 監査ログ / Cost Insights）で、運用ガバナンスを可視化する。

Backstage の導入段階は 3 段階とする。Phase 1a〜1b では Backstage を導入せず、GitHub Wiki / Confluence で代替する。Phase 1c で単 Pod 構成の Backstage を導入し、Software Catalog / TechDocs / Templates の 3 機能を稼働させる。Phase 2 で HA 化（Backend 2 レプリカ + PostgreSQL）し、プラグイン全載せ + カスタムプラグイン投入する。

## Software Catalog の設計

Software Catalog は `catalog-info.yaml` ファイルで各サービスをエンティティ登録する。エンティティの種別は Component（サービス）、API（Protobuf / OpenAPI 定義）、System（複数 Component の集合）、Domain（事業領域）、User / Group（組織）の 5 種類とする。

Component の必須フィールドは以下の通りとする。`name`（一意）、`owner`（Group 参照、オーナーチーム）、`lifecycle`（production / experimental / deprecated）、`system`（所属 System）、`dependsOn`（依存する他 Component）、`providesApis` / `consumesApis`（公開 / 利用 API）、`tags`（tier1 / tier2 / tier3、言語、フェーズ）。このメタデータで「どのサービスが誰に所有され、どの API を提供 / 利用しているか」を全社横断で可視化する。

Catalog の登録ルールは以下の通りとする。GitOps リポジトリ（`k1s0-gitops`）の `catalog/` ディレクトリに `catalog-info.yaml` を配置し、Backstage が自動スキャンする。サービス新規作成時は Software Template が自動生成する雛形に `catalog-info.yaml` が含まれ、開発者が手動登録する必要はない。catalog 未登録のサービスは Kyverno ポリシーで Kubernetes へのデプロイを拒否する（Phase 2）。

## TechDocs の設計

TechDocs は Markdown から HTML ドキュメントを自動ビルドする仕組みで、Backstage に MkDocs ベースのレンダラが組み込まれている。各サービスの `docs/` ディレクトリを Backstage が検知し、UI 上で検索・閲覧可能な静的サイトに変換する。

ドキュメント規約は以下の通りとする。必須ファイルは `docs/index.md`（概要）、`docs/api.md`（API 仕様）、`docs/runbook.md`（障害対応手順）、`docs/adr/`（ADR 履歴）、`docs/getting-started.md`（ローカル起動手順）の 5 種類である。これらは Software Template が雛形として生成し、開発者は中身を埋めるだけで済む。

検索は Lunr.js（全文検索）を標準とし、Phase 2 で Elasticsearch バックエンドに切り替える。検索対象は全 TechDocs + Software Catalog のメタデータで、「k1s0 State API の使い方」などの自然な質問で関連ドキュメントが上位表示される。検索性能の目標はクエリ 500ms 以内で 10 件以上のヒットを返すこと、とする。

## Software Templates の設計

Software Templates は「雛形から新規リポジトリを作成する」仕組みで、Golden Path 10 分ルールの技術的な基盤である。Backstage の Scaffolder バックエンドが Cookiecutter 相当の雛形処理を実行し、GitHub リポジトリを作成・初期コミットを push する。

k1s0 で提供する標準テンプレートは以下 5 種類である。

| テンプレート | 対象 | 技術スタック |
| --- | --- | --- |
| tier2 Microservice Go | 新規業務サービス | Go + Dapr + gRPC + PostgreSQL |
| tier2 Microservice C# | レガシー移行向け | .NET 8 + Dapr + gRPC |
| tier3 Web SPA React | 業務画面 | React + Vite + k1s0 SDK |
| tier3 Native MAUI | ネイティブ業務アプリ | .NET MAUI + k1s0 SDK |
| バッチジョブ | 定期バッチ | Go + k1s0 Workflow API |

各テンプレートは以下を自動生成する。GitHub リポジトリ + ブランチ保護設定、`.github/workflows/ci.yaml`（CI 定義）、`Dockerfile` + `Tiltfile`（ローカル開発）、`deploy/` 配下の Helm Chart / Kubernetes マニフェスト、tier1 SDK 呼び出し雛形コード、`catalog-info.yaml`（Backstage Catalog 登録）、`docs/` 5 種類の雛形、である。

テンプレート実行の引数は以下 5 項目を標準とする。`name`（サービス名、kebab-case）、`owner`（所有チーム、Group 参照）、`description`（日本語説明）、`system`（所属 System）、`lifecycle`（experimental / production）。これらを Backstage UI のフォームから入力すると、1 分以内に GitHub リポジトリが作成される。詳細は [06_ゴールデンパス方式.md](06_ゴールデンパス方式.md) で扱う。

## プラグイン構成

Backstage は豊富なプラグインエコシステムを持つ。k1s0 では以下のプラグインを段階的に導入する。

| プラグイン | 役割 | 導入フェーズ |
| --- | --- | --- |
| GitHub Actions | ビルド状況表示 | Phase 1c |
| Argo CD | デプロイ状況・同期トリガ | Phase 1c |
| Harbor | イメージ一覧・脆弱性表示 | Phase 1c |
| Grafana | ダッシュボード埋め込み | Phase 1c |
| Kafka Topics | トピック一覧・メッセージ数 | Phase 2 |
| OpenBao | Secret 一覧（値は非表示） | Phase 2 |
| flagd（自作） | Feature Flag 管理 UI | Phase 2 |
| Cost Insights | 推定コスト可視化 | Phase 2 |
| TechRadar | 技術選定ガイドライン | Phase 2 |

プラグインは Backstage Backend プラグインと Frontend プラグインに分かれる。Backend プラグインは Kubernetes 経由で各システムの API を叩き、Frontend プラグインが結果を UI 上に表示する。認証は Backstage 本体の OIDC（Keycloak）セッションを引き継ぐ。

## SSO（Keycloak 連携）

Backstage へのログインは Keycloak OIDC（OpenID Connect）で実装する。社員はシングルサインオンで Backstage へ入り、Keycloak の Group 属性が Backstage の User / Group エンティティにマッピングされる。権限制御は Backstage Permissions プラグインで実装し、例えば「tier1 コンポーネント編集は tier1 チームのみ許可」「Feature Flag の本番環境切替は SRE チームのみ許可」などを宣言的に定義する。

認証の詳細は [../30_共通機能方式設計/01_認証方式.md](../30_共通機能方式設計/01_認証方式.md) と連動する。Backstage は Keycloak の client として登録し、client secret は OpenBao で管理、Kubernetes Secret 経由で Backstage Pod にマウントする。

## デプロイ形態

Phase 1c は単 Pod デプロイとする。Backstage（フロントエンド + バックエンド同居）1 Pod + PostgreSQL 1 Pod で合計 2 Pod、メモリ 2GB、CPU 1 コアで運用する。SLO は 99%（月 7.2 時間停止許容）、バックアップは日次 PostgreSQL ダンプ、復旧は手動（Runbook 整備）とする。

Phase 2 は HA 化する。Backstage Backend 2 レプリカ + Frontend CDN（MinIO + Envoy）、PostgreSQL は CloudNativePG の 3 ノード構成、SLO は 99.9%（月 43 分停止許容）、バックアップは継続 WAL アーカイブ（RPO 10 秒）、復旧は自動フェイルオーバー、とする。Phase 2 での稼働水準は tier1 本体と同等で、Backstage の停止が開発全般を止める影響を考慮する設計である。

## カスタムプラグインの設計

k1s0 固有の課題に対応するため、2 種類のカスタムプラグインを Phase 2 で開発する。

第 1 は API Documentation Auto-Extract プラグインである。各サービスの `.proto` ファイルから gRPC API ドキュメントを自動抽出し、Software Catalog の API エンティティとして登録する。開発者が .proto を更新すると、CI が自動再生成し、Backstage 上の API ドキュメントが即時更新される。手動のドキュメント同期が不要となり、「コードと仕様書の乖離」問題が構造的に発生しない。

第 2 は Feature Flag Management UI プラグインである。flagd の設定ファイル（Git 管理）を UI で編集し、PR を自動作成する。Argo CD が PR マージを検知して flagd 設定を配信する。Git 操作に不慣れな運用担当者（ビジネス部門の担当者を含む）でも Feature Flag を操作できる導線を提供する。詳細は [../30_共通機能方式設計/09_FeatureManagement方式.md](../30_共通機能方式設計/09_FeatureManagement方式.md) と連動する。

両プラグインは Backstage の Plugin API（TypeScript + React）で実装し、`plugins/k1s0-api-docs/` と `plugins/k1s0-feature-flag/` に配置する。ソースは Git 管理し、Backstage 本体と同じ CI パイプラインでビルド・デプロイする。

## 設計 ID 一覧

| 設計 ID | 設計項目 | 確定フェーズ | 対応要件 |
| --- | --- | --- | --- |
| DS-DEVX-BS-001 | Backstage 5 層構成 | Phase 1c | DX-BS-001 |
| DS-DEVX-BS-002 | Software Catalog（5 種類エンティティ） | Phase 1c | DX-BS-002 |
| DS-DEVX-BS-003 | TechDocs（MkDocs + 全文検索） | Phase 1c | DX-BS-002 |
| DS-DEVX-BS-004 | Software Templates（5 種類標準） | Phase 1c | DX-BS-003 / DX-GP-001 |
| DS-DEVX-BS-005 | プラグイン構成（GHA / Argo / Harbor / Grafana） | Phase 1c | DX-BS-004 |
| DS-DEVX-BS-006 | Keycloak OIDC SSO | Phase 1c | DX-BS-005 |
| DS-DEVX-BS-007 | Phase 1c 単 Pod デプロイ | Phase 1c | DX-BS-006 |
| DS-DEVX-BS-008 | Phase 2 HA デプロイ | Phase 2 | DX-BS-006 |
| DS-DEVX-BS-009 | API Documentation Auto-Extract Plugin | Phase 2 | DX-BS-004 |
| DS-DEVX-BS-010 | Feature Flag Management UI Plugin | Phase 2 | DX-FM-002 / DX-FM-003 |
| DS-DEVX-BS-011 | Kafka / OpenBao / Cost Insights Plugin | Phase 2 | DX-BS-004 |

## 対応要件一覧

本ファイルは要件定義書 50_開発者体験 DX-BS-001〜006（Backstage ポータル）、DX-GP-001〜006（Golden Path）、DX-FM-002〜003（Feature Flag UI 管理）に直接対応する。30_非機能要件 NFR-C-001（開発者体験）、60_事業契約 BC-UX-002（ポータル体験）、BC-TRN-002（教育訓練導線）とも連動する。構想設計 ADR-BS-001（Backstage 採用）、ADR-FM-001（flagd 採用）、ADR-OBS-001（Grafana LGTM）を前提とする。10 分ルール DX-GP-003 の実現において、Software Templates からの雛形生成（30 秒〜1 分）が重要な時間予算を占める。
