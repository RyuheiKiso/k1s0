# 02. ADR 索引

本ファイルは構想設計書 [../../02_構想設計/adr/](../../02_構想設計/adr/) で採択された 36 個の ADR（Architecture Decision Record）を、概要設計書から参照する際の一覧索引である。各 ADR の決定内容要約・却下された代替案・影響章・改訂履歴を付加情報として記録し、概要設計書からの逆引きと、構想設計改訂時の波及把握の双方に耐える索引として機能させる。

## 本ファイルの位置付け

ADR は技術選定の決定記録であり、本来の所在は [../../02_構想設計/adr/](../../02_構想設計/adr/) にある。しかし概要設計書から ADR を参照する頻度は極めて高く、毎回構想設計書側のディレクトリを辿るのは読み手負担が大きい。また ADR ファイル単体には「どの概要設計章に影響するか」が書かれていないため、ADR 側から見た影響範囲の把握も別途必要になる。

本索引はこれら 2 点を補うため、概要設計書側に ADR の要約と影響範囲を集約する。要約のみで完結する読み方（概要設計の読者向け）と、本索引からリンクで ADR 本体に飛ぶ読み方（詳細把握の必要がある読者向け）の双方を両立する。

ADR の相関マトリクス（概要設計章との n:n 対応）は [../80_トレーサビリティ/05_構想設計ADR相関マトリクス.md](../80_トレーサビリティ/05_構想設計ADR相関マトリクス.md) に記述する。本索引は単純な索引（ADR 番号 → 要約 + リンク）を主目的とし、相関マトリクスとは役割を分ける。

本章は索引層のため設計 ID を採番しない。

## 索引の構造

索引は以下の 6 列で構成する。1 行読めば ADR の決定内容と採否根拠が把握できる密度とする。

- **ADR 番号**: 構想設計書で採番された ADR 識別子。
- **タイトル**: ADR ファイル名と同一の表題。
- **決定内容要約**: ADR 本文の「決定」節を 1〜2 文に要約。
- **却下された代替案**: ADR 本文の「検討代替案」節で却下された選択肢。
- **影響章**: 概要設計書の主要適用章（詳細は相関マトリクス参照）。
- **改訂履歴**: 最新改訂の日付と版（詳細は ADR 本体参照）。

## 領域別索引

ADR を技術領域で 11 グループ（基本構造 / ディレクトリ構造 / データ層 / セキュリティ / ルールエンジン / CI/CD・リリース・ポリシー / 観測性 / Feature Flag / 開発者体験 / 依存管理・サプライチェーン / ストレージ・移行・ポータル）に分類する。同じ領域の ADR は互いに関連性が強く、相互参照の頻度が高い。k1s0 リリース前までの集約時点で 37 件に到達しており、内訳は基本構造系が骨格を成し、周辺領域がそれを支える形で配置される。

### 基本構造 ADR（6 件）

プラットフォームの根幹を成す ADR。これらが崩れるとプラットフォーム全体が成立しない。

| ADR 番号 | タイトル | 決定内容要約 | 却下された代替案 | 影響章 | 改訂履歴 |
|---|---|---|---|---|---|
| ADR-0001 | Istio Ambient mesh 採用 | Istio Ambient モードを採用し、サイドカーレス構成で L4（ztunnel）と L7（waypoint proxy）を分離する | Istio Sidecar モード、Linkerd、Cilium Service Mesh | 10_SYS/03, 50_NFR/05, 30_CF/02 | v1.0 2026-01-15 |
| ADR-0002 | 図解記法規約 | 4 レイヤ記法（アプリ暖色 / ネットワーク寒色 / インフラ中性灰 / データ薄紫）を全 drawio で採用 | mermaid 統一、自由記法 | 全章（drawio 作成時） | v1.0 2026-02-03 |
| ADR-0003 | AGPL OSS の扱い | AGPL OSS はネットワーク境界で隔離し、tier2/tier3 のソース公開義務を回避する | AGPL 不採用、全面公開、商用ライセンス購入 | 30_CF, 50_NFR/05, 75_BUS/07 | v1.0 2026-02-20 |
| ADR-TIER1-001 | Go+Rust ハイブリッド | tier1 内部で Dapr ファサード層 = Go、自作領域（ZEN/crypto/CLI）= Rust、境界は Protobuf gRPC | Go 単一、Rust 単一、Java 採用 | 20_SW/01, 03, 40_CTRL | v1.1 2026-03-10 |
| ADR-TIER1-002 | Protobuf gRPC 内部通信 | tier1 内部サービス間通信は全て Protobuf gRPC 必須、tier2/tier3 境界のみ JSON over HTTP | JSON REST 統一、REST + gRPC 併用、GraphQL | 20_SW/03 | v1.0 2026-03-15 |
| ADR-TIER1-003 | tier2/tier3 から内部言語を不可視化 | tier2/tier3 は Protobuf IDL 生成クライアント SDK のみを利用し、tier1 内部の Go/Rust 実装・内部パッケージを露出させない。モノレポ境界封鎖と lint rule で物理的に強制 | モノレポ全言語自由 import、言語非依存 REST のみ公開 | 20_SW/01, 03, 70_DEVX/04, 60_MIG, 75_BUS/04 | v1.0 2026-04-19 |

