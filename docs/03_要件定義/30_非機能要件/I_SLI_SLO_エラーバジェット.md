# I. SLI / SLO / エラーバジェット

本書は k1s0 プラットフォームのサービスレベル指標（SLI）、サービスレベル目標（SLO）、エラーバジェット運用を定義する。A 可用性の NFR-A-CONT-001 は対外 SLA（24h 基準 99%、月 7.2 時間）を定め、B 性能拡張性は SLA 水準の p99 目標を定める。本書はこれらを SRE プラクティスに則って「何を測るか・どう測るか・超過したら何をするか」のオペレーショナブルな契約に翻訳し、SLA より 0.9 ポイント厳しい SLO（99.9%、月 43 分）を内部運用目標として設定する。2 層運用の数値対応は A 可用性の冒頭節を参照。

## 本書の位置付け

SLO は単なる目標値ではなく、「この値を割った時にどう行動するか」の運用契約である。申請書で「稼働率 99.9%」と宣言しても、それを測定する指標（SLI）、SLI の計算ウィンドウ、違反時の対応が定まっていなければ、実運用では形骸化する。本書は A 可用性・B 性能拡張・E セキュリティ・G データ保護の目標値を SLI/SLO/エラーバジェットの枠組みで再整理し、チーム間の責任境界を明確にする。

SLO 設定は Google SRE 書籍群および DORA State of DevOps Report の Elite 基準を参照する。tier1 API 群はプラットフォームのコアのため、Elite 水準（99.9% 以上・p99 < 500ms・デプロイ失敗率 < 15%）を目指す。

## 参照文献一覧（版号・発行年・参照範囲）

SLO 数値の根拠を逆引きできるよう、参照元文献の版号・発行年・該当章節を明示する。採用検討レビュアーや監査担当者が「この 99.9% はどこから来たか」を問うた際、本一覧から該当章節まで一意に辿れる状態を維持する。版が改訂された場合は四半期レビューで本一覧を更新する。

| 略称 | 正式名 | 発行元 | 版・発行年 | 参照章節 | 用途 |
|---|---|---|---|---|---|
| SRE Book | Site Reliability Engineering | O'Reilly | 初版 2016 | 第 4 章「Service Level Objectives」、第 6 章「Monitoring Distributed Systems」 | SLI/SLO/SLA の概念定義、バーンレート計算式の原典 |
| SRE Workbook | The Site Reliability Workbook | O'Reilly | 初版 2018 | 第 2 章「Implementing SLOs」、第 5 章「Alerting on SLOs」 | マルチバーンレートアラート（14.4x / 6x）の根拠、28 日ウィンドウの推奨 |
| DORA Report | Accelerate State of DevOps Report | DORA（Google Cloud） | 2023 年版（最新は 2024 年版を四半期レビュー時に更新） | Performance Clusters（Elite/High/Medium/Low）定義、Four Keys 指標 | Elite 水準の定量基準（変更リードタイム 1 日以内、デプロイ失敗率 < 15%、復旧時間 1 時間以内） |
| Kafka Docs | Apache Kafka Documentation | Apache Software Foundation | 3.7（2024 リリース） | Producer acks=all の意味論、min.insync.replicas 推奨値 | PubSub API の p99 Publish レイテンシ 50ms の算定根拠（acks=all 前提） |
| Temporal Docs | Temporal Documentation | Temporal Technologies | 1.23（2024 リリース） | Workflow Determinism、Replay 仕様 | Workflow API の Determinism 100% 要件、Replay 不一致 0% の根拠 |
| ZEN Engine README | GoRules ZEN Engine GitHub | GoRules | v0.36（2024 リリース、四半期更新） | ベンチマーク章（100 ルール p99 測定結果） | Decision API の in-process p99 < 1ms の根拠 |
| Google SLO Guide | Google Cloud Architecture Center: SRE Best Practices | Google Cloud | 2024 更新版 | 「Setting SLOs」「Handling Overloads」 | コントロールプレーン個別 SLO（Keycloak/OpenBao 99.9%）の整合確認 |
| RFC 7231 | HTTP/1.1 Semantics | IETF | 2014 | 6.6 Server Error 5xx | Availability SLI の「5xx のみ失敗に計上」の原典 |
| 非機能要求グレード 2018 | 非機能要求グレード 2018 | IPA | 2018 改訂版 | A 可用性、B 性能拡張性 | モデル②該当性、目標稼働率の業界比較基準 |

