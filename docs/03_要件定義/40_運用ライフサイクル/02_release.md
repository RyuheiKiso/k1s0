# OPS-REL: リリース要件

本ファイルは、**既定のリリース戦略（カナリア）と自動ロールバック、リリースノート、Feature Flag による切替** を要件化する。CI/CD パイプラインのゲート条件は [`01_CICD.md`](./01_CICD.md) で、環境別設定管理は [`03_environment_config.md`](./03_environment_config.md) で扱い、本ファイルは**トラフィックの段階的切替と問題検知時の戻し方**に集中する。

リリース要件の骨子は「(a) いきなり 100% にしない、(b) 問題検知時は人の判断を待たず戻す、(c) 戻せない変更は Feature Flag で可逆化する」の 3 点である。これが崩れると Phase 2 以降の同時稼働業務が連鎖停止するため、Phase 1c の本番稼働前に全 MUST 要件の達成が必要。

---

## 前提

- [`01_CICD.md`](./01_CICD.md) — デプロイパイプラインの前段
- [`../20_品質特性/03_sla_slo.md`](../20_品質特性/03_sla_slo.md) — SLO 違反の判定基準
- [`../20_品質特性/02_observability.md`](../20_品質特性/02_observability.md) — Argo Rollouts の AnalysisTemplate が参照する Prometheus 指標
- [`../50_開発者体験/02_feature_management.md`](../50_開発者体験/02_feature_management.md) — OpenFeature + flagd

---

## 要件本体

### OPS-REL-001: 既定リリース戦略はカナリア（5% → 25% → 100%）

- 優先度: MUST（本番 100% 即時切替は SLO 違反を全業務に波及させるため、Phase 1c 稼働前に必達）
- Phase: Phase 1c（業務稼働開始時）/ Phase 2 で Argo Rollouts + Istio 連携
- 関連: OPS-REL-002 / QUA-SLO-002

現状、tier1 は Deployment の RollingUpdate (maxSurge 25%) で段階的ではあるが、本番トラフィックは Pod 起動後に一気に流れるため、新バージョンのバグが全ユーザーに即波及する。

要件達成後の世界では、本番 tier1 / tier2 サービスのリリースは Argo Rollouts による **カナリア戦略** を既定とし、(1) カナリア 5% で 30 分観測、(2) 25% で 30 分観測、(3) 100% 昇格、の 3 ステップで段階実施する。各ステップの 30 分間は Argo Rollouts の `AnalysisTemplate` が Prometheus から `http_5xx_rate`（1% 上限）と `p99_latency_ms`（500ms 上限）を 1 分間隔で取得し、1 指標でも違反したら次ステップに進まない。Istio VirtualService の weight 分割で実現し、セッション粘着は不要（tier1 API はステートレス前提）。

崩れた時、新バージョンの不具合が本番全体に数秒で拡散し、例えば Go ファサードのメモリリーク修正で新たに導入されたデッドロックが全業務の Dapr サービス呼び出しを止める。業務側では稟議承認 3 か月フローが再試行不可状態になり、1 件の復旧に平均 2 時間を要する。

**受け入れ基準**

- 本番 Rollout リソースに `steps: [setWeight: 5, pause: 30m, analysis, setWeight: 25, pause: 30m, analysis, setWeight: 100]` が既定
- AnalysisTemplate が `http_5xx_rate < 0.01` と `http_request_duration_p99 < 500ms` を必須指標に含む
- カナリア中の違反検知で次ステップ移行が自動ブロックされ、Slack `#release-alerts` に通知
- ステートフルなサービス（ワークフロー等）はカナリア適用外を明示し、Blue-Green で代替（個別要件に記載）
- リリース回数の 80% 以上がカナリア戦略で実施されることを月次で確認

**検証方法**

- Argo Rollouts ダッシュボードで過去 30 日のリリース戦略内訳を集計
- 月次でカナリア適用率を Backstage Scorecard に表示

---

### OPS-REL-002: 自動ロールバック（SLO 違反検知 5 分以内）

- 優先度: MUST（人の判断を待つロールバックは平均 20 分を要し、エラーバジェットを一気に消費する）
- Phase: Phase 1c（本番稼働時）/ Phase 2 で全 tier 展開
- 関連: OPS-REL-001 / OPS-INC-002

現状、リリース後の異常は Prometheus アラート → Slack → 担当者の判断 → `kubectl rollout undo` というフローで、平均 20 分かかる。20 分間の P99 違反は QUA-SLO の月次エラーバジェット（43 分）の半分近くを消費する。

