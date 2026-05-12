---
id: plan.security.scenario_break_glass_post_review
axis: security
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.security.security_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [D, E]
  proof_classes: []
---

# break-glass emergency_step_up 事後レビュー

## 一文方針

`v1_emergency_step_up` が実行された audit_event を security 担当者が受け取ったタイミングで、OpenBao audit log + audit hash chain の一貫性検証 → break-glass 正当性判定 → postmortem PR 起票の事後レビューを ops 担当者との二者協調で実施し、不正 break-glass の場合は即時 `v1_privilege_escalation` incident class を発火する。

> 朝 8 時、Mattermost `#security-incident` に `authn_fact.class=v1_emergency_step_up` の audit_event 自動 forwarding が届く。security 担当者（シニア級）が ClickHouse に接続し、本番 PostgreSQL primary unreachable に伴う ops 担当者の break-glass session を時系列で確認し始める。OpenBao audit log との突合を行いながら、実行されたアクションが IR playbook の承認済み action と一致するかを 24 時間 SLA 内で判定する。

## ペルソナ要約

主役: security 担当者（シニア級）、目的: break-glass 緊急昇格後の post-review を形骸化させず action item を確定させる

## 現状業務での痛み

- break-glass 後の post-review が形骸化し、action item が出ないまま次の break-glass に同じ問題が繰り返される
- break-glass 操作の audit trail が不完全で、何をいつ操作したかの追跡が困難
- post-review の実施期限が管理されず、review が長期間未実施になる

## k1s0 でこう変わる

- break_glass.lock.yaml が操作の完全な audit trail を記録し、post-review の起点となる証跡が自動生成される
- post-review が break-glass 後 24h 以内の必須イベントとして CI で強制され、実施漏れが ship blocker になる
- action item が Backstage ticket で追跡され、改善サイクルが定量的に管理される

## Trigger（発火条件）

`v1_emergency_step_up` が実行され audit_event の `authn_fact.class=v1_emergency_step_up` が 09 観測 SoR に着信した時（break-glass 実行は ops 担当者主導、事後 audit レビューは security 担当者主導）。

## 想定頻度 / 典型きっかけ

想定頻度: イベント駆動（年間 1-5 件、DR や重大インシデント対応時のみ）。典型きっかけ: 「本番 PostgreSQL primary が unreachable になり、ops 担当者が break-glass で本番 DB cluster-admin を取得した。audit_event が Mattermost `#security-incident` に自動 forwarding されてきた」

## 主役 / 関与者

- 主役: security 担当者（シニア級、事後 audit レビューリード）
- 協調: ops 担当者（break-glass 実行者として事後説明の義務あり）
- 立会（任意）: formal 担当者（proof_class bind が期待通りに強制 audit emit として機能したか観察）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（security）| シニア | 本社 IT 室 / リモート | Mattermost #security-incident / break_glass_review.lock.yaml | audit_event 取得・hash chain 整合確認・正当性判定・dual sign-off |
| 協調（ops）| シニア〜ミドル | 本社 IT 室 / リモート | OpenBao audit log / IR incident ticket | break-glass 実行理由の説明・実行アクションの一覧提出 |
| 立会（formal）| シニア〜ミドル | 本社 IT 室 / リモート | proof_class bind dashboard | 強制 audit emit の物理機能確認・観察記録 |

## 個人 KPI / 達成感

- post-review 24h 以内実施率を定量確認でき、break-glass ガバナンスの達成感を得られる
- post-review action item 消化率を追跡でき、緊急対応品質の継続的改善を実感できる

## 工数 / 関与人数 / コスト感

- 工数: 1〜2h（audit trail 確認 30min + post-review 1h + action item 起票 30min）
- 関与人数: 3〜4 名（security 担当者・操作実施者・dual reviewer）
- コスト感: 低。audit trail が自動記録されるため post-review の準備コストが最小化される

## 前提

- break-glass 実行時に「強制 audit emit」が物理的に emit されている（実行者が emit を skip する経路は存在しない）
- OpenBao で発行された emergency token の audit log が OpenBao audit backend（file + syslog）に記録済み
- `escalation_policy.lock.yaml` に break-glass 実行後のセキュリティ審査 = security 担当者主導が明示済み
- 審査は break-glass 実行後 24 時間以内に開始する

## 流れ

1. **audit_event 取得**: ClickHouse で `SELECT * FROM audit_events WHERE fact_class='v1_emergency_step_up' AND event_id >= '<break-glass-time>'` を実行し、break-glass session の全 action を時系列で取得する
2. **hash chain 整合確認**: 対象 event_id 区間の hash chain が連続していること（シナリオ 09 の手順で確認）を確認する。break-glass 中に chain が diverge していた場合はシナリオ 09 を並行発火する
3. **OpenBao audit log 照合**: OpenBao audit backend の log（`/var/log/openbao/audit.log`）と ClickHouse の audit_event を突合し、emergency token で実行された全 API call が一致していることを確認する
4. **正当性判定**:
   - ops 担当者から実行理由（incident ticket + IR playbook の step 番号）を受け取る
   - 実行したアクション（`kubectl get`, `pg_dump`, `vault kv get` 等）が IR playbook の承認済み action と一致しているか確認する
   - break-glass session の duration（≤ IR playbook で指定された最大時間）を確認する