## 4 層契約の全体像

観測層のメトリクス源から SLI を計算し、SLI を SLO と比較してエラーバジェット消費を追跡、バーンレートでアラート、SLA は SLO より緩い値を対外契約として設定する。この 4 層の関係を以下の図に示す。

![SLI / SLO / SLA / エラーバジェット階層](img/slo_hierarchy.svg)

矢印は情報の流れ（観測 → 集計 → 比較 → 契約化）を示す。左右の並びはレイヤ内でのカテゴリ並列（Availability/Latency/Freshness/Correctness の 4 SLI など）。色は責務領域を分離する（観測＝灰、SLI＝青、SLO＝紫、エラーバジェット＝赤、SLA＝橙）。

## SLI（Service Level Indicator）の定義

SLI は「ユーザーが体験するサービス品質」を数値で表す指標。k1s0 では以下の 4 種類を基本とする。

- **Availability SLI**: 総リクエスト中の成功応答の割合（2xx/3xx/認可拒否を除く 4xx は成功扱い、5xx のみ失敗）
- **Latency SLI**: 指定応答時間以内に応答した割合（例: p99 < 500ms を満たした割合）
- **Freshness SLI**: データの新鮮度（バックアップ完了からの経過時間、イベント配信遅延など）
- **Correctness SLI**: 意味的に正しい応答を返した割合（契約テスト合格率、ルール評価の一致率など）

SLI は Prometheus 互換メトリクス（Grafana Mimir 集約）から計算する。tier1 ファサード層でメトリクス発行は一元化され、tier2/tier3 は Telemetry API 経由で自動的に SLI 対象になる。

## SLO（Service Level Objective）の設定

k1s0 の主要コンポーネント別に SLO を定める。SLO は複数の SLI の組合せで定義され、ローリング 28 日ウィンドウで評価する。

### SLO 値の選定根拠表

各 SLO 値は「業界標準 → k1s0 特有事情による調整 → 選定数値」の 3 段階で根拠化する。数値は申請・監査で「なぜこの値を選んだか」を一意に説明できる粒度まで分解する。実測データが蓄積される リリース時点 パイロット 1Q 後に再検証し、必要なら本表を改訂する。

