---
id: plan.security.scenario_chaos_failure_drill
axis: security
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.security.security_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [C, D, E]
  proof_classes: []
---

# chaos_failure_drill / atomic 三表書込 P1-P4 検証

## 一文方針

月次 cycle の `v1_chaos_failure_drill` を security 視点で実施し、atomic 三表書込（P1-P4）の integrity / KEK shamir M-of-N threshold / audit hash chain continuity が failure injection 下でも defense-in-depth 層 D / E に物理 enforce されることを Litmus chaos experiment で検証し、SLO budget 消費が target 以内であることを `drill_progress.lock.yaml` に記録する。

> 朝 10 時半、infra 担当者が今月の Chaos drill（`v1_persistence` surface 向け network partition）を終えたとの通知が届く。security 担当者（シニア級）が ClickHouse の audit_event dashboard を開き、failure injection 中の hash chain continuity を確認する。`audit_ingest_gap_monitor` のアラートが静かなことを確認しながら、atomic 三表書込 P2 / P4 の ClickHouse 確認クエリを実行し始める。

## ペルソナ要約

主役: security 担当者（シニア級）、目的: chaos drill を SLO 評価と組み合わせ、security 視点での耐障害性を定量確認する

## 現状業務での痛み

- chaos drill 結果を観察するだけで SLO 評価がなく、security 観点での耐障害性が主観的な判断になる
- drill 後の security incident 検知能力の確認が不十分で、攻撃時の検知遅延が把握できない
- drill 結果が security チームと infra チームで共有されず、security 視点の改善が infra に伝わらない

## k1s0 でこう変わる

- chaos drill 中の security alert 発火率が Perses で計測され、検知能力の定量評価が可能になる
- drill 結果が security と infra で共有され、security 視点の改善 action が infra の drill 設計に反映される
- SLO 評価と security 検知評価の両方が必須化され、drill の品質が担保される

## Trigger（発火条件）

月次 cycle 到来時（毎月第 3 営業日）。infra 担当者が当月の Chaos drill（[infra-06](../04_infra担当者シナリオ/06_Chaos_drill実行.md)）を実施した後に security 担当者が cross-axis bind の観点で追加検証を行う。

## 想定頻度 / 典型きっかけ

想定頻度: 月次。典型きっかけ: 「今月の chaos は `v1_persistence` surface 向け。PostgreSQL primary の network partition を注入し、atomic 三表書込（state / outbox / audit table）が P2（Outbox 投入失敗で rollback）と P4（単一経路 emit）を物理で守れているか、audit_event が欠落しないかを確認する」

## 主役 / 関与者

- 主役: security 担当者（シニア級）
- 関与: infra 担当者（Litmus chaos experiment の実行、failover 操作）
- 関与: data 担当者（atomic 三表書込の integrity 確認、audit hash chain の継続確認）
- 関与: ops 担当者（SLO budget 消費の監視、escalation 対応）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（security）| シニア | 本社 IT 室 / リモート | ClickHouse audit dashboard / drill_progress.lock.yaml | hash chain continuity・atomic P2/P4・KEK threshold 保全確認 |
| 関与（infra）| シニア〜ミドル | 本社 IT 室 / リモート | Litmus dashboard | chaos experiment 実行・failover 操作 |
| 関与（data）| シニア〜ミドル | 本社 IT 室 / リモート | ClickHouse / pgaudit | atomic 三表書込 integrity 確認・audit hash chain 継続確認 |
| 関与（ops）| シニア〜ミドル | 本社 IT 室 / リモート | Prometheus SLO dashboard | SLO budget 消費監視・escalation 対応 |

## 個人 KPI / 達成感

- chaos drill 中の security alert 発火率を定量確認でき、検知能力改善の達成感を得られる
- drill 後の action item 消化率を追跡でき、security 耐障害性の継続的改善を実感できる

## 工数 / 関与人数 / コスト感

- 工数: 半日〜1 日（drill 準備 1h + 実施 2h + SLO / security 評価 2h + action 起票 1h）
- 関与人数: 3〜4 名（security 担当者・infra 担当者・ops 担当者）
- コスト感: 低〜中。Litmus と Perses で自動評価されるため手動コストが最小化される

## 前提

- infra-06（Chaos drill）が当月 green であり、`failover_drill.lock.yaml` が更新済み
- staging cluster で atomic 三表書込 P1-P4 の property test が CI で常時 green
- `drill_progress.lock.yaml` の前月 `v1_chaos_failure_drill.last_green_at` が有効期限内
- 09 観測 SoR（ClickHouse）で audit_event の ingest gap monitor が稼働中

## 流れ

1. Litmus chaos experiment を staging cluster で実行する（network partition / pod kill / CPU hog の 3 種から当月 rotation で選択）。実行コマンド: `litmus run --experiment chaos-drill-security-$(date +%Y%m) --namespace staging`
2. failure injection 中、以下の security invariant を並行監視する:
   - audit hash chain: pgaudit の audit_event が連続して chain を維持しているか（`audit_ingest_gap_monitor` のアラートが出ていないか）
   - atomic 三表書込 P2: Outbox 投入失敗が発生した場合に state table への write がそれと同一 transaction で rollback されているか
   - atomic 三表書込 P4: Domain Event が Outbox 以外の経路から emit されていないか（ClickHouse audit クエリで単一経路 emit を確認）
