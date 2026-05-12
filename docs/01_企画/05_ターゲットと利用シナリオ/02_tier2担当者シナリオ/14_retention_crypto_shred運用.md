---
id: plan.tier2.scenario_retention_crypto_shred
axis: tier2
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.tier2.tier2_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, E]
  proof_classes: []
---

# 業務データ retention・crypto-shred 運用

## 一文方針

tier2 担当者が業務データの retention 期限到来または業務終了イベントを受けて、対象テナント・業務区分の DEK revoke 承認を data 担当者に依頼し、crypto-shred（物理データを残しつつ DEK 削除で実質的に不可読化）の完了を audit hash chain で確認する。

> 月末の金曜午後、中堅 tier2 担当者が Backstage の retention calendar アラートで「A 工場テナントの受注アーカイブデータが 7 年保持期限に到達」と通知を受け取る。手元には Backstage プラグイン画面と Mattermost、対応する data 担当者が `#data-lifecycle` チャンネルで待機している。

## Trigger（発火条件）

業務データの retention 期限到来通知（Backstage retention calendar アラート）または、業務終了・テナント縮小に伴い特定業務区分のデータを crypto-shred する業務判断が下された時。

## 想定頻度 / 典型きっかけ

- 想定頻度: 月次〜四半期（retention policy の期限到来時）
- 典型きっかけ: 「A 工場テナントの受注データが 7 年保持期限に到達し、法務部門の確認後に crypto-shred を実施する必要が生じた」「製造ライン廃止に伴い、当該ラインの SCADA テレメトリデータを業務終了日から 2 年後に crypto-shred する計画が策定された」

## 主役 / 関与者

- 主役: tier2 担当者（中堅級、業務データ retention の業務オーナー）
- 関与: data 担当者（DEK revoke 実施担当）
- 関与: security 担当者（DEK revoke の承認）
- 関与: 法務 / コンプライアンス担当者（retention 期限の法的根拠確認）
- 承認: dual reviewer（tier2 担当者 2 名）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（tier2）| 中堅 | 本社 IT 室 | Backstage retention calendar | DEK revoke 依頼書作成 / data 担当者との調整 / audit 確認 |
| 関与（data）| シニア | 本社 IT 室 / リモート | OpenBao / `data_lifecycle.lock.yaml` | DEK revoke 実行 / audit emit 確認 |
| 関与（security）| シニア | 本社 / リモート | OpenBao policy dashboard | DEK revoke の承認 |
| 関与（法務）| — | 本社 法務部 | — | 保持期限の法的根拠確認 |

## 前提

- Backstage retention calendar プラグインが稼働しており、業務データの期限アラートが発火済み
- 対象データの業務区分・テナント・期間が業務 DB schema の `retention_class` フィールドで識別可能
- data 担当者側で crypto-erase シナリオ（data-04）の手順が確立済み
- audit hash chain が稼働しており、crypto-shred 操作の emit が保証されている

## 流れ

1. Backstage retention calendar のアラートを確認し、対象テナント・業務区分・データ期間を特定する
2. 法務 / コンプライアンス担当者に retention 期限の法的根拠を確認する（GMP / HACCP / 民法 商事法定保存期間など）
3. crypto-shred 依頼書を作成する（対象: テナント名 / 業務区分 / 期間 / データ種別 / retention 根拠条文 / 依頼者名）
4. data 担当者に Mattermost `#data-lifecycle` で DEK revoke を依頼する（依頼書を添付）
5. security 担当者に DEK revoke 承認を依頼する
6. data 担当者が DEK revoke を実施（data-04 crypto-erase 手順に従う）
7. audit hash chain に crypto-shred 操作が emit されたことを security 担当者と共同確認する
8. `data_lifecycle.lock.yaml` の更新（data 担当者が実施）を確認する
9. Backstage の retention calendar アラートを「完了」に更新する
10. dual reviewer sign-off を取得する

## 業界 9 業務との紐付け

- **受注管理**: 受注データの 7 年商事記録保存義務満了後の crypto-shred が主な対象。受注 DB の DEK を revoke することで取引記録が実質削除される
- **品質検査結果**: ISO 9001 の検査記録保持義務（最低 3 年）を満たした後の crypto-shred。医薬品 GMP の場合は 15 年保持が必要なため期限判定に注意
- **SCADA テレメトリ**: 製造ライン廃止後の長期テレメトリデータの crypto-shred。大容量のため archive_to_offline 済みデータの場合は data-04 と data-13 を組み合わせる

## 関連適合仕様 / 関連 OSS

- データ保全適合仕様: [データ保全適合仕様](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)
- 関連 OSS: OpenBao（DEK 管理）/ Backstage（retention calendar プラグイン）/ audit hash chain

## 期待結果 / 観測指標

- artifact: `data_lifecycle.lock.yaml` に crypto-shred 完了エントリが追記済み（対象テナント / 業務区分 / 期間 / 実施日）
- audit: audit hash chain に crypto-shred 操作が emit 済み（security 担当者確認）
- artifact: Backstage retention calendar のアラートが「完了」状態
- sign-off: dual reviewer（tier2 担当者 2 名）sign-off 完了

## 失敗時の挙動 / escalation

- **法務部門が retention 期限の法的根拠を確認できない**: crypto-shred を保留し、法務部門の判断を待つ（**SLA: 5 営業日以内**）。保留期間中はデータを存続させ、アラートを「確認中」状態に更新する
- **DEK revoke が失敗する**: data 担当者 + security 担当者で Mattermost `#data-incident` に集合し、OpenBao の状態を確認する（**SLA: 30 分以内**）
- **audit hash chain への emit が失敗**: crypto-shred 操作を一時停止し、security / ops 担当者に即時 escalate（**SLA: 1h 以内**）。Backstage runbook `audit-chain-integrity-check` を参照

## 関連参照

- [tier2 担当者シナリオ index](./README.md) — tier2 担当者シナリオ全体の構成
- [crypto-erase / archive_to_offline（data-04）](../05_data担当者シナリオ/04_crypto_erase_archive_to_offline.md) — data 担当者側の DEK revoke 実施手順（本シナリオから依頼する対象手順）
- [archive_to_offline 復元（data-13）](../05_data担当者シナリオ/13_archive_to_offline復元.md) — archive_to_offline 済みデータを crypto-shred する場合は事前に復元フローを確認
- [データ保全適合仕様](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md) — retention_class と crypto-shred の設計根拠