| 指標 | 選定値 | 業界標準・先行事例 | k1s0 特有調整 | 出典 |
|---|---|---|---|---|
| tier1 Service Invoke 稼働率 | 99.9% | SRE Book の Elite 水準、AWS 内部サービス標準 | 単一クラスタ構成のため 99.99% は構成上到達不可。SLA（99%）に対し 0.9pt 厳しい内部目標として設定 | SRE Book 第 4 章、[A_可用性.md](A_可用性.md) 数値根拠表 |
| tier1 State 稼働率 | 99.95% | AWS RDS Multi-AZ SLA（99.95%）、Azure SQL DB SLA | データ層は「失うと復旧困難」のため API 層より 0.05pt 厳しく設定。リリース時点 で HA 移行完了前は目標値として運用（実測 SLO は リリース時点 中に別途） | AWS RDS SLA、[A_可用性.md](A_可用性.md) NFR-A-FT-003 |
| tier1 Service Invoke p99 レイテンシ | 500ms | SRE Book の Web サービス典型値、DORA Elite 水準 | Dapr + Istio Ambient 経由のオーバーヘッド（約 10〜20ms）込みで余裕を持たせた値。構想設計の Protobuf gRPC 前提で到達可能 | SRE Book 第 4 章、[B_性能拡張性.md](B_性能拡張性.md) NFR-B-PERF-001 |
| tier1 State Valkey p99 レイテンシ | 10ms | Redis ベンチマーク（AWS ElastiCache 標準 1〜2ms）に対し余裕確保 | Valkey をクラスタ内設置する前提で network RTT 含め 10ms 上限。Istio Ambient ztunnel 追加分 3〜5ms を織り込み | Redis ベンチマーク、Istio Ambient POC 結果（採用後の運用拡大時 に確定予定） |
| tier1 PubSub Publish p99 レイテンシ | 50ms | Kafka Producer acks=all の典型値（ブローカー 3 台で 10〜30ms） | 1 ブローカー構成（採用初期）では書込み待ち時間増加を織り込み 50ms 上限に設定。採用側のマルチクラスタ移行時 の 3 ブローカー化で 20ms 以内を目指す | Kafka Docs 3.7 Producer 章、[B_性能拡張性.md](B_性能拡張性.md) NFR-B-PERF-005 |
| tier1 PubSub メッセージ損失率 | < 0.001% | Kafka at-least-once 設定（acks=all、min.insync.replicas 2）の理論値 | 業務データ逸失は監査インシデント扱いのため、実運用で「2 週間に 1 件以下」を閾値 | Kafka Docs、OR-INC-005（メッセージ逸失の Severity 基準） |
| tier1 Workflow 開始レイテンシ | 2 秒 | Temporal 公式ベンチマーク（通常 500ms〜1.5 秒） | k1s0 では Keycloak トークン検証 + テナント識別の追加処理で 500ms 増、余裕確保で 2 秒上限 | Temporal Docs 1.23、構想設計 tier1_workflow_design.md |
| tier1 Decision in-process p99 | 1ms | ZEN Engine README ベンチマーク（100 ルール規模で 0.2〜0.5ms） | Rust in-process 実装のため JIT コスト無し。複雑ツリー（nested 3 階層以上）では 5ms まで許容 | ZEN Engine v0.36 ベンチマーク、[B_性能拡張性.md](B_性能拡張性.md) NFR-B-PERF-004 |
| tier1 Audit 永続化遅延 | 5 秒 | WORM ストレージ（MinIO Object Lock）の書込み遅延、Kafka 経由の伝播時間 | Kafka → Audit Consumer → MinIO の 3 段構成で各段 1〜2 秒、余裕含め 5 秒上限。法令（個情法 30 日保管義務）は別要件 | MinIO Object Lock Docs、[G_データ保護とプライバシー.md](G_データ保護とプライバシー.md) |
| Kubernetes API Server 稼働率 | 99.95% | kubeadm 構築の single-master 構成は 99.9%、HA 構成で 99.95% | 運用蓄積後の control-plane HA 化前提で 99.95%。リリース時点 は 99.9% を暫定目標、Runbook で手動復旧 | Kubernetes コミュニティガイド、[A_可用性.md](A_可用性.md) NFR-A-FT-001 |
| Keycloak 認証 稼働率 | 99.95% | Keycloak 公式 HA ガイドの 2-node 構成で 99.95% | SSO 障害は全 API 停止に直結するため 99.9% でなく 99.95% に設定。リリース時点 で HA 移行 | Keycloak HA Guide、構想設計 keycloak_sizing.md |

### バーンレート閾値の根拠

Fast burn 14.4x・Slow burn 6x は SRE Workbook 第 5 章「Multi-window Multi-burn-rate Alerts」の推奨値を採用している。14.4x は「1 時間で月間予算の 2% 消費」を MTTD 5 分以内で検知できる値、6x は「慢性的劣化を 6 時間以内に検知」する値。k1s0 独自の調整は行わず、SRE Workbook の値をそのまま使用する。これは「先に k1s0 独自に最適化するより、業界標準を採用して運用経験を蓄積する方が安全」との判断による。



### tier1 Service Invoke API

- 目標稼働率: 99.9%（月間ダウンタイム 43 分以内）
- 目標レイテンシ: p50 < 50ms、p95 < 200ms、p99 < 500ms
- 測定ウィンドウ: 28 日ローリング
- エラー分類: mTLS 検証失敗、ルーティング失敗、タイムアウト、upstream 5xx を失敗に計上

### tier1 State API（Valkey / PostgreSQL）

