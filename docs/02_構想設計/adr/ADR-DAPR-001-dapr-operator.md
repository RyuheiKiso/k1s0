# ADR-DAPR-001: 分散ランタイムに Dapr Operator を採用する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: tier1 開発チーム / システム基盤チーム / 採用検討組織

## コンテキスト

k1s0 は tier1 で 12 公開 API（state / pubsub / serviceinvoke / secrets / binding / workflow / log / telemetry / decision / audit / feature / pii）を gRPC ファサードとして提供する設計（ADR-TIER1-001 / ADR-TIER1-002 / ADR-TIER1-003）になっている。各 API の実体は Postgres / Kafka / OpenBao / Temporal / 自作 Rust crates 等のヘテロジニアスな backend で、tier2/tier3 からは内部言語と backend の差し替えが完全に不可視である必要がある。

この設計には「アプリケーション言語と backend を疎結合化する分散ランタイム層」が必要で、以下を満たすことが採用条件となる。

- **多 backend 抽象化**（state / pubsub / secrets / binding 等を component 切替で対応）
- **Kubernetes ネイティブ**（Operator / CRD で宣言的構成、Argo CD と整合）
- **mTLS の自動付与**（NFR-E-* セキュリティ要件）
- **observability の統合**（OTel と整合、ADR-OBS-002）
- **完全 OSS、業界標準**（採用組織の長期保守性、CNCF ステージ）
- **多言語 SDK**（Go / Rust / .NET / TypeScript の 4 言語、ADR-TIER1-001）

加えて k1s0 は tier1 内部の **stable な Go SDK** が必要で、Rust のような stable status が未確立の SDK だと Go ファサードの基盤として使えない。tier1-state / tier1-secret / tier1-workflow の 3 Go Pod は Dapr SDK を直接呼ぶ前提で実装されている（SHIP_STATUS § tier1）。

分散ランタイムの選択は tier1 ファサードの土台になるため **one-way door** に近く、後から差し替えると全 tier1 Pod / SDK / Helm chart / Component CR の総書き換えが発生する。リリース時点で確定し、採用組織の世代交代後も保守できる構造を残す。

## 決定

**tier1 の分散ランタイムは Dapr（CNCF Graduated）を採用し、Operator 経由で control-plane を HA 3 replica 運用する。**

- Dapr 1.17 LTS（control-plane: operator / placement / sentry / sidecar-injector / scheduler の 5 component すべて HA 3 replica）
- mTLS 必須有効（global.mtls.enabled: true、workloadCertTTL 24h、allowedClockSkew 15m）
- Component CRD で backend を差替可能（state.postgresql / pubsub.kafka / secrets.openbao / binding.s3 等）
- tier1 の 3 Go Pod（state / secret / workflow）は Dapr Go SDK を直接利用、tier2 .NET / Go サービスは Dapr SDK をクライアントから呼ぶ
- Subscription CRD で declarative な pubsub 接続を構成
- Prometheus メトリクス常時有効、JSON ログ出力
- ローカル開発（kind）は HA off / 1 replica で `tools/local-stack/manifests/45-dapr/values.yaml` に分離

`infra/dapr/control-plane/values.yaml` で production の Helm values を確定、`infra/dapr/components/` で 7 Component CRD を配置（state/postgres / state/redis-cache / pubsub/kafka / secrets/vault / binding/s3-inbound / binding/smtp-outbound / configuration/default）、`infra/dapr/subscriptions/` で 2 Subscription を配置。

## 検討した選択肢

### 選択肢 A: Dapr Operator（採用）