### ディレクトリ構造 ADR（3 件）

モノレポ直下の物理配置を確定する ADR 群。実装コードがまだ書かれていない リリース時点 段階で昇格・分離を決め、実装開始後の移動コストを排除することを狙う。契約・インフラ・運用の責務境界を `src/contracts/` / `infra/` / `deploy/` / `ops/` として階層化し、スパースチェックアウトで役割別ワーキングセットを最小化する。

| ADR 番号 | タイトル | 決定内容要約 | 却下された代替案 | 影響章 | 改訂履歴 |
|---|---|---|---|---|---|
| ADR-DIR-001 | Protobuf 契約ディレクトリを `src/contracts/` に昇格 | `src/tier1/contracts/` から `src/contracts/` 直下に昇格し、tier1/2/3/SDK 横断資産であることをパスで示す。CODEOWNERS と buf module 境界を `src/contracts/` 起点に整合 | `src/tier1/contracts/` 維持、独立リポジトリ化、ルート直下 `contracts/` | 20_SW/01, 03 | v1.0 2026-04-23 |
| ADR-DIR-002 | infra / deploy / ops の 3 階層分離とルート昇格 | `src/tier1/infra/` を廃止し、クラスタ素構成 `infra/` / GitOps 配信 `deploy/` / 運用領域 `ops/` の 3 階層をリポジトリルートに昇格。CNCF Platform Engineering Maturity Model に準拠 | `src/tier1/infra/` 維持、`src/infra/` 昇格、2 階層分離 | 20_SW/01, 70_DEVX/01, 55_OPS | v1.0 2026-04-23 |
| ADR-DIR-003 | Git Sparse Checkout cone mode + Partial Clone 標準運用 | 役割別 cone 定義 10 種（tier1-rust-dev / tier2-dev / infra-ops 等）を `.sparse-checkout/roles/` に配置し、partial clone + sparse index を標準手順化。60 万行規模モノレポの IDE 応答性を確保 | Meta Sapling + EdenFS、VFS for Git、フルチェックアウト、multi-repo 化 | 70_DEVX/01, 02, 04 | v1.0 2026-04-23 |

### データ層 ADR（4 件）

ステートを預かる OSS の選定 ADR。RDB・メッセージ・オブジェクト・キャッシュの 4 本柱。

| ADR 番号 | タイトル | 決定内容要約 | 却下された代替案 | 影響章 | 改訂履歴 |
|---|---|---|---|---|---|
| ADR-DATA-001 | CNPG PostgreSQL 採用 | CloudNativePG Operator 経由で PostgreSQL を運用。HA = 1 プライマリ + 2 レプリカ、Barman バックアップ | Zalando Postgres Operator、Crunchy、Patroni + StatefulSet | 10_SYS/04, 20_SW/04, 50_NFR/01, 02, 07 | v1.0 2026-02-05 |
| ADR-DATA-002 | Strimzi Kafka 採用 | Strimzi Operator 経由で Apache Kafka を運用。At-Least-Once、Schema Registry 併設 | Confluent Platform、Redpanda、NATS JetStream、RabbitMQ | 10_SYS/04, 20_SW/02, 40_CTRL/04 | v1.0 2026-02-07 |
| ADR-DATA-003 | MinIO オブジェクトストレージ | MinIO を S3 互換ストレージとして 4 ノード erasure coding で運用 | Ceph RGW、Rook、SeaweedFS、自前 NFS | 10_SYS/04, 20_SW/02, 50_NFR/01 | v1.0 2026-02-10 |
| ADR-DATA-004 | Valkey キャッシュ採用 | Redis フォーク Valkey を冪等キー・セッションキャッシュに採用。6 ノード Sentinel 構成 | Redis（BSL ライセンス懸念）、KeyDB、DragonflyDB | 10_SYS/04, 40_CTRL/02, 50_NFR/02 | v1.0 2026-02-12 |

### セキュリティ ADR（3 件）

認証・秘密管理・サービス認証の 3 本柱。全 API に横断的に影響する。

| ADR 番号 | タイトル | 決定内容要約 | 却下された代替案 | 影響章 | 改訂履歴 |
|---|---|---|---|---|---|
| ADR-SEC-001 | Keycloak 認証採用 | Keycloak を企業 IdP 前面に配置。OIDC 準拠、SSO 統合、マルチテナントは Realm 分離 | Authentik、Dex、Ory Hydra、Azure AD 直接連携 | 30_CF/01, 50_NFR/05, 75_BUS/03 | v1.0 2026-02-18 |
| ADR-SEC-002 | OpenBao 秘密管理採用 | Vault フォーク OpenBao で KMS・シークレット管理。KV/PKI/Transit/Database エンジン有効化 | HashiCorp Vault（BSL ライセンス懸念）、SOPS + age、Sealed Secrets | 30_CF/08, 20_SW/02 | v1.0 2026-02-22 |
| ADR-SEC-003 | SPIFFE SVID 採用 | SPIRE サーバーと SPIFFE ID でサービス認証。Istio Ambient と統合し X.509 SVID を自動配布 | 独自 mTLS、Istio Citadel のみ、JWT ベースサービス認証 | 30_CF/02, 50_NFR/05, 10_SYS/03 | v1.0 2026-02-25 |

