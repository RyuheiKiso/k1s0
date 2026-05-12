---
id: plan.tier2.scenario_emergency_override
axis: tier2
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier2.tier2_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers:
    - D
    - E
  proof_classes: []
---

# Emergency Override 実行

## 一文方針

tier2 担当者（またはプラットフォーム運営者）が Emergency Override（break-glass）経路を通じて本番 DB cluster-admin 権限を取得し、緊急対応を実施した後に強制 audit emit と事後報告を行う。

## Trigger（発火条件）

通常の権限では対応できない本番 DB 緊急操作（data corruption 修正 / 重大 bug の hotfix / セキュリティインシデント対応）が必要になった時

## 想定頻度 / 典型きっかけ

- 想定頻度: 不定期（緊急時のみ）
- 典型きっかけ: 「製造業 pack の受注 DB に data corruption が発生し、通常の tier2 API では修正できないため break-glass で直接 SQL 修正が必要になった」「本番環境の認可バグで一部テナントの PII が別テナントに漏洩していることが発覚し、即時の DB レベル修正が必要になった」

## 主役 / 関与者

- 主役: tier2 担当者（中堅級）またはプラットフォーム運営者
- 関与: security 担当者（emergency 承認）/ ops 担当者（on-call）
- 承認: security 担当者の承認必須（break-glass は通常の dual reviewer とは別の承認経路）

## 前提

- [管理境界方針](../../../03_概要設計/03_tier2設計方針/14_管理境界.md)の emergency 経路が確立済み
- OpenBao の response wrapping が設定済みで、break-glass role の取得には time-limited token が必要
- audit hash chain が稼働中

## 流れ

1. 緊急対応の必要性と scope を明確化する（何の問題を / どのデータに対して / なぜ直接操作が必要か）
2. security 担当者に Emergency Override 承認を Mattermost `#security-incident` で依頼する（security 担当者が承認しない限り break-glass 実行不可）
3. security 担当者承認後、OpenBao response wrapping で break-glass cluster-admin token を取得する（time-limited: 最大 2 時間）
4. 取得した token での操作開始と同時に audit emit が自動的に行われることを確認する（全操作が audit hash chain に記録される）
5. 最小限の操作で緊急対応を完了する（直接 SQL 実行は最終手段。可能な限り tier2 Admin API 経由を優先）
6. 操作完了後に break-glass token を即時 revoke する（Backstage runbook `break-glass-revoke` を実行）
7. audit trail を確認し、全操作が記録されていることを検証する
8. ops 担当者と security 担当者に完了報告を行い、postmortem を 1 営業日以内に提出する

## 関連適合仕様 / 関連 OSS

- 関連適合仕様: [認証適合仕様](../../../04_詳細設計/01_適合仕様/04_認証適合仕様.md) / [鍵管理適合仕様](../../../04_詳細設計/01_適合仕様/05_鍵管理適合仕様.md) / [tier2 強制機構](../../../04_詳細設計/02_強制機構/02_tier2強制機構.md)
- 関連 OSS: OpenBao（break-glass token）/ CloudNativePG（DB 操作）/ Backstage（runbook）

## 期待結果 / 観測指標

- audit: 全操作が audit hash chain に記録済み（`SELECT hash FROM audit_chain ORDER BY seq DESC LIMIT 10` で連続性確認）
- token: break-glass token が revoke 済み（`bao token lookup <token>` が 404 を返す）
- report: postmortem が 1 営業日以内に Backstage TechDocs に提出済み
- compliance: security 担当者の承認記録と audit trail が forensics 用に保全済み

## 失敗時の挙動 / escalation

- **break-glass token 取得後に audit chain が記録されない**: 操作を即時中断し security 担当者と ops 担当者に Mattermost `#compliance-incident` で即時通報（**SLA: 操作中断後 5 分以内**）。audit chain の修復が完了するまで break-glass 操作は禁止。
- **緊急操作でさらなる data corruption が発生**: 操作を中断し data 担当者 + security 担当者に Mattermost `#data-incident` で即時通報（**SLA: 5 分以内**）。restore drill（data-03）を緊急実施。**postmortem 期限: 1 営業日以内**。

## 関連参照

- [tier2 担当者シナリオ index](./README.md) — tier2 担当者シナリオ全体の構成と dual reviewer 規約
- [権限モデル変更](./11_権限モデル変更_ABAC_delegation.md) — 通常の権限変更シナリオ（break-glass なしで対応可能な場合）
- [tier2 設計方針 管理境界](../../../03_概要設計/03_tier2設計方針/14_管理境界.md) — Emergency Override の設計方針 SoT
