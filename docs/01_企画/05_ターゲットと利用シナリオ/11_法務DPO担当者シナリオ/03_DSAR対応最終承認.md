---
id: plan.overview.scenario_legal_dpo_dsar_final_approval
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

# DSAR対応最終承認

## 一文方針

data subject access request の最終承認 + 業務管理者 02 / data 軸 15 との連携を法務 DPO 担当者として主導し、GDPR Art.15-22 の 30d deadline を厳守する。

> 月曜午前 9 時、法務 DPO 担当者の Mattermost に「テナント D の従業員 E 氏から DSAR（データアクセス権行使）申請が届いた」という通知が届く。業務管理者 02 が申請内容を確認し、data 担当者 15 が ClickHouse から E 氏の PII データを抽出している。法務 DPO 担当者は抽出データの法的妥当性（過不足・サードパーティデータの除外）を確認し、30d deadline 内の提供完了に向けて最終承認を実施する。

## ペルソナ要約

主役: 法務 DPO 担当者（シニア級）、目的: DSAR 申請に対して法的妥当性を確認し、GDPR Art.15-22 の 30d deadline 内に最終承認を完了する

## 現状業務での痛み

- DSAR 申請の受付 → データ抽出 → 法務確認 → 提供のフローが文書化されておらず、担当者間の handover で情報が抜け落ちる
- 抽出データにサードパーティ情報が混入しており、法的に提供できないデータを削除する判断が属人化している
- 30d deadline の管理が散在し、期限が迫ってから気づくケースがある
- DSAR 対応件数が月次で増加している中、法務 DPO の review ボトルネックが解消されていない

## k1s0 でこう変わる

- DSAR 管理台帳が GitHub Project として管理され、申請受領日・30d deadline・対応ステータスが可視化される
- data 担当者 15 の PII data export 機能（シナリオ 15）が DSAR 対応のデータ抽出を標準化し、サードパーティデータ除外も自動化される
- 法務 DPO の最終承認 checklist が Backstage TechDocs に整備され、review 品質が標準化される
- 30d deadline 超過リスクが 7 日前に自動 alert され、escalation が早期化される

## Trigger

DSAR 申請受領時

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 月次〜イベント駆動
- 典型きっかけ: 「テナントの従業員から DSAR 申請が届いた」「breach 後に申請件数が増加した」
- 頻度根拠: 月次 1〜5 件程度の DSAR 申請を想定

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| 法務 DPO 担当者（主役） | シニア | 本社 / リモート | DSAR 管理台帳 / Mattermost | 申請内容確認・法的妥当性チェック・最終承認 |
| 業務管理者（シナリオ 08-02）| シニア | テナント事業部 | Backstage admin UI | 申請者確認・申請内容の業務的妥当性確認 |
| data 担当者（シナリオ 05-15）| シニア | 本社 IT 室 / リモート | ClickHouse | PII データ抽出・サードパーティデータ除外 |

## 個人 KPI / 達成感

- DSAR 対応 30d deadline 遵守率: 100%
- 法的妥当性チェックの完全性スコア（checklist 充足率）: 100%
- DSAR 管理台帳のステータス更新タイムラグ: 1 営業日以内

## 工数 / 関与人数 / コスト感

- 法的妥当性チェック: 1〜2 時間
- 申請者への提供パッケージ確認: 30 分
- 関与人数: 3〜4 名（法務 DPO + 業務管理者 + data 担当者 + 必要時 security）

## 前提

- DSAR 管理台帳（GitHub Project）が存在し、申請受領日・deadline・ステータスが管理されている
- data 担当者 15 の PII data export 機能が正常に動作している
- DSAR 対応 checklist が Backstage TechDocs に整備されている

## 流れ

1. **申請受領 + 台帳登録**: DSAR 申請を受領し、DSAR 管理台帳に登録する。30d deadline を設定する
2. **申請者確認（業務管理者と協働）**: 申請者が「正当なデータ主体」であることを業務管理者と確認する（本人確認）
3. **データ抽出指示（data 担当者へ）**: data 担当者 15 に PII データ抽出を依頼する（ClickHouse クエリ + サードパーティデータ除外）
4. **法的妥当性チェック**: 抽出データの法的妥当性を確認する
   - 提供データの範囲が申請内容と一致しているか
   - サードパーティ（第三者）のデータが除外されているか
   - GDPR Art.15 で提供が義務付けられる情報がすべて含まれているか
   - 提供が制限される情報（法的手続き中 / 商業機密等）が除外されているか