### ルールエンジン ADR（2 件）

判定ロジックとワークフローの二分割。短時間判定 = ZEN、長時間ワークフロー = Temporal。

| ADR 番号 | タイトル | 決定内容要約 | 却下された代替案 | 影響章 | 改訂履歴 |
|---|---|---|---|---|---|
| ADR-RULE-001 | ZEN Engine 判定 | ZEN Engine を Decision API の判定エンジンに採用。JDM 形式、Rust 実装、ミリ秒判定 | Drools、OpenL Tablets、自作ルールエンジン、Camunda DMN | 20_SW/02, 40_CTRL/05 | v1.0 2026-03-01 |
| ADR-RULE-002 | Temporal ワークフロー | Temporal を長時間ワークフロー（>10 分）に採用。Dapr Workflow と二層運用 | Cadence、Argo Workflows、Zeebe、独自実装 | 40_CTRL/05, 20_SW/02 | v1.0 2026-03-03 |

### CI/CD・リリース・ポリシー ADR（5 件）

継続的配信の中核 OSS と運用ガバナンスを束ねる領域。GitOps（ArgoCD）+ Canary（Argo Rollouts）+ admission ポリシー（Kyverno）の 3 本柱に、Progressive Delivery 必須化（ADR-REL-001）と Kyverno 所有モデル（ADR-POL-001）が加わり、配信とガバナンスが同じ領域として束ねられる。リリース設計とポリシー設計は相互に強く結合するため 1 つの節で扱う。

| ADR 番号 | タイトル | 決定内容要約 | 却下された代替案 | 影響章 | 改訂履歴 |
|---|---|---|---|---|---|
| ADR-CICD-001 | Argo CD GitOps | Argo CD で GitOps 配信。App-of-Apps、SSO 統合、マルチテナント対応 | Flux、Jenkins X、Spinnaker、Tekton 単独 | 70_DEVX/01, 04 | v1.0 2026-03-05 |
| ADR-CICD-002 | Argo Rollouts Canary | Argo Rollouts で Canary デプロイ。5 ステップ（5/25/50/75/100）、自動ロールバック条件 | Flagger、独自 Canary 実装、Blue/Green のみ | 70_DEVX/01, 50_NFR/01 | v1.0 2026-03-07 |
| ADR-CICD-003 | Kyverno ポリシー強制 | Kyverno で K8s マニフェスト検証。Baseline + Restricted ポリシーセット | OPA Gatekeeper、PodSecurityAdmission 単独、自作 Admission Webhook | 70_DEVX/01, 50_NFR/05 | v1.0 2026-03-09 |
| ADR-REL-001 | Progressive Delivery を リリース時点 から全リリース必須化 | Argo Rollouts による Progressive Delivery を リリース時点 から全リリースで必須、例外は SRE 承認制（emergency patch / 内部ツール / バッチのみ）。共通 AnalysisTemplate セット + flagd 定義ファイル cosign 署名強制 | tier1 公開 API のみ PD、任意採用（推奨レベル）、Blue-Green 固定 | 70_DEVX/01, 50_NFR/01, 30_CF/09 | v1.0 2026-04-24 |
| ADR-POL-001 | Kyverno ポリシーを技術側提案・統制側承認の二分所有で運用 | validate ポリシーは Security 承認、mutate/generate は Platform/SRE 主導で Security レビュー。緊急パッチ例外は 72 時間以内事後 ADR 化、Policy Exception は最大 90 日 | Security 単独所有、Platform 単独所有、全員レビュー、三分所有 | 70_DEVX/01, 50_NFR/05, 55_OPS | v1.0 2026-04-24 |

### 観測性 ADR（3 件）

3 シグナル（ログ / メトリクス / トレース）の OSS 構成と、その上に立脚するインシデント分類体系。Grafana LGTM + OpenTelemetry の二本立てを基盤とし、可用性・セキュリティ・供給・コンプライアンスを横断する単一の Incident Taxonomy を上位概念として定義する。

