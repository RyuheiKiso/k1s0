---
id: plan.overview.scenario_legal_dpo_v2_industry_pack_hipaa_sox
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

# v2業界pack_HIPAA_SOX_法務評価

## 一文方針

v2 候補業界 pack（医療 HIPAA / 金融 SOX）の法務評価を実施し、業界規制適合のための要件を確定して v2 開発の前提条件として sign-off する。

> 年次事業計画の月曜、経営から「v2 で医療業界（HIPAA）と金融業界（SOX）に対応したい」という方針が届く。法務 DPO 担当者は HIPAA Privacy Rule / Security Rule と SOX Section 404 の要件を評価し、現在の k1s0 アーキテクチャで何が不足しているかを特定する。HIPAA の PHI（Protected Health Information）定義と k1s0 の PII class の対応、SOX の audit trail 保持要件（7 年）と現在の data 保持ポリシーのギャップを分析する。

## ペルソナ要約

主役: 法務 DPO 担当者（シニア級）、目的: v2 業界 pack（HIPAA / SOX）の法務要件を評価し、開発の前提条件として sign-off する

## 現状業務での痛み

- HIPAA / SOX の要件を正確に理解できる担当者が社内に少なく、要件の解釈が不正確になるリスクがある
- 現在のアーキテクチャと規制要件のギャップ分析が、開発着手後に発覚するケースがある
- 外部法務顧問への依頼コストが高く、要件確認に時間がかかる
- v2 開発の要件定義に法務 DPO が参加する仕組みがなく、法務 risk が後付けで発覚する

## k1s0 でこう変わる

- 法務 DPO 担当者が v2 業界 pack の要件定義フェーズから参加し、法務 risk を事前に特定できる
- HIPAA / SOX の要件チェックリストが Backstage TechDocs に整備され、アーキテクチャとのギャップ分析が体系化される
- 外部法務顧問との協働フローが確立され、専門知識が必要な部分を効率的に活用できる
- v2 開発の前提条件として法務 DPO sign-off が必要となり、規制適合が構造的に担保される

## Trigger

v2 候補業界 pack 検討開始時

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 年次〜不定期
- 典型きっかけ: 「経営から HIPAA / SOX 対応業界 pack の v2 開発方針が示された」「医療 / 金融テナントから業界規制対応の問い合わせが増加した」
- 頻度根拠: v2 業界 pack 追加は事業拡張の意思決定に伴うイベント

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| 法務 DPO 担当者（主役） | シニア | 本社 / リモート | v2_legal_assessment.lock.yaml / Mattermost | HIPAA / SOX 要件評価・ギャップ分析・前提条件 sign-off |
| 外部法務顧問 | 外部 | リモート | — | HIPAA / SOX の専門法務解釈（必要時）|
| security 担当者 | シニア | 本社 IT 室 / リモート | security dashboard | security 要件（HIPAA Security Rule）のアーキテクチャ評価 |
| tier1 担当者 | シニア | 本社 IT 室 / リモート | GitHub | 技術的適合の実現可能性確認 |

## 個人 KPI / 達成感

- HIPAA / SOX 要件評価完了: 経営方針決定から 30 営業日以内
- ギャップ分析の完全性: 全規制要件の 100% をカバー
- 法務 DPO sign-off 前の外部法務顧問レビュー率: 100%

## 工数 / 関与人数 / コスト感

- HIPAA 要件評価: 3〜5 営業日
- SOX 要件評価: 2〜4 営業日
- ギャップ分析 + 前提条件まとめ: 3〜5 営業日
- 関与人数: 4〜6 名（法務 DPO + 外部法務顧問 + security + tier1 + 経営）

## 前提

- HIPAA / SOX の基礎知識を持つ外部法務顧問がいる、またはアクセス可能
- 現在の k1s0 アーキテクチャの詳細（PII class / データ保持ポリシー / audit trail 仕様）が文書化されている
- `v2_legal_assessment.lock.yaml` が build artifact として存在する

## 流れ

