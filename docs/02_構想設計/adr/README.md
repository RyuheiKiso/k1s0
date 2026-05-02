# ADR 索引（軽量版）

本 README は `docs/02_構想設計/adr/` 配下の Architecture Decision Record を、ファイル本体に最短距離で到達するための軽量索引である。k1s0 では ADR を技術選定と決定根拠の一次記録として扱い、`ADR-DOMAIN-NNN-short-name.md` 形式のファイルをリリース時点で 47 件蓄積している。新規参画者・外部レビュワー・監査担当がディレクトリを直接辿って目的の ADR を探す際、ファイル名のみでは何が書いてあるか把握しづらいため、本 README が「ファイル名 + 1 行要約」の段で探索を支援する。

本索引は軽量版であり、各 ADR の決定内容要約・却下された代替案・影響章・改訂履歴といった詳細情報は扱わない。それらは概要設計側の詳細索引 [../../04_概要設計/90_付録/02_ADR索引.md](../../04_概要設計/90_付録/02_ADR索引.md) が担う。両索引は重複ではなく役割分担の関係にある。本 README は「ADR 本体に辿り着く入口」、詳細索引は「ADR 本体を読まずに要点を掴むための要約集」として機能する。概要設計の読者は詳細索引から先に見て、そこから本 README 経由で ADR 本体に飛ぶ導線を想定している。

## ADR の命名規約

ADR ファイルは `ADR-DOMAIN-NNN-short-name.md` 形式で命名する。`DOMAIN` はドメイン系列（例: `TIER1`、`CICD`、`DATA`、`DIR`）を指し、`NNN` は各系列内の連番、`short-name` はケバブケース小文字で決定内容を短く示す。横断的な初期決定は `ADR-000x` 系列（例: `ADR-0001-istio-ambient-vs-sidecar.md`）を用いる。ドメイン系列の採番ルールの詳細は Skill `docs-adr-authoring`（`.claude/skills/docs-adr-authoring/SKILL.md`）を参照。

## ADR のステータス

ADR は `Proposed`（起案中、レビュー待ち）/ `Accepted`（採択済、本プロジェクトの決定事項）/ `Deprecated`（廃止、代替 ADR なし）/ `Superseded by ADR-MMMM`（後継 ADR によって置換）の 4 ステータスで運用する。決定済み ADR は原則として書き換えない。変更が必要な場合は新規 ADR を起票し、旧 ADR のステータスを `Superseded by ADR-NNNN` に更新する。

## 領域別 ADR 一覧

本 README では ADR を 15 領域に分類する。領域分類は概要設計側の詳細索引 [../../04_概要設計/90_付録/02_ADR索引.md](../../04_概要設計/90_付録/02_ADR索引.md) と完全に一致しており、両者のクロスリファレンスを容易にする。

### 基本構造（6 件）

プラットフォームの根幹を成す ADR。これらが崩れるとプラットフォーム全体が成立しない。

- [ADR-0001](./ADR-0001-istio-ambient-vs-sidecar.md) — Istio Ambient mesh を採用しサイドカーレス構成にする
- [ADR-0002](./ADR-0002-diagram-layer-convention.md) — 全 drawio 図で 4 レイヤ記法を必須化する
- [ADR-0003](./ADR-0003-agpl-isolation-architecture.md) — AGPL OSS をネットワーク境界で隔離する
- [ADR-TIER1-001](./ADR-TIER1-001-go-rust-hybrid.md) — tier1 を Go ファサード + Rust 自作領域で構成
- [ADR-TIER1-002](./ADR-TIER1-002-protobuf-grpc.md) — tier1 内部は全て Protobuf gRPC 通信
- [ADR-TIER1-003](./ADR-TIER1-003-language-opacity.md) — tier2/3 から内部言語を物理的に不可視化

### ディレクトリ構造（3 件）

モノレポ直下の物理配置を確定する ADR 群。実装開始前に昇格・分離を決めて移動コストを排除する。

