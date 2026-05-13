---
id: plan.security.scenario_threat_model_monthly_review
axis: security
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.security.security_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, C]
  proof_classes: []
---

# threat_model 625 cell 月次レビュー

## 一文方針

5⁴ = 625 cell の threat catalog を月次で差分確認し、新規 threat cell の登録・mitigation pointer の更新・unreachable marking の根拠再確認を行い、`threat_model.lock.yaml` と `mitigation_bindings.lock.yaml` の整合を CI で物理担保したままクローズする。

> 朝 9 時、本社 IT 室の security 担当者（シニア級）が月次レビューの calendar notification を受け、`threat_model.lock.yaml` の diff と前月 PR merge 一覧を照合し始める。Mattermost `#security-weekly` には dual reviewer の security 担当者 2 名がいる。手元の GitHub PR search で「新規 Kafka topic 追加に伴う surface 変化が未登録のまま CI warning が出ている」ことを確認し、generator script を起動する準備を整えている。

## ペルソナ要約

主役: security 担当者（シニア級）、目的: 625 cell の threat catalog を月次で差分確認し mitigation pointer を最新状態に維持する

## 現状業務での痛み

- threat catalog を Excel で管理しており、cell 更新漏れが規制当局の監査指摘になるまで気付かれない
- surface / asset の追加時に新規 cell の登録が属人的で、担当者によってカバレッジが異なる
- mitigation pointer の陳腐化が検知できず、参照先 artifact が削除されていても CI が fail しない

## k1s0 でこう変わる

- generator script が 625 cell の cross-product を自動展開し、cell 更新漏れが CI の coverage check で即時検知される
- 新規 surface / asset 追加時の cell 登録が必須化され、CI が未登録 cell を merge 前に拒否する
- mitigation_bindings.lock.yaml の xref check が CI で実行され、参照先 artifact の削除が即時検知される

## Trigger（発火条件）

月次 cycle の到来（毎月第 1 営業日）、または CI の threat_model coverage check が `explicit_unreachable` rate 上昇を検知した時。

## 想定頻度 / 典型きっかけ

想定頻度: 月次。典型きっかけ: 「新規 Kafka topic（tier2 の業界 pack 追加）が追加されたため、対応する `v1_north_south` / `v1_persistence` surface cell の mitigation pointer が未登録のまま CI warning が出ている」

## 主役 / 関与者

- 主役: security 担当者（シニア級）
- 関与: tier1 / tier2 担当者（新規機能の surface 変化の説明）
- 関与: formal 担当者（mitigation の proof_class bind 更新が必要な場合）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（security）| シニア | 本社 IT 室 | threat_model.lock.yaml diff / GitHub PR list | 新規 threat cell 登録・mitigation pointer 更新・unreachable 根拠確認・PR 起票 |
| 関与（tier1 / tier2）| シニア〜ミドル | 本社 IT 室 / リモート | GitHub PR list | 新規 surface 変化の説明・mitigation pointer 候補の提示 |
| 関与（formal）| シニア | 本社 IT 室 / リモート | proof_class bind dashboard | proof_class bind 更新要否の確認・sign-off |

## 個人 KPI / 達成感

- 全 625 cell の mitigation_class が null でないことを CI で定量確認でき、threat catalog 完全性の達成感を得られる
- explicit_unreachable rate が前月以下で推移することを数値で確認でき、threat 管理精度の向上を実感できる

## 工数 / 関与人数 / コスト感

- 工数: 半日〜1 日（diff 抽出 1h + cell 登録 2h + mitigation pointer 更新 2h + PR review 1h）
- 関与人数: 3〜4 名（security 担当者 2 名・tier1 / tier2 担当者・formal 担当者）
- コスト感: 低〜中。generator script が cell 生成を自動化するため手動作業が最小化される

## 前提

- `threat_model.lock.yaml` が build script の生成物として存在し、手書き drift は CI で物理拒否される状態
- `mitigation_bindings.lock.yaml` が存在し、mitigation pointer ↔ 物理 artifact の双方向 lock が CI 検証済み
- 前月の `last_green_at` が threat_model coverage check で green だった状態
- 対象月に追加・変更された 19 軸の PR 一覧を GitHub PR search で事前取得済み

## 流れ

