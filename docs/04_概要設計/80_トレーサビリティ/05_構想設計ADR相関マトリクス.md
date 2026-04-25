# 05. 構想設計 ADR 相関マトリクス

本ファイルは構想設計書で採択された 27 個の ADR（Architecture Decision Record）と、概要設計書の各章・設計 ID との対応関係を固定化する相関マトリクスである。ADR は技術選定の「なぜ」を決定記録として残し、概要設計は選定を受けた「どう組み立てるか」を方式として固定化する。両者の対応を明文化することで、選定根拠を失った設計、逆に設計変更で崩れた選定前提を機械的に検出できる状態を目指す。

## 本ファイルの位置付け

構想設計書の ADR と概要設計書の設計 ID は本来、1:n の対応を持つ。たとえば ADR-TIER1-001（Go+Rust ハイブリッド）は、ソフトウェア方式設計のコンポーネント境界（DS-SW-COMP-*）・内部インタフェース方式（DS-SW-IIF-*）・制御方式（DS-CTRL-*）に一斉に影響する。逆方向では、1 つの概要設計 ID が複数 ADR に基づくこともある（例: DS-SW-EIF-012 State API は CNPG 採用 ADR と Valkey 採用 ADR の双方を前提とする）。

この n:n の対応関係が文書化されていない場合、以下 3 つの事故が発生する。第一に、ADR が改訂された際、概要設計への波及箇所が見落とされる。第二に、概要設計で別技術を採用しようとする PR が ADR との矛盾を抱えたまま通過する。第三に、監査で「なぜこの設計判断か」を問われた際、ADR への逆引きができない。

本ファイルはこれら 3 事故を防ぐため、27 個の ADR ごとに「適用章」「影響する設計 ID 範囲」「反映内容」「矛盾検出有無」を記録する。本章は索引層のため設計 ID を採番しない。

## マトリクスの構造

マトリクスは以下の 6 列で構成する。

- **ADR 番号**: 構想設計書で採番された ADR 識別子。
- **ADR タイトル**: 構想設計書の見出しをそのまま転記。
- **適用章**: 概要設計書で本 ADR を参照している章。
- **影響する設計 ID 範囲**: 本 ADR を前提に組まれた設計 ID の範囲。
- **反映内容**: 本 ADR の決定を概要設計でどう具体化したかの要点。
- **矛盾検出有無**: 概要設計と ADR の間に矛盾が検出されたか（`なし` / `要確認` / `矛盾あり`）。

矛盾検出は四半期棚卸しで実施する。検出された場合は、ADR 改訂または設計改訂のいずれかで解消し、解消 PR を Product Council 承認の対象とする。

## 基本構造 ADR（6 件）

tier1 プラットフォームの基盤となる 6 個の ADR は、概要設計書のほぼ全章に影響する。これらは 採用検討で最も重点的に審査される根幹判断である。

| ADR 番号 | タイトル | 適用章 | 影響する設計 ID 範囲 | 反映内容 | 矛盾検出 |
|---|---|---|---|---|---|
| ADR-0001 | Istio Ambient mesh 採用 | 10_SYS/03 処理、50_NFR/05 セキュリティ、04 監視、30_CF/02 認可 | DS-SYS-PROC-003, DS-NFR-SEC-001 〜 005, DS-NFR-OBS-002, DS-CF-AUTHZ-001 | ztunnel + waypoint proxy での L4/L7 分離、mTLS 必須、ネットワークポリシー強制 | なし |
| ADR-0002 | 図解記法規約 | 全章（drawio 作成時） | ‐（記法ルール） | 4 レイヤ記法（アプリ / ネットワーク / インフラ / データ）と色分けの厳格化 | なし |
| ADR-0003 | AGPL OSS の扱い | 30_CF 全般、50_NFR/05 セキュリティ、75_BUS/07 OSS ライセンス | DS-CF-*（AGPL 含む場合）, DS-NFR-SEC-002, DS-BUS-LIC-001 〜 004 | AGPL コンテナをネットワーク境界で隔離、tier2/tier3 ソース公開義務を回避 | なし |
| ADR-TIER1-001 | Go+Rust ハイブリッド | 20_SW/01 コンポーネント、03 内部 IF、40_CTRL 全般 | DS-SW-COMP-010 〜 030, DS-SW-IIF-001 〜 010, DS-CTRL-* 実装基盤 | Dapr ファサード層 = Go、自作領域（ZEN/crypto/CLI）= Rust、境界は Protobuf gRPC | なし |
| ADR-TIER1-002 | Protobuf gRPC 内部通信 | 20_SW/03 内部 IF、全 API 実装 | DS-SW-IIF-003 〜 008 | tier1 内部サービス間通信は全て Protobuf gRPC。JSON over HTTP は tier2/tier3 境界のみ | なし |
| ADR-TIER1-003 | tier2/tier3 から内部言語を不可視化 | 20_SW/01 コンポーネント、03 内部 IF、70_DEVX/04 Backstage、60_MIG、75_BUS/04 教育訓練 | DS-SW-COMP-060, DS-SW-IIF-001 〜 010（境界）, DS-DEVX-BS-001 〜 005（SDK テンプレート）, DS-MIG-CUT-001 〜 003（SDK 段階移行）, DS-BUS-TRN-001 〜 003（tier2/3 教育範囲） | tier2/tier3 は Protobuf IDL 生成クライアント SDK のみを利用、tier1 内部 Go/Rust を露出させない。モノレポ境界封鎖 + 内部レジストリ配布 + lint rule で物理的強制 | なし |

