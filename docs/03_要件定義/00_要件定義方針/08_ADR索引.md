# 08. ADR 索引

本書は k1s0 プロジェクトの Architecture Decision Records（ADR）を一元索引する。ADR の本体は構想設計段階で発番・記述するが、要件定義時点で既に確定している技術判断を ADR 化することで、要件と判断理由の双方向トレーサビリティを担保する。

## 本書の位置付け

要件定義は「何を作るか」、ADR は「なぜその技術で作るか」を記述する。両者を分離することで、要件は変わっても技術判断が有効であり続ける場合や、逆に技術判断が更新されても要件は維持される場合を明確にできる。ADR 索引は要件定義から技術判断へのリンク集として機能する。

## ADR 記述ルール

k1s0 の ADR は以下の構造で記述する。

- **Title**: `ADR-<NUMBER>: <短い判断名>`
- **Status**: Proposed / Accepted / Deprecated / Superseded
- **Context**: なぜこの判断が必要になったか、関連する要件・制約
- **Decision**: 選択した方針
- **Consequences**: この判断による正負の影響
- **Alternatives**: 検討した他の選択肢と却下理由
- **Related Requirements**: 要件 ID のリスト

ADR ファイルは `docs/02_構想設計/` 配下の適切なサブフォルダに配置する。本索引はリンク集であり、ADR 本体のリポジトリではない。

### ADR 番号体系

k1s0 の ADR 番号は以下の 2 体系を併用する。混用はあいまいにするためではなく、判断のスコープを番号自体で区別するための明示的な設計である。

**横断 ADR（`ADR-NNNN` 4 桁フラット通番）**: 複数領域に同時に影響する基盤級の判断に付与する。例: `ADR-0001`（Istio Ambient Mesh、全通信レイヤに影響）、`ADR-0002`（4 レイヤ図解規約、全ドキュメントに影響）、`ADR-0003`（AGPL 分離、全 OSS 選定に影響）。新規発番は Product Council 承認必須、採番は 0004 以降を連番で消費する。

**領域別 ADR（`ADR-<DOMAIN>-NNN` カテゴリネームスペース + 3 桁通番）**: 単一技術領域内で閉じる判断に付与する。`<DOMAIN>` は以下のいずれか。

- `TIER1`: tier1 設計固有（例: `ADR-TIER1-001` Go/Rust ハイブリッド方針）
- `DATA`: データストア選定（`ADR-DATA-001` PostgreSQL、`ADR-DATA-002` Kafka、`ADR-DATA-003` MinIO、`ADR-DATA-004` Valkey）
- `SEC`: セキュリティ基盤（`ADR-SEC-001` Keycloak、`ADR-SEC-002` OpenBao、`ADR-SEC-003` SPIFFE/SPIRE）
- `MIG`: 移行方式（`ADR-MIG-001` サイドカー、`ADR-MIG-002` API Gateway 共存）
- `RULE`: ルール・ワークフローエンジン（`ADR-RULE-001` ZEN Engine、`ADR-RULE-002` Temporal）
- `CICD`: CI/CD・配信（`ADR-CICD-001` Argo CD、`ADR-CICD-002` Argo Rollouts、`ADR-CICD-003` Kyverno）
- `OBS`: 可観測性（`ADR-OBS-001`〜`ADR-OBS-003`）
- `FM`: Feature Management（`ADR-FM-001` flagd / OpenFeature）
- `BS`: Backstage 周辺（`BS-001`）
- `STOR`: ストレージ・ネットワーク（`ADR-STOR-001` Longhorn、`ADR-STOR-002` MetalLB）
- `TEST`: テスト戦略（`ADR-TEST-001` CI 留保 + qualify portable 設計、後続 002〜009 で開発環境 / テストピラミッド / 二層 E2E / 環境マトリクス / chaos+scale+soak / upgrade+DR / コンプライアンス / 観測性 E2E を補強）

**どちらを選ぶかの判定**: 判断の影響が「2 領域以上の DOMAIN を跨ぐ」または「全ドキュメント・全 OSS に波及する」場合は横断 ADR を選択する。それ以外は領域別 ADR とする。迷った場合は領域別を既定とし、Product Council レビュー時に昇格可否を議論する。

**採番責任**: 横断 ADR は起案者が Product Council に諮り、承認後に 08_ADR索引.md で番号を予約する。領域別 ADR は各領域リード（tier1 設計リード、データ基盤リード、セキュリティリード等）が直接採番し、本索引への登録 PR を合わせて出す。