- [ADR-DIR-001](./ADR-DIR-001-contracts-elevation.md) — Protobuf 契約を `src/contracts/` へ昇格
- [ADR-DIR-002](./ADR-DIR-002-infra-separation.md) — infra / deploy / ops をルート直下に 3 階層分離
- [ADR-DIR-003](./ADR-DIR-003-sparse-checkout-cone-mode.md) — Sparse Checkout cone + partial clone を標準化

### データ層（4 件）

ステートを預かる OSS の選定 ADR。RDB・メッセージ・オブジェクト・キャッシュの 4 本柱。

- [ADR-DATA-001](./ADR-DATA-001-cloudnativepg.md) — CloudNativePG で PostgreSQL を HA 運用
- [ADR-DATA-002](./ADR-DATA-002-strimzi-kafka.md) — Strimzi Operator で Apache Kafka を運用
- [ADR-DATA-003](./ADR-DATA-003-minio.md) — MinIO を S3 互換オブジェクトストレージに採用
- [ADR-DATA-004](./ADR-DATA-004-valkey.md) — Valkey を冪等キー・セッションキャッシュに採用

### セキュリティ（3 件）

認証・秘密管理・サービス認証の 3 本柱。全 API に横断的に影響する。

- [ADR-SEC-001](./ADR-SEC-001-keycloak.md) — Keycloak を企業 IdP として採用
- [ADR-SEC-002](./ADR-SEC-002-openbao.md) — OpenBao で KMS・シークレット管理
- [ADR-SEC-003](./ADR-SEC-003-spiffe-spire.md) — SPIFFE/SPIRE でサービス認証の SVID 配布

### ルールエンジン（2 件）

判定ロジックとワークフローの二分割。短時間判定 = ZEN、長時間ワークフロー = Temporal。

- [ADR-RULE-001](./ADR-RULE-001-zen-engine.md) — ZEN Engine を Decision API の判定基盤に採用
- [ADR-RULE-002](./ADR-RULE-002-temporal.md) — Temporal を長時間ワークフローに採用

### CI/CD・リリース・ポリシー（5 件）

継続的配信の中核 OSS と運用ガバナンスを束ねる領域。配信とポリシーは相互に結合するため 1 節で扱う。

- [ADR-CICD-001](./ADR-CICD-001-argocd.md) — Argo CD で GitOps 配信を行う
- [ADR-CICD-002](./ADR-CICD-002-argo-rollouts.md) — Argo Rollouts で Canary デプロイを行う
- [ADR-CICD-003](./ADR-CICD-003-kyverno.md) — Kyverno で admission ポリシーを強制する
- [ADR-REL-001](./ADR-REL-001-progressive-delivery-required.md) — Progressive Delivery を全リリースで必須化
- [ADR-POL-001](./ADR-POL-001-kyverno-dual-ownership.md) — Kyverno を技術提案・統制承認の二分所有で運用
- [ADR-POL-002](./ADR-POL-002-local-stack-single-source-of-truth.md) — ローカル kind cluster の構成 SoT を tools/local-stack/up.sh に統一

### 観測性（3 件）

3 シグナルの OSS 構成とインシデント分類体系。Grafana LGTM + OpenTelemetry を基盤に単一分類を乗せる。

- [ADR-OBS-001](./ADR-OBS-001-grafana-lgtm.md) — 観測性スタックに Grafana LGTM を採用
- [ADR-OBS-002](./ADR-OBS-002-otel-collector.md) — OpenTelemetry Collector で計装を統一
- [ADR-OBS-003](./ADR-OBS-003-incident-taxonomy-unified.md) — 可用性とセキュリティを単一分類体系で管理

### Feature Flag（1 件）

Feature Management の OSS 選定。OpenFeature 準拠。

- [ADR-FM-001](./ADR-FM-001-flagd-openfeature.md) — flagd を OpenFeature 準拠の FM として採用

### 開発者体験（3 件）

DX の思想と計測基盤、ローカル開発のホスト Docker ランタイム選定。Paved Road で Golden Path を一本化し、DX メトリクスを稼働 SLI と分離し、Windows + WSL2 ホストの Docker ランタイムを WSL ネイティブ docker-ce で固定する。