- 目標稼働率: 99.95%（月間 22 分以内、データ層は厳しく）
- 目標レイテンシ: Valkey p99 < 10ms、PostgreSQL p99 < 100ms
- 目標 Freshness: レプリケーション遅延 < 5 秒（PostgreSQL Streaming Replication）
- データ整合性: ETag 不一致による書込衝突検出率 100%（衝突は CONFLICT を返す）

### tier1 PubSub API（Kafka）

- 目標稼働率: 99.9%
- 目標 Publish レイテンシ: p99 < 50ms（acks=all、ブローカー書込完了まで、NFR-B-PERF-005 と整合）
- 目標配信遅延: p99 < 1 秒（Publish から Subscriber 受信まで、End-to-End Freshness SLI）
- 目標 At-least-once 保証: メッセージ損失率 < 0.001%（2 週間に 1 件以下相当）

### tier1 Workflow API（Temporal）

- 目標稼働率: 99.9%
- 目標開始レイテンシ: Start から running まで p99 < 2 秒
- 目標 Determinism: Replay 不一致 0%（非決定コード混入時は CI/CD で検出）
- 目標永続性: Workflow 状態の耐久性 99.999%（年間 5 分の状態損失以内）

### tier1 Decision API（ZEN Engine）

- 目標稼働率: 99.9%
- 目標レイテンシ（in-process 評価のみ）: p99 < 1ms（NFR-B-PERF-004 と整合、100 ルール規模）
- 目標レイテンシ（gRPC 経由 End-to-End）: p99 < 50ms（シンプル）、p99 < 200ms（複雑なツリー）
- 目標 Correctness: 同一入力・同一ルールバージョンで 100% 同一出力

### tier1 Log / Telemetry API

- 目標稼働率: 99.9%
- 目標 Ingest レイテンシ: p99 < 1 秒（受信から Loki/Mimir 検索可能まで）
- 目標損失率: < 0.01%（バックプレッシャ時のドロップ許容）

### tier1 Secrets / Audit / Pii / Feature API

- 目標稼働率: 99.9%
- Secrets レイテンシ: p99 < 50ms（OpenBao キャッシュあり）
- Audit 永続化: 受付から WORM 永続化まで p99 < 5 秒、損失 0%
- Feature 評価レイテンシ: p99 < 10ms（flagd ローカルキャッシュ）

### データ平面（Kubernetes / Istio / Longhorn / MetalLB）

- Kubernetes API Server: 99.95%、kubectl 応答 p99 < 500ms
- Istio Ambient データプレーン: ztunnel 転送レイテンシ増加 p99 < 5ms
- Longhorn ストレージ: IOPS 低下時 < 20%、復旧 RTO < 10 分
- MetalLB: VIP フェイルオーバー < 30 秒

### コントロールプレーン（Keycloak / OpenBao / Argo CD）

- Keycloak 認証: 99.95%（SSO 経路の稼働率）、トークン発行 p99 < 200ms
- OpenBao: 99.9%、シークレット取得 p99 < 100ms
- Argo CD Sync: Git push から apply 完了まで p99 < 2 分

## エラーバジェット

エラーバジェットは「許容される違反時間」。例えば稼働率目標 99.9%（月間）の場合、月間 43 分までのダウンタイムが許容される。これを超過しない限り新機能リリースは許可され、超過したらリリース凍結して信頼性改善に集中する。

### バジェット計算方式

- 月次バジェット = (1 - SLO 目標稼働率) × 28 日
- 例: SLO = 99.9%、バジェット = 0.1% × 40,320 分 = 40.3 分/月
- バジェット消費は Availability・Latency・Correctness の合算で計算
- 消費率は Grafana ダッシュボードで常時可視化

### バジェット消費時の対応

- **消費 50% 未満**: 通常運用、新機能リリース継続
- **消費 50〜80%**: 注意レベル、リリース前レビューを強化、カナリア拡張
- **消費 80〜100%**: 警告レベル、非クリティカル機能のリリース凍結、信頼性タスクを優先
- **消費 100% 超過**: バーンダウン期間、全リリース凍結、インシデントレビュー必須、ポストモーテム実施