- 概要: Microsoft 発の分散アプリケーション ランタイム、CNCF Graduated
- メリット:
  - **CNCF Graduated**（最高成熟度ステージ）、長期保守の信頼性
  - state / pubsub / secrets / binding / workflow / configuration の **6 building block を統一抽象**
  - Component CRD で backend を実装非依存に切り替え可能（Postgres / Kafka / OpenBao / S3 / 30+ providers）
  - **Go / .NET / Java / Python / JavaScript / Rust（community） / PHP の SDK が公式**（k1s0 の 4 言語要件を満たす）
  - mTLS 自動付与、SPIFFE 互換、Sentry が CA を運用
  - Kubernetes Operator + sidecar injector で declarative 構成、Argo CD 完全対応
  - 業界実績豊富（Microsoft / Alibaba / Diagrid 等）
- デメリット:
  - Dapr sidecar 自体のメモリ overhead（〜100 MB / Pod）
  - sidecar 呼び出しのレイテンシ（localhost gRPC 経由、〜1ms）
  - Component CRD の挙動が provider バージョンに依存し、provider 互換性追従が必要

### 選択肢 B: 自作 abstraction（k1s0 専用ファサード）

- 概要: tier1 の各 API を Dapr に依存せず自前で抽象化（Postgres / Kafka / OpenBao を直接呼ぶ）
- メリット:
  - 外部依存ゼロ、k1s0 独自最適化が可能
  - sidecar overhead なし
- デメリット:
  - state / pubsub / secrets / binding の各 backend 抽象を自前で書く労力が膨大
  - 多 backend サポートを自前で維持する 10 年の負債
  - 4 言語 SDK を自作する必要、Dapr の 30+ provider エコシステム放棄
  - 業界標準を捨てるため、採用組織の人材流動性が下がる

### 選択肢 C: Spring Cloud + Spring Cloud Stream

- 概要: Java エコシステムの分散アプリケーション フレームワーク
- メリット: Java 圏で実績豊富、Spring エコシステムとの統合
- デメリット:
  - **Java/Kotlin 中心の設計**で、Go / Rust / .NET / TypeScript の 4 言語混在には向かない
  - K8s ネイティブではなく Spring Boot 中心、Argo CD / GitOps との整合が薄い
  - k1s0 の言語選定（ADR-TIER1-001 Go + Rust hybrid）と乖離

### 選択肢 D: Knative Eventing + Serving

- 概要: K8s 上の serverless / event-driven プラットフォーム
- メリット:
  - K8s ネイティブ、CNCF Incubating
  - event-driven autoscaling（KEDA と統合）が標準
- デメリット:
  - state / secrets / binding の抽象がない（pubsub / event 駆動のみ）
  - k1s0 の 12 API のうち state / secret / workflow / decision / audit / pii / feature 等の **大半をカバーできない**
  - Knative Serving の autoscale-to-zero は tier1 のような core サービスには合わない

### 選択肢 E: Service Mesh（Istio / Linkerd）の sidecar 直接

- 概要: Service Mesh の L7 機能のみで多 backend を抽象化
- メリット: mTLS / トラフィック管理は強力
- デメリット:
  - Service Mesh は **トラフィック層**に特化しており、state / secrets / binding 等の **アプリ層抽象は提供しない**
  - k1s0 の building block 要件を満たさない
  - Istio Ambient（ADR-0001）は東西通信、Dapr はアプリ層 building block で **責務が異なり競合しない**（むしろ併存）

## 決定理由

選択肢 A（Dapr Operator）を採用する根拠は以下。

- **building block の網羅性**: k1s0 が必要とする state / pubsub / secrets / binding / workflow / configuration / observability の 6 building block を **Dapr 単体で網羅**する。自作（B）/ Spring（C）/ Knative（D）/ Service Mesh（E）はいずれも一部のみカバーで、複数を組み合わせるとアーキテクチャの複雑度が爆発する
- **多言語 SDK**: 公式 SDK が Go / .NET / Java / Python / JavaScript を完備し、k1s0 の 4 言語要件（Go / Rust / .NET / TypeScript）と整合する。Rust SDK は community 製だが、k1s0 の Rust 領域は Dapr SDK を直接使わず gRPC ファサード経由で Go から呼ぶ設計（ADR-TIER1-001）のため問題にならない
- **CNCF Graduated**: 最高成熟度ステージで、10 年保守の前提で安心して採用可能。自作（B）の負債は採用組織が抱えきれない
- **業界標準への乗り続け権**: 業界が Dapr に集約しつつあり、採用組織の人材流動性・周辺ツールの整備が将来も改善される見込みが高い
- **Service Mesh との責務分離**: Dapr（アプリ層 building block）と Istio Ambient（トラフィック層 mTLS / Authorization）は責務が直交し、併存させることで観測性・セキュリティの二層防御が成立する。E は責務範囲が異なるため代替にならない
- **退路の確保**: Dapr Component CRD は backend 切替の抽象を提供するため、特定 backend（Postgres → CockroachDB 等）への移行コストは Component 差し替えに局所化される。Dapr 自体からの撤退コストは大きいが、building block ごとの backend 差し替えは柔軟

