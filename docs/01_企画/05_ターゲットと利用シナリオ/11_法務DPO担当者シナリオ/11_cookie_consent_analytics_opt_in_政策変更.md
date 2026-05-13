---
id: plan.overview.scenario_legal_dpo_cookie_consent_policy
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

# cookie_consent_analytics_opt_in_政策変更

## 一文方針

cookie consent / analytics opt-in 政策変更の sign-off を法務 DPO 担当者として実施し、GDPR / ePrivacy Directive の要件を満たした政策を確定する。

> 半期の政策見直しで、k1s0 の web UI が利用する analytics cookies（Matomo）と tier3 SPA のセッション管理 cookie の consent 政策を見直す必要が生じた。EU テナントの従業員が利用する SPA に対して、GDPR / ePrivacy Directive の要件に基づく opt-in 型の consent mechanism が実装されているかを確認する。法務 DPO 担当者は tier3 担当者と連携し、consent banner の文言・opt-in 動作・同意記録の保管を法的要件に適合させる。

## ペルソナ要約

主役: 法務 DPO 担当者（シニア級）、目的: cookie consent / analytics opt-in 政策を GDPR / ePrivacy Directive の要件に適合させ、sign-off を完了する

## 現状業務での痛み

- cookie consent の法的要件（必須 cookie と非必須 cookie の区別 / opt-in と opt-out の使い分け）が担当者間で共有されていない
- consent banner の文言が法的要件を満たしていない可能性があるが、定期的な review が行われていない
- 同意記録（誰が・いつ・何に同意したか）が保管されておらず、規制当局からの要求に応答できない
- analytics ツールの変更（Matomo → 別ツール）が consent 政策の変更を伴うことが認識されていない

## k1s0 でこう変わる

- cookie consent policy が年次 / 半年次の定期 review 対象として `consent_policy.lock.yaml` で管理される
- 同意記録が data 層に保管され、監督機関または DSAR での照会に即座に応答できる
- consent banner の文言が法務 DPO 承認テンプレートから自動生成され、法的要件の逸脱を防止する
- analytics ツール変更時に法務 DPO への通知が自動化され、政策変更が漏れなく行われる

## Trigger

cookie consent 政策変更時（年次 / 半年次の定期見直し / analytics ツール変更時）

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 年次〜不定期
- 典型きっかけ: 「EU 監督機関から cookie consent に関する新ガイドラインが発行された」「analytics ツールを Matomo から GA4 に変更する提案が来た」「半年次の consent 政策 review 日程が到来した」
- 頻度根拠: 年次または半年次の定期 review が基本。規制変更 / ツール変更はイベント駆動

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| 法務 DPO 担当者（主役） | シニア | 本社 / リモート | consent_policy.lock.yaml / Mattermost | 政策 review・consent banner 文言承認・sign-off |
| tier3 担当者 | シニア | 本社 IT 室 / リモート | GitHub PR list | consent banner / opt-in 機能の技術実装 |
| data 担当者 | シニア | 本社 IT 室 / リモート | ClickHouse | 同意記録の保管設計・確認 |

## 個人 KPI / 達成感

- consent 政策の年次 / 半年次 review 実施率: 100%
- consent banner の法的要件充足率: 100%
- 同意記録の保管完全性: 100%

## 工数 / 関与人数 / コスト感

- consent 政策 review + 文言確認: 2〜3 時間
- 技術実装確認（tier3 と協働）: 1〜2 時間
- 関与人数: 3 名（法務 DPO + tier3 + data）

## 前提

- `consent_policy.lock.yaml` が存在し、consent 政策の管理が可能
- tier3 SPA に consent banner が実装されている
- 同意記録の保管設計が data 担当者によって確認されている

## 流れ