バジェット超過時の凍結解除は Product Council の決裁を要する。凍結中でもセキュリティ修正と信頼性改善は優先的にリリース可能。

### バジェット回復

ローリング 28 日ウィンドウなので時間経過で自動回復する。ただし回復を待たずに信頼性改善を実施することで、体質的な改善を図る。ポストモーテム（OR-INC-004）で root cause を特定し、再発防止策を次四半期の OKR に組み込む。

## マルチバーンレートアラート

SLO 違反は早期検知のためマルチバーンレートで監視する。Google SRE Workbook の推奨に従い以下 2 段階を設定する。

- **Fast burn**: 1 時間で月間バジェットの 2% を消費（バーンレート 14.4x）→ 即時ページング、Sev2 起票
- **Slow burn**: 6 時間で月間バジェットの 5% を消費（バーンレート 6x）→ 30 分以内のレビュー、Sev3 起票

バーンレートは以下の式で計算する: `実際のエラー率 / (1 - SLO 目標)`。Fast burn は致命的障害を短時間検出、Slow burn は慢性的劣化を検出する。

## SLA との関係

対外（テナントとの契約）SLA は SLO より低く設定する。SLO は内部目標、SLA は対外約束で違反時にペナルティ（課金割引 10%）が発生する契約条項。BC-LGL-005（責任分界）で具体条項を定める。k1s0 の 2 層運用の数値は [A_可用性.md](A_可用性.md) の「数値根拠表」が単一の真実源（single source of truth）であり、本書の以下の値は A_可用性.md と一致していなければならない。不一致はリリース凍結ルールの発散を招くため四半期レビューで検証する。

- **tier1 全般 SLA**: 99%（月間ダウンタイム許容 7.2 時間）— 申請書提示値、[A_可用性.md](A_可用性.md) NFR-A-CONT-001
- **tier1 全般 SLO**: 99.9%（月間 43 分以内）— 本書 NFR-I-SLO-001
- **マージン**: 月 約 6.5 時間（7.2 h − 43 min）— 層別割当は [A_可用性.md](A_可用性.md) の「マージン 6.5 時間の層別割当」参照

SLA は BC-LGL-005 の契約書で明文化、SLO はプラットフォームチーム内の運用指針とする。過去に本書が SLA 99.5%（月 216 分）と記載していた版があったが、企画書・A 可用性との不整合だったため是正した。

## ダッシュボードと可視化

全 SLO を Grafana の共通ダッシュボードで可視化する。ダッシュボードは以下の構成を含む。

- **SLO 概要**: 全主要 SLO の現在稼働率・バジェット消費率
- **バーンレート**: Fast/Slow burn の直近 1h/6h/24h 値
- **違反履歴**: 過去 90 日の違反イベントとポストモーテムリンク
- **コンポーネント別ドリルダウン**: クリックで詳細メトリクス画面へ

ダッシュボードはテナント向けにはステータスページ（NFR-H-TRN-001）経由で簡易版を公開、内部向けは詳細版を常時監視する。

## 要件 ID

tier1 公開 11 API それぞれに NFR-I-SLO-NNN を採番し、本書の各節の数値と一対一対応させる。同じ数値が A 可用性・B 性能拡張で再引用される際は、本要件 ID を介して参照する（数値の複数箇所分散を回避するため）。

### SLI 計測要件

- **NFR-I-SLI-001**: 全 tier1 公開 API で Availability / Latency / Freshness / Correctness の 4 SLI を Prometheus 互換メトリクスで計測（MUST、リリース時点）

### SLO 要件（tier1 公開 11 API 別）

