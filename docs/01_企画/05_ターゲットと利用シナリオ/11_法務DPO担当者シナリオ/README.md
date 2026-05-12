---
id: plan.overview.scenario_legal_dpo_index
axis: overview
phase: plan
kind: index
status: draft
depends_on:
  - plan.target_use_case
  - plan.legal_check
  - arch.security.security_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, D, E]
  proof_classes: []
---

# 法務 DPO 担当者シナリオ INDEX

## 一文方針

法務 DPO 担当者（シニア級、法務専門職 / DPO 認定）が GDPR 72h 通知 / 個人情報保護法 30d 報告 / DSAR 最終承認 / OSS ライセンス承認 / 業界規制対応 sign-off を担う 11 シナリオを 1 ファイル 1 シナリオで列挙する。regulatory deadline の物理遵守と dual sign-off 規律が本軸の核心であり、全シナリオに blameless culture と記録の完全性を通底させる。

## 担当者プロフィール

| 属性 | 内容 |
|------|------|
| 級 | シニア（法務専門職 / DPO 認定） |
| 必須スキル | GDPR / 個人情報保護法 / 業界規制（GMP/ISO 9001/HACCP）/ OSS ライセンス階層 (a)-(e) / HIPAA（v2 候補）/ SOX（v2 候補） |
| 想定人数 | 2-3 名 |
| 責務 | GDPR 72h / 個人情報保護法 30d 通知 deadline / DSAR 最終承認 / OSS ライセンス承認 / 業界規制対応 sign-off |
| 主要デバイス | ラップトップ（GitHub / Backstage / Mattermost 常時参照） |
| 認証手段 | パスワード + WebAuthn（FIDO2 セキュリティキー） |

詳細は [層別エンジニア要件](../../../02_要件定義/05_開発体制要件/01_層別エンジニア要件.md) を参照。

## 通知 deadline 早見表

| 規制 | 対象 | deadline | 通知先 |
|------|------|----------|--------|
| GDPR Art.33 | 監督機関通知 | breach 認識から 72h | EU 監督機関（例: BSI / CNIL） |
| GDPR Art.34 | 本人通知 | 不当な遅延なく | breach 対象本人 |
| 個人情報保護法 | 個人情報保護委員会報告 | 概算報告 30d、確報 60d | 個人情報保護委員会 |
| 個人情報保護法 | 本人通知 | 速やかに（合理的期間） | breach 対象本人 |
| HIPAA（v2 候補） | 監督機関通知 | 60d | HHS OCR |

## OSS ライセンス階層 (a)-(e) 定義

| 階層 | 代表ライセンス | 使用可否 |
|------|--------------|--------|
| (a) Permissive | MIT / Apache 2.0 / BSD | 使用可 |
| (b) Weak Copyleft | LGPL / MPL | 条件付き使用可 |
| (c) Strong Copyleft | GPL v2 / v3 | tier1 には採用不可（L3 閉鎖コードとの混合禁止） |
| (d) Network Copyleft | AGPL v3 | 採用禁止（SaaS 提供形態に適用） |
| (e) Source Available / Proprietary | SSPL / BSL / BUSL | 採用禁止（商用利用制限条項） |

## シナリオ一覧

| # | シナリオ名 | trigger | 想定頻度 | 主たる関連規制 | 種別 |
|---|-----------|---------|---------|--------------|------|
| 01 | [GDPR_72h_breach通知](01_GDPR_72h_breach通知.md) | data breach 発生時 | イベント駆動 | GDPR Art.33/34 | [緊急] |
| 02 | [個人情報保護法30d報告](02_個人情報保護法30d報告.md) | 日本向け data breach 発生時 | イベント駆動 | 個人情報保護法 | [緊急] |
| 03 | [DSAR対応最終承認](03_DSAR対応最終承認.md) | DSAR 申請受領時 | 月次〜イベント駆動 | GDPR Art.15-22 | [計画]+[緊急] |
| 04 | [right_to_be_forgotten_KEK_destroy承認](04_right_to_be_forgotten_KEK_destroy承認.md) | 削除権行使申請受領時 | 月次〜イベント駆動 | GDPR Art.17 | [計画] |
| 05 | [OSSライセンス階層_新規採用承認](05_OSSライセンス階層_新規採用承認.md) | 新規 OSS 採用提案時 | 四半期〜年次 | OSS ライセンス規律 | [計画] |
| 06 | [業界規制_監督官庁対応](06_業界規制_監督官庁対応.md) | 監督官庁査察時 | 年次〜不定期 | GMP / ISO 9001 / HACCP | [計画]+[緊急] |
| 07 | [商用PenTest結果_法務review](07_商用PenTest結果_法務review.md) | PenTest 結果受領時 | 年次〜不定期 | 契約上通知義務 | [計画] |
| 08 | [cross_region_data_locality対応](08_cross_region_data_locality対応.md) | テナント region 指定変更時 | 四半期〜年次 | GDPR / Schrems II | [計画] |
| 09 | [partner契約_DPA_締結](09_partner契約_DPA_締結.md) | 新規 partner 契約時 | 四半期〜年次 | GDPR Art.28 | [計画] |
| 10 | [v2業界pack_HIPAA_SOX_法務評価](10_v2業界pack_HIPAA_SOX_法務評価.md) | v2 業界 pack 検討開始時 | 年次〜不定期 | HIPAA / SOX | [計画] |
| 11 | [cookie_consent_analytics_opt_in_政策変更](11_cookie_consent_analytics_opt_in_政策変更.md) | cookie consent 政策変更時 | 年次〜不定期 | GDPR / ePrivacy | [計画] |

