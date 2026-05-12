---
id: plan.security.scenario_audit_hash_chain_tamper_detection
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

# audit hash chain 改竄検知 → 外部公証

## 一文方針

`audit_ingest_gap_monitor` の hash chain verification fail または ingest gap heartbeat miss を検知した時に `v1_audit_chain_break` incident class の 6 phase playbook を発火し、divergence point の特定 → WORM Object Lock アーカイブによる chain root 再 verify → RFC 3161 trusted timestamp + Sigstore Rekor への外部公証 attestation publish までを完結させ、監査証跡の immutability 三重化を回復する。

## Trigger（発火条件）

`audit_ingest_gap_monitor` の hash mismatch alert（Prometheus alert rule `audit_hash_chain_divergence`）が発火した時、または ingest gap heartbeat が 30 秒以上 miss した時。

## 想定頻度 / 典型きっかけ

想定頻度: イベント駆動（年間 0-2 件）。典型きっかけ: 「Prometheus alert `audit_hash_chain_divergence` が Mattermost に page。ClickHouse の audit_event テーブルで `event_id = 2024-11-15T10:32:07.000Z` 付近の hash が前後 event と一致しない」

## 主役 / 関与者

- 主役: security 担当者（シニア級）
- 関与: data 担当者（ClickHouse / PostgreSQL pgaudit の操作）
- 関与: infra 担当者（WORM Object Lock へのアクセス、network forensics）
- 関与: ops 担当者（escalation 継続、SLO 監視）

## 前提

- `audit_ingest_gap_monitor` が 09 cross-cutting 適合仕様に沿って稼働中
- WORM Object Lock（Rook+Ceph）に最低 30 日分の audit_event snapshot が保全済み
- RFC 3161 trusted timestamp と Sigstore Rekor への定期 publish が稼働中
- ClickHouse に audit_event の full-text が hot tier で保持済み（retention policy 内）

## 流れ

1. **divergence point の特定**: ClickHouse で `SELECT * FROM audit_events WHERE hash != expected_hash ORDER BY event_id` を実行し、divergence が始まった `event_id`（時刻）を特定する
2. **ingest gap の有無確認**: `audit_ingest_gap_monitor` の heartbeat log（09 cross-cutting 参照）から gap 区間を特定する。gap があれば ingest 経路の 4 経路（pgaudit / Debezium CDC / Envoy access log / app emit）を個別に確認する
3. **WORM Object Lock による chain root 再 verify**: Rook+Ceph の WORM archive から divergence 前の最後の valid snapshot を取得する（`aws s3 cp s3://audit-worm/snapshot-<date>.json.gz .`）。snapshot の hash を Sigstore Rekor entry と照合し、archive の改竄がないことを確認する
4. **divergence 区間の分類**:
   - 改竄（意図的な event_id 欠落 / hash 書き換え）の可能性がある場合: `v1_data_tampering` + `v1_audit_chain_break` の dual class として 14（data breach）も発火する
   - system fault（ClickHouse insert failure / network partition）の場合: ingest 4 経路から missing event を補完し、chain を re-seal する
5. **chain 再構築**:
   - divergence 区間を separate sub-chain として管理する（メイン chain に接合しない）
   - sub-chain の root を `cosign sign` で署名し、Sigstore Rekor に新エントリとして publish する
   - RFC 3161 trusted timestamp を Autoridade Certificadora 外部 TSA から取得し、sub-chain の existence proof を確立する
6. **全 ingest 経路のカバレッジ確認**: `audit_ingest_gap_monitor` の coverage gap count が 0 に戻るまで 4 経路を個別に確認する
7. postmortem PR を起票し、root cause（改竄 / fault）を明示する。改竄の場合は action item に侵入経路の物理 block を必ず含める

## 関連適合仕様 / 関連 OSS

- 脅威モデル適合仕様: [../../../04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md](../../../04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md)
- security 強制機構: [../../../04_詳細設計/02_強制機構/06_security強制機構.md](../../../04_詳細設計/02_強制機構/06_security強制機構.md)
- 関連 OSS: ClickHouse / Rook+Ceph（WORM Object Lock）/ Cosign / Sigstore Rekor / RFC 3161 TSA / Prometheus / pgaudit

## 期待結果 / 観測指標

- divergence point が特定済みで `audit_chain_break.lock.yaml` に記録済み
- WORM アーカイブとの照合で改竄 / system fault の分類が確定済み
- 再構築 sub-chain が cosign signed + RFC 3161 timestamp + Rekor entry を持つ
- `audit_ingest_gap_monitor` の coverage gap count が 0
- postmortem PR が起票済み

## 失敗時の挙動 / escalation

- **WORM アーカイブが改竄されていた（Object Lock が bypass された）**: L3 escalation（tech lead + 法務）を即時起動（SLA: 30 分）。Rook+Ceph の Object Lock bypass log を infra 担当者と共同で forensic する。外部監査人への通知を検討する
- **ingest gap が 24 時間以上に及ぶ**: audit_event の欠落規模に応じて regulatory 通知（v1_pii 含有の event が欠落している場合は法務 / DPO 連携）。L3 escalation + data 担当者・ops 担当者の全員招集（SLA: 60 分）
- **chain 再構築ができない（divergence が広範囲すぎる）**: 再構築を断念し、divergence 区間を `irrecoverable_gap` として `audit_chain_break.lock.yaml` に記録する。外部公証が存在する範囲を明示し、監査人向けの証跡説明文書を作成する

## 関連参照

- [security 担当者シナリオ index](./README.md) — security 担当者シナリオ全体の構成と主要分類一覧
- [audit_ingest_gap_monitor](../../../04_詳細設計/03_クロスカッティング適合仕様/09_audit_ingest_gap_monitor.md) — hash chain + ingest gap heartbeat の二重防御 structural spec
- [security 設計方針: 監査方針](../../../03_概要設計/07_security設計方針/05_監査方針.md) — audit_event subject の immutability 三重化設計
- [security 担当者シナリオ: data breach response](./14_data_breach_privacy_incident_response.md) — 改竄が PII 漏洩経路の場合の escalation 先