## ADR 一覧

以下は要件定義時点で既に確定している、または リリース時点 で確定すべき主要 ADR のリストである。

### アーキテクチャ基盤

- **ADR-0001: Istio Ambient Mesh 採用**
  - 判断: サービスメッシュに Istio Ambient（sidecar なし）を採用する
  - 理由: sidecar 版の運用負担と比べ、ztunnel + waypoint 構成の簡潔性、メモリ効率が 採用側組織の規模に適合
  - 関連要件: NFR-B-PERF-001（p99 レイテンシ）、NFR-C-NOP-001（運用負荷）

- **ADR-0002: 4 レイヤ図解規約**
  - 判断: drawio 図は 4 レイヤ（アプリ / ネットワーク / インフラ / データ）で色分け
  - 理由: レイヤ間の責務境界を視覚的に明確化、認識齟齬を回避
  - 関連要件: ドキュメント規約（CLAUDE.md）

- **ADR-0003: AGPL 分離アーキテクチャ**
  - 判断: AGPL OSS（Grafana 等）はプロセス分離してリンクを回避
  - 理由: AGPL 義務の自社コード波及を防ぐ、法務リスク低減
  - 関連要件: BC-LIC-004（AGPL 分離の技術判定）、BC-LIC-005（禁止ライセンスリスト）、NFR-E-NW-003（外部境界遮断）

### tier1 設計

- **ADR-TIER1-001: Go + Rust ハイブリッド方針**
  - 判断: Dapr ファサード層は Go（stable SDK）、ZEN Engine/crypto/雛形 CLI/採用側組織の固有は Rust
  - 理由: Go Dapr SDK の成熟度と、Rust 領域での性能・型安全性の両取り
  - 関連要件: FR-T1-INVOKE-001〜005、FR-T1-STATE-001〜005、FR-T1-PUBSUB-001〜005、FR-T1-SECRETS-001〜004、FR-T1-BINDING-001〜004、FR-T1-WORKFLOW-001〜005、FR-T1-LOG-001〜004、FR-T1-TELEMETRY-001〜004、FR-T1-DECISION-001〜004、FR-T1-AUDIT-001〜003、FR-T1-PII-001〜002、FR-T1-FEATURE-001〜004

- **ADR-TIER1-002: 内部通信は gRPC/Protobuf 必須**
  - 判断: tier1 内部サービス間は Protobuf gRPC、型安全と性能の両立
  - 理由: スキーマ駆動で破壊的変更を CI 検出、バイナリプロトコルで低レイテンシ
  - 関連要件: NFR-B-PERF-001、OR-EOL-001（SemVer 互換）

- **ADR-TIER1-003: tier2/tier3 からは言語不可視**
  - 判断: クライアント SDK と gRPC 公開 API のみ公開、内部 Go/Rust は不可視
  - 理由: tier2/tier3 実装言語を自由化、内部再実装を透過可能
  - 関連要件: FR-T1-INVOKE-001〜005、OR-EOL-001

### データとストレージ

- **ADR-DATA-001: PostgreSQL 採用（CloudNativePG）**
  - 判断: 業務データの RDB は PostgreSQL、運用は CloudNativePG Operator
  - 理由: AGPL 相当の強い copyleft を持たず、Kubernetes Operator が成熟
  - 関連要件: NFR-A-DR-002、FR-T1-STATE-001〜005

- **ADR-DATA-002: Kafka 採用（Strimzi）**
  - 判断: イベントバスは Kafka、運用は Strimzi Operator
  - 理由: 業界デファクト、Strimzi で Kubernetes ネイティブ運用、Dapr PubSub 対応
  - 関連要件: FR-T1-PUBSUB-001〜005、NFR-B-WL-001〜002

- **ADR-DATA-003: MinIO 採用**
  - 判断: オブジェクトストレージは MinIO、S3 互換 API
  - 理由: OSS で S3 互換、オンプレ運用確立、Longhorn との相性
  - 関連要件: FR-T1-BINDING-001〜004

- **ADR-DATA-004: Valkey 採用（Redis 代替）**
  - 判断: インメモリ KVS は Valkey（Redis の BSD フォーク後継）
  - 理由: Redis が RSAL ライセンスに変更されたため、真の OSS である Valkey を採用
  - 関連要件: FR-T1-STATE-001〜005、NFR-B-PERF-002〜003

