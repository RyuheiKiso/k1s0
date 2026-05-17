---
id: plan.infra.scenario_canary_progressive_rollback
axis: infra
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.infra.infra_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, C, D]
  proof_classes: []
---

# Canary / Progressive Delivery 観察と切戻し

## 一文方針

infra 担当者が Argo Rollouts による canary progressive delivery の SLI 観察を実施し、エラーレートまたはレイテンシ SLI が閾値を超えた場合に自動ロールバックまたは手動 `abort` を発動して旧バージョンに戻し、全手順を GitOps commit として記録する。

> 火曜昼 14 時、infra 担当者（シニア級）が Perses の Argo Rollouts ダッシュボードを定期確認していると、受注サービス `v1.3.2` の canary weight が 20% に達した時点でエラーレートが 0.3%→1.2% に跳ね上がっているのに気付く。自動分析が `fail` を返す前に、Mattermost `#infra-rollout` で tier2 担当者にトラフィック分析を依頼しながら手動 abort のスタンバイに入る。

## ペルソナ要約

主役: infra 担当者（シニア級）、目的: カナリア切り戻しを Argo Rollouts の自動・手動ゲートで安全かつ迅速に実施する

## 現状業務での痛み

- カナリア切り戻し手順が不明確で、エラー率上昇時に手順を探す時間がインシデント影響を拡大させる
- 手動切り戻しのタイミング判断が担当者の主観に依存し、切り戻し遅延が SLO 違反を引き起こす
- 切り戻し後の原因分析が属人的で、同じ問題が繰り返し発生する

## k1s0 でこう変わる

- Argo Rollouts の analysis run がエラー率を自動評価し、閾値超過で即時自動切り戻しを実施する
- 切り戻し手順が Backstage runbook に定義され、誰でも同一手順で緊急切り戻しを実行できる
- 切り戻し後の postmortem が Backstage ticket で追跡され、再発防止 action item が管理される

## Trigger（発火条件）

Argo Rollouts の canary analysis が fail を返した時（自動 rollback トリガー）、または infra 担当者が Perses dashboard で SLI 劣化を検出し手動 abort を判断した時。

## 想定頻度 / 典型きっかけ

想定頻度: デプロイ頻度に比例（週次〜月次のサービス更新のたびに canary が走る。切戻しが必要になるのは月次〜四半期に 1 回程度）。典型きっかけ:「受注サービスの新バージョンを canary weight 20% で配信した直後にエラーレートが 1% を超え、Argo Rollouts の AnalysisRun が fail し自動 rollback が発動した。infra 担当者は rollback の完全性を確認し tier2 担当者に原因調査を依頼した」

## 主役 / 関与者

- 主役: infra 担当者（シニア級、GitOps / Argo Rollouts オーナー）
- 関与: ops 担当者（SLI アラート検知 / on-call 通報）
- 関与: tier2 担当者（新バージョンの変更内容の説明 / 原因調査）
- 承認: dual reviewer（rollback 完了の sign-off）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（infra）| シニア | 本社 IT 室 / 自宅 on-call | Perses の Argo Rollouts ダッシュボード | SLI 観察 / abort 発動 / rollback 確認 |
| 関与（ops）| シニア | 本社 / リモート | Mattermost `#infra-rollout` + PagerDuty | アラート通報 / incident 管理 |
| 関与（tier2）| 中堅 | 本社 IT 室 | GitHub PR / Backstage Docs | 新バージョンの変更点説明 / 原因調査 |

## 個人 KPI / 達成感

- mean time to rollback が閾値以内であることを Perses で定量確認でき、インシデント対応品質の達成感を得られる
- 自動切り戻し成功率の向上を数値で確認でき、リリース安全性の改善を実感できる

## 工数 / 関与人数 / コスト感

- 工数: 1〜2 日（Argo Rollouts 切り戻し設定 4h + runbook 整備 2h + analysis run テスト 4h）
- 関与人数: 3 名（infra 担当者・ops 担当者・dual reviewer）
- コスト感: 低〜中。設定後は自動化で継続コストが最小化される

## 前提

- `Rollout` CRD と `AnalysisTemplate` が GitOps 管理下で存在する（`infra/gitops/rollouts/<service>.yaml`）
- Perses に Argo Rollouts の canary weight / error rate / P99 latency パネルが設定済み
- canary analysis の success / fail 閾値（例: error rate < 0.5%、P99 < 500ms）が `AnalysisTemplate` に定義済み
- `Rollout` の `strategy.canary.steps` が段階的（weight 10% → 20% → 50% → 100%）に設定されている

## 流れ

1. canary デプロイ開始を確認する（Argo CD が新バージョンを deploy し、`Rollout` が canary phase に入ったことを Argo CD UI で確認）
2. Perses dashboard で以下の SLI を観察する（canary weight が上がるたびに確認）:
   - error rate（canary 版 vs stable 版を比較）
   - P99 / P95 latency
   - Kafka consumer lag（canary が consumer の場合）