## 帰結

### ポジティブな帰結

- 12 公開 API の backend 切替が Component CRD 差し替えで完結（採用組織が既存 RDB / Message Bus と統合する際のコストが極小）
- mTLS / SPIFFE が自動有効、tier1 全 RPC で暗号化が標準
- Argo CD ApplicationSet で control-plane / Component / Subscription を一元 GitOps 管理
- 業界実績豊富で、採用組織の運用エンジニアが標準スキルで保守できる
- ADR-OBS-002（OTel Collector）と統合し、Dapr sidecar の observability が LGTM スタックに流れる

### ネガティブな帰結 / リスク

- Dapr sidecar の memory overhead（〜100 MB / Pod）。tier1 6 Pod × HA 3 = 18 sidecar で 1.8 GB を占有
- localhost gRPC 経由のレイテンシ（〜1ms）。NFR-B-PERF-001 の p99 < 500ms 達成には十分余裕があるが、極端な低レイテンシ要件には不向き
- Component provider 互換性追従の運用コスト（Postgres / Kafka / OpenBao 等の SDK 互換）
- Dapr 自体からの撤退は大規模で実質 one-way door。Component 切替（backend 差し替え）は柔軟だが、Dapr → 自作 → Spring 等の方式変更は不可逆

### 移行・対応事項

- `infra/dapr/control-plane/values.yaml` で HA 3 replica + mTLS + Prometheus + JSON ログを固定（既存実装あり）
- `infra/dapr/components/` で 7 Component CRD（state/postgres / state/redis-cache / pubsub/kafka / secrets/vault / binding/s3-inbound / binding/smtp-outbound / configuration/default）を Argo CD で管理（既存実装あり）
- `tools/local-stack/manifests/45-dapr/values.yaml` で kind 用の HA off / 1 replica 構成を分離（既存実装あり）
- Dapr バージョン追従手順を Runbook 化（minor upgrade 時の sidecar inject 互換確認、NFR-C-NOP-003）
- Component provider のバージョン互換性を CI で確認するゲート整備（IMP-CI-* 系）
- tier1 各 Pod で Dapr SDK の `DAPR_GRPC_ENDPOINT` 未設定時の in-memory fallback を実装済（SHIP_STATUS § tier1）

## 関連

- ADR-TIER1-001（Go + Rust hybrid）— Go 側で Dapr SDK 直接利用
- ADR-TIER1-002（Protobuf gRPC）— tier1 内部通信規約
- ADR-TIER1-003（言語不可視）— tier2/tier3 から Dapr SDK 直接参照は禁止
- ADR-0001（Istio Ambient）— トラフィック層との責務分離（併存）
- ADR-DATA-001/002/003/004 — Dapr Component の backend 選定
- ADR-SEC-002（OpenBao）— secrets/vault Component
- ADR-OBS-002（OTel Collector）— Dapr observability の統合
- DS-SW-IIF-* — tier1 内部 IF 設計

## 参考文献

- Dapr 公式: dapr.io
- CNCF Project Maturity Levels（Dapr は 2024 Graduated）
- Dapr Building Blocks: docs.dapr.io/concepts/building-blocks-concept/