### セキュリティと認証

- **ADR-SEC-001: Keycloak 採用**
  - 判断: ID プロバイダは Keycloak、OIDC 準拠
  - 理由: 成熟 OSS、Federation 機能で既存 AD/LDAP 統合容易、Apache License 2.0
  - 関連要件: NFR-E-AC-001〜005、FR-EXT-IDP-001〜002

- **ADR-SEC-002: OpenBao 採用（HashiCorp Vault 代替）**
  - 判断: Secret 管理は OpenBao（HashiCorp Vault の MPL 2.0 時代のフォーク）
  - 理由: Vault の BUSL ライセンス変更を回避、コミュニティ主導の OpenBao で OSS 継続性を確保
  - 関連要件: NFR-E-ENC-001〜003、FR-T1-SECRETS-001〜004

- **ADR-SEC-003: SPIFFE/SPIRE + Istio ワークロード ID**
  - 判断: ワークロード ID は SPIFFE 標準、Istio Ambient に統合
  - 理由: 業界標準、マルチクラウド移行時の互換性確保
  - 関連要件: NFR-E-AC-001〜005、NFR-F-STD-001

### CI/CD と配信

- **ADR-CICD-001: Argo CD 採用**
  - 判断: GitOps ツールは Argo CD
  - 理由: CNCF Graduated、Kubernetes ネイティブ、成熟度と機能性で Flux を上回る実績
  - 関連要件: OR-ENV-002、NFR-C-MNT-001〜003

- **ADR-CICD-002: Argo Rollouts 採用**
  - 判断: Canary / Blue-Green デプロイは Argo Rollouts
  - 理由: Argo CD との統合、progressive delivery の実績
  - 関連要件: NFR-D-MTH-002、DX-FM-004

- **ADR-CICD-003: Kyverno ポリシーエンジン**
  - 判断: Kubernetes Admission Controller は Kyverno
  - 理由: 宣言的ポリシー（YAML）で Rego 言語学習不要、OPA 比で運用負荷低減
  - 関連要件: NFR-E-AC-002、NFR-E-AC-004、OR-ENV-006

- **ADR-POL-002: ローカル kind cluster の構成 SoT を tools/local-stack/up.sh に統一**
  - 判断: `tools/local-stack/up.sh` を local kind cluster の唯一の構成 SoT とし、helm release は `apply_*` 関数経由 / argocd Application 経由でのみ許可。手動 `helm install` / `kubectl apply` は ephemeral namespace（`tmp-*` / `dev-*`）に限定する。
  - 理由: drift の再構築コスト（1.5〜2.5h × 再発回数）が 10 年保守期間で累積するのを防ぐ。Kyverno `block-non-canonical-helm-releases` (runtime) + CI drift-check workflow (PR) + `up.sh --mode {dev,strict}` (運用境界) の三層防御で再発を構造的に阻止する。
  - 関連要件: NFR-D-MTH-002、OR-ENV-002、NFR-E-AC-002、IMP-DEV-POL-006

### ワークフローとルール

- **ADR-RULE-001: ZEN Engine + JDM 採用**
  - 判断: ルールエンジンは ZEN Engine、判断モデルは JDM（JSON Decision Model）
  - 理由: Rust 実装で性能良好、JDM エディタの UX、DMN 標準より軽量
  - 関連要件: FR-T1-DECISION-001〜004

- **ADR-RULE-002: Temporal 採用**
  - 判断: 複雑な業務ワークフローは Temporal、軽量は Dapr Workflow
  - 理由: 業界トップクラスの Durable Execution、エンタープライズ実績
  - 関連要件: FR-T1-WORKFLOW-001〜005

### 可観測性

- **ADR-OBS-001: OpenTelemetry 標準準拠**
  - 判断: 全言語の計装は OpenTelemetry SDK
  - 理由: CNCF 標準、ベンダーロックイン回避、Trace/Metric/Log の統一
  - 関連要件: FR-T1-TELEMETRY-001〜004、NFR-F-STD-001

- **ADR-OBS-002: Grafana LGTM スタック**
  - 判断: Loki（ログ）、Grafana（可視化）、Tempo（トレース）、Mimir（メトリクス）
  - 理由: 統合された OSS 観測基盤、低コスト、Kubernetes ネイティブ
  - 関連要件: NFR-C-NOP-001〜003、FR-T1-LOG-001〜004