1. 対象月の PR 差分から surface / asset の変化を抽出する（`git log --merges --since="first of last month"` で merge commit を列挙し、`threat_model.lock.yaml` に未反映な surface を特定）
2. 未登録の cell を `threat_model.lock.yaml` の generator script（`tools/lock_yaml_generator/threat_model_gen`）に input として追加する。generator は cross-product を展開し、各 cell の `mitigation_class` フィールドを未指定 (`null`) で生成する
3. 新規 cell ごとに mitigation_class（`v1_authn_authz` / `v1_crypto` / `v1_isolation` / `v1_admission_block` / `v1_audit_detect`）のいずれかを決定し、対応する物理 artifact の pointer を `mitigation_bindings.lock.yaml` に追記する
4. unreachable と判定する cell は `explicit_unreachable=true` フラグと `unreachable_reason` を必ず記入する。理由なしの unreachable は CI fail
5. `explicit_unreachable` count が前月比で増加している場合は必ず根拠文を security 担当者間で dual review し、count 増加を commit message に記録する
6. generator script を実行して `threat_model.lock.yaml` と `mitigation_bindings.lock.yaml` を再生成し、CI の coverage check（`tools/docs_lint` 相当）をローカルで pass させる
7. PR を作成し、security 担当者 2 名の dual reviewer sign-off を取得する（LLM 単独 sign-off 禁止）
8. `release_gate.lock.yaml` の threat_model cell = `coverage_green` を確認してクローズ

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | security 担当者 | 月次 cadence 到来を確認し threat_model.lock.yaml diff を前月 PR と照合 | `月次 threat review 開始 / diff 抽出完了` |
| 2h | security 担当者 | 未登録 cell を generator script で追加し mitigation_class を決定 | `新規 cell N 件登録 / mitigation_class 設定完了` |
| 4h | security 担当者 | generator 実行 / CI coverage check green を確認し PR 提出 | `CI green / PR #NNN 提出` |
| 1d | dual reviewer (security 2名) | 全 cell の mitigation pointer と unreachable 根拠を確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

全 9 業務に横断的に影響する（threat model は全業務の attack surface / asset を網羅するため）。特に影響度が高い 3 業務:

- **FA 生産指示**: 設備制御は `v1_authn_authz` の mitigation が必須。制御系 threat cell（surface=`v1_ot_south` / asset=`v1_control_command`）の漏れが直接的な設備誤動作 risk に繋がる。
- **警報配信**: 警報は availability / integrity が最重要であり、DoS / 改竄 threat cell の mitigation pointer が欠落すると配信停止・誤報の attack surface になる。
- **SCADA テレメトリ収集**: テレメトリは `v1_bulk_upload` surface における supply chain attack の入口になり得る。surface / asset の新規追加時に cell 漏れが生じやすい最重要領域。

## 関連適合仕様 / 関連 OSS

- 脅威モデル適合仕様: [../../../04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md](../../../04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md)
- security 強制機構: [../../../04_詳細設計/02_強制機構/06_security強制機構.md](../../../04_詳細設計/02_強制機構/06_security強制機構.md)
- 関連 OSS: tools/lock_yaml_generator / GitHub Actions CI

## 期待結果 / 観測指標

- `threat_model.lock.yaml` の全 625 cell が `mitigation_class != null` または `explicit_unreachable=true` のいずれかを持つ
- `mitigation_bindings.lock.yaml` の全 pointer が実在する artifact を参照し、CI の xref check が green
- `explicit_unreachable` rate が前月以下で推移
- security 担当者 2 名の sign-off が記録済み

## 失敗時の挙動 / escalation

- **generator script 実行失敗（新 surface が schema 外）**: `classes.yaml` の enum に surface / asset を追加してから再実行。schema 拡張 PR は security 担当者 + tier1 担当者の joint review が必要
- **mitigation pointer が実在しない artifact を参照**: 参照先 PR の merge status を確認し、未 merge ならば threat_model PR の merge を blocker 付き draft にする。参照先が削除済みなら pointer を更新して security 担当者 2 名に再 review 依頼
- **CI coverage check が red のまま push**: CI が fail した状態で merge を試みると `Required Status Check` がブロックする。root cause を特定して coverage check を green にしてから再試行。Mattermost `#security` channel に状況を即時報告（SLA: 4 時間以内）
- **`explicit_unreachable` rate が急増**: セキュリティ訓練方針の table_top（シナリオ 03）を前倒しで実施し、rate 増加の根拠を全員で合意してから commit する。postmortem は翌営業日まで

## 失敗パターン (anti-pattern)

- generator を使わない直接 YAML 編集: 手書き編集は CI の build-script 整合チェックが merge 阻止する
- unreachable_reason なしの unreachable 設定: 理由なしの unreachable は CI fail になる

## 関連参照

- [security 担当者シナリオ index](./README.md) — security 担当者シナリオ全体の構成と主要分類一覧
- [脅威モデル適合仕様](../../../04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md) — 625 cell の正典定義と mitigation pointer binding 規則
- [security 強制機構](../../../04_詳細設計/02_強制機構/06_security強制機構.md) — coverage check / cross-axis bind の物理 enforce 一覧
- [security 設計方針](../../../03_概要設計/07_security設計方針/01_脅威モデル方針.md) — 脅威モデルの設計思想と 5 enum 軸の定義
