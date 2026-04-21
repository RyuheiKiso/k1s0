# 11. SLI / SLO / エラーバジェット方式設計

本ファイルは要件定義書の SLO 関連要件（NFR-I 系 8 要件および NFR-A-CONT-001 / NFR-A-CONT-002 と連携）を受けて、tier1 11 API の SLI 定義、SLO 目標、Error Budget Policy、Burn Rate Alert の方式を確定する。[04_監視と観測性方式.md](04_監視と観測性方式.md) が監視の**実装**を扱うのに対し、本章は「何を約束し、その約束を守るためにどう運用するか」の**契約層**を扱う。

## 本ファイルの位置付け

SLO（Service Level Objective）は運用と開発の「共通言語」である。SLO 数値が曖昧だと、「速くない」「落ちすぎ」といった主観的な衝突が発生し、リリース判断・機能追加判断が停滞する。本章は SLI（Indicator）を測定可能な指標として定義し、SLO 目標値を具体で書き、予算消費時のアクションを Error Budget Policy として確定する。

Google SRE 本の「エラーバジェットを全部使い切ってもいい、ただし使い切ったら新機能開発を止めて信頼性向上に回す」という契約文化を k1s0 でも採用する。これは開発と運用の衝突を数値で解決するメカニズムであり、2 名運用を守る命綱でもある。

## SLI の定義方式

SLI は「何をもって成功とするか」を客観的に測定する指標である。k1s0 の tier1 API では、可用性（Availability）、遅延（Latency）、品質（Quality、データ新鮮度等）の 3 種類を採用する。

**設計項目 DS-NFR-SLO-001 SLI の 3 種類**

- Availability SLI: `good_requests / total_requests`。good は HTTP 2xx もしくはビジネス成功の 4xx（404 等は total から除外）。
- Latency SLI: `fast_requests / total_requests`。fast は p99 目標値以内のリクエスト。p99 を直接 SLI にすると直感的だが、boolean 化した方が Multi Burn Rate Alert が組みやすい。
- Quality SLI（一部 API のみ）: データ新鮮度（Cache Hit 率 / データ同期遅延）の boolean 化。

**設計項目 DS-NFR-SLO-002 測定点**

- Envoy Gateway の access log: 外部から見た成功率・レイテンシを測定。外形 SLO の主要情報源。
- tier1 内部の OTel メトリクス: 内部処理の成功率・レイテンシを測定。内部 SLO 算定に利用。
- Synthetic Monitoring: 5 分間隔で 11 API 全てを実行、外部から見た SLI を補強。
- 3 点の数値は近似するが完全一致しない。外形（Envoy）を真値とし、内部と Synthetic はクロスチェック。
- 確定フェーズ: Phase 1b で Envoy + 内部、Phase 1c で Synthetic 統合。

## SLO 目標値

tier1 公開 11 API 全てに SLO を設定する。API ごとに目標が異なり、Decision は厳しく（p99 1ms）、Workflow Start は緩く（p99 500ms）。

**設計項目 DS-NFR-SLO-003 SLO 目標マトリクス**

| API | Availability | Latency 目標 | Latency SLO |
| -- | -- | -- | -- |
| Service Invoke | 99.9% | p99 500ms | 95% |
| State Get | 99.95% | p99 10ms | 99% |
| State Set | 99.9% | p99 50ms | 99% |
| PubSub Publish | 99.9% | p99 50ms | 99% |
| Secrets Get | 99.95% | p99 100ms | 99% |
| Binding Invoke | 99.9% | p99 500ms | 95% |
| Workflow Start | 99.9% | p99 500ms | 95% |
| Log Append | 99.99% | p99 10ms | 99% |
| Telemetry Push | 99.9% | p99 50ms | 99% |
| Decision Evaluate | 99.95% | p99 1ms | 99% |
| Audit-Pii Scan | 99.9% | p99 50ms | 99% |
| Feature Evaluate | 99.95% | p99 10ms | 99% |

（表は章末サマリとして位置づけ）。Availability は「成功リクエストの割合」、Latency SLO は「目標レイテンシ以内のリクエストの割合」を指す。