- **ADR-OBS-003: Prometheus + Mimir**
  - 判断: メトリクス収集は Prometheus、長期保存は Mimir
  - 理由: デファクト標準、PromQL エコシステム、スケーラブル長期保存
  - 関連要件: NFR-B-PERF-001〜007、NFR-C-MNT-001〜003

### Feature Management

- **ADR-FM-001: flagd 採用（OpenFeature 準拠）**
  - 判断: Feature Flag バックエンドは flagd
  - 理由: OpenFeature 標準（CNCF）、ベンダーロックイン回避
  - 関連要件: FR-T1-FEATURE-001〜004、DX-FM-001〜007

### Backstage

- **ADR-BS-001: Backstage 採用**
  - 判断: Developer Portal は Backstage
  - 理由: CNCF Incubating、エコシステム豊富、Software Template と TechDocs 統合
  - 関連要件: DX-GP-001〜006、BC-ONB-002

### ストレージ基盤

- **ADR-STOR-001: Longhorn 採用**
  - 判断: Kubernetes CSI ストレージは Longhorn
  - 理由: オンプレ運用で実績、レプリケーション・スナップショット機能
  - 関連要件: NFR-A-CONT-001、NFR-F-CHR-002

- **ADR-STOR-002: MetalLB 採用**
  - 判断: オンプレ LoadBalancer は MetalLB
  - 理由: L2 モード（リリース時点）、BGP モード（採用側のマルチクラスタ移行時+）、オンプレ標準
  - 関連要件: NFR-F-CHR-003

### 移行と共存

- **ADR-MIG-001: .NET Framework サイドカー方式**
  - 判断: 既存 .NET Framework 資産統合の第一選択はサイドカー
  - 理由: 既存アプリに最小変更で統合、HTTP/1.1 で k1s0 API 呼出
  - 関連要件: NFR-D-MTH-001、FR-EXT-DOTNET-001

- **ADR-MIG-002: .NET Framework API Gateway 方式**
  - 判断: VM 直接稼働の既存アプリは Envoy Gateway 経由
  - 理由: Pod 化できない資産も k1s0 統合可能
  - 関連要件: NFR-D-MTH-001、FR-EXT-DOTNET-002

### Kubernetes 基盤・ネットワーク

- **ADR-INFRA-001: Kubernetes クラスタを kubeadm + Cluster API で構築**
  - 判断: production K8s ブートストラップは Cluster API + kubeadm（KubeadmControlPlane）を標準とし、kubeadm 直接実行を小規模オンプレ向けに併存維持
  - 理由: vanilla K8s 互換維持・宣言的 cluster lifecycle・環境別 overlay の 3 軸で 採用組織の標準スキルが流用可能
  - 関連要件: NFR-F-SYS-001（オンプレ完結）、NFR-A-CONT-001（HA 3 control-plane）、IMP-DEV-POL-006

- **ADR-CNCF-001: vanilla Kubernetes（CNCF Conformance 互換）を維持**
  - 判断: 独自 admission / API server 改造 / upstream-incompatible distribution は採用せず、CNCF Conformance テスト pass 状態を維持
  - 理由: 採用組織の K8s 標準スキル流用性・周辺エコシステム互換性・upstream 進化への追従権を確保
  - 関連要件: NFR-F-SYS-001、NFR-F-STD-001（業界標準）

- **ADR-NET-001: production CNI に Cilium、kind 検証用に Calico を使い分け**
  - 判断: production = Cilium（eBPF dataplane）、kind multi-node = Calico、kind single-node = kindnet
  - 理由: production の eBPF 性能と Hubble 観測性、kind での NetworkPolicy 実機検証性（H3a 実証）の両取り
  - 関連要件: NFR-E-AC-003（tenant 越境防止）、NFR-B-PERF-001（性能）

### 分散ランタイム・スケール

- **ADR-DAPR-001: 分散ランタイムに Dapr Operator を採用**
  - 判断: tier1 の分散ランタイムは Dapr 1.17 LTS、Operator HA 3 replica + mTLS 必須
  - 理由: state / pubsub / secrets / binding / workflow / configuration の 6 building block を単一機構で網羅、4 言語 SDK 公式、CNCF Graduated
  - 関連要件: FR-T1-* 全 12 API、NFR-E-ENC-001〜003（mTLS）、ADR-TIER1-001/002/003 と整合

