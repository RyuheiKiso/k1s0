---
id: plan.overview.scenario_legal_dpo_cross_region_data_locality
axis: overview
phase: plan
kind: plan_doc
status: draft
depends_on:
  - plan.legal_check
  - arch.security.security_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, D]
  proof_classes: []
---

# cross_region_data_locality対応

## 一文方針

テナント単位 region 指定の法的妥当性を review し、GDPR / Schrems II に適合した data locality 設定を確認して法務 DPO の sign-off を完了させる。

> 四半期 review の金曜、EU 加盟国（ドイツ）のテナント H から「データを EU 外（日本リージョン）に置きたい」というリクエストが届く。法務 DPO 担当者は GDPR 第 5 章（第三国移転）の要件と Schrems II 判決の影響を確認する。日本は GDPR の「十分性認定」を受けているため移転は可能だが、標準契約条項（SCC）の適用と追加安全保護措置の確認が必要と判断し、infra 担当者に region 設定変更の条件を伝える。

## ペルソナ要約

主役: 法務 DPO 担当者（シニア級）、目的: テナントの region 変更リクエストの GDPR / Schrems II 法的妥当性を確認し、sign-off を完了させる

## 現状業務での痛み

- GDPR の第三国移転要件（十分性認定 / SCC / BCR / 適切な保護措置）を正確に評価できる担当者が少ない
- Schrems II 判決の影響（EU-US Privacy Shield の無効化）が既存の region 設定に与える影響を把握していない
- テナント単位の region 変更リクエストが技術的に承認され、法務確認が後付けになるケースがある
- region 変更後の法的適合性を継続的に監視する仕組みがない

## k1s0 でこう変わる

- テナント region 変更リクエストが infra 担当者（シナリオ 04-infra-07）から法務 DPO に自動転送され、法務確認が必須 gate になる
- GDPR 第三国移転要件のチェックリストが Backstage TechDocs に整備され、担当者間で判断基準が統一される
- `data_locality.lock.yaml` に各テナントの region 設定と法務 DPO sign-off が記録され、継続的な適合性追跡が可能になる
- 十分性認定の更新情報（EU の決定改訂等）が `data_locality.lock.yaml` の自動更新に連動する

## Trigger

テナント region 指定変更時

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 四半期〜年次
- 典型きっかけ: 「EU テナントが日本リージョンへの移転を要求した」「US テナントが EU リージョンへの移転を要求した」
- 頻度根拠: region 変更はテナントのビジネス要件変化に伴う不定期イベント。四半期 review が典型

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| 法務 DPO 担当者（主役） | シニア | 本社 / リモート | data_locality.lock.yaml / Mattermost | 第三国移転要件確認・Schrems II 評価・sign-off |
| infra 担当者 | シニア | 本社 IT 室 / リモート | Kubernetes / OpenTofu | region 変更の技術的実装（法務 sign-off 後） |
| data 担当者 | シニア | 本社 IT 室 / リモート | ClickHouse | 移転対象 PII データの確認 |

## 個人 KPI / 達成感

- region 変更リクエストへの法務 DPO 回答: 10 営業日以内
- `data_locality.lock.yaml` の更新完了率: 100%
- region 変更後の法的適合性違反: 0 件

## 工数 / 関与人数 / コスト感

- 第三国移転要件確認 + Schrems II 評価: 2〜4 時間
- SCC / 追加保護措置確認: 1〜2 時間
- 関与人数: 3〜4 名（法務 DPO + infra + data + 必要時 外部法務顧問）

## 前提

- GDPR 第三国移転要件チェックリストが Backstage TechDocs に整備されている
- `data_locality.lock.yaml` が各テナントの region 設定と法務 sign-off を管理している
- 標準契約条項（SCC）テンプレートが法務 DPO 手元にある

## 流れ