1. **review トリガ確認**: 定期 review / 規制変更 / ツール変更のどのトリガで発火したかを確認する
2. **現状 consent 政策確認**: `consent_policy.lock.yaml` の現在の政策（必須 / 非必須 cookie 分類 / opt-in / opt-out 設定）を確認する
3. **ePrivacy / GDPR 要件確認**: 最新の EU 監督機関ガイドライン（EDPB）と ePrivacy Directive の要件を確認する
4. **consent banner 文言確認**: tier3 SPA の consent banner 文言が法的要件（処理目的の明示 / 同意撤回の容易性）を満たしているか確認する
5. **opt-in / opt-out 動作確認**: 非必須 cookie（analytics / marketing）が opt-in 型で実装されているか確認する（EU 向け）
6. **同意記録保管確認**: data 担当者と協力し、同意記録（タイムスタンプ / 同意内容 / 同意者 ID）が data 層に保管されているか確認する
7. **変更が必要な場合の PR 依頼**: tier3 担当者に consent banner の文言修正 / opt-in 動作修正の PR を依頼する
8. **sign-off + `consent_policy.lock.yaml` 更新**: 法務 DPO sign-off を記録し、`consent_policy.lock.yaml` を更新する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| Day 0 | 法務 DPO | review トリガ確認・現状政策確認 | — |
| Day 2 | 法務 DPO | GDPR / ePrivacy 要件確認完了 | — |
| Day 3 | 法務 DPO + tier3 | consent banner 文言 / opt-in 動作確認 | — |
| Day 5 | 法務 DPO + data | 同意記録保管確認 | — |
| Day 7 | tier3 担当者 | 必要な変更を PR 化 | GitHub: PR open |
| Day 10 | 法務 DPO | PR 内容確認・sign-off・consent_policy 更新 | 「[cookie consent] 政策更新完了。opt-in 実装確認済み」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 全 9 業務（EU テナント）| 高 | EU テナントが利用する tier3 SPA すべてに consent 政策が適用 |
| 受注 sub | 中 | 受注 UI での analytics cookie 同意が必要 |
| 図面 collaborative review | 中 | 図面 review UI での session cookie consent 確認 |

## 通知 deadline timeline

- 通知 deadline は本シナリオには適用されない（定期 review または規制変更に対応する政策変更が目的）

## 監督官庁との接点

- **接点内容**: EU 監督機関（EDPB ガイドライン遵守 / 国別 DPA への compliance 報告）
- **行政処分リスク**: 無効な consent mechanism（opt-in なし / 撤回困難）は GDPR / ePrivacy 違反として行政処分の対象

## dual sign-off 規律

- 法務 DPO（法的要件の充足確認）+ tier3 担当者（技術実装の正確性確認）

## 関連適合仕様 / 関連 OSS

- security 強制機構（同意記録の改竄不可能性）
- 関連 OSS: Matomo / GA4（analytics）/ 同意管理プラットフォーム / Backstage TechDocs（consent policy template）

## 期待結果 / 観測指標

- EU 向け tier3 SPA の非必須 cookie が opt-in 型で実装されている
- 同意記録が data 層に保管されている
- `consent_policy.lock.yaml` に最新の法務 DPO sign-off が記録されている

## 失敗時の挙動 / escalation

- **opt-in なしで analytics cookie が設定されていることが発覚**: tier3 担当者に即時修正 PR を依頼する。SLA: 5 営業日以内に修正完了。EU テナントの影響を法務 DPO が評価し、必要に応じて自主報告を検討する
- **同意記録が保管されていないことが発覚**: data 担当者に同意記録の保管設計を依頼する。DSAR で同意確認を求められる前に整備する
- **EU 監督機関から consent banner に関する問い合わせが来た**: 外部法務顧問と連携し、30 日以内に回答する

## 失敗パターン (anti-pattern)

1. **必須 cookie と analytics cookie を同一バナーで opt-out にする**: EU では analytics cookie の opt-in が必須。opt-out は認められない
2. **同意記録の保管を「将来対応」にする**: DSAR や監督機関からの調査で同意証明ができなくなる
3. **consent banner を「一度実装したから大丈夫」と見直さない**: 規制ガイドラインの更新や analytics ツールの変更で要件が変わるため、年次 review が必須

## 関連参照

- [法務 DPO 担当者シナリオ index](README.md)
- [GDPR_72h_breach通知](01_GDPR_72h_breach通知.md)
- [DSAR対応最終承認](03_DSAR対応最終承認.md)
