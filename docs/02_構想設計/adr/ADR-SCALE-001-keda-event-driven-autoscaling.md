# ADR-SCALE-001: Event-driven autoscaling に KEDA を採用する

- ステータス: Accepted
- 起票日: 2026-05-02
- 決定日: 2026-05-02
- 起票者: kiso ryuhei
- 関係者: tier1 / tier2 開発チーム / インフラ運用チーム / 採用検討組織

## コンテキスト

k1s0 は tier1（公開 12 API gRPC ファサード）/ tier2（ドメインサービス）/ tier3（BFF / SPA）の 3 階層で構成され、各層の負荷特性が異なる。

- **tier1**: gRPC RPS / per-tenant trafic で線形スケール、p99 < 500ms（NFR-B-PERF-001）の達成に multi-replica + LB 分散が必要（SHIP_STATUS G7）
- **tier2**: Kafka topic lag / Postgres queue depth / 業務イベント駆動でバースト負荷が発生
- **tier3**: HTTP RPS、ユーザーアクセス時間帯で 10:1 程度の変動

このうち tier2 / tier1 の event 駆動スケールに対しては、Kubernetes 標準の **Horizontal Pod Autoscaler（HPA）単独では対応できない**：

- HPA は CPU / メモリ / カスタムメトリクスのみ対応で、Kafka topic lag / Redis queue / Postgres job count などの **外部メトリクス** をネイティブに扱えない
- HPA はメトリクスサーバ経由の pull 型で、event 駆動のバースト（10 秒以内に 100 倍負荷）に追従するには間隔が長すぎる
- 0 replica からの起動（scale-from-zero）が HPA にはない

加えて、autoscaling の選択は dataplane（Cilium、ADR-NET-001）/ Service Mesh（Istio Ambient、ADR-0001）/ Dapr Operator（ADR-DAPR-001）と協調動作する必要があり、**業界標準の整合**が要件になる。

autoscaling 機構の選択は **two-way door** 寄り（後から差し替え可能）だが、tier2 サービスの ScaledObject 定義を全部書き直す移行コストは無視できない。リリース時点で確定し、採用組織の世代交代後も保守できる構造を残す。

## 決定

**Event-driven autoscaling には KEDA（Kubernetes Event-driven Autoscaling、CNCF Graduated）を採用し、HPA を補完する形で運用する。**

- KEDA 2.16 LTS、operator HA（leader election で active 1 + standby 2）+ metrics-apiserver HA 2 + admission webhook
- ScaledObject CRD で declarative にスケール定義（外部メトリクス: Kafka / Postgres / Redis / Prometheus / cron 等 60+ scaler）
- HPA は KEDA から自動生成されるため、CPU / メモリベースの単純スケールも KEDA 経由で統一管理
- scale-from-zero を許容する tier2 サービス（業務 batch）と許容しない core サービス（tier1 facade）を区別
- ServiceMonitor で KEDA 自身の SLI を Prometheus に取り込み、LGTM（ADR-OBS-001）で可視化

`infra/scaling/keda/values.yaml` で production の Helm values を確定（既存実装あり）。

## 検討した選択肢

### 選択肢 A: KEDA（採用）

- 概要: Microsoft / Red Hat 発の Event-driven Autoscaling、CNCF Graduated（2023）
- メリット:
  - **CNCF Graduated**、長期保守の信頼性
  - **60+ scaler**（Kafka / RabbitMQ / Redis / Postgres / Prometheus / AWS SQS / Azure Service Bus / cron / external HTTP / etc）
  - HPA を内部で生成するため Kubernetes ネイティブ、kubectl autoscale との互換性維持
  - **scale-from-zero** 対応（業務 batch / 低頻度サービスでコスト削減）
  - admission webhook で ScaledObject の妥当性検証
  - Prometheus メトリクスで KEDA 自身の動作可視化
  - Apache 2.0、完全 OSS
- デメリット:
  - operator + metrics-apiserver + webhook の 3 component を運用する必要
  - scaler ごとの認証情報管理（TriggerAuthentication CRD）の学習コスト
  - 一部の特殊な scaler（カスタム HTTP / クラウド固有）はバージョン互換性追従が必要

### 選択肢 B: HPA 単独

- 概要: Kubernetes 標準の Horizontal Pod Autoscaler のみで運用
- メリット:
  - 標準機能、追加 component なし
  - kubectl autoscale で declarative 定義
- デメリット:
  - **Kafka topic lag / Postgres queue depth 等の外部メトリクスを扱うには Custom Metrics API + 自作 adapter が必要**
  - scale-from-zero 不可
  - event 駆動のバースト（10 秒以内に 100 倍）に追従が遅い（pull 間隔 15 秒〜）
  - tier2 業務イベント駆動の autoscale 要件を満たせない

### 選択肢 C: Knative Serving

- 概要: K8s 上の serverless プラットフォーム、自動スケール内蔵
- メリット:
  - scale-from-zero / scale-to-zero が標準
  - HTTP request-based autoscale が内蔵
  - CNCF Incubating
