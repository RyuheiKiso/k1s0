# QUA-OBS: 可観測性要件

本ファイルは、k1s0 プラットフォームの **メトリクス / ログ / トレース** 3 本柱の収集粒度・保持期間・統合基盤を定義する。可観測性は `QUA-PRF` の数値を測る土台であり、かつ `QUA-SLO` のバーンレート計算・エラーバジェット運用の原資である。ここが崩れると他の品質要件はすべて「根拠なし」で扱われる。

本プロジェクトには特殊な制約がある。Grafana Loki は 2024 年以降 AGPL で公開されており、オンプレ自社運用だとソースコード提供義務の解釈リスクが発生する（`COM-RSK-005`）。このため Phase 1a では Prometheus 素 UI + `kubectl logs` 運用を暫定採用し、Phase 1b 早期に SaaS Grafana Cloud または Community Edition 相当の AGPL 回避策に移行する。本カテゴリはその Phase 分割を要件化し、Phase 1a でも最低限の可観測性が保たれるよう規定する。

OTel（OpenTelemetry）は計装標準として Phase 1a から全 tier1 / tier2 で必須とする。バックエンドが Loki / Tempo / Mimir と入れ替わっても、アプリ側の計装は不変で済む。これは `COM-GLO-005` Dapr ファサードの差し替え可能性と同じ哲学（抽象を固定し実装を差し替える）である。

---

## 前提

- `COM-RSK-005` AGPL Phase 分割毀損リスク
- `QUA-PRF-*` レイテンシ計測の基盤
- `QUA-SLO-*` SLO / バーンレート計算の根拠
- `SEC-AUD-*` 監査ログは本カテゴリとは別物（`COM-GLO-008` 参照）
- [`../../01_企画/05_法務サマリ/01_OSSライセンス適合.md`](../../01_企画/05_法務サマリ/01_OSSライセンス適合.md) AGPL Phase 分割
- [`../../02_構想設計/01_アーキテクチャ/02_可用性と信頼性/02_プラットフォーム自己監視.md`](../../02_構想設計/01_アーキテクチャ/02_可用性と信頼性/02_プラットフォーム自己監視.md)
- [`../../02_構想設計/01_アーキテクチャ/04_非機能とデータ/01_非機能要件.md`](../../02_構想設計/01_アーキテクチャ/04_非機能とデータ/01_非機能要件.md) 5 節 運用性要件

---

## 要件本体

### QUA-OBS-001: OTel によるメトリクス・ログ・トレース統合

- 優先度: MUST（可観測性の計装標準。未達でバックエンド差し替えが不能になる）
- Phase: Phase 1a
- 関連: `COM-RSK-005`, `QUA-OBS-005`

現状、Prometheus クライアントライブラリ、Zap / Slog、OTLP SDK など計装ライブラリが複数候補あり、統一されていない。計装がバラつくとバックエンド差し替え時に tier1 / tier2 の全コード変更が必要になる。

本要件が満たされた世界では、全 tier1 / tier2 サービスが OTel SDK（Go は `opentelemetry-go`、Rust は `opentelemetry-rust`）で計装を行い、バックエンドは OTel Collector → Prometheus / Loki / Tempo（または代替 SaaS）に流す。結果として AGPL 回避のための Loki → SaaS 移行が OTel Collector 設定変更だけで完結する。

未達時、バックエンド差し替えのたびに数百 Pod の計装コード修正が必要となり、AGPL リスクの解消が数人月遅延する。

**受け入れ基準**

- 測定対象: tier1 / tier2 全サービスのメトリクス・ログ・トレース
- 測定地点: 各サービスの OTel SDK → OTel Collector（DaemonSet）→ バックエンド
- 計装標準: OTel 仕様（Semantic Conventions 準拠）
- 許容違反: 計装欠落サービスはビルド時の lint で検出し CI で拒否
- Phase 1a では OTLP → Prometheus + `kubectl logs` への流し込み。Phase 1b で Loki / Tempo または SaaS

**検証方法**

- CI で OTel SDK の import チェック（Go: `go.opentelemetry.io/otel`、Rust: `opentelemetry`）
- OTel Collector のパイプライン設定を E2E でテスト

---

### QUA-OBS-002: メトリクス保持期間と階層化

- 優先度: MUST（SLO 計算に 30 日ウィンドウが必要。コールド保管は法令対応）
- Phase: Phase 1a
- 関連: `QUA-SLO-001`, 構想設計 04_非機能とデータ 5.1 節