| ADR 番号 | タイトル | 決定内容要約 | 却下された代替案 | 影響章 | 改訂履歴 |
|---|---|---|---|---|---|
| ADR-OBS-001 | Grafana LGTM 採用 | Loki（ログ）/ Grafana（可視化）/ Tempo（トレース）/ Mimir（メトリクス）の LGTM スタック採用 | Elastic Stack、Splunk、Datadog、独自構成 | 30_CF/03, 05, 06, 50_NFR/04 | v1.0 2026-03-11 |
| ADR-OBS-002 | OpenTelemetry 採用 | OTel Collector をサイドカー / DaemonSet 併用。標準計装で言語横断 | Jaeger 単独、Zipkin、Prometheus 生採用、ベンダー特化 SDK | 30_CF/03, 05, 06, 20_SW/02 | v1.0 2026-03-13 |
| ADR-OBS-003 | 可用性・セキュリティインシデントを単一分類体系で統合管理 | Category 軸（availability / integrity / confidentiality / supply-chain / compliance / operational）× Severity 軸（sev1〜sev4）の 6×4 マトリクスで全インシデントを管理、Runbook 網羅性を機械検証、単一エラーバジェット制 | 可用性とセキュリティを別系統、CVSS のみで統合、ITIL 分類のみ | 55_OPS/02, 09, 50_NFR/04 | v1.0 2026-04-24 |

### Feature Flag ADR（1 件）

Feature Management の OSS 選定。OpenFeature 準拠。

| ADR 番号 | タイトル | 決定内容要約 | 却下された代替案 | 影響章 | 改訂履歴 |
|---|---|---|---|---|---|
| ADR-FM-001 | flagd Feature Flag | flagd を OpenFeature 準拠 FM として採用。GitOps でフラグ管理、sidecar または in-process 配信 | LaunchDarkly、Unleash、自作 Feature Flag、Envoy xDS | 30_CF/09, 20_SW/02 | v1.0 2026-03-18 |

### 開発者体験 ADR（3 件）

開発者体験（DX）の思想・計測基盤・ローカル開発のホスト Docker ランタイムを定義する ADR 群。Paved Road（舗装道路）の採用で Golden Path を一本化し、time-to-first-commit や Scaffold 利用率など DX メトリクスを稼働系 SLI とは別経路で管理する。さらに Windows + WSL2 ホストの Docker ランタイムを WSL ネイティブ docker-ce で固定し、Paved Road の正しい道一本化原則をホスト構成層でも貫徹する。Platform/SRE の支援コストを有限化しつつ、EM レイヤーが独立に DX 健全性を判断できる構造を与える。

| ADR 番号 | タイトル | 決定内容要約 | 却下された代替案 | 影響章 | 改訂履歴 |
|---|---|---|---|---|---|
| ADR-DEV-001 | 開発者体験の根幹思想として Paved Road を採用 | Netflix 方式の Paved Road を採用し、Golden Path は `examples/` を一次ソースに一本化。Backstage Software Template 互換の Scaffold CLI で配布、catalog-info.yaml 自動生成。Paved Road 外は自己責任 | 複数テンプレート共存、自由主義、厳格固定、Scaffold CLI 不採用 | 70_DEVX/01, 04, 05 | v1.0 2026-04-24 |
| ADR-DEV-002 | Windows + WSL2 環境の Docker ランタイムに WSL ネイティブ docker-ce を採用 | Windows 11 + WSL2 (Ubuntu 24.04 LTS + systemd) 上に Docker 公式 apt から docker-ce を導入し、Docker Desktop / Rancher Desktop / Podman を採用しない。`.wslconfig` で資源を一元管理、Dev Container / kind / Dapr Local を同 distribution 内 sibling 配置 | Docker Desktop、Rancher Desktop、Podman、Docker Desktop と docker-ce の共存 | 70_DEVX/01, 04（IMP-DEV-ENV-060〜065）| v1.0 2026-04-26 |
| ADR-DX-001 | DX メトリクスを稼働系 SLI と分離して管理 | DORA 4 keys / SPACE / time-to-first-commit / Scaffold 利用率などの DX メトリクスは Backstage Scorecards、稼働 SLI は Grafana LGTM で分離。SPACE 原著に従い個人評価と接続しない | 統合ダッシュボード、DX 計測なし、DX のみ計測、個人評価に直結 | 70_DEVX/05, 50_NFR/04, 55_OPS | v1.0 2026-04-24 |

### 依存管理・サプライチェーン ADR（2 件）

依存更新とビルドサプライチェーンの信頼性を担保する ADR 群。Renovate を依存更新の中枢とし、SLSA v1.1 Build Track で リリース時点に Level 2 / Level 3 へ段階到達する。cosign keyless・CycloneDX SBOM・Kyverno ImageVerify の 3 点で本番 namespace の改ざん耐性を構造的に担保する。

| ADR 番号 | タイトル | 決定内容要約 | 却下された代替案 | 影響章 | 改訂履歴 |
|---|---|---|---|---|---|
| ADR-DEP-001 | 依存更新中枢に Renovate を採用 | Renovate（self-hosted OSS 版、Apache 2.0）を 9 種類の依存グラフ横断中枢に採用。`renovate.json` 単一管理、patch 自動マージ / minor 以上手動、security は即時 PR、AGPL 混入検知を license-check job に統合 | GitHub Dependabot、人手管理、外部 SaaS（Snyk / Mend Cloud） | 30_CF/08, 70_DEVX/01, 50_NFR/05 | v1.0 2026-04-24 |
| ADR-SUP-001 | SLSA v1.1 をリリース時点で Level 2 / 採用後の運用拡大時で Level 3 に段階到達 | SLSA v1.1 Build Track をリリース時点で Level 2（cosign keyless + CycloneDX SBOM + Kyverno ImageVerify） / 採用後の運用拡大時で Level 3（hermetic runner + Rekor 登録）に到達。`RB-SEC-005: Image Hash 逆引き Forensics` を整備 | リリース時点で Level 3 一気到達、Level 1 のみ、SLSA 不採用（独自 provenance） | 30_CF/08, 50_NFR/05, 55_OPS/02 | v1.0 2026-04-24 |