- デメリット:
  - **Knative Serving 全体を採用する必要**があり、Pod ライフサイクル / Ingress / Routing が Knative 固有モデルに置換される
  - tier1 の core サービス（常時稼働、HA 必須）に scale-to-zero は不適切
  - Argo Rollouts（ADR-CICD-002）/ Istio Ambient（ADR-0001）との統合に追加レイヤが要る
  - Dapr（ADR-DAPR-001）との二重抽象になり、アーキテクチャ複雑度が爆発

### 選択肢 D: 自作 controller（k1s0 専用）

- 概要: ScaledObject 相当の CRD と controller を自作
- メリット: k1s0 独自最適化が可能
- デメリット:
  - 60+ scaler を自前で書く労力が膨大、10 年維持の負債
  - 業界標準を捨てるため採用組織の人材流動性が下がる
  - kubectl autoscale 等の標準ツールとの互換性を自前で確保する必要

### 選択肢 E: クラウド事業者の autoscaler（EKS Karpenter / GCP / Azure）

- 概要: 各クラウド事業者が提供する Pod / Node autoscaler
- メリット: 事業者統合、マネージド
- デメリット:
  - **オンプレ完結要件（NFR-F-SYS-001）に違反**、選択肢として成立しない
  - クラウドベンダーロック

## 決定理由

選択肢 A（KEDA）を採用する根拠は以下。

- **CNCF Graduated**: 2023 年に Graduated に昇格しており、10 年保守の前提で安心して採用可能。Knative Serving（C、Incubating）より成熟度が高い
- **scaler エコシステムの完備**: 60+ scaler により tier2 業務イベント駆動（Kafka topic lag / Postgres queue / Redis stream）と tier1 RPS ベースの autoscale を**単一機構で網羅**できる。HPA 単独（B）では Custom Metrics adapter を 60 個自作することになり破綻する
- **業界標準性**: Microsoft / Red Hat / Google 等が共同メンテナンスしており、採用組織の運用エンジニアが標準スキルで保守できる。自作（D）の負債は 10 年で破綻する
- **K8s ネイティブ統合**: KEDA は内部で HPA を生成するため、kubectl autoscale / kubectl get hpa 等の標準ツールがそのまま動く。ADR-CNCF-001 の vanilla K8s 維持と整合
- **責務の単一性**: Knative Serving（C）は autoscaling 以外に Ingress / Routing / 履歴管理を含むため、Argo Rollouts / Istio Ambient / Dapr との責務競合が発生する。KEDA は autoscaling に特化し、責務が直交する
- **scale-from-zero の現実性**: 業務 batch / 低頻度 tier2 サービスでは scale-from-zero がコスト削減に直結する。HPA 単独（B）では実現できない
- **退路の確保**: KEDA は HPA を内部で生成するため、KEDA から撤退する場合も生成済 HPA が残り、段階的に退路が確保される

## 帰結

### ポジティブな帰結

- tier2 業務イベント駆動 autoscale が ScaledObject CRD で declarative に定義可能
- tier1 facade の per-tenant RPS による HPA 補完が KEDA 経由で統一管理
- scale-from-zero で業務 batch のコスト削減が可能
- KEDA 自身の SLI が Prometheus に取り込まれ、LGTM で可視化（ADR-OBS-001）
- Argo Rollouts（ADR-CICD-002）の canary 戦略と整合（KEDA は Rollout の replica を直接操作しない、HPA 経由）

### ネガティブな帰結 / リスク

- KEDA control plane 3 component の運用負担（operator / metrics-apiserver / webhook）
- scaler ごとの認証管理（TriggerAuthentication CRD）の学習コスト
- KEDA バージョン追従時の ScaledObject API 互換性確認が必要

### 移行・対応事項

- `infra/scaling/keda/values.yaml` で operator HA + metrics-apiserver + admission webhook を固定（既存実装あり）
- ScaledObject の標準テンプレートを `deploy/charts/tier2-go-service/` / `tier2-dotnet-service/` / `tier1-rust-service/` の Helm chart に組み込み（採用初期で実装）
- KEDA バージョン追従手順を Runbook 化（NFR-C-NOP-003）
- scaler ごとの認証情報を OpenBao（ADR-SEC-002）経由で管理する経路を整備
- ServiceMonitor で KEDA 自身を Prometheus 監視

## 関連

- ADR-INFRA-001（K8s クラスタ）— bootstrap 後の autoscaling 機構として KEDA を install
- ADR-CNCF-001（CNCF Conformance）— vanilla K8s 上の autoscaling 標準
- ADR-CICD-002（Argo Rollouts）— Rollout の replica 操作との整合
- ADR-DATA-002（Strimzi Kafka）— Kafka scaler の主要利用先
- ADR-DAPR-001（Dapr）— Subscription 経由の event 駆動と integration
- IMP-DIR-INFRA-* — infra/scaling/keda/ 配置

## 参考文献

- KEDA 公式: keda.sh
- KEDA Scalers Catalog: keda.sh/docs/scalers/
- CNCF Graduation Announcement: cncf.io/projects/keda/
