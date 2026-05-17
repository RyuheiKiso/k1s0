---
id: plan.overview.scenario_legal_dpo_oss_license_approval
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

# OSSライセンス階層_新規採用承認

## 一文方針

tier1 軸 01 と連動し、(a)-(e) 階層判定 / AGPL/SSPL 拒否 / 商用利用条項 review の法務 DPO 承認を完了させ、OSS ライセンスリスクを法的に遮断する。

> 月曜午前 11 時、tier1 担当者から「Redpanda を L1+ として採用したい。ライセンスは Business Source License (BSL) 2.1 で 4 年後に AGPL v3 に移行予定」という提案が法務 DPO 担当者の inbox に届く。法務 DPO 担当者は (a)-(e) 階層判定を行い、BSL は階層 (e) に該当（Source Available / 商用利用制限条項）として採用拒否の判断を下し、tier1 担当者に代替 OSS を提案するよう依頼する。

## ペルソナ要約

主役: 法務 DPO 担当者（シニア級）、目的: 新規 OSS 採用提案の (a)-(e) 階層判定を実施し、AGPL/SSPL/BSL 拒否と商用利用条項 review の承認を完了する

## 現状業務での痛み

- AGPL / SSPL / BSL 等の「非 permissive ライセンス」の法的リスクを正確に評価できる担当者が少ない
- tier1 担当者が技術的採用可否を判断し、法務確認が後付けになるケースがある
- ライセンス変更（例: HashiCorp の BUSL 移行）に対応する仕組みがなく、採用済み OSS のライセンス変更リスクが管理されていない
- 商用利用条項の解釈が担当者によって異なり、判断が属人化している

## k1s0 でこう変わる

- OSS ライセンス階層 (a)-(e) 判定フローが Backstage TechDocs に定義され、担当者間で判断基準が統一される
- tier1 の OSS 採用 PR が法務 DPO 承認を必須 gate として設定され、採用前の法務確認が構造的に担保される
- CI の依存導入 lint が (d)(e) 階層ライセンスを物理拒否し、法務 DPO 承認前の merge を防止する
- `oss_lifecycle.lock.yaml` に採用済み OSS のライセンス変更監視が組み込まれ、HashiCorp 型の突然のライセンス変更に即時対応できる

## Trigger

新規 OSS 採用提案時（tier1 担当者から法務 DPO 承認依頼が届いた時）

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 四半期〜年次
- 典型きっかけ: 「tier1 担当者から新規 L1+ OSS の採用提案が届いた」「採用済み OSS のライセンス変更通知が届いた」
- 頻度根拠: 新規 OSS 採用は四半期〜年次（tier1 シナリオ 01 と連動）。ライセンス変更はイベント駆動

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| 法務 DPO 担当者（主役） | シニア | 本社 / リモート | GitHub PR（OSS 採用提案）/ oss_lifecycle.lock.yaml | (a)-(e) 階層判定・商用利用条項 review・承認 or 拒否 |
| tier1 担当者 | シニア | 本社 IT 室 / リモート | GitHub PR | OSS 採用提案・代替 OSS 選定（拒否時） |

## 個人 KPI / 達成感

- OSS 採用 PR の法務 DPO 承認応答時間: 5 営業日以内
- (d)(e) 階層ライセンスの採用拒否率: 100%（採用 0 件）
- ライセンス変更検知から法務確認完了: 10 営業日以内

## 工数 / 関与人数 / コスト感

- (a)-(e) 階層判定 + 商用利用条項 review: 1〜3 時間
- 拒否時の代替案検討（tier1 と協働）: 1〜2 時間
- 関与人数: 2 名（法務 DPO + tier1）

## 前提

- OSS ライセンス階層 (a)-(e) 判定フローが Backstage TechDocs に定義されている
- tier1 の OSS 採用 PR に法務 DPO を reviewer として必須設定されている
- CI の依存導入 lint が (d)(e) 階層ライセンスを物理拒否するよう設定されている

## 流れ

1. **採用提案確認**: tier1 担当者からの OSS 採用 PR を確認し、対象 OSS のライセンス種別と条項を確認する
2. **(a)-(e) 階層判定**: 以下の判定フローを実施する
   - (a) Permissive（MIT / Apache 2.0 / BSD）→ 承認可
   - (b) Weak Copyleft（LGPL / MPL）→ 条件付き承認（tier1 との境界設計確認後）
   - (c) Strong Copyleft（GPL v2/v3）→ tier1 facade での採用は不可（L3 閉鎖コードとの混合禁止）
   - (d) Network Copyleft（AGPL v3）→ 採用拒否
   - (e) Source Available / Proprietary（SSPL / BSL / BUSL）→ 採用拒否
