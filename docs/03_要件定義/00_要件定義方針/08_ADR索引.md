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
- `TEST`: テスト戦略（`ADR-TEST-001` Test Pyramid + testcontainers、後続 002〜007 で E2E 自動化 / CNCF Conformance / Chaos / Upgrade-DR / 観測性 E2E / テスト属性タグを補強予定）

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

テスト戦略系列（`ADR-TEST-*`）はリリース時点でテスト層構成・ツール選定・CI 時間予算を ADR で正典化し、4 言語並走（Rust / Go / .NET / TypeScript）の品質保証を単一の基本方針で統制する。tier1 の不具合が全 tier2 / tier3 に波及する構造に対し、層別の責務分界（UT / 契約 / 結合 / E2E）と横断軸（Chaos / SAST / SCA / DAST）を本系列で確定する。

- **ADR-TEST-001: Test Pyramid（UT 70% / 結合 20% / E2E 10%）+ testcontainers でテスト戦略を正典化**
  - 判断: Mike Cohn Test Pyramid を採用（UT 70 / 契約 5 / 結合 20 / E2E 5 の実数比）+ testcontainers を結合テストの標準ツール（mock/stub のみによる結合層は許容しない）。CI 時間予算 PR 5 分 / main 10 分 / 夜間バッチ 30 分を IMP-CI-RWF-010 / IMP-CI-QG-065 と整合。カバレッジ硬性基準は 行 80% / ブランチ 70%。Chaos / DAST は採用後の運用拡大時に追加し本 ADR では具体ツール選定しない（別 ADR-TEST-* で確定）。**ADR-DEVEX-003（テスト戦略標準化）を吸収**して docs-orphan を 2 件解消
  - 理由: Test Trophy（B、integration 重視）/ Diamond（C、UT 圧縮）は CI 時間予算（PR 5 分）に収まらず Lead Time 1h を破壊。自由形式（D）は属人化し採用組織の世代交代で崩壊。Test Pyramid + testcontainers のみが ① CI 時間予算 ② 業界標準スキル流用性 ③ 本番乖離の構造的回避 ④ 4 言語の成熟ライブラリ ⑤ ADR-TIER1-002 / IMP-CI-RWF-010 / IMP-CI-QG-065 との整合 の 5 点を同時に満たす
  - 関連要件: DX-TEST-001〜008（テスト戦略）、DX-MET-003（Lead Time 1h）、DX-MET-005（Change Failure Rate 5%）、NFR-E-RSK-002（ペネトレーションテスト）、NFR-E-AV-001/002（コンテナイメージスキャン）、NFR-E-WEB-001（OWASP Top 10）、NFR-A-CONT-001（tier1 SLA）

- **ADR-TEST-002: E2E テストを kind cluster + tools/local-stack + reusable workflow で自動化**
  - 判断: `tools/local-stack/up.sh --role e2e` を新設して kind cluster + 本番再現スタック（Argo CD / Istio Ambient / Dapr / CNPG / Strimzi / MinIO / Valkey / OpenBao / Backstage / Grafana LGTM / Keycloak）を起動、`tests/e2e/scenarios/` の Go test を実行。`.github/workflows/_reusable-e2e.yml`（IMP-CI-RWF-010 4 本構成への 5 本目追加）を `.github/workflows/nightly.yml`（cron 03:00 JST）から呼ぶ。failure artifact（screenshot / HAR / k6-summary / cluster-logs）を 14 日保存、`make verify-e2e` 1 コマンドでローカル再現。L4 と L5（CNCF Conformance、ADR-TEST-003）の cluster fidelity 責務分界は `docs/05_実装/30_CI_CD設計/30_quality_gate/02_test_layer_responsibility.md` で別途明文化
  - 理由: testcontainers 単一プロセス E2E（B）は K8s 上の Service Mesh / NetworkPolicy / Operator / SPIRE workload identity を再現できず本番乖離。multipass kubeadm（C）は GitHub Actions runner で nested virtualization 不可 + local-stack SoT から外れる。CI 自動化なし（D）は ADR-TEST-001 の「夜間バッチ全シナリオ実行」決定と矛盾し E2E 5% 比率が空洞化。kind + local-stack + reusable workflow のみが ① ADR-POL-002 SoT 拡張 ② 既存 IMP-CI-RWF-010 パターン整合 ③ ローカル / CI 同一 shell script による再現性 ④ ADR-TEST-001 CI 時間予算整合 ⑤ 採用検討者の `make verify-e2e` 1 コマンド再現容易性 を同時に満たす
  - 関連要件: DX-TEST-001〜008、DS-DEVX-TEST-006（Playwright）、DS-DEVX-TEST-007（k6）、IMP-DIR-COMM-112（tests 配置）、IMP-CI-RWF-010、ADR-POL-002（local-stack SoT）

