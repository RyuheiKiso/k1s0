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

## ペルソナ要約

主役: tier2 担当者（中堅級）、目的: 業務データの retention 期限到来を受けて対象テナント・業務区分の DEK revoke 承認を data 担当者に依頼し crypto-shred 完了を audit hash chain で確認する

## 現状業務での痛み

- retention 期限の到来を手動でカレンダー管理しており、見落としが発生して法定保存期間超過のリスクがある
- DEK revoke の依頼書フォーマットが標準化されておらず、情報の抜け漏れで手戻りが発生する
- audit hash chain への crypto-shred 操作の emit が確認されないまま「完了」とされ、コンプライアンス証跡が不完全になる
- 法務確認のプロセスが曖昧で、法的根拠が不明なまま crypto-shred が実施されるリスクがある

## k1s0 でこう変わる

- Backstage retention calendar プラグインがアラートを自動発火し、期限到来の見落としを排除する
- crypto-shred 依頼書フォーマットが標準化され、テナント名 / 業務区分 / 期間 / retention 根拠条文が必須フィールドとして明文化される
- audit hash chain への emit 確認を security 担当者と共同実施し、コンプライアンス証跡の完全性を担保する
- 法務確認を crypto-shred 実施の物理前提条件として設定し、根拠不明の crypto-shred を構造的に防止する

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

## 個人 KPI / 達成感

- `data_lifecycle.lock.yaml` に crypto-shred 完了エントリが追記済み
- audit hash chain に crypto-shred 操作が emit 済み（security 担当者確認）
- Backstage retention calendar のアラートが「完了」状態
- dual reviewer（tier2 担当者 2 名）sign-off 完了

## 工数 / 関与人数 / コスト感

- 通常: 半日〜1 日（法務確認・依頼書作成・DEK revoke 依頼・audit 確認）、関与 5〜6 名（主役 + data 担当者 + security 担当者 + 法務担当者 + dual reviewer 2 名）
- 重大（audit chain emit 失敗・DEK revoke 失敗）: +半日、関与 5〜6 名（即時 escalate）
- 法務確認待ち（期限不明確）: +5 営業日、データは保留

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

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | tier2 担当者 | retention calendar アラート確認・対象テナント / 業務区分 / 期間 特定 | `A 工場 受注データ 7 年期限到来 / 対象: 2017-2019 受注データ` |
| 1 日 | 法務担当者 | retention 期限の法的根拠確認（商事法 / 民法 等）| `法的根拠確認 OK: 商法 36 条 / 保存期間満了確認` |
| 2 日 | tier2 担当者 | 依頼書作成・data 担当者 + security 担当者に DEK revoke 依頼 | `#data-lifecycle DEK revoke 依頼送付 / 依頼書: 添付済` |
| 3 日 | data 担当者 + security 担当者 | DEK revoke 実施・audit hash chain emit 確認 | `DEK revoke 完了 / audit hash chain emit 確認済` |
| 3 日 | tier2 担当者 | Backstage retention calendar「完了」更新・dual sign-off | `retention calendar: 完了 / dual sign-off 完了` |

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

## 失敗パターン (anti-pattern)

- **法務確認なしに crypto-shred を実施**: 法定保存期間がまだ残っているデータを誤って crypto-shred し、法務リスクが発生する。法務担当者の法的根拠確認を DEK revoke 依頼書の必須記載事項とし、根拠条文が空欄の依頼書を受け付けない。
- **audit hash chain への emit を確認せずに「完了」とする**: コンプライアンス証跡が不完全なまま「完了」と記録され、監査時に証跡不備として指摘される。audit hash chain への emit を security 担当者と共同確認し、確認完了後に初めて Backstage アラートを「完了」に更新する。
- **crypto-shred 後にデータが実際に不可読か確認しない**: DEK revoke は成功したが別の経路で暗号化されていないデータが残存しているケースを見落とす。`data_lifecycle.lock.yaml` の完了エントリに「不可読性確認済み」フィールドを追加し、data 担当者の確認を必須とする。

## 関連参照

- [tier2 担当者シナリオ index](./README.md) — tier2 担当者シナリオ全体の構成
- [crypto-erase / archive_to_offline（data-04）](../05_data担当者シナリオ/04_crypto_erase_archive_to_offline.md) — data 担当者側の DEK revoke 実施手順（本シナリオから依頼する対象手順）
- [archive_to_offline 復元（data-13）](../05_data担当者シナリオ/13_archive_to_offline復元.md) — archive_to_offline 済みデータを crypto-shred する場合は事前に復元フローを確認
- [データ保全適合仕様](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md) — retention_class と crypto-shred の設計根拠