- 確定フェーズ: Phase 1b で全 API 実装、Phase 1c で初回 SLO レビュー。

**設計項目 DS-NFR-SLO-004 算定ウィンドウ**

- 30 日ローリングウィンドウ: 主要算定期間、Error Budget 算定に使用。
- 7 日ローリング: 短期トレンド把握、バーンレート計算のソース。
- 90 日ローリング: 長期トレンド、四半期レビュー用。
- 算定結果は Mimir recording rule で事前計算、ダッシュボード表示は即時。
- 確定フェーズ: Phase 1b 実装。

## Error Budget の計算

Error Budget は「SLO 未達成を許容する数値範囲」である。99.9% SLO なら月あたり 0.1% = 約 43 分の未達成を許容する。

**設計項目 DS-NFR-SLO-005 Error Budget 計算**

- 30 日 Error Budget = (1 - SLO) × 30 日 × 24 時間 × 60 分。
- 99.9% SLO: 43.2 分 / 30 日。
- 99.95% SLO: 21.6 分 / 30 日。
- 99.99% SLO: 4.3 分 / 30 日。
- Consumed Budget: `(1 - actual availability) / (1 - SLO) × 100%`。
- 残予算: 100% − Consumed。
- 確定フェーズ: Phase 1b で recording rule 実装。

## Error Budget Policy

予算消費率に応じてアクションを段階化する。この段階化が開発と運用の衝突回避のエンジンである。

**設計項目 DS-NFR-SLO-006 予算消費率別アクション**

- 消費 0〜50%（正常）: 通常開発・通常運用。新機能リリース自由。
- 消費 50〜80%（注意）: SRE が原因分析、Product Council に注意報告。開発は継続可。
- 消費 80〜100%（警戒）: 新機能凍結、信頼性向上作業のみ。Bug fix と性能改善を優先。
- 消費 100% 超過（超過）: 全リリース停止（Argo CD auto-sync 無効化）、復旧完了まで緊急対応のみ。Product Council Chair 承認でのみ例外許可。
- 新月次サイクル開始時: Budget リセット、0% から再スタート。

**設計項目 DS-NFR-SLO-007 Policy の強制**

- 消費 80% 到達時: Argo CD ApplicationSet の sync-policy を manual に切替（自動化スクリプト）。
- 消費 100% 到達時: Kyverno で新規 Deployment 作成を block（ラベル `signalid/release-freeze: true` を namespace に自動付与）。
- 例外解除: Product Council Chair の承認コメントが付いた PR で自動解除。
- 解除記録: 全ての例外解除は監査ログに残す。
- 確定フェーズ: Phase 1c で自動化実装。

## Burn Rate Alert

Error Budget を一気に消費する障害を早期検知するため、Multi Burn Rate Alert を設定する。速度の違う 2 つのウィンドウで異常を検知する方式は SRE Workbook の推奨パターンである。

**設計項目 DS-NFR-SLO-008 Multi Burn Rate Alert 設定**

- Fast burn: 1 時間ウィンドウ、14.4x 倍速（1h で月予算 2% 消費）→ SEV1。
- Slow burn: 6 時間ウィンドウ、6x 倍速（6h で月予算 5% 消費）→ SEV2。
- 計算式: `burn_rate = (1 - availability) / (1 - SLO)`。14.4 倍速は 1 時間 = 月の 1/720 を考慮し、(1/720) × 14.4 = 2% に対応する。
- 両方を満たす場合のみ発報（AND 条件）: 瞬間的なスパイクで発報せず、実際にバジェット消費しているケースのみ発報。
- 確定フェーズ: Phase 1b で実装、Phase 1c で係数調整。

**設計項目 DS-NFR-SLO-009 アラート通知経路**

- SEV1（Fast burn）: PagerDuty 即時 + Slack `#k1s0-sev1` + 電話（オンコール SRE）。応答 SLA 15 分。
- SEV2（Slow burn）: Slack `#k1s0-sev2` + メール。応答 SLA 2 時間。
- 誤発報率目標: SEV1 月 1 回以下、SEV2 週 2 回以下。
- 誤発報時の対応: 2 週連続で閾値見直しレビュー、係数調整。
- 確定フェーズ: Phase 1b 実装。