1. **リクエスト受領 + 移転先確認**: region 変更リクエストの移転先 region と対象 PII データを確認する
2. **十分性認定確認**: 移転先国が EU の十分性認定を受けているか確認する（日本: 認定済み / 米国: Privacy Shield 無効）
3. **Schrems II 評価**: 移転先国の政府によるデータアクセスリスクを Schrems II の枠組みで評価する
4. **移転根拠の確定**: 十分性認定 / SCC / BCR / Art.49 例外の中から適用可能な移転根拠を確定する
5. **SCC / 追加保護措置確認（必要時）**: SCC が必要な場合はテンプレートを適用し、Schrems II 対応の追加保護措置（暗号化 / アクセス制限）を確認する
6. **sign-off 記録**: `data_locality.lock.yaml` に法務 DPO sign-off を記録し、GitHub commit として残す
7. **infra 担当者への承認通知**: region 変更の技術的実装を infra 担当者に承認し、条件（SCC 適用 / 暗号化要件等）を伝える

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| Day 0 | 法務 DPO | リクエスト受領・移転先確認 | — |
| Day 3 | 法務 DPO | 十分性認定 / Schrems II 評価完了 | — |
| Day 7 | 法務 DPO | 移転根拠確定・SCC 適用可否判断 | — |
| Day 10 | 法務 DPO | sign-off 記録・infra 担当者に承認通知 | 「[data locality] テナント H: EU → JP 移転 承認（十分性認定適用）」 |
| Day 15 | infra 担当者 | region 変更の技術的実装 | — |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 受注 sub | 高 | EU テナントの受注データ region 指定変更が GDPR 第三国移転要件に該当 |
| 品質検査結果配信 | 中 | 検査記録の region 移転が業界規制（GMP）の記録保持要件に影響 |
| FA 生産指示 | 中 | 生産指示データの region 移転 |

## 通知 deadline timeline

- 通知 deadline は本シナリオには適用されない（テナントリクエストへの法務回答 10 営業日 SLA が基準）

## 監督官庁との接点

- **発動条件**: 第三国移転の妥当性に疑義がある場合、EU 監督機関への事前相談が必要になるケースがある
- **接点内容**: EU 監督機関への BCR 承認申請 / 第三国移転の個別許可申請

## dual sign-off 規律

- 法務 DPO（GDPR 第三国移転の法的妥当性確認）+ data 担当者（移転対象 PII データの確認）

## 関連適合仕様 / 関連 OSS

- security 強制機構（データ暗号化の確認）
- 関連 OSS: OpenTofu（region 変更の GitOps 実装）/ Backstage TechDocs（移転要件チェックリスト）/ GitHub（sign-off 記録）

## 期待結果 / 観測指標

- region 変更リクエストへの法務 DPO 回答が 10 営業日以内に完了
- `data_locality.lock.yaml` に法務 DPO sign-off が記録されている
- region 変更後に GDPR 違反事例が発生していない

## 失敗時の挙動 / escalation

- **移転先国の政府アクセスリスクが Schrems II で高リスクと判断される**: 外部法務顧問に判断を依頼し、SCC への追加条項や技術的保護措置の適切性を評価する
- **テナントが法務 DPO の承認前に infra 担当者に直接 region 変更を依頼する**: infra 担当者は法務 DPO sign-off なしに region 変更を実施できないよう設計されている。bypass 試みがあれば ops 担当者に報告する
- **十分性認定が EU に取り消された（外部環境変化）**: 移転を即時停止し、SCC への切替または EU リージョンへの移転を検討する

## 失敗パターン (anti-pattern)

1. **「日本は十分性認定あるから大丈夫」と確認をスキップする**: 十分性認定の範囲外（例: 医療記録の特定 PII class）がある可能性を確認する
2. **Schrems II の評価を形式的に「問題なし」にする**: 移転先の政府アクセス法（例: 米国 FISA 702）は具体的に評価する必要がある
3. **SCC を締結せずに US への移転を承認する**: Privacy Shield 無効後の US 移転は SCC または other valid mechanism が必須

## 関連参照

- [法務 DPO 担当者シナリオ index](README.md)
- [partner契約_DPA_締結](09_partner契約_DPA_締結.md)
- [infra: network変更](../04_infra担当者シナリオ/07_network変更.md)
