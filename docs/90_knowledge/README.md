# 90_knowledge — 技術学習用ドキュメント

本ディレクトリは k1s0 の設計・実装で参照する技術要素を、新規参画者でも前提なしに読み進められる粒度で整理した学習資料群である。要件定義 (`03_要件定義/`) や構想設計 (`02_構想設計/`) で前提として扱われる用語・パターン・プロダクトを、独立した知識ノードとして切り出すことで、設計文書側の散文を本質的な意思決定に集中させることを目的とする。

各ドキュメントは `/knowledge` スラッシュコマンドで生成される。命名は `<対象>_<切り口>.md` 形式で、`<切り口>` には `basics`（概論・採用判断の前提）または特定トピック名（例: `ownership`、`ambient-mode`、`virtual-dom`）を用いる。図表は同階層の `img/` 配下に drawio + SVG として配置し、本文に SVG を埋め込む。

学習資料の置き場をカテゴリで分けているのは、参照される文脈が異なるためである。要件定義から逆引きされるのはほとんどが `infra/` と `protocol/` であり、設計判断の根拠を辿るのは `architecture/` と `methodology/`、実装時の言語仕様参照は `language/` と `library/`、CI/CD やビルド時に参照されるのは `tool/` である。`other/` はプロジェクト運営の思想・キャリア観など、技術カテゴリに収まらないがチームの共通認識として残しておく価値がある資料を置く。

## カテゴリ別ドキュメント

### architecture — アーキテクチャ思想

長期的に保守可能なシステム構造の前提となる設計思想を扱う。k1s0 の tier1/tier2/tier3 階層構造や Dapr ファサード方針の正当化根拠を追跡する際に参照する。

- [clean-architecture_basics.md](architecture/clean-architecture_basics.md)
- [vertical-slice-architecture_basics.md](architecture/vertical-slice-architecture_basics.md)

### framework — フレームワーク

UI / アプリケーションフレームワークの基礎概念。tier3 のオペレーション UI を構築する際の前提知識として整理する。

- [react_basics.md](framework/react_basics.md)
- [react_virtual-dom.md](framework/react_virtual-dom.md)

### infra — インフラ・運用基盤

k1s0 の tier1 を構成する CNCF プロダクト群と、その周辺で運用判断に関わる SRE 概念を集約する。要件定義の非機能要件 (`30_非機能要件/`) で「Dapr/Istio Ambient/CloudNativePG/Keycloak を採用する」と書かれた際に、その採用根拠と動作モデルを辿るために用いる。

#### 制御基盤

- [kubernetes_basics.md](infra/kubernetes_basics.md)
- [k3s_basics.md](infra/k3s_basics.md)
- [kubeadm_basics.md](tool/kubeadm_basics.md)
- [kube-vip_basics.md](infra/kube-vip_basics.md)
- [metallb_basics.md](infra/metallb_basics.md)
- [headlamp_basics.md](infra/headlamp_basics.md)
- [kube-state-metrics_basics.md](infra/kube-state-metrics_basics.md)

#### アプリ基盤

- [dapr_basics.md](infra/dapr_basics.md)
- [istio_ambient-mode.md](infra/istio_ambient-mode.md)
- [keda_basics.md](infra/keda_basics.md)
- [temporal_basics.md](infra/temporal_basics.md)
- [argo-rollouts_basics.md](infra/argo-rollouts_basics.md)

#### データ基盤

- [cloudnativepg_basics.md](infra/cloudnativepg_basics.md)
- [postgresql_basics.md](infra/postgresql_basics.md)
- [pg_partman_basics.md](infra/pg_partman_basics.md)
- [pgvector_basics.md](infra/pgvector_basics.md)
- [longhorn_basics.md](infra/longhorn_basics.md)
- [valkey_basics.md](infra/valkey_basics.md)
- [minio_basics.md](infra/minio_basics.md)
- [emqx_basics.md](infra/emqx_basics.md)

#### 認証・秘匿

- [keycloak_basics.md](infra/keycloak_basics.md)
- [openbao_basics.md](infra/openbao_basics.md)
- [external-secrets-operator_basics.md](infra/external-secrets-operator_basics.md)
- [kyverno_basics.md](infra/kyverno_basics.md)

#### 観測・監査

- [otel-collector_basics.md](infra/otel-collector_basics.md)
- [grafana-tempo_basics.md](infra/grafana-tempo_basics.md)
- [grafana-pyroscope_basics.md](infra/grafana-pyroscope_basics.md)
- [actuator_basics.md](infra/actuator_basics.md)
- [kubeshark_basics.md](infra/kubeshark_basics.md)

#### 配布・更新

- [harbor_basics.md](infra/harbor_basics.md)
- [mender_basics.md](infra/mender_basics.md)
- [feature-flag_basics.md](infra/feature-flag_basics.md)

#### 開発者基盤

- [backstage_basics.md](infra/backstage_basics.md)
- [idp_basics.md](infra/idp_basics.md)
- [nodejs_basics.md](infra/nodejs_basics.md)

#### アーキテクチャ概念

- [microservice-architecture_basics.md](infra/microservice-architecture_basics.md)
- [cncf_basics.md](infra/cncf_basics.md)
- [oasis_basics.md](infra/oasis_basics.md)
- [vendor-lockin_basics.md](infra/vendor-lockin_basics.md)

#### SRE

- [sre_basics.md](infra/sre_basics.md)
- [sre_toil.md](infra/sre_toil.md)
- [sli_basics.md](infra/sli_basics.md)
- [slo_basics.md](infra/slo_basics.md)
- [sla_basics.md](infra/sla_basics.md)
- [runbook_basics.md](infra/runbook_basics.md)
- [chaos-engineering_basics.md](infra/chaos-engineering_basics.md)