## SLO ダッシュボード

SLO を組織の共通言語にするため、全員が見るダッシュボードを用意する。Product Council・SRE・Dev が同じ数値で議論できる状態を作る。

**設計項目 DS-NFR-SLO-010 SLO ダッシュボード構成**

- `k1s0-slo-overview`: 11 API 全てのリアルタイム SLO 消費率（%、残日数換算）、Burn Rate 現状値。
- `k1s0-slo-<api_name>`: 各 API の詳細、30 日 / 7 日 / 90 日のトレンド、Error Budget 残。
- `k1s0-slo-monthly-report`: 月次 SLO レポート、前月比較、改善アクション一覧。
- 色分け: 緑（消費 0〜50%）、黄（50〜80%）、赤（80% 以上）、黒（100% 超過）。
- 確定フェーズ: Phase 1b で overview、Phase 1c で詳細・月次レポート。

## SLO レビューサイクル

SLO は一度決めたら終わりではない。月次で達成度レビュー、四半期で目標値レビューを行う。

**設計項目 DS-NFR-SLO-011 月次 SLO レビュー**

- 月末に前月の実績を集計、SRE が 2 時間でまとめ。
- 内容: 各 API の達成度、未達成原因の Top 3、翌月の改善アクション。
- Product Council 共有、Dev チームへのフィードバック。
- 月内 Incident との紐付け: 各 incident がどの API の SLO 消費に貢献したかを分析。
- 確定フェーズ: Phase 1c 運用開始。

**設計項目 DS-NFR-SLO-012 四半期 SLO 目標レビュー**

- SLO 目標値の調整可否を四半期ごとに判定。
- 調整条件: (1) 3 四半期連続で目標達成 → 引き上げ検討、(2) 3 四半期連続で未達成 → 目標見直しか信頼性投資。
- 調整判定者: Product Council。
- 変更履歴: SLO 変更は監査ログ + Git PR で記録、長期比較可能。
- 確定フェーズ: Phase 1c 第 1 四半期から開始。

## SLO と SLA の関係

SLA（外部契約、99.0%）と SLO（内部目標、99.9%）を 10 倍の差で設計する理由は、先行指標としての SLO 運用である。SLO 消費 80% の段階（まだ SLA 未達成）で新機能凍結を発動することで、SLA 未達成を構造的に防ぐ。

**設計項目 DS-NFR-SLO-013 SLA・SLO の役割分担**

- SLA 99.0%: 外部コミット、JTC 顧客との契約。月間 7.2 時間までのダウンタイム許容。
- SLO 99.9%: 内部目標、先行指標。月間 43 分までのダウンタイム許容。
- Internal Buffer: SLA の 10 倍厳しい SLO により、SLO 未達成でも SLA には余裕あり、対処時間を確保。
- 外部報告: SLA は月次公開、SLO は内部のみ。
- 確定フェーズ: Phase 1a SLA 確定、Phase 1b SLO 運用開始。

## 年間 SLO Report

監査対応パック（[09_完整性とコンプライアンス方式.md](09_完整性とコンプライアンス方式.md) DS-NFR-COMP-011）の一環として年次 SLO Report を自動生成する。

**設計項目 DS-NFR-SLO-014 年次 SLO Report**

- 生成時期: 事業年度末から 20 営業日以内。
- 内容: 全 API の年間達成度、月次トレンド、重大インシデント一覧と原因、翌年度改善計画。
- 形式: PDF + データ CSV、Backstage portal に保管。
- 公開範囲: Product Council、外部監査役、年次報告書（抜粋）。
- 確定フェーズ: Phase 1c 末の初回発行、以降毎年。

## SLI 計測方式の具体化

要件定義 NFR-I-SLI-001 は「全 tier1 公開 API で Availability / Latency / Freshness / Correctness の 4 SLI を Prometheus 互換メトリクスで計測（Phase 1a MUST）」を採番する。DS-NFR-SLO-001 では 3 種類（Availability / Latency / Quality）を定義したが、要件側は 4 SLI を明示しているため、Freshness と Correctness を独立した SLI として追加する。