要件達成後の世界では、Argo Rollouts の `AnalysisTemplate` がカナリア昇格後にも 10 分間の post-deploy analysis を継続し、SLO 違反（5xx > 1% または P99 > 500ms が 2 分連続）を検知した瞬間に自動で `Abort` → 前 ReplicaSet への切り戻しが発火する。切り戻し完了までの目標時間は検知から 5 分以内（Abort 判定 2 分 + Rollout 戻し 3 分）。同時に Alertmanager 経由で PagerDuty に通知が飛び、on-call 当番が 5 分以内に事後確認を開始する。

崩れた時、検知と判断に 20 分を要しエラーバジェットが 1 回のリリースで半減する。月 3 回以上の違反で月次バジェットを食い潰し、翌月のデプロイ凍結トリガとなる。

**受け入れ基準**

- 全本番 Rollout に post-deploy analysis（10 分）が必須
- SLO 違反検知から Abort 発火までが 2 分以内、Rollout 完了までが 5 分以内（P95）
- 自動ロールバック発火時は Slack + PagerDuty 両方に通知
- 月次でロールバック発火件数と平均切り戻し時間を集計、5 分超は原因分析
- 「ロールバック不可能な変更」（DB スキーマ Contract 等）は OPS-REL-004 の Feature Flag 経由に強制

**検証方法**

- 四半期ごとに staging で意図的にバグ版を投入し、5 分以内ロールバックを訓練
- Grafana ダッシュボード `rollout-mttr` で切り戻し時間 P95 を週次レビュー

---

### OPS-REL-003: リリースノート自動生成と顧客通知

- 優先度: SHOULD（手動リリースノートは書き忘れが起き、tier2/3 開発者が変更に気付けない）
- Phase: Phase 1c（tier1 リリース開始時）
- 関連: OPS-CID-001（Conventional Commits）/ OPS-SUP-003

現状、リリースノートは担当者が手動で Confluence に記載しており、Phase 1a のテンポでは「書き忘れ」が月 2 回発生している。tier2/3 開発者は tier1 API の breaking change を GitHub Releases を見ないと把握できない。

要件達成後の世界では、`release-please` / `git-cliff` が Conventional Commits 形式のコミットメッセージからリリースノートを自動生成し、GitHub Releases に push する。`feat!:` や `BREAKING CHANGE:` タグは別セクションで強調され、API 契約変更は Backstage の Announcement 機能で tier2/3 ポータルに掲出、Slack `#k1s0-announce` チャンネルに BOT 投稿される。

崩れた時、tier2/3 開発者が breaking change を知らずに本番デプロイし、連鎖的な API 不整合が発生する。Phase 2 以降で稼働業務数が増えると、1 件の未告知 breaking change が 3-5 業務を同時停止させる。

**受け入れ基準**

- 全 tier1 サービスの GitHub Releases に自動生成リリースノートが 100%
- `BREAKING CHANGE:` 含むリリースは Backstage Announcement + Slack BOT 通知が 1 時間以内
- リリースノートには変更箇所の PR 番号 / 著者 / Breaking 有無 / 関連 ADR 番号が含まれる
- tier2/3 向け API に影響する変更は 2 週間前の事前告知が OPS-EOL-003 と連動
- 月次でリリースノート生成率（対 本番リリース件数）が 95% 以上

**検証方法**

- GitHub Releases API で自動生成率を週次集計
- tier2/3 開発者への四半期アンケートで「breaking change を事前に把握できたか」を調査

---

### OPS-REL-004: Feature Flag によるリリースと本番切替の分離

- 優先度: MUST（大きな機能は段階的公開が必要。flag なしでは「コード merge = 即本番公開」となりリスクが集中する）
- Phase: Phase 1c（最小 flag 構成）/ Phase 2 で全 tier 展開
- 関連: DEV-FM-001（OpenFeature + flagd）/ OPS-REL-001

現状、大規模な新機能（例: tier1 の PII 自動マスキング追加）は 1 PR で merge → 即本番公開となり、問題発覚時の切り戻しがコード revert に依存する。DB スキーマ変更を伴う場合は revert 不可能で、詰む。