- **ADR-TEST-003: CNCF Conformance を Sonobuoy + kind multi-node + Calico で月次実行**
  - 判断: Sonobuoy v0.57+ を `--mode certified-conformance` で kind multi-node cluster（control-plane 1 + worker 3、Calico CNI、vanilla K8s のみ）上で月次実行。`tools/local-stack/up.sh --role conformance`（新設、フルスタック非起動）+ `_reusable-conformance.yml` + `conformance.yml`（cron 月初 03:00 JST + workflow_dispatch）+ `tests/.conformance/<YYYY-MM>/` に results.tar.gz + summary.md を 12 ヶ月分 git LFS で版管理。IMP-CI-CONF-001〜005 を本 ADR で確定（実装段階の正典記述は別 commit で `90_対応IMP-CI索引/01_対応IMP-CI索引.md` へ展開）。ADR-TEST-002 で約束した `30_quality_gate/02_test_layer_responsibility.md` を本 commit で同時整備
  - 理由: production cluster での実行（B）はリリース時点で cluster 自体が無い + 業務影響リスク。Conformance 省略（C）は ADR-CNCF-001「移行・対応事項」を未充足のまま放置し CNCF Sandbox 申請が通らない。自前実装（D）は upstream e2e テストスイート（年 4 回更新）の追従工数で破綻。Sonobuoy on kind multi-node + Calico のみが ① ADR-CNCF-001 / ADR-NET-001 と完全整合 ② Actions runner 6 時間制約に余裕 ③ 12 ヶ月時系列証跡で採用検討者向け testing maturity 評価可能 ④ 個人 OSS の運用工数で持続可能（月次 1 回）を同時に満たす
  - 関連要件: ADR-CNCF-001（CNCF Conformance 維持）、ADR-NET-001（kind = Calico）、ADR-INFRA-001（kubeadm 本番）、IMP-CI-CONF-001〜005、NFR-F-STD-001（業界標準）、NFR-C-NOP-003（upstream バージョン追従）

- **ADR-TEST-004: Chaos Engineering を LitmusChaos で実装し、概要設計の Chaos Mesh 記述を訂正**
  - 判断: LitmusChaos v3+（CNCF Incubating、Apache 2.0、k8s ネイティブ CRD + Web UI）を採用。`infra/chaos/`（ADR-DIR-002 予約）配下に 5 シナリオ最低セット（Pod Kill / Network Latency / Network Partition / CPU Stress / Disk IO Stress）の ChaosEngine / ChaosExperiment CRD を配置。採用後の運用拡大時に `operation` namespace へデプロイし週次 CronChaosEngine で実行。リリース時点では実装ゼロ、本 ADR + 概要設計 `05_テスト戦略方式.md` の Chaos Mesh → LitmusChaos 訂正のみ
  - 理由: Chaos Mesh（B）は構想設計 02_周辺OSS で「次点」と明示済 + 概要設計の記述が古い drift。Toxiproxy（C）はネットワーク chaos 特化で 5 シナリオの 4/5 を満たせず ADR-DIR-002 の k8s ネイティブ前提と不整合。自前実装（D）は維持工数で破綻。LitmusChaos のみが ① 構想設計 03_周辺OSS / 04_CICDと配信 の既存決定と完全整合 ② ADR-DIR-002 / ADR-OPS-001（Chaos Drill）と接続 ③ Web UI で採用組織運用チームの学習曲線緩和 ④ ChaosHub 公開シナリオ流用で実装工数最小 を同時に満たす
  - 関連要件: DX-TEST-008、DS-DEVX-TEST-008（LitmusChaos に訂正）、ADR-DIR-002（infra/chaos/）、ADR-OPS-001（Chaos Drill 四半期実施）、NFR-A-CONT-001（HA / 縮退動作）