**設計項目 DS-NFR-SLO-015 4 SLI の計測実装**

- Availability SLI: `k1s0_api_requests_total{status="success"} / k1s0_api_requests_total` を Prometheus recording rule で事前計算。success は HTTP 2xx + business-success 4xx。
- Latency SLI: `k1s0_api_request_duration_seconds{le="<target>"}` の histogram から fast_requests を boolean 化、target は API 毎に DS-NFR-SLO-003 の目標値。
- Freshness SLI: データ同期遅延（State の Cache Miss Rate、Audit ログの Publish Lag）を `k1s0_data_freshness_seconds` で計測。State は 5 秒以内、Audit は 5 秒以内を boolean 判定。
- Correctness SLI: Decision 評価・業務処理の正解一致率を `k1s0_correctness_total{match="true"} / k1s0_correctness_total`。Decision は自動テスト母集団との一致、Workflow は Determinism 検証。
- 全 4 SLI を Prometheus 互換（Mimir 保管）で統一、OpenMetrics 仕様準拠。
- 確定フェーズ: Phase 1a で Availability / Latency、Phase 1b で Freshness / Correctness。

## 個別 API の SLO 採番と整合

要件定義 NFR-I-SLO-001〜011 は tier1 公開 11 API 各々に SLO を採番し、NFR-I-SLO-101〜107 は tier1 基盤 OSS（k8s API Server / Istio ztunnel / Longhorn / MetalLB / Keycloak / OpenBao / Argo CD）に SLO を採番する。DS-NFR-SLO-003 の SLO 目標マトリクスで数値は記載済みだが、各 API の SLO ID と本書の設計項目の紐付けが未確定のため、本節で個別採番する。

**設計項目 DS-NFR-SLO-016 tier1 公開 11 API の SLO 個別確定**

tier1 公開 API はそれぞれ要件定義 NFR-I-SLO-001〜011 で SLO 数値が採番されている。これらは本書 DS-NFR-SLO-003 のマトリクスに集約されているが、API 毎の例外条件（Workflow の Determinism 100%、Audit の損失 0% など）を個別に設計項目として残す必要がある。

- Service Invoke (NFR-I-SLO-001): 99.9% 稼働、p99 < 500ms SLO 95%。成功判定は gRPC status OK のみ。
- State (NFR-I-SLO-002): 99.95% 稼働、Valkey p99 < 10ms / PostgreSQL p99 < 100ms / レプリ遅延 < 5 秒。データストア別に SLO 分離。
- PubSub (NFR-I-SLO-003): 99.9% 稼働、Publish p99 < 50ms、配信遅延 p99 < 1 秒、損失率 < 0.001%。損失率は Kafka ISR 監視で担保。
- Secrets (NFR-I-SLO-004): 99.9% 稼働、取得 p99 < 50ms。OpenBao の DS-NFR-SLO-106 と整合。
- Binding (NFR-I-SLO-005): 99.9% 稼働、p99 < 200ms。外部依存先（SaaS / メール送信等）のレイテンシを含むため緩め設定。
- Workflow (NFR-I-SLO-006): 99.9% 稼働、Start p99 < 2 秒、Determinism 100%、永続性 99.999%。Determinism 違反は即 SEV1。
- Log (NFR-I-SLO-007): 99.9% 稼働、Ingest p99 < 1 秒、損失率 < 0.01%。Loki の Ingester 冗長で損失低減。
- Telemetry (NFR-I-SLO-008): 99.9% 稼働、Ingest p99 < 1 秒、損失率 < 0.01%。Collector の queue + back-pressure で損失吸収。
- Decision (NFR-I-SLO-009): 99.9% 稼働、in-process p99 < 1ms / gRPC p99 < 50ms（シンプル）< 200ms（複雑）、Correctness 100%。Correctness 違反は Decision の業務ルール逸脱として SEV1。
- Audit-Pii (NFR-I-SLO-010): 99.9% 稼働、Audit 永続化 p99 < 5 秒、損失 0%。損失 0% は acks=all + ハッシュチェーンで担保。
- Feature (NFR-I-SLO-011): 99.9% 稼働、評価 p99 < 10ms。キャッシュヒット 95% 以上で達成。
- 確定フェーズ: Phase 1a で Log SLO（`k1s0.Log` が MVP-0 の唯一の tier1 Go 実装 API のため）、Phase 1b で Telemetry SLO、Phase 2 で残 9 API（Service Invoke / State / PubSub / Secrets / Binding / Workflow / Decision / Audit-Pii / Feature）の SLO。SLO 目標値自体は要件定義書 NFR-I-SLO-001〜011 で Phase 0 時点に固定済みで、本確定フェーズは SLO の**運用開始時期**を指す。