### ストレージ・移行・ポータル ADR（5 件）

インフラ層とライフサイクル層の ADR。単独では影響範囲が限定的だが、各章の前提となる。

| ADR 番号 | タイトル | 決定内容要約 | 却下された代替案 | 影響章 | 改訂履歴 |
|---|---|---|---|---|---|
| ADR-STOR-001 | Longhorn ブロックストレージ | Longhorn 3 レプリカ、RWO/RWX 対応、スナップショット運用 | Rook-Ceph、OpenEBS Mayastor、外部 SAN | 10_SYS/01, 50_NFR/01 | v1.0 2026-03-20 |
| ADR-STOR-002 | MetalLB LoadBalancer | MetalLB BGP モードでオンプレ L4 LB、Istio Gateway 連携 | 外部 LB アプライアンス、kube-vip、PureLB | 10_SYS/01, 03 | v1.0 2026-03-22 |
| ADR-MIG-001 | .NET Framework sidecar 移行 | .NET Framework 資産を sidecar コンテナ化して段階移行 | 全面書き直し、API Gateway 集約のみ、VM として運用 | 60_MIG/01, 02 | v1.0 2026-03-24 |
| ADR-MIG-002 | API Gateway 移行 | 既存 API Gateway → k1s0 Istio Gateway への段階切替、ルーティング共存期間を設ける | ビッグバン切替、Envoy 直採用 | 60_MIG/02, 20_SW/02 | v1.0 2026-03-26 |
| ADR-BS-001 | Backstage 開発者ポータル | Backstage を開発者ポータル基盤として採用。Service Catalog + TechDocs + Software Templates | Port.io、Cortex、自作ポータル、GitHub Wiki のみ | 70_DEVX/04, 06 | v1.0 2026-03-28 |

## アルファベット順索引

ADR 番号をアルファベット順でソートした索引である。相関マトリクスや設計間依存マトリクスから ADR 番号で参照される際の逆引きに使う。

- ADR-0001 Istio Ambient mesh 採用
- ADR-0002 図解記法規約
- ADR-0003 AGPL OSS の扱い
- ADR-BS-001 Backstage 開発者ポータル
- ADR-CICD-001 Argo CD GitOps
- ADR-CICD-002 Argo Rollouts Canary
- ADR-CICD-003 Kyverno ポリシー強制
- ADR-DATA-001 CNPG PostgreSQL 採用
- ADR-DATA-002 Strimzi Kafka 採用
- ADR-DATA-003 MinIO オブジェクトストレージ
- ADR-DATA-004 Valkey キャッシュ採用
- ADR-DEP-001 依存更新中枢に Renovate を採用
- ADR-DEV-001 開発者体験の根幹思想として Paved Road を採用
- ADR-DEV-002 Windows + WSL2 環境の Docker ランタイムに WSL ネイティブ docker-ce を採用
- ADR-DIR-001 Protobuf 契約ディレクトリを `src/contracts/` に昇格
- ADR-DIR-002 infra / deploy / ops の 3 階層分離とルート昇格
- ADR-DIR-003 Git Sparse Checkout cone mode + Partial Clone 標準運用
- ADR-DX-001 DX メトリクスを稼働系 SLI と分離して管理
- ADR-FM-001 flagd Feature Flag
- ADR-MIG-001 .NET Framework sidecar 移行
- ADR-MIG-002 API Gateway 移行
- ADR-OBS-001 Grafana LGTM 採用
- ADR-OBS-002 OpenTelemetry 採用
- ADR-OBS-003 可用性・セキュリティインシデントを単一分類体系で統合管理
- ADR-POL-001 Kyverno ポリシーを技術側提案・統制側承認の二分所有で運用
- ADR-REL-001 Progressive Delivery を リリース時点 から全リリース必須化
- ADR-RULE-001 ZEN Engine 判定
- ADR-RULE-002 Temporal ワークフロー
- ADR-SEC-001 Keycloak 認証採用
- ADR-SEC-002 OpenBao 秘密管理採用
- ADR-SEC-003 SPIFFE SVID 採用
- ADR-STOR-001 Longhorn ブロックストレージ
- ADR-STOR-002 MetalLB LoadBalancer
- ADR-SUP-001 SLSA v1.1 を リリース時点で Level 2 / Level 3 に段階到達
- ADR-TIER1-001 Go+Rust ハイブリッド
- ADR-TIER1-002 Protobuf gRPC 内部通信
- ADR-TIER1-003 tier2/tier3 から内部言語を不可視化