5. **不正 break-glass の場合**: `v1_privilege_escalation` incident class を発火する（IR commander = security 担当者）。該当 identity の token / SSH / DB password を OpenBao で即時 revoke し、cluster RBAC を Argo CD force sync で正常状態に戻す
6. **正当 break-glass の場合**: 正当性を security 担当者が確認し、postmortem PR（break-glass を必要とした incident の根本対策）に security 観点のコメントを付与して sign-off する
7. dual cosign sign-off（ops 担当者 + security 担当者）で事後レビュー record を `break_glass_review.lock.yaml` に記録する（手書き禁止、Tekton job が更新）

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | security 担当者 | break_glass.lock.yaml で操作 audit trail を確認し post-review を開始 | `break-glass post-review 開始 / audit trail 確認` |
| 30分 | security 担当者 + 操作実施者 | 操作の正当性と最小権限の遵守を確認 | `正当性確認完了 / 最小権限確認` |
| 1h | security 担当者 | post-review 結果と action item を lock.yaml に記録 | `post-review 完了 / action item N 件起票` |
| 24h | dual reviewer | post-review 記録と action item を確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

break-glass は緊急時の業務継続手段であり、特に DB / 制御系の障害時に発生する可能性が高い業務:

- **FA 生産指示**: 設備制御系の DB が unreachable になった場合に break-glass が発動しやすく、緊急権限行使後の security 担当者による事後レビューは FA 業務継続の正当性確認として機能する。
- **品質検査結果配信**: 検査 DB への break-glass は検査データへの不正アクセス経路になりうるため、審査では RLS bypass が行われていないかを ClickHouse audit query で特定する。
- **受注**: 受注データベースへの break-glass は business_data の整合性 audit として最高優先度であり、不正 break-glass の場合は 受注 DB の全 PII / 取引データを影響範囲として扱う。

## 関連適合仕様 / 関連 OSS

- 認証適合仕様: [../../../04_詳細設計/01_適合仕様/04_認証適合仕様.md](../../../04_詳細設計/01_適合仕様/04_認証適合仕様.md)
- security 強制機構: [../../../04_詳細設計/02_強制機構/06_security強制機構.md](../../../04_詳細設計/02_強制機構/06_security強制機構.md)
- 関連 OSS: OpenBao / ClickHouse / Argo CD / Cosign / Tekton

## 期待結果 / 観測指標

- audit hash chain が break-glass session を通じて連続（欠落なし）
- OpenBao audit log と ClickHouse audit_event が全 API call で一致
- 正当 / 不正の判定が dual sign-off で `break_glass_review.lock.yaml` に記録済み
- 不正の場合は `v1_privilege_escalation` 発火が即時実施済み
- postmortem PR が merge 済みで action item が GitHub Issue 化済み

## 失敗時の挙動 / escalation

- **audit_event が存在しない（emit が skip された）**: 強制 audit emit は物理経路上で skip 不可能なため、audit_event がない場合は emit 機構の故障として扱う。シナリオ 09 を発火し、ingest 経路の gap を特定する（SLA: 2 時間）
- **OpenBao audit log と ClickHouse が一致しない（API call の欠落）**: 改竄を疑い `v1_data_tampering` incident class を発火。infra 担当者と共同で forensic を実施し、L3 escalation（tech lead + 法務）を起動する（SLA: 30 分）
- **事後レビューが 24 時間を超えて開始できない（security 担当者不在）**: `escalation_policy.lock.yaml` に定義されたバックアップ security 担当者（ops 担当者が代行可）が 24 時間 cap で開始する。24 時間超過は ship blocker として `release_gate.lock.yaml` に記録される

## 失敗パターン (anti-pattern)

- post-review の省略: break-glass 後に post-review なしでは CI が 24h 以内の必須チェックを ship blocker として検知する
- action item なしの post-review: 形骸化した post-review は CI が 0-action-item として警告する

## 関連参照

- [security 担当者シナリオ index](./README.md) — security 担当者シナリオ全体の構成と主要分類一覧
- [security 設計方針: インシデント対応方針](../../../03_概要設計/07_security設計方針/07_インシデント対応方針.md) — `v1_privilege_escalation` class の contain action
- [認証適合仕様](../../../04_詳細設計/01_適合仕様/04_認証適合仕様.md) — `v1_emergency_step_up` auth class と強制 audit emit の structural spec
- [security 担当者シナリオ: audit hash chain 改竄検知](./09_audit_hash_chain改竄検知_外部公証.md) — hash chain 整合確認の詳細手順