- **ADR-SCALE-001: Event-driven autoscaling に KEDA を採用**
  - 判断: tier2 業務 event 駆動と tier1 RPS ベース autoscale を KEDA で統一管理、HPA は KEDA から自動生成
  - 理由: 60+ scaler で外部メトリクス autoscale を網羅、CNCF Graduated、scale-from-zero 対応
  - 関連要件: NFR-B-WL-001〜002（バースト負荷）、NFR-B-PERF-002（スループット）

### tier3 アーキテクチャ

- **ADR-TIER3-001: tier3 client ごとに専用 BFF を配置**
  - 判断: Web SPA は Go BFF（GraphQL + REST）、Native は SDK + tier1 直接、Legacy wrap は .NET サイドカー BFF
  - 理由: client 多様性への最適化、認証経路分離、cross-tenant boundary の per-request 強制点を BFF 段に置ける
  - 関連要件: NFR-E-AC-003（tenant 越境防止）、FR-T3-* 系（tier3 機能要件）

- **ADR-TIER3-002: tier3 Web を React + Vite SPA + Go BFF で構成**
  - 判断: React 18+ + TypeScript + Vite + pnpm workspace、SSR 不採用（docs-site のみ Astro pre-rendering を将来検討）
  - 理由: 採用組織のフロントエンド人材プール最大、ADR-TIER3-001 の責務分離と整合、Vite 開発体験
  - 関連要件: DX-GP-001〜006、ADR-TIER3-001 と整合

- **ADR-TIER3-003: tier3 Native アプリに .NET MAUI を採用**
  - 判断: .NET MAUI 8 LTS、Android / iOS / macOS Catalyst / Windows の 4 OS 対応、tier2 .NET SDK を ProjectReference 共有
  - 理由: tier2 .NET 生態系との完全統合、採用組織の C# 人材プール活用、Microsoft 公式 + .NET LTS 3 年サイクル
  - 関連要件: FR-T3-NATIVE-*、ADR-TIER1-001 と整合

### 運用ライフサイクル

- **ADR-OPS-001: Runbook を 8 セクション + YAML frontmatter + Chaos Drill で標準化**
  - 判断: Markdown + Backstage TechDocs + 8 セクション固定（前提 / 対象事象 / 初動 5 分 / 原因特定 / 復旧 / 検証 / 予防 / 関連）+ YAML frontmatter（runbook_id 等 10 項目）+ 1 Runbook 1 事象 / 1 ステップ 3 分以内 / 1 Runbook 全体 1 時間以内 + 四半期 Chaos Drill 検証 + Alertmanager `runbook_url` ラベル必須 + 品質指標 4 種を Grafana 月次計測
  - 理由: 起案者不在の夜間休日 SEV1 で協力者が単独対応するためのバス係数 2 を構造的に担保。Confluence/Notion（オンプレ完結要件 NFR-F-SYS-001 違反）、自由形式（属人化）、ITIL（小規模運用 NFR-C-NOP-001 と矛盾）を退ける
  - 関連要件: NFR-A-REC-002（Runbook 15 本整備）、NFR-A-CONT-001（RTO 4 時間）、NFR-C-NOP-001（小規模運用）、NFR-C-OPS-001（運用プロセス基盤）

### テスト戦略

テスト戦略系列（`ADR-TEST-*`）はリリース時点で CI を持たない個人 OSS という特殊解にもかかわらず、採用検討組織が「10 年保守する」前提で testing maturity を評価できる構造を組み立てるための ADR 群である。一次経路はローカルマシン上の `make qualify-release`、二次経路は CI への将来移行（Phase 1 以降の客観条件で起動）という非対称な配置を取り、CI 障害が品質崩壊に直結しない頑健さと、Phase 移行時の低コスト性を両立する。

- **ADR-TEST-001: リリース時点では CI を導入せず、qualify 基盤を CI portable に設計**
  - 判断: 採用検討者の信頼は release artifact（qualify report + Sonobuoy + SLSA + SBOM）で得る。qualify 基盤は POSIX shell + GNU Make + env var 抽象化 + JSON+Markdown 双方出力 + artifact upload 抽象化 + devcontainer 固定 + OpenBao secret + matrix.yaml 外出しの 7 制約を満たし、Phase 1 以降の CI 移行を YAML 1 本の追加で済ませる。Phase 0–4 の移行条件を客観条件（contributor 数 / sponsor 月予算）で明示
  - 理由: コスト 0 円・bus 係数 2・将来移行コスト最小・先送り言い訳化防止の四立を唯一実現する選択。CI 完全不採用（A）は採用検討者信頼喪失、リリース時点 GitHub Actions（B）は二重実装と runner 差異で個人 OSS の運用工数破綻、self-hosted runner（D）は public repo セキュリティと bus 係数 2 と矛盾
  - 関連要件: NFR-F-SYS-001（オンプレ完結）、NFR-C-NOP-001（小規模運用）、NFR-A-CONT-001（HA / RTO 4 時間 = 協力者単独実行可能性）