3. SLI が閾値内であれば `kubectl argo rollouts promote <rollout-name>` で次の weight step に進める
4. **自動 rollback トリガー**: `AnalysisRun` が `fail` を返した場合は Argo Rollouts が自動で rollback を開始する
   - rollback 完了を Argo CD UI で確認する（`Rollout` の `status.currentStableHash` が旧バージョンに戻っていること）
   - 影響を受けたリクエスト数を Perses の `error-burst` パネルで確認する
5. **手動 abort**: infra 担当者が SLI 劣化を検出した場合は `kubectl argo rollouts abort <rollout-name>` を実行する
6. rollback 後の確認:
   - `Rollout` が `Degraded` status から `Healthy` に戻ること
   - stable 版のエラーレートが元の閾値内に戻ること（Perses で 10 分観察）
7. tier2 担当者に Mattermost `#infra-rollout` で rollback 完了と原因調査依頼を通知する
   - 通知例: `@tier2-oncall 受注サービス v1.3.2 canary rollback 完了。error rate 1.2% @weight20%。新バージョンの変更内容を確認して修正 PR を提出してください`
8. Argo CD の GitOps commit 履歴で rollback 操作が記録されていることを確認する
9. dual reviewer sign-off を取得する

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | infra 担当者 | Argo Rollouts analysis run でエラー率監視を設定し staging でテスト | `analysis run 設定完了 / staging テスト実行` |
| 5分 | Argo Rollouts | エラー率閾値超過を検知し自動切り戻しを開始 | `自動切り戻し開始 / エラー率 N% 検知` |
| 10分 | infra 担当者 | 切り戻し完了を Perses で確認し postmortem ticket を起票 | `切り戻し完了 / postmortem ticket #NNN 起票` |
| 1営業日 | infra 担当者 | postmortem で root cause と action item を確定し lock.yaml を更新 | `postmortem 完了 / action item N 件` |

## 業界 9 業務との紐付け

- **受注管理**: 受注サービスの新バージョン canary 配信で最もリスクが高い業務。エラーレート上昇が注文受付の停止に直結するため、canary weight の段階移行は受注数の低い時間帯（例: 深夜〜早朝）に実施することが推奨される
- **警報配信**: 警報配信サービスの canary 配信で失敗した場合、工場の警報通知が停止する可能性がある。警報配信は P99 latency の SLI 閾値を他サービスより厳しく設定する（例: P99 < 200ms）
- **設備リモート操作（FA）**: bidi 双方向ストリームを使う設備 remote 操作サービスの canary は connection continuity の SLI を別途 AnalysisTemplate に定義する

## 関連適合仕様 / 関連 OSS

- クラスタ位相適合仕様: [../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md](../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md)
- infra 強制機構: [../../../04_詳細設計/02_強制機構/04_infra強制機構.md](../../../04_詳細設計/02_強制機構/04_infra強制機構.md)
- 関連 OSS: Argo Rollouts（canary / blue-green）/ Argo CD（GitOps）/ Perses（SLI 観察）/ Istio（traffic split）

## 期待結果 / 観測指標

- runtime: rollback 後 10 分で stable 版エラーレートが閾値（0.5%）以下に回帰（Perses `error-rate` パネル）
- runtime: `Rollout` が `Healthy` status に戻っていること（Argo Rollouts UI / `kubectl argo rollouts status`）
- gitops: Argo CD の commit 履歴に rollback 操作が記録済み
- notify: tier2 担当者への Mattermost 通知が完了
- sign-off: dual reviewer 2 名 sign-off 完了

## 失敗時の挙動 / escalation

- **自動 rollback が 5 分以内に完了しない**: Argo Rollouts の `Rollout` CRD の状態を確認し、stuck している場合は `kubectl argo rollouts undo <rollout-name>` で強制ロールバック（**SLA: 5 分以内**）。ops 担当者に Mattermost `#infra-incident` で通報
- **rollback 後も stable 版のエラーレートが高い**: stable 版の問題の可能性があるため ops 担当者と共に前バージョンまで rollback するか判断（**SLA: 15 分以内**）。P1 incident として扱う
- **AnalysisRun fail の原因が不明**: tier2 担当者に 48h 以内の原因調査 PR を依頼（**SLA: 48h 以内**）。分析結果を Backstage TechDocs に postmortem として記録する

## 失敗パターン (anti-pattern)

- analysis run なしの手動監視: エラー率監視を人手で行うと切り戻しタイミングが遅延し SLO 違反になる
- postmortem の省略: 切り戻し後の分析なしでは同一問題が再発する

## 関連参照

- [infra 担当者シナリオ index](./README.md) — infra 担当者シナリオ全体の構成
- [GitOps 配信管理（infra-05）](./05_GitOps配信_progressive_delivery.md) — 通常の GitOps 配信フロー（canary 開始前の前提手順）
- [Chaos drill（infra-06）](./06_Chaos_drill実行.md) — canary rollback と類似した復旧確認手順
- [クラスタ位相適合仕様](../../../04_詳細設計/01_適合仕様/12_クラスタ位相適合仕様.md) — topology_class と traffic split の設計根拠