現状、Prometheus のローカル TSDB は 15 日のデフォルト保持で、30 日ウィンドウの SLO バーンレート計算に足りない。Phase 5 の 90 日長期保管も未整備。

本要件が満たされた世界では、メトリクスはホット（Prometheus ローカル 15 日）とコールド（Thanos / Mimir などオブジェクトストア 1 年）の 2 階層で保管され、SLO 月次レビュー・四半期トレンド分析・年次 TCO 分析すべてが根拠データを持って可能になる。

未達時、SLO 30 日ウィンドウの計算が不能で、`QUA-AVL-001` の月次稼働率の確定ができず、SLA 合意の裏付けが消滅する。

**受け入れ基準**

- ホット保持: 15 日（MVP Phase 1a、構想設計 5.1 節）、30 日（Phase 1b、SLO 計算のため延長）
- コールド保持: 1 年（Phase 1c 以降、Thanos または Mimir でオブジェクトストレージへ）
- 測定期間: 常時。古いデータの自動削除ジョブが動作
- 許容違反: データ欠落は 1 分以内のギャップまで許容（Prometheus のスクレイプ間隔が 15 秒）
- 保持期間設定は GitOps 管理

**検証方法**

- Prometheus `prometheus_tsdb_head_max_time` - `prometheus_tsdb_head_min_time` の差分を監視
- Thanos の object storage 側に 1 年前のデータが存在することを月次で抜き取り確認

---

### QUA-OBS-003: ログ保持期間と法令適合

- 優先度: MUST（アプリログは 15 日、監査ログは別要件で 7 年）
- Phase: Phase 1a
- 関連: `SEC-AUD-*`, `COM-GLO-008`, 構想設計 5.1 節

現状、アプリログと監査ログの保管要件が混同される傾向があり、「全ログ 7 年保持」といった過剰設計か「全ログ 15 日」といった過小設計が発生しがち。監査ログは別カテゴリ（`SEC-AUD`）で 7 年保持を扱うため、本要件はアプリログに限定する。

本要件が満たされた世界では、アプリログは 15 日（MVP）→ 90 日（Phase 5）のローリング保持で、障害調査・SLO 違反調査に必要十分な期間を確保する。ストレージコストが過剰にならず、監査要件は別経路で担保される。

未達時、アプリログを 7 年保持にするとストレージコストが年間数百万円規模で膨らむ一方、監査ログを 15 日にすると電子帳簿保存法の 7 年保管要件を満たせず法令違反。

**受け入れ基準**

- アプリログ（Loki または代替）: 15 日（MVP Phase 1a）、90 日（Phase 5）
- 監査ログ（`SEC-AUD-*` 別要件）: 7 年（電子帳簿保存法 10 条の 7 年保管要件に整合）
- PII を含むログはマスキング後に保管（PII 原文は保管しない）
- 測定地点: Loki / 代替 SaaS のインデックスサイズと保存ポリシー
- 許容違反: 保管期限切れデータは 24 時間以内に削除

**検証方法**

- Loki の `retention_period` 設定を GitOps で検証
- 四半期に 1 回、古いログの削除ジョブが動作しているか監査

---

### QUA-OBS-004: 分散トレースのサンプリング率

- 優先度: MUST（全量サンプリングは Phase 1a の小規模ならば実行可能、本番ではコスト爆発）
- Phase: Phase 1a
- 関連: `QUA-PRF-001`, 構想設計 5.1 節

現状、Grafana Tempo の保持 7 日・サンプリング率無指定で、本番 500 RPS 時にディスク容量が破綻する可能性がある。一方で全量サンプリングを外すと p99 の外れ値（500ms 超過トレース）を捕捉できない。

本要件が満たされた世界では、通常リクエストは 1% サンプリング、p99 超過リクエスト（500ms 超）は 100% サンプリングの **ハイブリッドサンプリング（tail-based sampling）** が適用され、外れ値調査の完全性とストレージコストの両立が実現する。

未達時、1% 固定サンプリングだと 500ms 外れ値を 99% 見逃し、原因調査が困難化。100% サンプリングだと Phase 5 の 500 RPS × 86,400 秒 × 7 日 = 3 億スパンでストレージが破綻。

**受け入れ基準**

- 通常リクエスト: 1% サンプリング
- p99 閾値超過（レイテンシ 500ms 超 / HTTP 5xx / 監査ログ失敗）: 100% サンプリング
- サンプリング判定は OTel Collector の tail_sampling プロセッサで実装
- トレース保持: 7 日（MVP）、30 日（Phase 5）
- 測定地点: OTel Collector の `otelcol_processor_tail_sampling_count_traces_sampled`