**設計項目 DS-NFR-SLO-017 基盤 OSS 7 種の SLO**

要件定義 NFR-I-SLO-101〜107 は tier1 基盤 OSS の SLO を採番する。tier1 API の SLO を守るためには基盤 OSS が崩れない前提が必要で、基盤 OSS 側も独立した SLO で運用する。

- Kubernetes API Server (NFR-I-SLO-101): 99.95% 稼働、kubectl 応答 p99 < 500ms（Phase 1b は 99.9% 暫定、Phase 1c で 99.95% 昇格）。
- Istio Ambient ztunnel (NFR-I-SLO-102): レイテンシ増加 p99 < 5ms（Phase 2 SHOULD）。tier1 API p99 の積算モデル（DS-NFR-PERF-001）に収まる制約。
- Longhorn (NFR-I-SLO-103): IOPS 低下 < 20%、復旧 RTO < 10 分（Phase 1b MUST）。データ層の I/O 性能を保証。
- MetalLB (NFR-I-SLO-104): VIP フェイルオーバー < 30 秒（Phase 1b MUST）。外部入口の切替性能。
- Keycloak (NFR-I-SLO-105): 99.95% 稼働、トークン発行 p99 < 200ms（Phase 1c MUST）。認証は全 API の前段のため厳しめ。
- OpenBao (NFR-I-SLO-106): 99.9% 稼働、シークレット取得 p99 < 100ms（Phase 1b MUST）。
- Argo CD Sync (NFR-I-SLO-107): Git push から apply 完了まで p99 < 2 分（Phase 1b MUST、MVP-1a の GitOps 導入に合わせる）。GitOps のリードタイム保証。要件定義書側の MUST 採番は Phase 1a だが、Argo CD 自体が [01_MVPスコープ.md](../../../01_企画/03_ロードマップと体制/01_MVPスコープ.md) の MVP-1a（Phase 1b）初投入のため、概要設計側は Phase 1b で SLO 運用を開始する（Phase 0.3 要件改訂時に要件定義側の Phase 指定も再調整予定）。
- 個別ダッシュボード: Grafana `k1s0-infra-slo` で 7 基盤 OSS の SLO を統合表示。
- 確定フェーズ: Phase 1b で Argo CD / Longhorn / MetalLB（いずれも MVP-1a 投入）、Phase 1c で Keycloak HA / OpenBao（MVP-1b で HA 化または初投入）、Phase 1c 後半で k8s API Server SLO 昇格（99.9% → 99.95%）、Phase 2 で Istio Ambient。

## Error Budget 運用の補強

要件定義 NFR-I-EB-001〜004 は「Grafana 常時可視化 / リリース凍結 Runbook / Multi Burn Rate Alert / 月次マージン層別消費レビュー」を採番する。DS-NFR-SLO-005〜010 で大枠は設計済みだが、層別消費レビューの方式が未確定のため本節で補強する。

**設計項目 DS-NFR-SLO-018 Error Budget 可視化の常時性**

要件定義 NFR-I-EB-001 は「Grafana で常時可視化」を要求する。DS-NFR-SLO-010 の overview ダッシュボードと連動し、リアルタイム性を担保する。

- 更新頻度: Prometheus scrape 15 秒、Grafana refresh 30 秒。リアルタイム予算消費を表示。
- 残予算表示: 秒単位ではなく「残 X 時間 Y 分」の直感的表示に変換。
- モバイル対応: Grafana Cloud の Mobile App でオンコール SRE が外出先でも確認可。
- 確定フェーズ: Phase 1b。