- **ADR-TEST-002: 開発環境を devcontainer で固定し、ハードウェア最低要件を ADR で正典化**
  - 判断: SoT は `.devcontainer/devcontainer.json` + Dockerfile（Dev Containers Spec 準拠）。Ubuntu 24.04 LTS / mise + `.tool-versions` で toolchain 4 種固定 / 依存バイナリ 10+ 種を SHA256 検証付きインストール / image tag は git-sha 一意 / arm64 + amd64 マルチアーキ。HW 最低要件は起案者・協力者向けに 8 コア+32GB+NVMe 1TB（リファレンス機）、採用検討者試走向けに 4 コア+16GB+SSD 100GB を別表分離
  - 理由: release artifact 中心の品質公開（ADR-TEST-001）を成立させるには「採用検討者が手元で同じ環境を 5 分で立ち上げて再走できる」が必須条件。Nix flake（A）は学習コストと採用組織保守スキル流用性で 10 年保守に重荷、asdf+compose（C）は SoT 二重化で再現性が崩れ、各自任せ（D）は qualify 再現性を原理的に破壊する
  - 関連要件: NFR-F-SYS-001（オンプレ完結）、NFR-C-NOP-001（小規模運用）、NFR-A-REC-002（再現可能なテスト基盤）

- **ADR-TEST-003: テストピラミッドを L0–L10 の 11 層に階層化**
  - 判断: L0 contract / L1 unit / L2 integration / L3 smoke / L4 standard / L5 conformance / L6 portability / L7 chaos / L8 scale-soak / L9 upgrade / L10 DR の 11 層を `tests/e2e/<layer>/` のディレクトリ単位で物理分離。各層に gate（pre-commit / pre-push / make qualify / make nightly / make qualify-release / make qualify-soak / make qualify-portability-once）を 1:1 対応、所要時間 / 必要環境 / release blocking 可否を表で正典化
  - 理由: 4 層モデル（A）では conformance / chaos / DR / upgrade / portability が「e2e」に潰れて gate 配置が成立せず採用検討者が testing maturity を評価不能。7 層 Google モデル（B）は portability / DR / upgrade を未定義。階層化なし（D）は書きやすい層に偏り「未来への先送りは許さない」と全面衝突。11 層は CNCF Graduated 級 OSS（Kubernetes / Istio / Cilium / ArgoCD）の慣行と整合する最大値で、これ以上分割すると個人 OSS の運用工数で維持できない
  - 関連要件: NFR-A-CONT-001（HA / RTO 4 時間 = L9 / L10 検証対象）、NFR-B-PERF-001〜007（L8 scale soak）、NFR-E-AC-001〜005（L7 chaos network-partition）

- **ADR-TEST-004: E2E を kind + multipass kubeadm の二層構造で実装し、release tag 強制を 3 重防御で物理化**
  - 判断: kind = L3/L4/L7 速度層、multipass で立てる Ubuntu VM 3 control-plane + 2 worker + kubeadm = L5/L9/L10 本番 fidelity 層。multipass 停滞時の退路として lxd 経路を予備整備。release tag 強制は ① `core.hooksPath = .githooks` 強制 ② `.githooks/pre-push` で `make qualify-pre-push` 強制 ③ `tools/release/cut.sh` を `git tag` の唯一の入口にする ④ git wrapper で `--no-verify` を塞ぐ、の 4 経路（実質 3 重防御 + bypass 阻止）で物理化
  - 理由: kind 単一層（A）は本番 fidelity が原理的に取れず ADR-CNCF-001 / ADR-INFRA-001 と矛盾。kind+lima（B）は WSL2 上での安定性に課題。k3s/k3d（D）は vanilla K8s 派生で ADR-CNCF-001 違反。multipass は WSL2 / macOS / Linux の 3 環境で公式サポート、kubeadm 直叩きで ADR-INFRA-001 本番と等価
  - 関連要件: NFR-F-SYS-001（オンプレ完結）、NFR-A-CONT-001（HA / RTO 4 時間）、NFR-A-REC-002（再現可能なテスト基盤）