## 法務 DPO 担当者の決定権限境界

| 操作 | 法務 DPO 担当者の権限 | escalation 先 |
|------|---------------------|--------------|
| GDPR 72h 監督機関通知 | 最終承認・送信 | — |
| DSAR 対応の最終承認 | 実施可 | — |
| crypto-shred（KEK destroy）の dual sign-off | 実施可（data 担当者との dual）| — |
| OSS ライセンス階層 (d)(e) 採用拒否 | 単独判断可 | — |
| OSS ライセンス階層 (c) の採用可否 | 法務判断後に tier1 と協働 | tier1 担当者 |
| **新規 DPA 締結（金額 5000 万円超）** | **法務 DPO + 経営承認必須** | 経営 |
| **HIPAA / SOX 対応の最終 sign-off（v2）** | **外部法務顧問と協働** | 外部法務顧問 |

## dual sign-off 規律

本軸の全シナリオで適用される dual sign-off 規律:

- **GDPR 72h / 個人情報保護法 30d**: 法務 DPO + security 担当者（breach 技術詳細の確認）
- **crypto-shred（KEK destroy）**: 法務 DPO + data 担当者（削除対象データの確認）
- **OSS ライセンス承認**: 法務 DPO + tier1 担当者（技術的採用可否の確認）
- **PenTest 法務 review**: 法務 DPO + 外部監査人（security 評価の確認）

## 新規参画者向けオンボーディング

- **Day 1-3**: README 全体読了 → OSS ライセンス階層 (a)-(e) の暗記 → 01（GDPR 72h）+ 02（個人情報保護法 30d）の一文方針と deadline を通読
- **Day 4-7**: 03（DSAR 最終承認）→ 04（KEK destroy 承認）で data 削除フローを把握
- **Week 2**: 05（OSS ライセンス承認）で tier1 担当者との協働フローを確認
- **Week 3-4**: 09（DPA 締結）→ 08（cross-region data locality）で契約法務フローを習得
- **Month 2 以降**: 06（監督官庁対応）→ 07（PenTest 法務 review）→ 10（v2 HIPAA/SOX）は発生時に担当

## シナリオ間の依存関係

```
01 (GDPR 72h breach 通知)
  └─► 02 (個人情報保護法 30d 報告) — 同一 breach が日本法にも適用される場合

01 / 02 (breach 通知)
  └─► 03 (DSAR 最終承認) — breach 後に DSAR 申請が増加するケースがある

04 (KEK destroy 承認)
  └─► 03 (DSAR 最終承認) — 削除権行使後に DSAR で削除完了確認が来るケースがある

05 (OSS ライセンス承認)
  └─► 10 (v2 業界 pack HIPAA/SOX 法務評価) — v2 業界 pack の OSS ライセンス評価

09 (DPA 締結)
  └─► 08 (cross-region data locality) — partner との region 指定が DPA に反映される

10 (v2 HIPAA/SOX 法務評価)
  └─► 09 (DPA 締結) — v2 業界 pack 向け partner との DPA が必要になる
```

## 関連参照

- [層別エンジニア要件](../../../02_要件定義/05_開発体制要件/01_層別エンジニア要件.md)
- [ターゲットと利用シナリオ index](../README.md)
- `arch.security.security_index`
- `plan.legal_check`
