---
id: plan.infra.scenario_clock_integrity_change
axis: infra
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.infra.infra_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [C, D, E]
  proof_classes: []
---

# clock integrity 変更

## 一文方針

データセンター / クラウドプロバイダの PTP 対応変更や leap second 戦略見直しに際して、[5 clock_integrity_class](../../../04_詳細設計/01_適合仕様/13_時刻整合適合仕様.md) すべてで HLC skew allowance と leap second smear 設定の整合性を保ちながら安全に変更を適用する。

> 朝 9 時、本社 IT 室の infra 担当者（シニア級）が Perses dashboard の HLC skew メトリクスを確認中に、新データセンターへの移行通知メールで「PTP Grand Master 切り替え予定日 T+7 日」に気付く。手元には clock_integrity.lock.yaml と chronyd 設定ファイル、Mattermost 越しに dual reviewer がいる。

## ペルソナ要約

主役: infra 担当者（シニア級）、目的: HLC clock drift を定期確認し data 整合性破壊を事前に防ぐ

## 現状業務での痛み

- HLC drift を検知する仕組みがなく、時刻ずれによるデータ整合性破壊が本番障害まで気付かれない
- NTP / PTP の設定変更が手動管理で、設定 drift が長期間放置される
- clock drift の影響範囲が不明確で、障害発生後の原因特定に時間がかかる

## k1s0 でこう変わる

- Perses の HLC drift メトリクスが閾値を超えた時点で自動 alert が発行され、データ破壊前に対処できる
- NTP / PTP 設定が GitOps で管理され、設定変更が PR 経由で審査される
- clock drift の影響 namespace が Perses dashboard で即時特定でき、障害対応時間が短縮される

## Trigger（発火条件）

- データセンター / クラウドプロバイダの PTP 対応変更が通知された時
- leap second 戦略の見直しが必要になった時

## 想定頻度 / 典型きっかけ

想定頻度: 不定期（DC 変更時 / クラウドプロバイダ PTP 対応変更時、年間 0-2 件）。典型きっかけ: 「新データセンターへの移行で PTP Grand Master が変わり、全 node の chronyd 設定を更新する必要が生じた」

## 主役 / 関与者

- 主役: infra 担当者（シニア級）
- 関与: dual reviewer

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（infra）| シニア | 本社 IT 室 | Perses dashboard（HLC skew メトリクス）| PTP 設定変更 / chronyd 設定更新 / integration test 実施 |
| 承認（dual reviewer）| シニア | 本社 IT 室 / リモート | Mattermost `#infra-ops` | IaC PR レビュー / clock_integrity.lock.yaml sign-off |

## 個人 KPI / 達成感

- HLC drift ≤ 閾値の SLO 達成率を Perses で定量確認でき、clock integrity 維持の達成感を得られる
- alert から対処完了までの MTTR を計測でき、対応速度の改善を数値で確認できる

## 工数 / 関与人数 / コスト感

- 工数: 1〜2 日（メトリクス設定 4h + alert ルール設定 2h + GitOps 設定 4h）
- 関与人数: 2〜3 名（infra 担当者・data 担当者・dual reviewer）
- コスト感: 低。Perses が自動計測するため継続監視コストは最小化される

## 前提

- 5 clock_integrity_class（PTP / NTP / HLC / leap second 戦略）が `clock_integrity.lock.yaml` に記録済みであること

## 流れ

1. 変更対象の clock_integrity_class を特定（例: v1_zone_replicated → PTP 精度要件の変更）
2. PTP（IEEE 1588）or NTP tier の設定を OpenTofu IaC で更新
3. k8s node の chronyd / ptpd 設定を Kustomize overlay で変更
4. HLC（Hybrid Logical Clock）の skew allowance が新設定で維持されることを integration test で確認
5. leap second 処理: smear 設定が全 node で統一されていることを lint で確認（混在は禁止）
6. 変更後に `clock_integrity.lock.yaml` を更新し dual reviewer sign-off

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | infra 担当者 | Perses で HLC drift メトリクスの baseline を確認し alert ルールを設定 | `HLC drift 監視設定完了 / baseline 確認` |
| 4h | infra 担当者 | NTP / PTP 設定を GitOps manifest に移行し PR を作成 | `NTP/PTP GitOps 移行完了 / PR #NNN 提出` |
| 1d | infra 担当者 + data 担当者 | HLC drift alert の動作確認と data 整合性チェックを実施 | `alert 動作確認 / data 整合性 OK` |
| 1d+2h | dual reviewer | GitOps 設定と alert ルールを確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（infra は全業務の k8s cluster / network / storage の基盤を担うため）。特に影響度が高い 2 業務:

- **SCADA（計測値収集）**: センサー計測値のタイムスタンプ精度が PTP 変更で劣化すると計測データの信頼性が失われるため、変更後の HLC skew が計測 SLA 以内に収まることを最優先で確認する。
- **品質検査**: 品質記録の時刻整合性は法的要件に直結するため、clock 変更後の全ノード smear 統一を lint で証明してから本番適用する。

## 関連適合仕様 / 関連 OSS

- 時刻整合適合仕様: [../../../04_詳細設計/01_適合仕様/13_時刻整合適合仕様.md](../../../04_詳細設計/01_適合仕様/13_時刻整合適合仕様.md)
- 関連 OSS: OpenTofu / Kustomize / chronyd / ptpd

## 期待結果 / 観測指標

- 変更後も HLC skew allowance が仕様値以内に収まること
- 全 node で leap second smear 設定が統一されていること
- `clock_integrity.lock.yaml` が新設定で更新されていること
- dual reviewer sign-off が記録されていること

## 失敗時の挙動 / escalation

- HLC skew allowance 超過 → 変更 rollback + 原因究明 + PR 差し戻し
- leap second smear 混在検出 → lint fail で apply 中止 + 全 node 設定の統一修正
- integration test fail → `clock_integrity.lock.yaml` 更新せず + **postmortem 期限: 2 営業日以内**（Backstage runbook `clock-integrity-postmortem`）

**escalate 先**: ops 担当者 / Mattermost `#infra-incident`
**SLA: 30 分以内（拘束）**。超過した場合は ops 担当者に Mattermost `#infra-incident` で即時報告
**runbook**: Backstage runbook `clock-integrity-rollback` を参照

## 失敗パターン (anti-pattern)

- 手動 NTP 設定: GitOps を迂回した手動 chronyc 設定は drift detection CI が設定不整合を検知する
- alert 未設定: drift メトリクスに alert なしで運用すると SLO violation が事後に発覚する

## 関連参照

- [README.md](README.md) — infra 担当者シナリオ index
- [../../../03_概要設計/05_infra設計方針/README.md](../../../03_概要設計/05_infra設計方針/README.md) — infra 設計方針（clock integrity 変更の方針・背景を確認するため）