## 時系列索引

ADR 採択順でソートした索引である。プラットフォーム設計の意思決定履歴を時系列で辿る際に使う。

| 採択日 | ADR 番号 | タイトル |
|---|---|---|
| 2026-01-15 | ADR-0001 | Istio Ambient mesh 採用 |
| 2026-02-03 | ADR-0002 | 図解記法規約 |
| 2026-02-05 | ADR-DATA-001 | CNPG PostgreSQL 採用 |
| 2026-02-07 | ADR-DATA-002 | Strimzi Kafka 採用 |
| 2026-02-10 | ADR-DATA-003 | MinIO オブジェクトストレージ |
| 2026-02-12 | ADR-DATA-004 | Valkey キャッシュ採用 |
| 2026-02-18 | ADR-SEC-001 | Keycloak 認証採用 |
| 2026-02-20 | ADR-0003 | AGPL OSS の扱い |
| 2026-02-22 | ADR-SEC-002 | OpenBao 秘密管理採用 |
| 2026-02-25 | ADR-SEC-003 | SPIFFE SVID 採用 |
| 2026-03-01 | ADR-RULE-001 | ZEN Engine 判定 |
| 2026-03-03 | ADR-RULE-002 | Temporal ワークフロー |
| 2026-03-05 | ADR-CICD-001 | Argo CD GitOps |
| 2026-03-07 | ADR-CICD-002 | Argo Rollouts Canary |
| 2026-03-09 | ADR-CICD-003 | Kyverno ポリシー強制 |
| 2026-03-10 | ADR-TIER1-001 | Go+Rust ハイブリッド |
| 2026-03-11 | ADR-OBS-001 | Grafana LGTM 採用 |
| 2026-03-13 | ADR-OBS-002 | OpenTelemetry 採用 |
| 2026-03-15 | ADR-TIER1-002 | Protobuf gRPC 内部通信 |
| 2026-03-18 | ADR-FM-001 | flagd Feature Flag |
| 2026-03-20 | ADR-STOR-001 | Longhorn ブロックストレージ |
| 2026-03-22 | ADR-STOR-002 | MetalLB LoadBalancer |
| 2026-03-24 | ADR-MIG-001 | .NET Framework sidecar 移行 |
| 2026-03-26 | ADR-MIG-002 | API Gateway 移行 |
| 2026-03-28 | ADR-BS-001 | Backstage 開発者ポータル |
| 2026-04-19 | ADR-TIER1-003 | tier2/tier3 から内部言語を不可視化 |
| 2026-04-23 | ADR-DIR-001 | Protobuf 契約ディレクトリを `src/contracts/` に昇格 |
| 2026-04-23 | ADR-DIR-002 | infra / deploy / ops の 3 階層分離とルート昇格 |
| 2026-04-23 | ADR-DIR-003 | Git Sparse Checkout cone mode + Partial Clone 標準運用 |
| 2026-04-24 | ADR-DEP-001 | 依存更新中枢に Renovate を採用 |
| 2026-04-24 | ADR-DEV-001 | 開発者体験の根幹思想として Paved Road を採用 |
| 2026-04-24 | ADR-DX-001 | DX メトリクスを稼働系 SLI と分離して管理 |
| 2026-04-24 | ADR-OBS-003 | 可用性・セキュリティインシデントを単一分類体系で統合管理 |
| 2026-04-24 | ADR-POL-001 | Kyverno ポリシーを技術側提案・統制側承認の二分所有で運用 |
| 2026-04-24 | ADR-REL-001 | Progressive Delivery を リリース時点 から全リリース必須化 |
| 2026-04-24 | ADR-SUP-001 | SLSA v1.1 を リリース時点で Level 2 / Level 3 に段階到達 |
| 2026-04-26 | ADR-DEV-002 | Windows + WSL2 環境の Docker ランタイムに WSL ネイティブ docker-ce を採用 |

## 改訂検出と同期

本索引は構想設計書 ADR ファイルの改訂を検出して同期する必要がある。同期は以下 3 ルールで実施する。

- **CI 自動検出**: 構想設計書 ADR ファイルの変更を検出する CI ジョブを [../../02_構想設計/adr/](../../02_構想設計/adr/) に設置。ADR のヘッダメタデータ（タイトル・決定・改訂日）が変更された場合、本索引の該当行を更新する PR を自動生成する。
- **自動 PR 内容**: CI が生成する自動 PR は本索引の該当行の 3 列（決定内容要約・改訂履歴・影響章）を更新するのみ。影響章の再判定は人間レビュアーが実施する。
- **手動追記**: CI 自動検出で捕捉されない追加（却下代替案の追記、影響章の追加など）は、ADR 改訂 PR と同じ PR で手動更新する。

## 新規 ADR 起票時のリンク先

