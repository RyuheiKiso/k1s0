---
id: plan.overview.scenario_legal_dpo_right_to_be_forgotten_kek_destroy
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

# right_to_be_forgotten_KEK_destroy承認

## 一文方針

物理削除（crypto-shred）の dual sign-off（KEK destroy → DEK 失効）を法務 DPO 担当者として承認し、GDPR Art.17 の削除権を技術的に完全履行する。

> 木曜午前 10 時、テナント F の元従業員 G 氏から「GDPR Art.17 に基づく削除権（right to be forgotten）の行使」申請が届く。通常の DB レコード削除では不十分であり、crypto-shred（KEK destroy による DEK 失効）が必要と判断する。法務 DPO 担当者は削除権行使の法的妥当性を確認し、data 担当者との dual sign-off で KEK destroy を承認する。

## ペルソナ要約

主役: 法務 DPO 担当者（シニア級）、目的: 削除権行使申請の法的妥当性を確認し、crypto-shred（KEK destroy）の dual sign-off で GDPR Art.17 を完全履行する

## 現状業務での痛み

- 通常の DB レコード削除では backup にデータが残るため、GDPR Art.17 の「真の削除」に相当しないケースがある
- KEK destroy の技術的な意味（DEK 失効による暗号論的消去）を法務担当者が理解していないため、承認判断が困難
- 削除完了の証明を GDPR 規制当局に提示できる記録が残っていない
- 削除権と保持義務（法的保持要件）の競合判断が属人化している

## k1s0 でこう変わる

- crypto-shred（KEK destroy）の技術説明が Backstage TechDocs に法務 DPO 向けに整備され、承認判断が可能になる
- `crypto_shred_record.lock.yaml` が削除完了の証跡として機能し、規制当局への完了証明に活用できる
- 削除権と保持義務の競合判断フローが Backstage runbook に定義され、属人化が解消される
- dual sign-off が構造化され、法務 DPO + data 担当者の両者の確認が記録として残る

## Trigger

削除権（GDPR Art.17）行使申請受領時

## 想定頻度 / 典型きっかけ / 頻度根拠

- 想定頻度: 月次〜イベント駆動
- 典型きっかけ: 「元従業員・元取引先から GDPR Art.17 に基づく削除権行使申請が届いた」
- 頻度根拠: 月次 0〜3 件程度の削除権申請を想定

## 主役 / 関与者

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|------|---|------------|----------------|----------------------|
| 法務 DPO 担当者（主役） | シニア | 本社 / リモート | DSAR 管理台帳 / Mattermost | 削除権の法的妥当性確認・保持義務競合確認・KEK destroy dual sign-off |
| data 担当者 | シニア | 本社 IT 室 / リモート | ClickHouse / OpenBao | 削除対象データ特定・crypto-shred 実施・dual sign-off |

## 個人 KPI / 達成感

- 削除権対応 30d deadline 遵守率: 100%
- `crypto_shred_record.lock.yaml` への記録完了率: 100%
- 削除完了後の監督機関からの異議申し立て: 0 件

## 工数 / 関与人数 / コスト感

- 法的妥当性・保持義務競合確認: 1〜2 時間
- dual sign-off 手順実施: 30〜60 分
- 関与人数: 2〜3 名（法務 DPO + data 担当者 + 必要時 security）

## 前提

- crypto-shred（KEK destroy）の手順が Backstage TechDocs に整備されている
- `crypto_shred_record.lock.yaml` が build artifact として存在する
- 削除権と法的保持義務（税務記録 7 年 / 労働記録 5 年等）の競合判断フローが定義されている

## 流れ

1. **申請受領 + 台帳登録**: 削除権行使申請を受領し、DSAR 管理台帳に登録する。30d deadline を設定する
2. **法的妥当性確認**: Art.17 の削除権が適用される条件を確認する（処理の目的終了 / 同意撤回 / 異議申し立て）
3. **保持義務競合確認**: 法的保持義務（税務記録 / 労働記録 / 会計記録等）と削除権が競合していないか確認する。競合がある場合は削除を拒否できる範囲を特定する
4. **削除範囲特定（data 担当者と協働）**: 削除対象のデータ範囲（テナント / PII class / 保持期間）を data 担当者と確認する
5. **crypto-shred が必要かの判断**: backup を含む完全削除が必要な場合は crypto-shred（KEK destroy）を選択する
6. **dual sign-off 実施**: 法務 DPO（法的妥当性確認）と data 担当者（技術的削除の正確性確認）が `crypto_shred_record.lock.yaml` に署名する
7. **crypto-shred 実行（data 担当者）**: data 担当者が KEK destroy を実行し、DEK を失効させる
8. **完了確認 + 記録**: `crypto_shred_record.lock.yaml` に削除完了記録を追記し、申請者に完了通知を送付する

