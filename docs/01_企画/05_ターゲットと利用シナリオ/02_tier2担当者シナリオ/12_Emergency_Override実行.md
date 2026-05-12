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

> 深夜 2 時、自宅 on-call の tier2 担当者（中堅級）が PagerDuty アラートで「受注 DB data corruption」通知を受け取り、Backstage runbook `break-glass-revoke` を開く。手元には OpenBao CLI・`bao token` コマンド、Mattermost 越しに security 担当者と ops 担当者がいる。

## Trigger（発火条件）

通常の権限では対応できない本番 DB 緊急操作（data corruption 修正 / 重大 bug の hotfix / セキュリティインシデント対応）が必要になった時

## 想定頻度 / 典型きっかけ

- 想定頻度: 不定期（緊急時のみ）
- 典型きっかけ: 「製造業 pack の受注 DB に data corruption が発生し、通常の tier2 API では修正できないため break-glass で直接 SQL 修正が必要になった」「本番環境の認可バグで一部テナントの PII が別テナントに漏洩していることが発覚し、即時の DB レベル修正が必要になった」

## 主役 / 関与者

- 主役: tier2 担当者（中堅級）またはプラットフォーム運営者
- 関与: security 担当者（emergency 承認）/ ops 担当者（on-call）
- 承認: security 担当者の承認必須（break-glass は通常の dual reviewer とは別の承認経路）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier2）| 中堅 | 自宅 on-call | PagerDuty / Mattermost `#security-incident` | break-glass 手順開始 / 最小限操作 / token revoke / postmortem 提出 |
| 関与（security）| シニア | 自宅 on-call | PagerDuty / Mattermost `#security-incident` | Emergency Override 承認 / audit trail 確認 |
| 関与（ops）| 中堅 | 自宅 on-call | PagerDuty / Perses dashboard | on-call 調整 / 操作完了報告受領 |

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

## 業界 9 業務との紐付け

- **受注**: 受注 DB の data corruption 修正が break-glass の最も典型的な適用場面であり、緊急対応後に受注業務が正常稼働に復帰する。
- **FA 生産指示・設備操作**: 生産ラインに影響する重大 bug の hotfix を break-glass で実施し、FA 業務の停止時間を最小化する。
- **警報配信**: 認可バグによる PII 漏洩インシデント対応時に break-glass で DB レベル修正を行い、警報配信経路の完全性を回復する。
- **SCADA テレメトリ**: SCADA 連携 DB の corruption 修正により、テレメトリデータの正常取得が再開される。

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

## Timeline

| T+ | actor | action | Mattermost 投稿例 |
|---|---|---|---|
| 0 | tier2 担当者 | on-call 招集受領 / break-glass 手順開始 | `@security-oncall break-glass 開始 / 対象: 受注 DB / T+0 02:00` |
| 5 分 | security 担当者 | OpenBao break-glass 経路確認 / Emergency Override 承認 | `OpenBao break-glass 経路確認中 / 承認済み T+5` |
| 15 分 | tier2 担当者 | cluster-admin 取得 / 緊急操作開始 | `cluster-admin 取得 / 操作: data corruption 修正 SQL / T+15` |
| 60 分 | tier2 担当者 | 操作完了 / audit hash chain 確認 / break-glass token revoke | `操作完了 / audit emit 確認済 / break-glass 返却 T+60` |
| 1 営業日 | tier2 担当者 | postmortem 着手 / Backstage TechDocs 提出 | `#postmortem postmortem PR 作成済 / 対象インシデント: 受注 DB corruption` |

## 関連参照

- [tier2 担当者シナリオ index](./README.md) — tier2 担当者シナリオ全体の構成と dual reviewer 規約
- [権限モデル変更](./11_権限モデル変更_ABAC_delegation.md) — 通常の権限変更シナリオ（break-glass なしで対応可能な場合）
- [tier2 設計方針 管理境界](../../../03_概要設計/03_tier2設計方針/14_管理境界.md) — Emergency Override の設計方針 SoT