## データ層 ADR（4 件）

データ層 4 個の ADR は、状態ストア・メッセージブローカー・オブジェクトストレージ・キャッシュの採用 OSS を固定化する。概要設計では DB 方式・API 仕様・可用性方式に広範に波及する。

| ADR 番号 | タイトル | 適用章 | 影響する設計 ID 範囲 | 反映内容 | 矛盾検出 |
|---|---|---|---|---|---|
| ADR-DATA-001 | CNPG PostgreSQL 採用 | 10_SYS/04 DB、20_SW/04 DB 最上位、50_NFR/01 可用性、02 性能、07 環境 | DS-SYS-DB-001, DS-SW-DB-001 〜 010, DS-NFR-AVL-007, DS-NFR-PERF-005 | Kubernetes Operator 経由で CNPG を運用、HA = 1 プライマリ + 2 レプリカ、Barman バックアップ | なし |
| ADR-DATA-002 | Strimzi Kafka 採用 | 10_SYS/04 DB、20_SW/02 EIF（PubSub）、40_CTRL/04 MSG | DS-SYS-DB-003, DS-SW-EIF-020 〜 027, DS-CTRL-MSG-001 〜 005 | Strimzi Operator 経由で Kafka 運用、At-Least-Once、Schema Registry 併設 | なし |
| ADR-DATA-003 | MinIO オブジェクトストレージ | 10_SYS/04 DB、20_SW/02 EIF（Binding）、50_NFR/01 可用性 | DS-SYS-DB-005, DS-SW-EIF-032（Binding）, DS-NFR-AVL-008 | MinIO クラスタを S3 互換ストレージとして運用、4 ノード、erasure coding | なし |
| ADR-DATA-004 | Valkey キャッシュ採用 | 10_SYS/04 DB、40_CTRL/02 冪等、50_NFR/02 性能 | DS-SYS-DB-004, DS-CTRL-IDEM-001, DS-NFR-PERF-003 | Redis フォーク Valkey を冪等キー・セッションキャッシュに採用 | なし |

## セキュリティ ADR（3 件）

セキュリティ 3 個の ADR は、認証・秘密管理・サービス認証の 3 本柱を固定化する。全 tier1 API に横断的に影響する。

| ADR 番号 | タイトル | 適用章 | 影響する設計 ID 範囲 | 反映内容 | 矛盾検出 |
|---|---|---|---|---|---|
| ADR-SEC-001 | Keycloak 認証採用 | 30_CF/01 認証、50_NFR/05 セキュリティ、75_BUS/03 オンボーディング | DS-CF-AUTH-001 〜 005, DS-NFR-SEC-002, DS-BUS-ONB-002 | Keycloak を企業 IdP 前面に配置、OIDC 準拠、SSO 統合 | なし |
| ADR-SEC-002 | OpenBao 秘密管理採用 | 30_CF/08 暗号、20_SW/02 EIF（Secrets） | DS-CF-CRYPT-001 〜 005, DS-SW-EIF-028 〜 031 | Vault フォーク OpenBao で KMS・シークレット管理、KV/PKI/Transit エンジン | なし |
| ADR-SEC-003 | SPIFFE SVID 採用 | 30_CF/02 認可、50_NFR/05 セキュリティ、10_SYS/03 処理 | DS-CF-AUTHZ-001 〜 003, DS-NFR-SEC-001, DS-SYS-PROC-003 | SPIRE サーバー + SPIFFE ID でサービス認証、Istio Ambient と統合 | なし |

## ルールエンジン ADR（2 件）

ルールエンジン 2 個の ADR は、判定ロジックとワークフローの二分割を固定化する。短期実行判定 = ZEN Engine、長時間ワークフロー = Temporal の使い分けが基本方針。