5. **最終承認**: 法的妥当性チェックを完了し、最終承認 sign-off を DSAR 管理台帳に記録する
6. **申請者への提供**: 承認済みのデータパッケージを申請者に安全な方法で提供する
7. **台帳クローズ**: DSAR 管理台帳のステータスを「完了」に更新し、提供日時を記録する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| Day 0 | 法務 DPO | 申請受領・台帳登録・30d deadline 設定 | 「[DSAR #42] 申請受領: 2026-05-12。deadline: 2026-06-11」 |
| Day 3 | 業務管理者 | 申請者確認完了 | — |
| Day 5 | 法務 DPO | data 担当者に抽出指示 | Mattermost DM: 「DSAR #42: E 氏の PII データ抽出をお願いします」 |
| Day 10 | data 担当者 | データ抽出完了・サードパーティデータ除外済み | — |
| Day 12 | 法務 DPO | 法的妥当性チェック完了・最終承認 | — |
| Day 14 | 法務 DPO | 申請者へデータ提供 | — |
| Day 15 | 法務 DPO | 台帳クローズ | 「[DSAR #42] 完了: Day 15（30d deadline に対し余裕あり）」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 品質検査結果配信 | 高 | 検査員の個人情報を含む品質検査データが DSAR 対象 |
| 受注 sub | 高 | 取引先担当者の個人情報を含む受注データが DSAR 対象 |
| FA 生産指示 | 中 | 生産担当者の勤務記録が DSAR 対象になるケース |

## 通知 deadline timeline

| deadline | 内容 | 送信先 | 根拠条文 |
|----------|------|--------|--------|
| 30d（延長可: +60d） | DSAR への回答・データ提供 | 申請者（データ主体） | GDPR Art.12(3) |

## 監督官庁との接点

- **直接の接点なし**（申請者への対応が主）
- **行政処分リスク**: deadline 超過や不完全な回答は監督機関への申告リスクがある

## dual sign-off 規律

- 法務 DPO（法的妥当性確認）+ 業務管理者（業務的妥当性確認）

## 関連適合仕様 / 関連 OSS

- security 強制機構（PII data export の audit emit）
- 関連 OSS: ClickHouse（PII データ抽出）/ GitHub Project（DSAR 管理台帳）/ Backstage TechDocs（checklist）

## 期待結果 / 観測指標

- DSAR 対応が 30d deadline 内に完了している
- DSAR 管理台帳に全申請の対応記録が存在する
- 法的妥当性チェックの checklist がすべて pass している

## 失敗時の挙動 / escalation

- **Day 23（30d deadline の 7 日前）時点で未完了**: Mattermost に自動 alert。法務 DPO が対応状況を確認し、必要に応じて申請者に 60 日間の deadline 延長を通知する（GDPR Art.12(3)）
- **サードパーティデータの除外が不完全**: data 担当者に再抽出を依頼し、法務 DPO が再チェックを実施する
- **申請者の本人確認が取れない**: 業務管理者に追加確認を依頼する。確認取れずの場合は弁護士（外部法務顧問）に判断を仰ぐ

## 失敗パターン (anti-pattern)

1. **DSAR 管理台帳への登録を後回しにする**: deadline 管理ができなくなり、超過リスクが増大する
2. **データ抽出の依頼を「後でまとめて」にする**: data 担当者のデータ抽出には時間がかかるため、申請受領後 5 日以内に依頼する
3. **法的妥当性チェックを形式的に実施する**: サードパーティデータの混入やデータ不足を見落とし、申請者から監督機関への申告につながるリスクがある

## 関連参照

- [法務 DPO 担当者シナリオ index](README.md)
- [right_to_be_forgotten_KEK_destroy承認](04_right_to_be_forgotten_KEK_destroy承認.md)
- [data: PII_DSAR_export対応](../../05_data担当者シナリオ/15_PII_DSAR_export対応.md)