**設計項目 DS-NFR-SLO-019 リリース凍結 Runbook**

要件定義 NFR-I-EB-002 は「バジェット超過時のリリース凍結プロセスを Runbook で整備（Phase 1c MUST）」を要求する。DS-NFR-SLO-007 の自動化と合わせて、Runbook-SLO-FREEZE-001 を整備する。

- トリガー: Error Budget 消費 100% 到達。
- 手順: (1) Argo CD ApplicationSet の sync-policy を manual に切替、(2) Kyverno で新規 Deployment を block、(3) Product Council Chair へ通知、(4) 復旧計画の策定、(5) 復旧完了後の解除 PR 承認。
- 解除条件: 30 日ウィンドウの SLO 達成度が 95% 以上に回復、かつ復旧計画の Product Council Chair 承認。
- 解除記録: 全解除事案を ADR-SLO-NNN + Audit ログで永続記録。
- 確定フェーズ: Phase 1c Runbook 化 + 自動化。

**設計項目 DS-NFR-SLO-020 Multi Burn Rate Alert 係数**

要件定義 NFR-I-EB-003 は「Fast 14.4x / Slow 6x のマルチバーンレートアラート」を採番する。DS-NFR-SLO-008 で方式は確定済みだが、係数の根拠と調整サイクルを明示する。

- Fast burn 14.4x: 1 時間で月予算 2% 消費する速度、(1/720) × 14.4 = 2%。
- Slow burn 6x: 6 時間で月予算 5% 消費する速度、(6/720) × 6 = 5%。
- AND 条件: 瞬間スパイクで発報せず、短期と長期の両方で異常が継続するケースのみ発報。
- 係数調整: 誤発報率が目標を超える場合、SRE が 2 週連続レビュー後に係数見直し PR。係数変更は ADR-SLO-NNN で記録。
- 確定フェーズ: Phase 1b 実装、Phase 1c 以降係数調整。

**設計項目 DS-NFR-SLO-021 月次マージン層別消費レビュー**

要件定義 NFR-I-EB-004 は「月次マージン（SLA − SLO）の層別消費実績をレビュー、閾値 70% 超の層は翌月の体質改善タスクを OKR に組込み（Phase 1c MUST）」を要求する。層別消費を可視化しないと、どこで予算を食ったかを特定できず改善が回らない。

- 層別消費: DS-NFR-PERF-001 の 5 層（業務処理 / Dapr ファサード / OTel / 監査 / NW・DB）別に月次消費量を算出。
- 閾値: 層別消費率 70% 超の層を特定、Product Council に報告。
- OKR 組込: 翌月の体質改善タスクとして、該当層の改善施策を OKR アクションに追加。
- Grafana ダッシュボード: `k1s0-slo-margin-layer` で 5 層の消費率を時系列表示。
- 確定フェーズ: Phase 1c で月次レビュー開始、Phase 2 で四半期追跡。

## SLA 契約条項の具体化

要件定義 NFR-I-SLA-001 は「対外 SLA は SLO より 0.9 ポイント緩い設定（tier1 全般 99%）で BC-LGL-005 に明文化、違反時ペナルティ条項を契約書に含める（Phase 1b 契約締結時 MUST）」を採番する。DS-NFR-SLO-013 で役割分担は設計済みだが、契約書条項の具体化が不足する。

**設計項目 DS-NFR-SLO-022 SLA 契約条項とペナルティ**

- 稼働率条項: 「tier1 全 API の加重平均稼働率 99% を月次で保証、未達時は下記のサービスクレジット提供」。
- 計測条項: 「稼働率は Envoy Gateway の成功率を真値とし、計画停止（事前 72 時間告知）は除外」。
- ペナルティ条項: 月次稼働率 99% 未達で月額費用の 10% クレジット、98% 未達で 25%、95% 未達で 50%（業界標準 AWS/GCP の SLA 条項を参考に設計）。
- 免責条項: 天災・JTC 上位 NW 障害・顧客起因の障害は除外。
- 契約書レビュー: Phase 1b の契約締結前に法務部門 + Product Council 承認、BC-LGL-005 と整合。
- 違反時の運用: SLA 違反発生時は Runbook-SLA-BREACH-001 に従い、クレジット自動発行 + 再発防止 Post-Mortem 公開。
- 確定フェーズ: Phase 1b 契約締結時 MUST。