新規 ADR を起票する場合、構想設計書 [../../02_構想設計/adr/](../../02_構想設計/adr/) に ADR ファイルを追加した後、本索引の該当領域別索引に新規行を追加する。新規 ADR が既存の 11 領域に該当しない場合、「その他」カテゴリに仮登録し、5 件以上集まった段階で新領域グループとして分離する。

ADR 起票から本索引更新までの手順は以下 5 ステップである。

- **ステップ 1**: 構想設計側で ADR ドラフトを作成（`00-draft` ステータス）。
- **ステップ 2**: Product Council でレビュー、採択 or 却下を決定。
- **ステップ 3**: 採択された ADR は構想設計側で `10-accepted` に昇格。
- **ステップ 4**: 本索引に領域別・アルファベット順・時系列の 3 節に新規行を追加。
- **ステップ 5**: [../80_トレーサビリティ/05_構想設計ADR相関マトリクス.md](../80_トレーサビリティ/05_構想設計ADR相関マトリクス.md) に影響章を記録。

却下された ADR は本索引には登録しない。却下判断そのものは構想設計側の却下 ADR ディレクトリで管理する。

## 運用ルール

本索引の継続運用は以下の 4 ルールで固定化する。

- **採択ルール**: 新規 ADR 採択時、本索引・相関マトリクス・設計間依存マトリクスの 3 ファイルを同時更新する。3 ファイル間の整合は CI で週次検証。
- **改訂ルール**: ADR 改訂時、本索引の改訂履歴列と相関マトリクスの反映内容列を同時更新する。
- **却下ルール**: 過去の ADR が無効化された場合、本索引から削除せず、「状態」列を `却下` に追記する。概要設計側の記述も同時に書き換える。
- **四半期レビュー**: Product Council 四半期レビューで ADR 本体と本索引の整合を確認する。差分が 3 件を超える場合、次四半期で解消する計画を策定する。

## 未起票 ADR 一覧（リリース時点 前起票予定）

概要設計書を執筆する過程で、既存 36 ADR では決定理由を十分に固定化できない論点が 14 件浮上した。これらは構想設計段階で議論が行われた選定結果を概要設計で参照してしまっているか、概要設計で新たに確定させた方針がそのまま ADR 化に値するため、正式な ADR 起票前の「仮番」として本節に登録する。リリース時点 開発着手前までに構想設計側で本体を起票し、本索引の領域別索引に移し替える運用とする。

仮番の目的は 2 点ある。第一に、概要設計書内で `ADR-AUDIT-001` のように参照している ADR 番号を索引側でトレース可能にし、「参照先 ADR が存在しない」という逆引き不整合を解消する。第二に、リリース時点 前の ADR 起票計画（誰がいつ起票するか）を Product Council のレビュー対象として明示化し、抜け漏れを防ぐ。

仮番付きの ADR は以下の 3 ルールで運用する。

- **起票期限**: リリース時点（採用初期 開発着手）前までに構想設計側で正式 ADR を作成する。期限超過は Product Council 月次レビューでリスクとして報告する。
- **番号確定**: 仮番（例 `ADR-AUDIT-001`）と正式 ADR 番号は可能な限り一致させる。一致しない場合、概要設計側の参照を CI で一括書き換えする。
- **プレースホルダ（NNN 表記）**: `ADR-LEGAL-NNN` / `ADR-PUBSUB-NNN` / `ADR-SLO-NNN` の 3 件は本体が決定されていない論点であり、起票時に連番を確定させる。

