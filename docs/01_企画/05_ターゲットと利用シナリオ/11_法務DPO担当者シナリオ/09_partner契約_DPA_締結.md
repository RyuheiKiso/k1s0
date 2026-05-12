---
id: plan.overview.scenario_legal_dpo_partner_dpa
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

# partner契約_DPA_締結

## 一文方針

取引先との DPA（Data Processing Agreement）締結を業務管理者 04 と連動して法務 DPO 担当者として主導し、GDPR Art.28 の要件を満たす DPA を確定させる。

> 四半期の新規 partner 契約時、法務 DPO 担当者が業務管理者 04 と並走して DPA の締結を主導する。新規 partner は EU 域内のクラウドサービスプロバイダーであり、テナントの PII データを処理する sub-processor に該当する。法務 DPO 担当者は GDPR Art.28 の DPA 必須条項を確認し、partner から提示された DPA draft の法的妥当性を review する。sub-processor chain の透明性要件（GDPR Art.28(2)）も確認する。

## ペルソナ要約

主役: 法務 DPO 担当者（シニア級）、目的: GDPR Art.28 の DPA 必須条項を満たす DPA を partner と締結し、sub-processor chain の適切性を確認する

## 現状業務での痛み

- partner 各社の DPA draft の品質がまちまちで、GDPR Art.28 の必須条項が欠落しているケースがある
- sub-processor の変更通知義務（Art.28(2)）が DPA に明記されておらず、後から問題になるケースがある
- DPA 締結の状況が散在しており、どの partner と DPA を締結済みかの管理が困難
- partner の security 体制（Art.28(1) の十分な保証）の確認方法が明確でない

## k1s0 でこう変わる

- DPA 必須条項チェックリストが Backstage TechDocs に整備され、review 品質が標準化される
- `dpa_registry.lock.yaml` が partner 単位の DPA 締結状況を管理し、締結済み / 未締結 / 期限切れ を一元管理する
- partner の security 体制確認（ISO 27001 / SOC 2 / ISMS 等）を DPA 締結の前提条件として構造化する
- sub-processor chain の変更通知が自動的に法務 DPO に通知される仕組みを整備する

## Trigger

新規 partner 契約時

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 四半期〜年次
- 典型きっかけ: 「新規クラウドサービス partner との契約が決まり、PII データの処理を委託する」「既存 partner の DPA が更新期限を迎えた」
- 頻度根拠: partner 契約は事業拡大に伴い四半期〜年次で発生

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| 法務 DPO 担当者（主役） | シニア | 本社 / リモート | dpa_registry.lock.yaml / Mattermost | DPA 必須条項確認・partner DPA review・締結 sign-off |
| 業務管理者（シナリオ 08-04）| シニア | テナント事業部 | Backstage admin UI | partner 連携設定の業務的妥当性確認・連携 |
| security 担当者 | シニア | 本社 IT 室 / リモート | security dashboard | partner の security 体制確認（ISO 27001 等） |

## 個人 KPI / 達成感

- DPA 締結率（PII 処理 partner 全社）: 100%
- DPA review 応答時間: 10 営業日以内
- `dpa_registry.lock.yaml` の最新性（期限切れ DPA が 0 件）: 100%

## 工数 / 関与人数 / コスト感

- DPA 必須条項確認 + partner DPA review: 2〜4 時間
- 交渉・修正対応（必要時）: 1〜5 時間
- 関与人数: 3〜4 名（法務 DPO + 業務管理者 + security + 必要時 外部法務顧問）

## 前提

- DPA 必須条項チェックリストが Backstage TechDocs に整備されている
- `dpa_registry.lock.yaml` が存在し、partner 単位の DPA 管理が可能
- DPA テンプレート（k1s0 側が DPA を提示する場合）が整備されている

## 流れ