## 対応要件一覧

本ファイルは要件定義書の以下要件 ID に対応する。

本節は採番範囲 DS-NFR-SLO-001 〜 DS-NFR-SLO-022 をカバーする。要件定義のサブカテゴリ ID（NFR-I-SLI / NFR-I-SLO / NFR-I-EB / NFR-I-SLA）との対応は以下で明示する。

- NFR-A-CONT-001（tier1 対外 SLA 稼働率 99%）→ DS-NFR-SLO-013 / DS-NFR-SLO-022
- NFR-A-CONT-002（OpenBao 障害時の degrade 稼働を内部 SLO 99.9% 相当で維持）→ DS-NFR-SLO-005 / DS-NFR-SLO-013
- NFR-B-PERF-001〜005（tier1 個別 API の p99 性能 SLO）→ DS-NFR-SLO-003
- NFR-I-SLI-001（4 SLI 計測）→ DS-NFR-SLO-015
- NFR-I-SLO-001 Service Invoke → DS-NFR-SLO-016 / DS-NFR-SLO-003
- NFR-I-SLO-002 State → DS-NFR-SLO-016 / DS-NFR-SLO-003
- NFR-I-SLO-003 PubSub → DS-NFR-SLO-016 / DS-NFR-SLO-003
- NFR-I-SLO-004 Secrets → DS-NFR-SLO-016 / DS-NFR-SLO-003
- NFR-I-SLO-005 Binding → DS-NFR-SLO-016 / DS-NFR-SLO-003
- NFR-I-SLO-006 Workflow → DS-NFR-SLO-016 / DS-NFR-SLO-003
- NFR-I-SLO-007 Log → DS-NFR-SLO-016 / DS-NFR-SLO-003
- NFR-I-SLO-008 Telemetry → DS-NFR-SLO-016 / DS-NFR-SLO-003
- NFR-I-SLO-009 Decision → DS-NFR-SLO-016 / DS-NFR-SLO-003
- NFR-I-SLO-010 Audit-Pii → DS-NFR-SLO-016 / DS-NFR-SLO-003
- NFR-I-SLO-011 Feature → DS-NFR-SLO-016 / DS-NFR-SLO-003
- NFR-I-SLO-101 Kubernetes API Server → DS-NFR-SLO-017
- NFR-I-SLO-102 Istio Ambient ztunnel → DS-NFR-SLO-017
- NFR-I-SLO-103 Longhorn → DS-NFR-SLO-017
- NFR-I-SLO-104 MetalLB → DS-NFR-SLO-017
- NFR-I-SLO-105 Keycloak → DS-NFR-SLO-017
- NFR-I-SLO-106 OpenBao → DS-NFR-SLO-017
- NFR-I-SLO-107 Argo CD Sync → DS-NFR-SLO-017
- NFR-I-EB-001 Grafana 常時可視化 → DS-NFR-SLO-018 / DS-NFR-SLO-010
- NFR-I-EB-002 リリース凍結 Runbook → DS-NFR-SLO-019 / DS-NFR-SLO-007
- NFR-I-EB-003 Multi Burn Rate Alert → DS-NFR-SLO-020 / DS-NFR-SLO-008
- NFR-I-EB-004 月次マージン層別消費レビュー → DS-NFR-SLO-021
- NFR-I-SLA-001 SLA 契約条項とペナルティ → DS-NFR-SLO-022 / DS-NFR-SLO-013

逆引きは [../80_トレーサビリティ/02_要件から設計へのマトリクス.md](../80_トレーサビリティ/02_要件から設計へのマトリクス.md) を参照する。監視実装側は [04_監視と観測性方式.md](04_監視と観測性方式.md) と連動する。