- **ADR-TEST-005: 環境マトリクスを pairwise 抽出で 8〜12 jobs に圧縮し matrix.yaml で正典化**
  - 判断: 軸 8 種（K8s ver / CNI / CSI / LB / IP stack / OS / arch / runtime）× pairwise testing で Phase 0 ≤ 8 jobs / Phase 3 ≤ 30 jobs に圧縮。matrix.yaml で Phase 別 axes を併記、matrix-gen.py（allpairspy）で resolve、matrix-resolved.yaml を commit して PR レビュー対象化。重要な三つ組は must_include で固定。Python は portable 制約 1 の例外として明示
  - 理由: 完全 matrix 5184 組み合わせ（A）はリソース不可能 + flaky 数学的保証で常に赤、起案者主観選定（B）は属人化で contributor 増加時に説明不能、matrix なし（D）は ADR-INFRA-001 の environment overlay 思想と矛盾。pairwise はカバー率 100%（軸ペア）+ jobs 数 8〜30 のバランスを唯一実現し、業界標準（CNCF プロジェクト多数採用）
  - 関連要件: NFR-F-CHR-002（環境差分への耐性）、ADR-INFRA-001 / ADR-NET-001 / ADR-STOR-001/002 の代替候補検証

- **ADR-TEST-006: L7 chaos を Chaos Mesh、L8 scale を KWOK 1000 node + k6、soak を 24h 月次で構造化**
  - 判断: L7 chaos = Chaos Mesh on kind（PodChaos / NetworkChaos / IOChaos / TimeChaos / StressChaos / DNSChaos の CRD 宣言、6 シナリオ最低セット、`make qualify-nightly` 毎晩実行）。L8 scale = KWOK で 1000 node + 50000 pod + 1000 namespace（Phase 0、Phase 3 で 5000 node）+ k6 で 1000 RPS API 負荷。soak = k6 で 24h 連続負荷を月次 `make qualify-soak`、Phase 1 で chaos 複合シナリオ追加。Grafana LGTM 統合で SLO assertion を Prometheus クエリ判定
  - 理由: Chaos Mesh / KWOK / k6 はそれぞれ CNCF Incubating / SIG Scalability 公式 / Grafana Labs 製で 10 年保守の継続性最大。Litmus + kubemark + Locust（B）は kubemark が 2024 deprecated 路線、Toxiproxy 系（C）は IO/Time/Pod chaos を再現不能、放棄（D）は CNCF Sandbox 申請の testing maturity で失格 + ADR-TEST-003 の L7/L8 が「未来への先送り」化
  - 関連要件: NFR-A-CONT-001（L7 復旧可能性）、NFR-B-PERF-001〜007（L8 SLO assertion）、NFR-B-WL-001〜002（バースト負荷）

- **ADR-TEST-007: L9 upgrade を N-2→N→N+1 ローリング、L10 DR を Velero + minio + etcd PITR で構造化**
  - 判断: L9 = multipass kubeadm cluster で kubeadm 公式 `upgrade plan/apply/node` 経路 + control-plane 1→2→3 → worker drain/upgrade/uncordon、`make qualify-release` 必須（30〜45 分/pair）。assertion は availability ≥ 99% / API 常時応答 / CRD 破壊なし / 代表 L4 シナリオ成功。L10 = 4 シナリオ（namespace-restore / etcd-pitr / pv-data-restore / region-failover）を Velero + minio + etcdctl で実装、resource diff + PV SHA256 + etcd diff で integrity 機械判定、`make qualify-release` 必須
  - 理由: kubeadm 公式 + Velero + minio + etcdctl は ADR-INFRA-001 / ADR-CNCF-001 / ADR-DATA-003 と完全整合し本番 fidelity 100%。Cluster API 経由（B）は management cluster が要り multipass 構成と整合しない、手動 upgrade + K8up（C）は ADR-INFRA-001 の宣言的 cluster lifecycle 思想と矛盾、放棄（D）は本番 upgrade で deprecated API バグ顕在化 + RTO 4 時間が机上化
  - 関連要件: NFR-A-CONT-001（HA / RTO 4 時間）、NFR-A-DR-002（RPO / バックアップ）、NFR-A-REC-002（復旧可能性検証）