**検証方法**

- E2E テストで意図的に 600ms の遅延を注入し、100% トレースが保存されることを確認
- Tempo のストレージ使用量を月次レビュー

---

### QUA-OBS-005: Phase 1a の AGPL 回避下での代替可観測性

- 優先度: MUST（AGPL リスク解消まで Phase 1a でも最低限の運用可視化を保証）
- Phase: Phase 1a
- 関連: `COM-RSK-005`, `QUA-OBS-001`

現状、Grafana Loki / Tempo が AGPL 化された影響で、Phase 1a では採用が保留されている。一方で Phase 1a パイロット運用で障害調査を `kubectl logs` 手動実行だけで回すのは現実的でなく、検知遅延のリスクがある（`COM-RSK-005`）。

本要件が満たされた世界では、Phase 1a では Prometheus 素 UI + SaaS 可観測性ツール（Grafana Cloud / Datadog の無料枠など AGPL 非該当）または Community Edition 相当の代替で最低限の可視化が確保される。Phase 1b で正式な Grafana スタック（AGPL 適合解釈確定後）または SaaS 本契約に移行する。

未達時、Phase 1a パイロット運用中に障害を検知できず、tier3 顧客が先に気付いて報告するという信頼失墜シナリオが発生する。

**受け入れ基準**

- Phase 1a での代替経路: Prometheus 素 UI（メトリクス）+ Grafana Cloud 無料枠 または Zabbix など AGPL 非該当 OSS（ログ・アラート）
- 代替経路でも以下が可視化されている: tier1 API の成功率 / p99 レイテンシ / Pod 異常 / ノード障害
- SaaS 採用時はオンプレからの外向き通信（HTTPS 送信）が corporate proxy を通過できること
- Phase 1b 終了時点で AGPL 問題が法務判断済みであり、正式可観測スタックが稼働している
- 許容違反: Phase 1a の代替経路で障害検知遅延が 10 分を超えることは許容しない

**検証方法**

- Phase 1a 終了時点で「代替可観測性スタックの稼働レビュー」を法務 + SRE 合同で実施
- 四半期に 1 回、模擬障害を発生させ検知時間を計測

---

### QUA-OBS-006: ダッシュボードとアラート

- 優先度: MUST（可観測性データがあっても可視化がなければ使われない）
- Phase: Phase 1a
- 関連: `QUA-SLO-*`, 構想設計 02_プラットフォーム自己監視

現状、メトリクスは Prometheus に入っても、ダッシュボードが整備されていないと tier1 チームが毎朝確認することができない。アラートも Slack / Teams 経由のルートが未確定。

本要件が満たされた世界では、tier1 チームが毎朝 Grafana の「SLO Overview」「API Latency」「Incident Timeline」の 3 ダッシュボードを確認し、Critical アラートは 5 分以内にオンコール担当に届く運用が成立する。

未達時、データはあるが誰も見ない状態になり、SLO バーンレート 14.4 倍（2 日でバジェット枯渇）を見逃して顧客から報告される。

**受け入れ基準**

- 必須ダッシュボード: SLO Overview / API Latency / Incident Timeline / Resource Usage
- アラート経路: Slack or Teams の tier1-oncall チャンネル + PagerDuty / Opsgenie
- アラート重要度: Critical（P1, バーンレート 14.4x）は 5 分以内に通知、Warning（P2, 6x）は 30 分以内、Info（P3, 1x）は翌営業日
- 許容違反: アラート未発火は事後 Postmortem の対象

**検証方法**

- 四半期に 1 回、意図的に SLO 違反を発生させアラート配信を検証
- Grafana ダッシュボードの利用統計で週次アクセスを確認

---

## 章末サマリ

### ID 一覧

| ID | タイトル | 優先度 | Phase |
|---|---|---|---|
| QUA-OBS-001 | OTel によるメトリクス・ログ・トレース統合 | MUST | 1a |
| QUA-OBS-002 | メトリクス保持期間と階層化 | MUST | 1a |
| QUA-OBS-003 | ログ保持期間と法令適合 | MUST | 1a |
| QUA-OBS-004 | 分散トレースのサンプリング率 | MUST | 1a |
| QUA-OBS-005 | Phase 1a の AGPL 回避下での代替可観測性 | MUST | 1a |
| QUA-OBS-006 | ダッシュボードとアラート | MUST | 1a |

### 優先度分布

| 優先度 | 件数 | 代表 ID |
|---|---|---|
| MUST | 6 | QUA-OBS-001, 002, 003, 004, 005, 006 |