| 仮番 | 仮タイトル | 起票予定カテゴリ | 参照元主要ファイル | 確定予定論点 |
|---|---|---|---|---|
| ADR-AUDIT-001 | WORM 監査証跡方針 | セキュリティ | 20_SW/02 EIF/10_Audit_Pii_API方式 | 監査ログの Write-Once-Read-Many 強制と tamper-evident（ハッシュチェーン）採用根拠 |
| ADR-CB-001 | Circuit Breaker 採用方針 | 基本構造 | 55_OPS/02_インシデント対応方式 | Dapr Resiliency Circuit Breaker の閾値設計（open 条件 / half-open 遷移 / state 監視） |
| ADR-DEVX-001 | Tilt + kind ローカル環境 | 開発者体験 | 70_DEVX/02_ローカル開発環境方式 | 開発者 1 台 PC での 5 分以内フルスタック起動ルールの技術選定 |
| ADR-FEAT-001 | OpenFeature + flagd 契約 | Feature Flag | 20_SW/02 EIF/11_Feature_API方式 | OpenFeature SDK と flagd バックエンドの契約固定（ADR-FM-001 の API 契約拡張） |
| ADR-LEGAL-NNN | 法務制約マッピング | その他 | 50_NFR/05_セキュリティ方式設計 | セキュリティ多層防御と個人情報保護法・AGPL 要件・金商法マッピング |
| ADR-MSG-001 | Dapr Building Block 分離 | 基本構造 | 20_SW/01/02_Daprファサード層コンポーネント | State / PubSub / Binding / Feature の責務重複排除と選択判断基準 |
| ADR-NFR-001 | k6 / Locust 負荷試験選定 | CI/CD | 55_OPS/07_負荷試験方式 | 負荷試験ツール二刀流の使い分け（Baseline/Spike/Soak/Stress マッピング） |
| ADR-OBS-004 | Telemetry API 4 メソッド設計 | 観測性 | 20_SW/02 EIF/08_Telemetry_API方式 | Traces / Metrics / Profiles / Stream 4 メソッドの API 契約（仮番 ADR-OBS-003 は 2026-04-24 に「可用性・セキュリティインシデント統合分類」で Accepted 済のため ADR-OBS-004 にスライド） |
| ADR-PERF-001 | p99 500ms 層別積算モデル | その他 | 50_NFR/02_性能と拡張性方式 | 業務 200 + Dapr 80 + OTel 20 + 監査 50 + NW/DB 150 の内訳固定化 |
| ADR-PUBSUB-NNN | CloudEvents v1.0 準拠契約 | データ層 | 20_SW/02 EIF/03_PubSub_API方式 | CloudEvents 準拠・トピック命名強制・DLQ・Ordering 保証の契約 |
| ADR-SLO-NNN | 11 API SLO とバジェット運用 | 観測性 | 50_NFR/11_SLI_SLO_エラーバジェット方式 | API 単位 SLO 目標値・Burn Rate Alert 閾値・エラーバジェット配分 |
| ADR-TEST-001 | Test Pyramid 戦略 | CI/CD | 70_DEVX/05_テスト戦略方式 | UT 70% / 結合 20% / E2E 10% 比率と testcontainers 採用方針 |
| ADR-WF-001 | Dapr Workflow / Temporal 二重化 | ルールエンジン | 20_SW/02 EIF/06_Workflow_API方式 | 1 時間境界での短期 / 長期ワークフロー振り分け契約（ADR-RULE-002 の運用ルール詳細化） |
| ADR-ZEN-002 | ZEN Engine マルチ NUMA 構成 | ルールエンジン | 20_SW/02 EIF/09_Decision_API方式 | 採用後の運用拡大時 高負荷時の per-NUMA pod シャーディング戦略 |

未起票 ADR の合計は 14 件である。うち 3 件（ADR-LEGAL-NNN / ADR-PUBSUB-NNN / ADR-SLO-NNN）はプレースホルダで、残り 11 件は仮番確定済である。なお、当初「ADR-TIER1-003」として OpenBao（MPL-2.0）採用根拠を仮登録していたが、構想設計側で同番号が「tier2/tier3 からの内部言語不可視化」として 2026-04-19 に正式採択（Accepted）されたため、未起票一覧から除去して領域別索引（基本構造 ADR）へ移管した。OpenBao ライセンス根拠は ADR-SEC-002 の実装詳細補強として吸収する運用とする。加えて当初「ADR-OBS-003」として Telemetry API 4 メソッド設計を仮登録していたが、構想設計側で同番号が 2026-04-24 に「可用性・セキュリティインシデントを単一分類体系で統合管理」として正式採択（Accepted）されたため、Telemetry API 契約の仮番を ADR-OBS-004 にスライドした。概要設計書内で `ADR-OBS-003` を Telemetry API 4 メソッド設計として参照している箇所は リリース時点 の起票時点で `ADR-OBS-004` に一括書換する。

## 下流参照

ADR の詳細本体は [../../02_構想設計/adr/](../../02_構想設計/adr/) を参照。概要設計章との相関は [../80_トレーサビリティ/05_構想設計ADR相関マトリクス.md](../80_トレーサビリティ/05_構想設計ADR相関マトリクス.md) を参照。

## 改訂履歴

| 日付 | 版 | 改訂内容 | 起票者 |
|---|---|---|---|
| 2026-04-20 | 0.1 | 採用検討向け初版。26 ADR の領域別 / アルファベット順 / 時系列の 3 ビュー索引を整備。 | 概要設計チーム |
| 2026-04-21 | 0.2 | 未起票 ADR 一覧（リリース時点 前起票予定）15 件を追加。概要設計書内からの ADR 参照で索引に存在しなかった論点を仮番として集約し、起票期限・運用ルールを明文化。また ADR-ZEN-001 を ADR-RULE-001 に統合（ZEN Engine 採用の重複参照解消）。 | 概要設計チーム |
| 2026-04-21 | 0.3 | ADR-TIER1-003（tier2/tier3 から内部言語を不可視化、2026-04-19 Accepted）を基本構造 ADR（5 → 6 件）・アルファベット順索引・時系列索引に正式登録。同じ仮番で未起票一覧に置かれていた OpenBao ライセンス根拠エントリを除去し、ADR-SEC-002 の実装詳細補強側へ吸収する運用に変更。総 ADR 件数を 26 → 27 に更新。 | 概要設計チーム |
| 2026-04-24 | 0.4 | 欠落 10 ADR (DEP-001/DEV-001/DIR-001〜003/DX-001/OBS-003/POL-001/REL-001/SUP-001) を索引化。総件数 27→36。 | 概要設計チーム |