- **NFR-I-SLO-001**: Service Invoke API は 99.9% 稼働率・p99 < 500ms を SLO として設定（MUST、リリース時点）
- **NFR-I-SLO-002**: State API は 99.95% 稼働率・Valkey p99 < 10ms・PostgreSQL p99 < 100ms・レプリ遅延 < 5 秒を SLO として設定（MUST、リリース時点）
- **NFR-I-SLO-003**: PubSub API は 99.9% 稼働率・Publish p99 < 50ms・配信遅延 p99 < 1 秒・損失率 < 0.001% を SLO として設定（MUST、リリース時点）
- **NFR-I-SLO-004**: Secrets API は 99.9% 稼働率・取得 p99 < 50ms を SLO として設定（MUST、リリース時点）
- **NFR-I-SLO-005**: Binding API は 99.9% 稼働率・p99 < 200ms を SLO として設定（MUST、リリース時点）
- **NFR-I-SLO-006**: Workflow API は 99.9% 稼働率・Start p99 < 2 秒・Determinism 100%・永続性 99.999% を SLO として設定（MUST、リリース時点）
- **NFR-I-SLO-007**: Log API は 99.9% 稼働率・Ingest p99 < 1 秒・損失率 < 0.01% を SLO として設定（MUST、リリース時点）
- **NFR-I-SLO-008**: Telemetry API は 99.9% 稼働率・Ingest p99 < 1 秒・損失率 < 0.01% を SLO として設定（MUST、リリース時点）
- **NFR-I-SLO-009**: Decision API は 99.9% 稼働率・in-process p99 < 1ms・gRPC 経由 p99 < 50ms（シンプル） / < 200ms（複雑ツリー）・Correctness 100% を SLO として設定（MUST、リリース時点）
- **NFR-I-SLO-010**: Audit-Pii API は 99.9% 稼働率・Audit 永続化 p99 < 5 秒・損失 0% を SLO として設定（MUST、リリース時点）
- **NFR-I-SLO-011**: Feature API は 99.9% 稼働率・評価 p99 < 10ms を SLO として設定（MUST、リリース時点）

### SLO 要件（インフラ層）

- **NFR-I-SLO-101**: Kubernetes API Server は 99.95% 稼働率・kubectl 応答 p99 < 500ms を SLO として設定（MUST、リリース時点。リリース時点 は 99.9% 暫定）
- **NFR-I-SLO-102**: Istio Ambient データプレーン（ztunnel）はレイテンシ増加 p99 < 5ms を SLO として設定（SHOULD、採用後の運用拡大時）
- **NFR-I-SLO-103**: Longhorn ストレージは IOPS 低下時 < 20%・復旧 RTO < 10 分を SLO として設定（MUST、リリース時点）
- **NFR-I-SLO-104**: MetalLB は VIP フェイルオーバー < 30 秒を SLO として設定（MUST、リリース時点）
- **NFR-I-SLO-105**: Keycloak 認証は 99.95% 稼働率・トークン発行 p99 < 200ms を SLO として設定（MUST、リリース時点）
- **NFR-I-SLO-106**: OpenBao は 99.9% 稼働率・シークレット取得 p99 < 100ms を SLO として設定（MUST、リリース時点）
- **NFR-I-SLO-107**: Argo CD Sync は Git push から apply 完了まで p99 < 2 分を SLO として設定（MUST、リリース時点）

### エラーバジェット要件

- **NFR-I-EB-001**: エラーバジェット消費率を Grafana で常時可視化（MUST、リリース時点）
- **NFR-I-EB-002**: バジェット超過時のリリース凍結プロセスを Runbook で整備（MUST、リリース時点）
- **NFR-I-EB-003**: マルチバーンレートアラート（Fast 14.4x / Slow 6x）を設定（SHOULD、リリース時点）
- **NFR-I-EB-004**: 月次マージン（SLA − SLO）の層別消費実績をレビュー、閾値 70% 超の層は翌月の体質改善タスクを OKR に組込み（MUST、リリース時点）

### SLA 要件

- **NFR-I-SLA-001**: 対外 SLA は SLO より 0.9 ポイント緩い設定（tier1 全般 99%）で BC-LGL-005 に明文化。違反時ペナルティ条項を契約書に含める（MUST、リリース時点 契約締結時）

## メンテナンス

SLO は四半期ごとに Product Council でレビュー、ビジネス要求と技術的可能性の変化に応じて調整する。SLO を緩める場合は BC-LGL-005 の SLA と連動して見直し、テナント通知を要する。新規コンポーネント追加時は SLO 設定を PR の必須項目とする。
