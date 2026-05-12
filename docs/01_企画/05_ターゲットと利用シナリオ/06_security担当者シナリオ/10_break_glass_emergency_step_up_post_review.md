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

## Trigger（発火条件）

`v1_emergency_step_up` が実行され audit_event の `authn_fact.class=v1_emergency_step_up` が 09 観測 SoR に着信した時（break-glass 実行は ops 担当者主導、事後 audit レビューは security 担当者主導）。

## 想定頻度 / 典型きっかけ

想定頻度: イベント駆動（年間 1-5 件、DR や重大インシデント対応時のみ）。典型きっかけ: 「本番 PostgreSQL primary が unreachable になり、ops 担当者が break-glass で本番 DB cluster-admin を取得した。audit_event が Mattermost `#security-incident` に自動 forwarding されてきた」

## 主役 / 関与者

- 主役: security 担当者（シニア級、事後 audit レビューリード）
- 協調: ops 担当者（break-glass 実行者として事後説明の義務あり）
- 立会（任意）: formal 担当者（proof_class bind が期待通りに強制 audit emit として機能したか観察）

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

## 関連参照

- [security 担当者シナリオ index](./README.md) — security 担当者シナリオ全体の構成と主要分類一覧
- [security 設計方針: インシデント対応方針](../../../03_概要設計/07_security設計方針/07_インシデント対応方針.md) — `v1_privilege_escalation` class の contain action
- [認証適合仕様](../../../04_詳細設計/01_適合仕様/04_認証適合仕様.md) — `v1_emergency_step_up` auth class と強制 audit emit の structural spec
- [security 担当者シナリオ: audit hash chain 改竄検知](./09_audit_hash_chain改竄検知_外部公証.md) — hash chain 整合確認の詳細手順