### language — プログラミング言語

k1s0 の自作領域 (Rust) と Dapr ファサード層 (Go) の言語仕様。Rust の所有権など、設計判断に直結するモデルは独立ノードとして切り出す。

- [rust_basics.md](language/rust_basics.md)
- [rust_ownership.md](language/rust_ownership.md)
- [rust_crates.md](language/rust_crates.md)
- [rust_zero-cost-abstractions.md](language/rust_zero-cost-abstractions.md)
- [go_basics.md](language/go_basics.md)
- [go_goroutines.md](language/go_goroutines.md)

### library — ライブラリ

実装で常用するライブラリの API モデルと採用理由。マイグレーション・テストコンテナなど、設計判断と実装慣習の境界に位置するものを扱う。

- [sqlx-migrate_basics.md](library/sqlx-migrate_basics.md)
- [testcontainers_basics.md](library/testcontainers_basics.md)

### methodology — 手法・方針

ADR・MVP・E2E テスト・OSS ライセンスなど、開発プロセスとガバナンスを支える方法論。要件定義の付録 (`90_付録/`) や ADR (`02_構想設計/`) と相互参照する。

#### 意思決定

- [adr_basics.md](methodology/adr_basics.md)
- [bus-factor_basics.md](methodology/bus-factor_basics.md)
- [champion-program_basics.md](methodology/champion-program_basics.md)

#### プロダクト企画

- [mvp_basics.md](methodology/mvp_basics.md)
- [persona_basics.md](methodology/persona_basics.md)
- [fr_basics.md](methodology/fr_basics.md)

#### 検証・レビュー

- [e2e-testing_basics.md](methodology/e2e-testing_basics.md)
- [lgtm_basics.md](methodology/lgtm_basics.md)

#### ライセンス

- [apache-license_basics.md](methodology/apache-license_basics.md)
- [mit-license_basics.md](methodology/mit-license_basics.md)

#### AI エージェント開発

- [harness-engineering_basics.md](methodology/harness-engineering_basics.md)

### protocol — 通信プロトコル

tier1 内部の gRPC、tier2/tier3 への公開 API、IoT 系の MQTT など、k1s0 が扱う通信プロトコルの仕様と選定根拠。要件定義の機能要件 (`20_機能要件/`) で API 形式を決定する際の前提となる。

- [gRPC_basics.md](protocol/gRPC_basics.md)
- [protobuf_basics.md](protocol/protobuf_basics.md)
- [proto3_basics.md](protocol/proto3_basics.md)
- [rest-api_basics.md](protocol/rest-api_basics.md)
- [websocket_basics.md](protocol/websocket_basics.md)
- [mqtt_basics.md](protocol/mqtt_basics.md)
- [rtt_basics.md](protocol/rtt_basics.md)
- [llms-txt_basics.md](protocol/llms-txt_basics.md)

### tool — ツール

CI/CD・パッケージング・ビルド関連の周辺ツール。実装時よりも CI/CD パイプライン (`02_構想設計/04_CICDと配信/`) の整備で参照する頻度が高い。

#### パッケージング・配布

- [docker_basics.md](tool/docker_basics.md)
- [helm_basics.md](tool/helm_basics.md)
- [kustomize_basics.md](tool/kustomize_basics.md)
- [argo-cd_basics.md](tool/argo-cd_basics.md)

#### CI/CD

- [github-actions_basics.md](tool/github-actions_basics.md)
- [renovate_basics.md](tool/renovate_basics.md)
- [tilt_basics.md](tool/tilt_basics.md)

#### ビルド・解析

- [buf_basics.md](tool/buf_basics.md)
- [llvm_basics.md](tool/llvm_basics.md)
- [roslyn-analyzer_basics.md](tool/roslyn-analyzer_basics.md)

#### インフラ

- [terraform_basics.md](tool/terraform_basics.md)
- [kubeadm_basics.md](tool/kubeadm_basics.md)
- [golang-migrate_basics.md](tool/golang-migrate_basics.md)

#### ライセンス

- [agpl-3.0_basics.md](tool/agpl-3.0_basics.md)

### other — その他

技術カテゴリには収まらないが、プロジェクトの言語化・思考様式・キャリア観の共通認識として保持する資料。`tractatus`（論理哲学論考）や `language-games`（言語ゲーム）は、生成 AI とプロンプトエンジニアリングを哲学的基盤から捉え直すための参照点として置いている。

#### 哲学・言語論

- [tractatus_basics.md](other/tractatus_basics.md)
- [tractatus_generative-ai-relation.md](other/tractatus_generative-ai-relation.md)
- [tractatus_prompt-engineering-relation.md](other/tractatus_prompt-engineering-relation.md)
- [language-games_prompt-engineering-relation.md](other/language-games_prompt-engineering-relation.md)

#### キャリア観

- [career-levels_junior.md](other/career-levels_junior.md)
- [career-levels_mid.md](other/career-levels_mid.md)
- [career-levels_senior.md](other/career-levels_senior.md)
- [career-levels_staff.md](other/career-levels_staff.md)
- [career-levels_principal.md](other/career-levels_principal.md)
- [career-levels_distinguished.md](other/career-levels_distinguished.md)
- [career-levels_junior-senior-principal.md](other/career-levels_junior-senior-principal.md)

## 追加方針

新しい技術要素を採用判断の俎上に載せる場合は、`02_構想設計/` や ADR で議論を始める前に、まず本ディレクトリに `basics` 資料を起こすこと。設計文書側で「Dapr とは何か」「ZEN Engine がなぜ Drools の代替たり得るか」といった前提説明を毎回繰り返さないために、知識ノードへの参照リンクで済ませられる状態を保つことが重要である。