## Timeline

| T+ | actor | action | 通知例 |
|----|-------|--------|--------|
| Day 0 | 法務 DPO | 申請受領・台帳登録・30d deadline 設定 | — |
| Day 5 | 法務 DPO | 法的妥当性・保持義務競合確認 | — |
| Day 7 | data 担当者 | 削除対象データ範囲特定 | — |
| Day 10 | 法務 DPO + data 担当者 | dual sign-off 実施 | GitHub: commit（dual sign-off 記録） |
| Day 11 | data 担当者 | crypto-shred（KEK destroy）実行 | 「[crypto-shred] G 氏 KEK destroy 実行完了」 |
| Day 12 | 法務 DPO | 完了確認・申請者への完了通知 | — |
| Day 13 | 法務 DPO | 台帳クローズ | 「[削除権 #43] 完了: Day 13（30d deadline に対し余裕あり）」 |

## 業界 9 業務との紐付け

| 業務名 | 影響度 | 紐付き内容 |
|--------|--------|----------|
| 受注 sub | 高 | 元取引先担当者の個人情報の削除権行使 |
| FA 生産指示 | 中 | 元従業員の勤務関連データの削除権行使 |
| 品質検査結果配信 | 中 | 元検査員の個人情報の削除権行使 |

## 通知 deadline timeline

| deadline | 内容 | 送信先 | 根拠条文 |
|----------|------|--------|--------|
| 30d（延長可: +60d） | 削除権への回答・削除完了通知 | 申請者（データ主体） | GDPR Art.12(3) / Art.17 |

## 監督官庁との接点

- **直接の接点なし**（申請者への対応が主）
- **行政処分リスク**: 削除を不当に拒否した場合や deadline 超過は監督機関への申告リスクがある

## dual sign-off 規律

- 法務 DPO（削除権の法的妥当性確認 + 保持義務競合なしの確認）+ data 担当者（crypto-shred 実施の技術的確認）

## 関連適合仕様 / 関連 OSS

- security 強制機構（KEK destroy の audit emit）
- 関連 OSS: OpenBao（KEK 管理）/ ClickHouse（削除対象データ特定）/ GitHub（dual sign-off 記録）

## 期待結果 / 観測指標

- 削除権対応が 30d deadline 内に完了している
- `crypto_shred_record.lock.yaml` に dual sign-off と削除完了記録が存在する
- 削除完了後に backup を含む対象データが暗号論的に復元不可能になっている

## 失敗時の挙動 / escalation

- **保持義務と削除権が競合し判断できない**: 外部法務顧問に判断を依頼する。SLA: 5 営業日以内に判断取得
- **crypto-shred 実行後に対象データが残存していることが発覚**: data 担当者と security 担当者が原因を特定し、追加の削除操作を実施する。法務 DPO が申請者に経緯を説明する義務がある
- **申請者が削除完了に異議を唱える（DSAR フォローアップ申請）**: 法務 DPO が `crypto_shred_record.lock.yaml` を根拠に削除完了を証明する。証明できない場合は外部法務顧問に対応を依頼する

## 失敗パターン (anti-pattern)

1. **通常の DB レコード削除で削除権対応を完了したと判断する**: backup に残存するデータが GDPR Art.17 の「真の削除」に相当しない場合がある。crypto-shred の必要性を必ず確認する
2. **保持義務の確認をスキップして削除を急ぐ**: 税務記録や労働記録の削除は別の法令違反を引き起こす可能性がある
3. **dual sign-off なしに data 担当者単独で KEK destroy を実行する**: 法的承認なしの削除となり、誤削除時の法的責任が曖昧になる

## 関連参照

- [法務 DPO 担当者シナリオ index](README.md)
- [DSAR対応最終承認](03_DSAR対応最終承認.md)
- [data: crypto_erase_archive_to_offline](../../05_data担当者シナリオ/04_crypto_erase_archive_to_offline.md)