- **ADR-TEST-005: Upgrade drill（kubeadm N-2→N→N+1）と DR drill（既存 barman-cloud / etcdctl / GitOps 経路の実機検証）を採用後の運用拡大時に実施**
  - 判断: Upgrade drill は kubeadm 公式 plan/apply/node 経路で staging cluster 上で月次実施、release tag 直前必須。DR drill は既存設計（`01_障害復旧とバックアップ.md` / `02_etcd全ノード障害.md`）の 4 経路（A: etcd snapshot / B: GitOps 完全再構築 / C: PostgreSQL barman-cloud / D: Keycloak Realm Export）を四半期ローテーションで実機検証。**Velero / Stash / K8up 等の汎用 K8s resource backup ツールは新規導入しない**（既存戦略のコンポーネント別バックアップで完結、SoT 二重化を回避）。リリース時点では Runbook skeleton のみ
  - 理由: Velero 統合（B）は既存 ADR-DATA-001（barman-cloud）/ ADR-DATA-003（MinIO）/ 既存設計の SoT を割る。drill 実施なし（C）は机上 RTO（PostgreSQL 15 分 / etcd 30 分 / GitOps 4 時間）が実証されず採用検討者の信頼が低下。Kasten 等専用 DR ツール（D）は ADR-0003 OSS 方針と乖離 + ADR-DATA-* 連鎖改訂が要る。既存設計の drill 実施方針確定（A）のみが ① ADR-DATA-001/003 / ADR-INFRA-001 を改訂せずに既存 SoT を保つ ② ADR-OPS-001 四半期 Chaos Drill とローテーション枠共有 ③ Velero 不採用を構造的判断軸として残す ④ 4 経路を SRE 学習機会として提供（バス係数 2 整合）を同時に満たす
  - 関連要件: NFR-A-CONT-001（HA / RTO 4 時間）、NFR-A-DR-002（RPO / バックアップ）、NFR-A-REC-002（復旧可能性検証）、ADR-INFRA-001（kubeadm）、ADR-DATA-001（CloudNativePG）、ADR-DATA-003（MinIO）、ADR-OPS-001（Runbook + Chaos Drill）

- **ADR-TEST-006: 観測性 E2E を OTLP trace 貫通 / Prometheus cardinality / log↔trace 結合 / SLO alert 発火 / dashboard goldenfile の 5 検証で構造化**
  - 判断: `tests/e2e/observability/` 配下に 5 検証を独立サブディレクトリで Go test として実装、ADR-TEST-002 の `_reusable-e2e.yml` 経由で nightly 実行。検証 1=Tempo API で span tree assert / 2=Prometheus baselines/*.json + 1.2 倍超過で fail / 3=Loki LogQL で trace_id フィールド ≥ 95% / 4=k6 で意図 SLO 違反 → fast burn alert 5 分窓発火 + `runbook_url` 必須 / 5=Grafana dashboard JSON を baselines/*.json と jq diff。リリース時点は skeleton のみ、採用初期で検証 1 開始、採用後の運用拡大時で 5 検証完備
  - 理由: trace のみ（B）は cardinality / SLO alert / dashboard 破壊を検出不能。観測性 E2E なし（C）は ADR-OBS-001/002/003 の決定が「実装かつ未検証」状態を放置。L2 integration 統合（D）は L2 defining property（同一プロセス）超え + 5 分予算超過で破綻。5 検証統合（A）のみが ① ADR-OBS-001/002/003 の継続検証完備 ② ADR-TEST-002 の `--role e2e` LGTM スタック再利用で追加 cluster コストなし ③ 検証 4 が ADR-OPS-001 の `runbook_url` 必須要件を機械検証 ④ dashboard goldenfile は本検証類型でしか実装できない独立価値 を同時に満たす
  - 関連要件: ADR-OBS-001（OTel）、ADR-OBS-002（Grafana LGTM）、ADR-OBS-003（インシデント分類）、ADR-OPS-001（Runbook runbook_url 必須）、ADR-SEC-002（OpenBao = PagerDuty API key 格納）、NFR-B-PERF-001〜007（SLI 定義）、NFR-C-NOP-001〜003（Runbook 連動）

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