| ADR 番号 | タイトル | 適用章 | 影響する設計 ID 範囲 | 反映内容 | 矛盾検出 |
|---|---|---|---|---|---|
| ADR-RULE-001 | ZEN Engine 判定 | 20_SW/02 EIF（Decision）、40_CTRL/05 振り分け | DS-SW-EIF-060 〜 064, DS-CTRL-WF-002 | ZEN Engine を Decision API の判定エンジンに採用、JDM 形式、ミリ秒判定 | なし |
| ADR-RULE-002 | Temporal ワークフロー | 40_CTRL/05 振り分け、20_SW/02 EIF（Workflow） | DS-CTRL-WF-003, DS-SW-EIF-044 | Temporal を長時間ワークフロー（>10 分）に採用、Dapr Workflow と二層運用 | なし |

## CI/CD ADR（3 件）

CI/CD 3 個の ADR は、継続的配信の中核 OSS を固定化する。DevEx 方式に集中的に影響するが、セキュリティ方式と可用性方式にも波及する。

| ADR 番号 | タイトル | 適用章 | 影響する設計 ID 範囲 | 反映内容 | 矛盾検出 |
|---|---|---|---|---|---|
| ADR-CICD-001 | Argo CD GitOps | 70_DEVX/01 CI/CD、04 Backstage | DS-DEVX-CICD-001 〜 005, DS-DEVX-BS-003 | Argo CD で GitOps 配信、App-of-Apps、SSO 統合、マルチテナント対応 | なし |
| ADR-CICD-002 | Argo Rollouts Canary | 70_DEVX/01 CI/CD、50_NFR/01 可用性 | DS-DEVX-CICD-003, DS-NFR-AVL-005 | Argo Rollouts で Canary デプロイ、5 ステップ、自動ロールバック条件 | なし |
| ADR-CICD-003 | Kyverno ポリシー強制 | 70_DEVX/01 CI/CD、50_NFR/05 セキュリティ | DS-DEVX-CICD-005, DS-NFR-SEC-008 | Kyverno で K8s マニフェスト検証、Baseline + Restricted ポリシーセット | なし |

## 観測性 ADR（2 件）

観測性 2 個の ADR は、ログ・メトリクス・トレースの 3 シグナルの OSS 構成を固定化する。

| ADR 番号 | タイトル | 適用章 | 影響する設計 ID 範囲 | 反映内容 | 矛盾検出 |
|---|---|---|---|---|---|
| ADR-OBS-001 | Grafana LGTM 採用 | 30_CF/03 ログ、05 トレース、06 メトリクス、50_NFR/04 監視 | DS-CF-LOG-003, DS-CF-TRACE-002, DS-CF-METRIC-002, DS-NFR-OBS-001 〜 003 | Loki / Grafana / Tempo / Mimir の LGTM スタックを採用、統一可視化 | なし |
| ADR-OBS-002 | OpenTelemetry 採用 | 30_CF/03 ログ、05 トレース、06 メトリクス、20_SW/02 EIF（Telemetry） | DS-CF-LOG-001, DS-CF-TRACE-001, DS-CF-METRIC-001, DS-SW-EIF-055 | OTel Collector をサイドカー / DaemonSet 併用、標準計装 | なし |

## その他 ADR（7 件）

基本構造 / データ / セキュリティ / ルール / CI/CD / 観測性 の 6 グループに分類されない 7 個の ADR。Feature Flag・ストレージ・LB・移行・ポータル領域に特化した ADR である。

| ADR 番号 | タイトル | 適用章 | 影響する設計 ID 範囲 | 反映内容 | 矛盾検出 |
|---|---|---|---|---|---|
| ADR-FM-001 | flagd Feature Flag | 30_CF/09 FM、20_SW/02 EIF（Feature） | DS-CF-FM-001 〜 003, DS-SW-EIF-070 | flagd を OpenFeature 準拠 FM として採用、GitOps でフラグ管理 | なし |
| ADR-STOR-001 | Longhorn ブロックストレージ | 10_SYS/01 HW、50_NFR/01 可用性 | DS-SYS-HW-008, DS-NFR-AVL-006 | Longhorn 3 レプリカ、RWO/RWX 対応、スナップショット運用 | なし |
| ADR-STOR-002 | MetalLB LoadBalancer | 10_SYS/01 HW、03 処理 | DS-SYS-HW-010, DS-SYS-PROC-005 | MetalLB BGP モードでオンプレ L4 LB、Istio Gateway 連携 | なし |
| ADR-MIG-001 | .NET Framework sidecar 移行 | 60_MIG/01 データ移行、02 切替 | DS-MIG-DATA-001 〜 003, DS-MIG-CUT-001 〜 003 | .NET Framework 資産を sidecar コンテナ化、段階移行 | なし |
| ADR-MIG-002 | API Gateway 移行 | 60_MIG/02 切替、20_SW/02 EIF | DS-MIG-CUT-004, DS-SW-EIF-085 | 既存 API Gateway → k1s0 Istio Gateway への切替、ルーティング共存期間 | なし |
| ADR-BS-001 | Backstage 開発者ポータル | 70_DEVX/04 Backstage、70_DEVX/06 Golden Path | DS-DEVX-BS-001 〜 005, DS-DEVX-GP-001 〜 002 | Backstage を開発者ポータル基盤として採用、Service Catalog + TechDocs + Software Templates | なし |

