# DR drill 結果サマリ（四半期ローテーション）

本書は ADR-TEST-005（Upgrade drill + DR drill 実施方針、Velero 不採用）の 4 経路四半期ローテーション結果を記録する live document。既存設計（barman-cloud / etcdctl / GitOps / Realm Export）の drill が机上 RTO（PostgreSQL 15 分 / etcd 30 分 / GitOps 4 時間）を実証していることを採用検討組織に公開する。

## 本書の位置付け

ADR-TEST-005 で確定した DR drill の 4 経路を四半期ローテーションで実施し、各経路の所要時間を実測値で記録する。机上 RTO 値が「採用検討者向けの主張」ではなく「drill 実走の実測」で裏付けられることが本書の価値。

採用組織の SRE が四半期に 1 経路を学べる構造とし、ADR-OPS-001 の四半期 Chaos Drill とローテーション枠を共有する設計。

## 4 経路ローテーション計画

| 四半期 | 経路 | 机上 RTO | 検証対象 |
|---|---|---|---|
| Q1 | 経路 A: etcd snapshot 復旧 | 30 分 | etcdctl snapshot save / restore + control-plane 再起動 |
| Q2 | 経路 B: GitOps 完全再構築 | 4 時間 | tofu apply + kubeadm init + Argo CD 同期で全 manifest 復元 |
| Q3 | 経路 C: PostgreSQL barman-cloud restore | 15 分 | base backup + WAL リプレイ |
| Q4 | 経路 D: Keycloak Realm Export restore | 15-30 分 | DB restore + Realm JSON import |

## 四半期サマリ

### 2026-Q2（リリース時点 / 初期、Drill 実走前）

- **状態**: ADR-TEST-005 で 4 経路の drill 実施方針を確定。リリース時点では Runbook skeleton 整備を進めるが、staging cluster 常設が採用後の運用拡大時 のため drill 実走は未着手。
- **完了済**: ADR-TEST-005 起票 / 既存設計（`02_etcd全ノード障害.md` / `01_障害復旧とバックアップ.md`）との整合確認 / Velero 不採用の構造的判断軸確立
- **採用後の運用拡大時**: staging cluster 常設後、初回 Q として経路 A（etcd snapshot）から drill 開始

## 四半期サマリ template（採用後の運用拡大時で本テンプレに従って追記）

```markdown
### YYYY-Qx

- **対象経路**: 経路 A / B / C / D
- **実施日**: YYYY-MM-DD
- **机上 RTO**: 30 分 / 4 時間 / 15 分 / 15-30 分
- **実測 RTO**: M 分（机上比 +X 分 / -Y 分）
- **失敗箇所**: <step / 概要 / 根本原因>
- **Runbook 修正**: <commit hash> <修正内容>
- **artifact**: <staging cluster の状態 dump>
- **以降 Q への申し送り**: <経路改善 / 自動化候補>
```

## 関連

- ADR-TEST-005（Upgrade drill + DR drill）
- ADR-INFRA-001（kubeadm + Cluster API）— Upgrade drill の前提
- ADR-DATA-001（CloudNativePG）— 経路 C の前提
- ADR-DATA-003（MinIO）— DR backup target、Velero 不採用
- ADR-OPS-001（Runbook 標準化 + Chaos Drill）— 四半期ローテーション枠の共有元
- 構想設計 `02_構想設計/01_アーキテクチャ/02_可用性と信頼性/01_障害復旧とバックアップ.md`
- 構想設計 `02_構想設計/01_アーキテクチャ/02_可用性と信頼性/06_壊滅的障害シナリオ/02_etcd全ノード障害.md`
- `ops/runbooks/RB-DR-001〜004`（採用初期で skeleton 整備）