3. **商用利用条項 review**: ライセンス文中の commercial use 制限・SaaS 提供制限・competitive use 制限を確認する
4. **承認 or 拒否の判断**: 判定結果を GitHub PR に comment として記録し、承認 sign-off または拒否理由を明記する
5. **拒否時の代替案提案**: 拒否した場合、同機能の (a) または (b) 階層ライセンス OSS の候補を tier1 担当者に提案する
6. **`oss_lifecycle.lock.yaml` 更新（承認時）**: 採用 OSS のライセンス種別・採用理由・次回 review 日を記録する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| Day 0 | tier1 担当者 | OSS 採用 PR 作成・法務 DPO を reviewer に assign | GitHub: PR open |
| Day 1 | 法務 DPO | 採用提案確認・(a)-(e) 階層判定開始 | — |
| Day 3 | 法務 DPO | 商用利用条項 review 完了 | — |
| Day 5 | 法務 DPO | 承認 sign-off または拒否 comment を PR に記録 | GitHub: 「[法務 DPO] BSL は階層 (e): 採用拒否。代替: Redpanda は SSPL のため Apache Kafka を継続推奨」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 全 9 業務 | 高 | tier1 Library / Server は全業務の通信基盤であり、OSS ライセンスリスクは全業務に波及する |

## 通知 deadline timeline

- 通知 deadline は本シナリオには適用されない（ライセンス承認は regulatory deadline なし）
- ただし採用 PR の SLA（5 営業日以内の法務回答）を維持する

## 監督官庁との接点

- **通常は監督官庁との接点なし**
- **ライセンス違反が発覚した場合**: OSS ベンダーからの legal notice が監督官庁を経由する可能性がある

## dual sign-off 規律

- 法務 DPO（ライセンスの法的妥当性確認）+ tier1 担当者（技術的採用可否の確認）

## 関連適合仕様 / 関連 OSS

- OSS ライフサイクル適合仕様（tier1 シナリオ 01 と連動）
- 関連 OSS: GitHub（PR 管理）/ Backstage TechDocs（判定フロー）/ cargo-deny / licensee（ライセンス自動チェック）

## 期待結果 / 観測指標

- (d)(e) 階層ライセンスの採用が 0 件
- 法務 DPO 承認応答が OSS 採用 PR の 5 営業日以内に完了
- `oss_lifecycle.lock.yaml` に承認済み OSS のライセンス種別が記録されている

## 失敗時の挙動 / escalation

- **CI lint が (d)(e) 階層を検知した後に法務 DPO 確認をスキップして merge しようとする**: merge がシステム的に不可。ops 担当者に status check 解除依頼があっても承認しない
- **採用済み OSS のライセンス変更が発覚（HashiCorp 型）**: `oss_lifecycle.lock.yaml` の当該 OSS の status を「要法務確認」に更新し、tier1 担当者に移行計画の作成を依頼する。SLA: 10 営業日以内に代替 OSS または移行計画を確定
- **法務 DPO 承認が 5 営業日を超過**: tier1 担当者が Mattermost で催促し、法務 DPO が回答期限を再設定する

## 失敗パターン (anti-pattern)

1. **「有名な OSS だから大丈夫」と判断をスキップする**: MongoDB（SSPL）や Elasticsearch（SSPL）は有名でも採用拒否対象。ライセンス文を必ず確認する
2. **ライセンス変更をリリースノートで発見してから対応する**: `oss_lifecycle.lock.yaml` の監視機能で事前検知する体制を維持する
3. **BSL の「Change Date 後に AGPL」という記述を見落とす**: BSL 採用時は Change Date 後のライセンスも確認し、将来的な AGPL 移行を織り込んで判断する

## 関連参照

- [法務 DPO 担当者シナリオ index](README.md)
- [tier1: 新規OSS採用評価](../01_tier1担当者シナリオ/01_新規OSS採用評価.md)
- [tier1: OSSライフサイクルイベント対応](../01_tier1担当者シナリオ/02_OSSライフサイクルイベント対応.md)