- [ADR-DEV-001](./ADR-DEV-001-paved-road.md) — 開発者体験の根幹思想に Paved Road を採用
- [ADR-DEV-002](./ADR-DEV-002-windows-wsl2-docker-runtime.md) — Windows 11 + WSL2 環境の Docker ランタイムに WSL ネイティブ docker-ce を採用
- [ADR-DX-001](./ADR-DX-001-dx-metrics-separation.md) — DX メトリクスを稼働 SLI と分離して管理する

### 依存管理・サプライチェーン（2 件）

依存更新とビルドサプライチェーンの信頼性を担保する領域。Renovate + SLSA で改ざん耐性を構造化する。

- [ADR-DEP-001](./ADR-DEP-001-renovate-central.md) — 依存更新中枢に Renovate を採用
- [ADR-SUP-001](./ADR-SUP-001-slsa-staged-adoption.md) — SLSA v1.1 を L2 でリリース、L3 を運用蓄積後の到達目標

### ストレージ・移行・ポータル（5 件）

インフラ層とライフサイクル層の ADR。単独では影響範囲が限定的だが各章の前提となる。

- [ADR-STOR-001](./ADR-STOR-001-longhorn.md) — Longhorn をブロックストレージに採用
- [ADR-STOR-002](./ADR-STOR-002-metallb.md) — MetalLB を L4 LoadBalancer に採用
- [ADR-MIG-001](./ADR-MIG-001-net-framework-sidecar.md) — .NET Framework 資産を sidecar で段階移行
- [ADR-MIG-002](./ADR-MIG-002-api-gateway.md) — 既存 API Gateway を Envoy Gateway へ段階切替
- [ADR-BS-001](./ADR-BS-001-backstage.md) — Backstage を開発者ポータル基盤に採用

### Kubernetes 基盤・ネットワーク（3 件）

vanilla K8s 上のクラスタ ブートストラップ・互換性維持・データプレーン CNI を確定する領域。tier1〜tier3 すべての前提となる。

- [ADR-INFRA-001](./ADR-INFRA-001-kubernetes-cluster-bootstrap.md) — Kubernetes クラスタを kubeadm + Cluster API で構築する
- [ADR-CNCF-001](./ADR-CNCF-001-cncf-conformance.md) — vanilla Kubernetes（CNCF Conformance 互換）を維持する
- [ADR-NET-001](./ADR-NET-001-cni-selection.md) — production CNI に Cilium、kind 検証用に Calico を使い分ける

### 分散ランタイム・スケール（2 件）

tier1 ファサード層の分散ランタイムと event 駆動スケールを束ねる領域。アプリ層の建付けを規定する。

- [ADR-DAPR-001](./ADR-DAPR-001-dapr-operator.md) — 分散ランタイムに Dapr Operator を採用する
- [ADR-SCALE-001](./ADR-SCALE-001-keda-event-driven-autoscaling.md) — Event-driven autoscaling に KEDA を採用する

### tier3 アーキテクチャ（3 件）

Web SPA / Native アプリの集約戦略と実装方式を確定する領域。client 多様性と採用組織の人材プールを橋渡しする。

- [ADR-TIER3-001](./ADR-TIER3-001-bff-pattern.md) — tier3 client ごとに専用 BFF を配置する
- [ADR-TIER3-002](./ADR-TIER3-002-spa-plus-bff.md) — tier3 Web を React + Vite SPA + Go BFF で構成する
- [ADR-TIER3-003](./ADR-TIER3-003-dotnet-maui-native.md) — tier3 Native アプリに .NET MAUI を採用する

### 運用ライフサイクル（1 件）

夜間休日 SEV1 を起案者不在で対応するためのプロセス資産を確定する領域。Runbook の形式・粒度・検証の構造化で属人性を排除する。

- [ADR-OPS-001](./ADR-OPS-001-runbook-standardization.md) — Runbook を 8 セクション + YAML frontmatter + Chaos Drill で標準化する

## 詳細索引への誘導

各 ADR の決定内容要約・却下された代替案・影響章・改訂履歴は、概要設計側の詳細索引 [../../04_概要設計/90_付録/02_ADR索引.md](../../04_概要設計/90_付録/02_ADR索引.md) を参照。アルファベット順索引・時系列索引・未起票 ADR 一覧（仮番）も同詳細索引に集約されている。