要件達成後の世界では、影響範囲が大きい機能（tier1 の新 API / 新 ZEN ルール / 破壊的な tier2 UI 変更）は OpenFeature + flagd の Feature Flag でガードされ、コード merge = 本番配置だが、**機能の有効化はフラグ操作で別途実施**。フラグの切替は環境別（dev/stg/prod）かつテナント別・ユーザー別に可能で、本番での段階的公開（テナント 1% → 10% → 100%）と、問題検知時の即時オフ（30 秒以内）が実現する。フラグのライフサイクルは「追加 → 段階公開 → 100% → 30 日後に flag 削除」の 4 段階を Runbook `feature-flag-lifecycle.md` で管理する。

崩れた時、DB スキーマ変更を伴う機能で本番不具合が発覚した際、コード revert だけでは戻せず、DB のマイグレーション戻しが必要となり復旧に 4-8 時間を要する（Expand-Contract パターンが取れない）。

**受け入れ基準**

- 影響範囲「大」（複数テナント影響 or DB スキーマ変更 or API 契約変更）の全機能は Feature Flag 必須
- flagd のフラグ評価レイテンシ P99 が 10ms 以内
- フラグオフ（本番切替）操作から配信反映までが 30 秒以内
- 陳腐化フラグ（100% 公開から 30 日経過）は Backstage Scorecard で可視化、四半期レビューで削除
- フラグ操作は Audit Log で 365 日保管、操作者と理由が記録される

**検証方法**

- 四半期ごとに「影響範囲大の PR に flag が付いているか」を抜き打ち監査
- Grafana ダッシュボード `feature-flag-staleness` で陳腐化フラグ数を週次レビュー

---

### OPS-REL-005: リリース凍結判定とエラーバジェット連動

- 優先度: SHOULD（エラーバジェットを使い切った月に追加リリースを許すと SLA 違反リスクが倍増）
- Phase: Phase 2（稼働業務 2 本以上でエラーバジェット運用開始時）
- 関連: QUA-SLO-002 / OPS-OPR-003

現状、エラーバジェット残が枯渇寸前でも追加リリースが通過してしまい、月次 SLA 違反の直接原因となる。

要件達成後の世界では、SRE 当番が毎朝 9:00 の BOT 通知（OPS-OPR-003）でエラーバジェット残を確認し、**残 10% 以下** であれば当日のリリース凍結を SRE リード + プロダクトオーナーに持ち込み、同意が取れればリリース凍結ラベル `release-freeze` を GitOps リポに付与する。ラベル付与中は Argo CD の prod Application が sync-wave で停止し、緊急パッチのみ「インシデントコマンダー 1 Approve」経路で通過可能。

崩れた時、月末のエラーバジェット枯渇日にリリースを続けることで SLA 違反が顕在化し、tier3 顧客向けの SLA クレジット支払い（契約上の返金）が発生する。

**受け入れ基準**

- エラーバジェット残 10% 以下で BOT が自動的に凍結提案 Slack メッセージを `#sre-daily` に投稿
- 凍結ラベル付与 → Argo CD prod sync 停止までが 5 分以内
- 凍結解除は翌月バジェットリセット時に自動解除、または SRE リード + PO の書面承認
- 四半期の凍結発動回数と継続時間を経営レビューで報告
- 緊急パッチ経路が凍結中も動作することを四半期テスト

**検証方法**

- エラーバジェット残と凍結発動の時系列を Grafana `error-budget-freeze` で監視
- 年次で SLA 違反件数とリリース凍結発動率の相関を確認

---

## 章末サマリ

### ID 一覧

| ID | タイトル | 優先度 | Phase |
|---|---|---|---|
| OPS-REL-001 | カナリア既定（5/25/100） | MUST | 1c |
| OPS-REL-002 | 自動ロールバック 5 分以内 | MUST | 1c |
| OPS-REL-003 | リリースノート自動生成 | SHOULD | 1c |
| OPS-REL-004 | Feature Flag による切替分離 | MUST | 1c/2 |
| OPS-REL-005 | エラーバジェット連動凍結 | SHOULD | 2 |

### 優先度分布

| 優先度 | 件数 | 代表 ID |
|---|---|---|
| MUST | 3 | OPS-REL-001, 002, 004 |
| SHOULD | 2 | OPS-REL-003, 005 |

### Phase 達成度

| Phase | 必達件数 | 未達影響 |
|---|---|---|
| 1c | 3 | 本番稼働時の段階公開 / 自動ロールバック不可、SLO 違反全業務波及 |
| 2 | 5 | 稼働業務数拡大時のリリースリスク管理不足、SLA 違反頻発 |