- **ADR-TEST-008: コンプライアンス検証を Sonobuoy / SLSA L3 / OSSF Scorecard / OpenSSF Best Practices Badge / FIPS 140-3 の 5 軸統合**
  - 判断: `make qualify-compliance` で 5 軸を順次実行し `tests/qualify-report/<version>/compliance/` に統合同梱。Sonobuoy は L5 内で `--mode certified-conformance`、SLSA L3 は in-toto attestation + cosign 署名 + Rekor 公開、OSSF Scorecard は 17 項目（CI-Tests 除く）で 8.0/10 以上、OpenSSF Best Practices Badge は Phase 0 で Silver 取得 / Phase 3 で Gold、FIPS 140-3 は aws-lc-rs（Rust）+ Microsoft Go Crypto fork（Go）で対応宣言
  - 理由: Sonobuoy のみ（B）/ CNCF 公式 2 軸（C）/ 全部放棄（D）はいずれも採用検討者の評価軸に欠落軸が出て CNCF Sandbox 申請 / エンタープライズ採用 / 政府系採用の選択肢を消す。5 軸統合は採用検討者が必要とする評価軸すべてを 1 ヶ所で確認可能にし、ADR-TEST-001 release artifact 中心モデルに外部評価軸の証跡を物理的に保有させる
  - 関連要件: NFR-E-ENC-001〜003（暗号要件 = FIPS 対応根拠）、NFR-F-STD-001（業界標準）、ADR-CNCF-001（CNCF Conformance）

- **ADR-TEST-009: 観測性 E2E を OTLP trace 貫通 / Prometheus cardinality / log↔trace 結合 / SLO burn rate alert / dashboard goldenfile の 5 検証で構造化**
  - 判断: `tests/e2e/observability/` 配下に 5 検証を独立サブディレクトリで配置、`make qualify-observability` で順次実行（kind + Grafana LGTM、所要 30〜45 分）、release blocking。trace 貫通は Tempo API で span tree assert、cardinality は baselines/*.json に上限値版管理し +20% 急増で fail、log↔trace は 95% 結合率 SLO、SLO alert は inject-slo-violation.sh で意図注入し fast burn 5 分窓発火を assert、dashboard は infra/observability/grafana/dashboards/*.json を baseline JSON と diff
  - 理由: trace のみ（B）は cardinality / SLO alert / dashboard 破壊を検出不能、観測性 E2E なし（C）は ADR-OBS-001/002/003 の決定が「実装かつ未検証」状態を放置、integration 統合（D）は L2 の defining property（同一プロセス）を超え時間窓を必要とする SLO alert が破綻。5 検証統合は CNCF Graduated 級 OSS の慣行と整合し observability の 5 主要側面を release ごとに継続検証
  - 関連要件: ADR-OBS-001（OTel）、ADR-OBS-002（Grafana LGTM）、ADR-OBS-003（インシデント分類）、NFR-B-PERF-001〜007（SLI 定義）、NFR-C-NOP-001〜003（Runbook 連動）

## ADR ライフサイクル

ADR は以下のステータスを持つ。

- **Proposed**: 提案中、Product Council でレビュー待ち
- **Accepted**: 承認済み、プラットフォームの現行判断
- **Deprecated**: 非推奨、新規適用禁止（既存利用は許容）
- **Superseded**: 後継 ADR に置換、旧 ADR として保全

ADR は削除せず、Superseded として保全する。ステータス変更は Product Council 承認が必要。

## ADR レビューサイクル

- **新規 ADR**: Product Council 月次ミーティングで承認
- **既存 ADR の陳腐化レビュー**: 年次で全 ADR を Product Council がレビュー、現行方針との整合を確認
- **要件改訂時**: 関連 ADR への影響を必ず確認、必要に応じて ADR 更新

## ADR と要件のトレーサビリティ

要件定義書の各要件は、関連する ADR へのリンクを持つことが望ましい。逆に ADR は影響する要件 ID のリストを `Related Requirements` セクションに記述する。これにより以下が実現する。

- 要件変更時に影響する ADR を即座に特定
- ADR 変更時に影響する要件を即座に特定
- 新規参画者が「この仕様の理由」を ADR 経由で理解

トレーサビリティの詳細は [../80_トレーサビリティ/](../80_トレーサビリティ/) を参照。
