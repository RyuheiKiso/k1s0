---
id: plan.security.scenario_data_breach_privacy_incident_response
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

# data breach / privacy incident response

## 一文方針

PII 漏洩疑いまたは breach notification 要件（規制 / 法律上の通知義務）が発火した時に、`v1_data_exfiltration` incident class の 6 phase playbook を最優先で発火し、L3 escalation（tech lead + 法務 / DPO）+ 影響テナントへの通知 + WORM アーカイブによる証跡保全 + postmortem PR merge まで、規制通知期限（GDPR 72 h / 個人情報保護法 30 日 etc.）以内に完結させる。

## Trigger（発火条件）

シナリオ 09（audit hash chain 改竄検知）で `v1_pii` asset が改竄経路に含まれていた時、または Falco / Tetragon が PII table への大量 SELECT / COPY を `v1_read` capability として検知した時、または外部報告（影響ユーザーからの申告 / セキュリティ研究者からの報告）があった時。

## 想定頻度 / 典型きっかけ

想定頻度: 非計画的（年間 0-1 件）。典型きっかけ: 「Tetragon eBPF probe が本番 PostgreSQL で `SELECT * FROM pii_basic WHERE tenant_id = '*'`（全テナント横断の意図的 query）を検知。`v1_external_auth` actor が RLS bypass を試みた可能性があり、audit hash chain に divergence が確認された」

## 主役 / 関与者

- 主役: security 担当者（シニア級、IR commander = `v1_data_exfiltration` class の自動 assign）
- 必須: L3 escalation（tech lead + 法務 / DPO）を即時起動
- 関与: data 担当者（影響範囲の DB forensic、WORM アーカイブ保全）
- 関与: infra 担当者（NetworkPolicy による egress block、Kyverno enforcement 強化）
- 関与: ops 担当者（SLO 監視、escalation engine 継続）

## 前提

- 7 incident class × 6 phase playbook が build artifact として存在し、`v1_data_exfiltration` の playbook pointer が有効
- `escalation_policy.lock.yaml` の L3 escalation（tech lead + 法務 / DPO）連絡先が最新
- WORM Object Lock（Rook+Ceph）に audit_event snapshot が 30 日分以上保全済み
- PII 専用クラスタ（`08_PII_dedicated_cluster`）が稼働し、WAL chain が分離済み

## 流れ

1. **検知 → L3 escalation 即時起動（phase 1 + L3 宣言）**:
   - alert 受信後 30 分以内に L3 escalation（tech lead + 法務 / DPO）を Mattermost `#security-breach` + 電話で起動する
   - IR commander（security 担当者）を自動 assign し、scope assessment を開始する
2. **triage（phase 2）**: 影響テナント / 影響 PII class / 影響期間を ClickHouse audit query で特定する（`SELECT tenant_id, pii_class, MIN(event_at), MAX(event_at) FROM audit_events WHERE asset='v1_pii' AND actor=... GROUP BY tenant_id, pii_class`）
3. **contain（phase 3）**:
   - 該当 actor identity の token / SSH を OpenBao で即時 revoke する
   - 影響テナントへの外向き egress を NetworkPolicy で即時 block する（除: 認証 / 監査 SoR への通信のみ許可）
   - PII 専用クラスタを read-only 化する（CloudNativePG で primary 切離 → standby のみアクセス）
4. **証跡保全**: WORM Object Lock の audit_event snapshot を litigation hold 状態に設定する（Object Lock の retention mode = COMPLIANCE に変更）。RFC 3161 trusted timestamp を取得し、外部公証 attestation を Sigstore Rekor に publish する
5. **breach 規模の確定と通知判断**: 法務 / DPO と共に breach 規模（PII record 件数 / PII class の機微度）を確定する。通知義務が発生する場合（GDPR 72 h / 個人情報保護法 30 日）はタイムラインを `breach_notification.lock.yaml` に記録し、期限管理を開始する
6. **eradicate（phase 4）**: RLS bypass が起きた場合は PostgreSQL RLS policy の gap を特定し修正 PR を起票する。tier2 担当者とペアで RLS FORCE の適用範囲を再確認する
7. **recover（phase 5）**: NetworkPolicy block を段階的に解除し、SLO 復帰 + audit_event 正常 ingest を確認する。影響テナントへの通知文書を法務と共同作成する
8. **postmortem（phase 6）**: blameless postmortem PR を起票。必須 section（timeline / root cause / PII impact scope / mitigation / lessons learned / action items）。breach notification を行った場合は通知時刻と対象テナントを action item に記録する。postmortem PR merge が次 release の物理 prerequisite

## 関連適合仕様 / 関連 OSS

- 脅威モデル適合仕様: [../../../04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md](../../../04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md)
- データ保全適合仕様: [../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)
- 関連 OSS: Tetragon / Falco / CloudNativePG（RLS FORCE）/ Rook+Ceph（WORM Object Lock）/ OpenBao / Sigstore Rekor / RFC 3161 TSA / Kyverno

## 期待結果 / 観測指標

- L3 escalation が alert 受信後 30 分以内に起動済み
- 影響テナント / PII class / 影響期間が ClickHouse audit query で確定済み
- WORM Object Lock が litigation hold 状態（COMPLIANCE mode）で RFC 3161 timestamp + Rekor entry を持つ
- 規制通知期限（GDPR 72 h 等）以内に通知判断と（必要な場合）通知が完了済み
- postmortem PR が merge 済み、action item 全件 GitHub Issue 化済み

## 失敗時の挙動 / escalation

- **影響範囲特定に 72 h を超える**: GDPR の notifiable breach である可能性が高い。法務 / DPO が「影響範囲未確定」として当局に事前報告し、確定次第 補足報告する手順（GDPR Article 33.4）を踏む
- **WORM Object Lock が Object Lock policy で変更できない（Rook+Ceph 故障）**: infra 担当者に Ceph cluster の緊急 recovery を依頼する（SLA: 2 時間）。snapshot が WORM 以外の場所（cold storage）にコピーされているか確認し、litigation hold を cold storage 側で設定する
- **RLS bypass の root cause が CloudNativePG 本体の bug**: data 担当者と共同でパッチ適用または paired OSS（StackGres）への緊急 migration を判断する。`dry_run.lock.yaml` の `last_green_at` が 365 日以内であれば migration 経路は担保されている

## 関連参照

- [security 担当者シナリオ index](./README.md) — security 担当者シナリオ全体の構成と主要分類一覧
- [security 設計方針: インシデント対応方針](../../../03_概要設計/07_security設計方針/07_インシデント対応方針.md) — `v1_data_exfiltration` class の contain action 詳細
- [security 設計方針: PII 保護方針](../../../03_概要設計/07_security設計方針/06_PII保護方針.md) — 5 PII class の機微度と通知義務の判断基準
- [security 担当者シナリオ: audit hash chain 改竄検知](./09_audit_hash_chain改竄検知_外部公証.md) — 改竄が PII 漏洩経路だった場合の本シナリオへの escalation
- [PII 専用クラスタ](../../../04_詳細設計/03_クロスカッティング適合仕様/08_PII_dedicated_cluster.md) — PII 専用 PostgreSQL cluster と WAL chain 分離の structural spec