1. **DPA 締結必要性確認**: 新規 partner が PII データを処理するかどうかを確認する（GDPR Art.28 の processor 該当性判断）
2. **partner DPA draft 受領**: partner から DPA draft を受領する（または k1s0 側の DPA テンプレートを提示する）
3. **DPA 必須条項チェック**: GDPR Art.28(3) の 7 項目（処理の目的 / 期間 / 性質 / PII の種別 / 義務と権利）が含まれているか確認する
4. **sub-processor chain 確認**: partner が使用する sub-processor のリスト取得と変更通知義務条項（Art.28(2)）を確認する
5. **security 体制確認（security 担当者と協働）**: partner の security 認証（ISO 27001 / SOC 2 等）を確認する（Art.28(1) の「十分な保証」）
6. **交渉・修正**: 不備がある条項について partner と交渉し、必須条項を補完する
7. **締結 sign-off**: DPA に署名し、`dpa_registry.lock.yaml` に締結日・有効期限・partner 名・処理 PII 種別を記録する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| Day 0 | 法務 DPO | partner DPA draft 受領・必須条項チェック開始 | — |
| Day 5 | 法務 DPO + security | security 体制確認完了 | — |
| Day 7 | 法務 DPO | 必須条項チェック完了・交渉項目特定 | — |
| Day 10 | 法務 DPO | partner との交渉・DPA 修正完了 | — |
| Day 12 | 法務 DPO | 締結 sign-off・dpa_registry 記録 | 「[DPA] partner X との DPA 締結完了。PII 処理: 受注データ」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 受注 sub | 高 | 受注処理 partner との DPA 締結が必須 |
| 品質検査結果配信 | 高 | 品質管理 SaaS partner との DPA 締結 |
| FA 生産指示 | 中 | 生産管理 partner との DPA 締結 |

## 通知 deadline timeline

- 通知 deadline は本シナリオには適用されない（partner との DPA 締結は事前の契約プロセス）

## 監督官庁との接点

- **通常は監督官庁との直接接点なし**
- **DPA なしに PII 処理が行われた場合**: GDPR 違反として監督機関からの処分リスクがある

## dual sign-off 規律

- 法務 DPO（DPA の法的妥当性確認）+ 業務管理者（連携業務の適切性確認）

## 関連適合仕様 / 関連 OSS

- security 強制機構（partner の security 体制確認と連動）
- 関連 OSS: GitHub（dpa_registry.lock.yaml）/ Backstage TechDocs（DPA チェックリスト）

## 期待結果 / 観測指標

- PII 処理 partner 全社との DPA 締結率 100%
- `dpa_registry.lock.yaml` に全 partner の DPA 締結記録が存在する
- DPA の有効期限切れが 0 件

## 失敗時の挙動 / escalation

- **partner が DPA 締結を拒否する**: PII データの処理委託を取りやめるか、外部法務顧問を通じて交渉する。DPA なしに PII 処理を委託することは GDPR 違反
- **DPA の必須条項が欠落しており partner が修正に応じない**: 欠落条項を別紙（Addendum）で補完する案を提示する。それでも応じない場合は弁護士を介入させる
- **DPA 期限切れを見落とす**: `dpa_registry.lock.yaml` の有効期限切れ 60 日前に自動 alert が Mattermost に届く

## 失敗パターン (anti-pattern)

1. **DPA を「後でまとめて」対応する**: PII 処理委託開始前に DPA が必須。開始後の締結は GDPR 違反期間が発生する
2. **partner が「GDPR 準拠です」と主張する口頭確認だけで DPA を省略する**: 口頭の確認は法的に有効でない。書面 DPA が必須
3. **sub-processor の変更通知義務を DPA に明記しない**: partner が sub-processor を変更しても通知がなく、GDPR Art.28(2) 違反になる

## 関連参照

- [法務 DPO 担当者シナリオ index](README.md)
- [cross_region_data_locality対応](08_cross_region_data_locality対応.md)
- [業務管理者: partner連携設定_IdP_federation](../../08_業務管理者シナリオ/04_partner連携設定_IdP_federation.md)