3. failure injection が終了したら、SLO budget 消費（Prometheus error_budget 消費率）を記録し、target（`v1_chaos_failure_drill` の success criteria: SLO budget 消費 ≤ target）と比較する
4. audit hash chain の continuity を WORM Object Lock アーカイブと照合し、failure injection 中に欠落した audit_event がないことを確認する
5. KEK shamir M-of-N threshold の保全確認: failure injection 中に shamir share を持つ node が閾値以上 down した場合、share reconstruction が物理不可能であること（OpenBao の threshold guard が正しく動作すること）を確認する
6. 検証結果を `drill_progress.lock.yaml` に Tekton job 経由で記録する。success criteria に反する項目が 1 つでもあれば `red` 判定
7. `red` 判定の場合は即 postmortem PR を起票し、release_gate.lock.yaml の `drill_overdue_count` が cap を超えないよう security 担当者 + tech lead で root cause を特定する

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | security 担当者 + infra 担当者 | chaos drill を開始し security alert 監視を Perses で開始 | `chaos drill 開始 / security alert 監視開始` |
| 30分 | security 担当者 | drill 中の security alert 発火数と検知時間を記録 | `alert 発火数 N 件 / 検知時間 平均 N 秒` |
| 1h | security 担当者 | SLO 評価と security 検知評価を chaos_drill.lock.yaml に記録 | `drill 完了 / lock.yaml 更新` |
| 1d | dual reviewer | drill 結果と security 評価を確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

chaos drill は SLO / 可用性・データ integrity に直結する業務に対して最も重大な影響を持つ。特に影響度が高い 3 業務:

- **ライン稼働監視 live tile**: 可用性 SLO に直結する監視系が failure injection 下でも正常に動作するか（audit_detect が欠落しないか）を chaos drill で毎月確認する最重要業務。
- **警報配信**: 警報配信の availability / integrity は failure injection 下でも SLO budget 内に収める必要があり、chaos drill で atomic 三表書込の保全と warning emission を確認する。
- **計量装置連続データ**: 計量データの business_data としての改竄検知は audit hash chain continuity に依存しており、chaos drill での hash chain 保全確認は計量データの信頼性保証の根拠になる。

## 関連適合仕様 / 関連 OSS

- 脅威モデル適合仕様: [../../../04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md](../../../04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md)
- データ保全適合仕様: [../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)
- 関連 OSS: Litmus / Prometheus / ClickHouse / CloudNativePG / OpenBao / Tekton

## 期待結果 / 観測指標

- SLO budget 消費が target 以内（Prometheus error_budget query で確認）
- audit hash chain が failure injection 中・後ともに連続（欠落 event ゼロ）
- atomic 三表書込 P2 / P4 が failure injection 下でも保全（ClickHouse audit クエリで確認）
- KEK shamir threshold guard が正しく機能（reconstruction 不可能ケースで OpenBao が revoke）
- `drill_progress.lock.yaml` が `v1_chaos_failure_drill.last_green_at` を当日日付で記録済み

## 失敗時の挙動 / escalation

- **audit hash chain に欠落を検出**: 即時シナリオ 09（audit hash chain 改竄検知）を発火。chaos drill を中断し、欠落経路の root cause を特定する（SLA: 4 時間以内）
- **atomic 三表書込 P4 が破れた（Outbox 以外の経路から emit）**: data 担当者 + tier2 担当者を Mattermost `#data-incident` に招集（SLA: 15 分）。発火経路を特定して tier2 強制機構の Domain Event emit 必須 check を修正する。postmortem は 2 営業日以内
- **SLO budget 消費が target 超過**: SLO error_budget が freeze 状態になる（feature merge stop）。ops 担当者が infra rollback または scale を実施し、budget 復帰を確認してから drill 結果を `red` で記録

## 失敗パターン (anti-pattern)

- SLO 評価のみの drill: security 検知評価なしの drill は security 視点の耐障害性が確認できない
- lock.yaml 更新なし: drill 結果を記録しないと security 改善の追跡が失われる

## 関連参照

- [security 担当者シナリオ index](./README.md) — security 担当者シナリオ全体の構成と主要分類一覧
- [audit_ingest_gap_monitor](../../../04_詳細設計/03_クロスカッティング適合仕様/09_audit_ingest_gap_monitor.md) — hash chain + ingest gap heartbeat の二重防御
- [data 担当者シナリオ: Outbox atomic 三表書込障害対応](../05_data担当者シナリオ/08_Outbox_atomic三表書込障害対応.md) — data 視点の atomic 三表書込障害対応
- [infra 担当者シナリオ: Chaos drill 実行](../04_infra担当者シナリオ/06_Chaos_drill実行.md) — infra 側の chaos drill 実行手順