1. **HIPAA 要件評価**: HIPAA Privacy Rule（PHI の定義・利用・開示制限）と Security Rule（administrative / physical / technical safeguards）を評価し、k1s0 の現状との対応を確認する
2. **SOX 要件評価**: SOX Section 302（CEO/CFO 認証）/ Section 404（内部統制）の要件を評価し、audit trail 保持要件（7 年）と現在のデータ保持ポリシーとのギャップを特定する
3. **ギャップ分析**: HIPAA / SOX の要件と k1s0 アーキテクチャのギャップを一覧化する（例: PHI の PII class への対応 / audit trail 7 年保持 / BAA 締結要件）
4. **外部法務顧問レビュー**: ギャップ分析を外部法務顧問にレビューしてもらい、解釈の正確性を確認する
5. **前提条件まとめ**: v2 開発着手前に必要な法的条件をまとめる（BAA テンプレート作成 / SOX 内部統制フレームワーク定義 / 保持ポリシー変更等）
6. **`v2_legal_assessment.lock.yaml` 更新**: 評価結果と前提条件を YAML に記録する
7. **経営報告 + sign-off**: 評価結果と前提条件を経営に報告し、v2 開発着手の法務 DPO sign-off を取得する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| Day 0 | 法務 DPO | HIPAA / SOX 要件評価開始 | — |
| Day 10 | 法務 DPO | HIPAA 要件評価完了 | — |
| Day 15 | 法務 DPO | SOX 要件評価完了 | — |
| Day 20 | 法務 DPO + 外部法務顧問 | ギャップ分析 外部レビュー | — |
| Day 25 | 法務 DPO | 前提条件まとめ・v2_legal_assessment 更新 | — |
| Day 30 | 法務 DPO + 経営 | 経営報告・sign-off | 「[v2 法務] HIPAA / SOX 評価完了。前提条件 8 件。sign-off 取得」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 品質検査結果配信 | 高 | HIPAA 対応では品質検査 PHI の厳密な利用制限が必要 |
| FA 生産指示 | 中 | SOX 対応では生産指示の内部統制証明が必要 |
| 受注 sub | 高 | SOX 対応では受注データの audit trail 7 年保持が必要 |

## 通知 deadline timeline

- 通知 deadline は本シナリオには適用されない（v2 開発の前提条件確定が目的）

## 監督官庁との接点

- **HIPAA**: HHS OCR（Office for Civil Rights）が enforcement 機関
- **SOX**: SEC（Securities and Exchange Commission）/ PCAOB（Public Company Accounting Oversight Board）が audit 機関
- **接点内容**: v2 対応後の BAA 締結 / SOX 内部統制評価のタイミングで外部監査人との接点が発生する

## dual sign-off 規律

- 法務 DPO（HIPAA / SOX 要件の法的妥当性確認）+ 外部法務顧問（専門規制解釈の確認）

## 関連適合仕様 / 関連 OSS

- security 強制機構（HIPAA Security Rule の技術的実装確認）
- 関連 OSS: ClickHouse（audit trail 7 年保持確認）/ Backstage TechDocs（要件チェックリスト）/ GitHub（v2_legal_assessment.lock.yaml）

## 期待結果 / 観測指標

- HIPAA / SOX の要件評価が 30 営業日以内に完了している
- `v2_legal_assessment.lock.yaml` に全要件のギャップ分析と前提条件が記録されている
- 経営と法務 DPO の sign-off が取得されている

## 失敗時の挙動 / escalation

- **HIPAA / SOX の解釈で社内判断が困難**: 外部法務顧問を早期に投入し、専門解釈を取得する
- **ギャップ分析で大規模なアーキテクチャ変更が必要と判明**: 経営に対して v2 開発スケジュールの延長または投資増加を提言する
- **外部法務顧問のレビューで評価の誤りが発見**: 評価を修正し、前提条件を再設定する

## 失敗パターン (anti-pattern)

1. **「HIPAA は US だから関係ない」と評価をスキップする**: 日本法人が US 医療機関と取引する場合でも HIPAA BAA が必要になるケースがある
2. **SOX を「大企業だけの問題」として評価から除外する**: k1s0 のテナントが上場企業の場合は SOX 適合が要求される可能性がある
3. **外部法務顧問レビューなしに法務 DPO 単独で sign-off する**: HIPAA / SOX は高度に専門的な規制であり、外部専門家の確認なしの単独判断はリスクが高い

## 関連参照

- [法務 DPO 担当者シナリオ index](README.md)
- [業界規制_監督官庁対応](06_業界規制_監督官庁対応.md)
- [商用PenTest結果_法務review](07_商用PenTest結果_法務review.md)