## ADR 影響度ランキング

26 個の ADR のうち、概要設計書への影響度が特に高いものを影響範囲順にランキングする。影響度は影響する設計 ID の件数で測り、上位の ADR は改訂時のレビュー負荷が高い。

| 順位 | ADR 番号 | 影響する設計 ID 件数（概算） | 影響章数 |
|---|---|---|---|
| 1 | ADR-TIER1-001 Go+Rust | 約 80 件 | 6 章 |
| 2 | ADR-DATA-001 CNPG | 約 40 件 | 5 章 |
| 3 | ADR-SEC-001 Keycloak | 約 35 件 | 5 章 |
| 4 | ADR-0001 Istio Ambient | 約 30 件 | 4 章 |
| 5 | ADR-SEC-003 SPIFFE | 約 25 件 | 4 章 |
| 6 | ADR-OBS-001 Grafana LGTM | 約 25 件 | 4 章 |
| 7 | ADR-OBS-002 OpenTelemetry | 約 25 件 | 4 章 |
| 8 | ADR-CICD-001 Argo CD | 約 20 件 | 3 章 |
| 9 | ADR-DATA-002 Strimzi Kafka | 約 18 件 | 3 章 |
| 10 | ADR-TIER1-002 Protobuf gRPC | 約 15 件 | 3 章 |

## 矛盾検出プロセス

ADR と概要設計の矛盾検出は、四半期棚卸しと PR 単位の 2 段階で実施する。

四半期棚卸しは Product Council の定例ゲートとして、全 26 個の ADR について概要設計書の反映状況を確認する。確認項目は以下 4 点。

- ADR の決定内容が概要設計書の該当章に正しく反映されているか。
- ADR の却下代替案が概要設計書で蘇っていないか。
- ADR が前提とした制約（OSS ライセンス・性能特性など）が概要設計で維持されているか。
- ADR の改訂履歴が概要設計書と同期しているか。

PR 単位のチェックは、概要設計書の PR が ADR への影響を持つ場合に必須とする。PR テンプレートに「影響する ADR 番号」列を設け、レビュアーが矛盾有無を判定する。

矛盾が検出された場合、以下 3 択のいずれかで是正する。第一に、ADR を改訂して新しい決定を記録する（設計の進化による選定変更）。第二に、概要設計を ADR に合わせて改訂する（設計の誤り）。第三に、概要設計で ADR を凌駕する新制約が見つかった場合、新規 ADR を起票して追加する。

## 運用ルール

本マトリクスの継続運用は以下の 5 ルールで固定化する。

- **新規 ADR 起票時**: 構想設計側で新規 ADR が採択された場合、同じ PR で本マトリクスに行を追加する。影響する設計 ID 範囲が リリース時点で未確定の場合は「リリース時点 時点で確定」と注記する。
- **ADR 却下時**: 却下された ADR は本マトリクスから削除せず、「状態」列を `却下` に書き換える。却下 ADR を参照している概要設計はその時点で非整合なので、別 PR で是正する。
- **概要設計側改訂時**: 概要設計書の改訂が ADR の前提を崩す場合、構想設計側の ADR 改訂を先行させる。概要設計のみの改訂で矛盾を解消することは禁止する。
- **四半期棚卸し**: Product Council 四半期レビューで矛盾検出列を再判定する。`要確認` または `矛盾あり` が 2 件を超える場合、次四半期で解消するロードマップを提示する。
- **段階切替時**: 採用初期 / 1c / 2 の切替時、全 ADR と概要設計の整合を確認する。整合未確認のまま 段階切替を承認することは禁止する。

## 下流参照

ADR 本体は [../../02_構想設計/adr/](../../02_構想設計/adr/) に格納。ADR 索引は [../90_付録/02_ADR索引.md](../90_付録/02_ADR索引.md) を参照。設計間依存は [04_設計間依存マトリクス.md](04_設計間依存マトリクス.md) を参照。

## 改訂履歴

| 日付 | 版 | 改訂内容 | 起票者 |
|---|---|---|---|
| 2026-04-20 | 0.1 | 採用検討向け初版。26 ADR × 概要設計の相関を整理。 | 概要設計チーム |
