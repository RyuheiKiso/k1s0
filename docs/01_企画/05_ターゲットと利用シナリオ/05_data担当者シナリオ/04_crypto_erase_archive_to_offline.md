---
id: plan.data.scenario_crypto_erase_archive
axis: data
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.data.data_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [E]
  proof_classes: []
---

# crypto-erase / archive_to_offline

## 一文方針

テナント offboarding による [crypto-shred / crypto-erase](../../../03_概要設計/06_data設計方針/06_ライフサイクル方針.md) と cold data の archive_to_offline を lifecycle 単一経路に従って実施し、全操作が audit hash chain に記録されていることを保証して dual reviewer sign-off まで完結させる。

> 午前 10 時、本社 IT 室の data 担当者（シニア級）が Mattermost `#data-ops` で法務部門からの「A 工場テナント閉鎖に伴う全データの crypto-erase 依頼」メッセージに気付く。手元には `data_lifecycle.lock.yaml` と OpenBao 管理画面、Mattermost 越しに security 担当者・infra 担当者・dual reviewer がいる。

## ペルソナ要約

主役: data 担当者（シニア級）、目的: crypto-erase で KEK destroy による物理削除を実施し GDPR の忘れられる権利に真に対応する

## 現状業務での痛み

- KEK destroy なしの logical delete で GDPR 対応を偽装しており、規制当局の監査で指摘を受けるリスクがある
- 暗号化されていないアーカイブデータが offline ストレージに残存し、将来的な PII 漏洩リスクがある
- crypto-erase の手順が文書管理で属人化し、削除実施の証跡が残らない

## k1s0 でこう変わる

- KEK destroy による crypto-erase が crypto_erase.lock.yaml で管理され、削除の物理的完全性が保証される
- offline archive への crypto-erase が CI の compliance check で確認され、残存 PII の検知が自動化される
- 削除実施の audit trail が audit hash chain に記録され、規制当局への証跡提出が即時に可能になる

## Trigger（発火条件）

テナント offboarding（完全データ削除要求）または long-term archive の offline 移行が必要になった時。

## 想定頻度 / 典型きっかけ

想定頻度: crypto-erase = 不定期（テナント offboarding 時、年間 0-5 件）/ archive_to_offline = 月次〜四半期（lifecycle policy で自動）。典型きっかけ: 「A 工場テナントが閉鎖され、全データの crypto-erase と監査記録の保全が求められた」

## 主役 / 関与者

- 主役: data 担当者（シニア級）
- 関与: security 担当者（DEK revoke の承認と audit 確認）
- 関与: infra 担当者（offline media への転送経路確保）
- 関与: dual reviewer（sign-off）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（data）| シニア | 本社 IT 室 | `data_lifecycle.lock.yaml` / Mattermost `#data-ops` | crypto-erase / archive 操作主導・lifecycle lock.yaml 記録 |
| 関与（security）| シニア | 本社 / リモート | OpenBao 管理画面 / Mattermost `#security-ops` | DEK revoke 承認・audit 確認 |
| 関与（infra）| シニア | 本社 IT 室 / リモート | Argo CD / Kyverno | offline media への転送経路確保 |
| 承認（dual reviewer）| シニア | 本社 / リモート | Mattermost `#data-ops` | sign-off レビュー |

## 個人 KPI / 達成感

- crypto-erase 完了の audit trail が 100% 記録されていることを確認でき、GDPR compliance 達成の達成感を得られる
- KEK destroy 後の復号不能確認が自動検証され、物理削除の完全性を定量的に確認できる

## 工数 / 関与人数 / コスト感

- 工数: 半日〜1 日（KEK destroy 手順確認 1h + crypto-erase 実施 2h + audit 確認 1h）
- 関与人数: 3〜4 名（data 担当者・security 担当者・compliance 担当者・dual reviewer）
- コスト感: 低〜中。手順が lock.yaml で管理されており実施コストは最小化される

## 前提

- lifecycle 単一経路が定義済み: `hot → warm → cold → archive_to_offline → purge`
- crypto-shred（DEK を削除して data を実質的に不可読化）が定義済み
- AES-256-GCM で全 data が暗号化済みであり、DEK は OpenBao で管理済み
- audit hash chain が稼働しており、削除操作の emit が保証済み
- `data_lifecycle.lock.yaml` が存在し、lifecycle 操作の記録フォーマットが定義済み

## 流れ

1. 操作種別を判定する: **crypto-erase**（テナント offboarding）または **archive_to_offline**（cold data の物理 offline 移行）
2. **crypto-erase の場合**: 対象テナントの DEK を OpenBao から revoke する → AES-256-GCM 暗号化済みデータが読み取り不能になる（物理的な purge は不要。DEK 削除で完結する）
3. **archive_to_offline の場合**: cold tier から object store（Rook+Ceph）経由で offline media（tape / S3 Glacier 等）にデータを転送し、転送完了と integrity を確認する
4. 転送完了後、online copy を crypto-erase して hot / warm / cold 経路から切り離す
5. lifecycle 完了の記録を `data_lifecycle.lock.yaml` に追記する
6. 監査確認: 削除・転送操作が audit hash chain に emit されていることを確認する。audit が欠落した offboarding は禁止とする
7. dual reviewer sign-off を得る

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | data 担当者 | crypto_erase.lock.yaml で対象 KEK と影響データを確認 | `crypto-erase 対象確認 / KEK: kek-XXXXXXXX / 影響データ確認` |
| 1h | data 担当者 | OpenBao で KEK destroy を実行し destroy 証跡を取得 | `KEK destroy 完了 / destroy 証跡取得` |
| 2h | data 担当者 | crypto-erase 後の復号不能を確認し audit hash chain への記録を確認 | `復号不能確認 / audit emit 確認` |
| 1d | dual reviewer + security 担当者 | crypto-erase 手順と audit trail を確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（data は全業務の PostgreSQL / Kafka / ClickHouse の永続化基盤を担うため）。特に影響度が高い 2 業務:

- **受注管理**: テナント閉鎖時に受注データの crypto-erase が確実に完了することで、顧客情報漏洩リスクを排除する。audit hash chain への記録が後日の法的証跡となる。
- **品質検査**: GMP 規制対象テナントの閉鎖時には検査記録の crypto-erase と同時に監査証跡の保全が必要であり、compliance 要件との整合が最重要となる。

## 関連適合仕様 / 関連 OSS

- データ保全適合仕様: [../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)
- 関連 OSS: OpenBao（DEK revoke）/ Rook+Ceph（object store）/ ClickHouse（analytics data lifecycle）

## 期待結果 / 観測指標

- crypto-erase: 対象テナントの DEK が OpenBao から revoke 済みであり、data が復号不能であることを確認できる
- archive_to_offline: online copy が存在せず、offline media に完全転送済みかつ integrity 確認済み
- audit hash chain に全操作が記録済み
- `data_lifecycle.lock.yaml` が更新済み
- dual reviewer 2 名の sign-off が記録済み

## 失敗時の挙動 / escalation

- **audit 欠落**: compliance incident として扱い、security / ops 担当者に即時 escalate する。操作を一時停止し audit の欠落原因を特定する。
- **DEK revoke 失敗**: OpenBao の状態を確認し、infra 担当者と協力して revoke を再試行する。revoke 完了まで offboarding を完了とみなさない。
- **offline 転送途中の障害**: 転送済み部分の integrity を確認したうえで、中断箇所から再開する。再開できない場合は全量を再転送する。
- **audit chain に削除操作が記録されていない**: compliance incident として security 担当者 + ops 担当者に Mattermost `#compliance-incident` で即時通報（**SLA: 1h 以内**）。Backstage runbook `audit-chain-integrity-check` を参照。**postmortem 期限: 2 営業日以内**。
- **offline 転送途中の障害**: 転送を中断し data 担当者 + infra 担当者で Mattermost `#data-incident` に集合（**SLA: 15 分以内**）。転送済みデータの完全性確認後に再開。

## 失敗パターン (anti-pattern)

- logical delete のみの GDPR 対応: KEK destroy なしの削除は compliance check が GDPR 違反を検知する
- audit trail なしの KEK destroy: destroy 後に audit emit を確認しないと証跡が欠落し規制対応が無効になる

## 関連参照

- [data 設計方針](../../../03_概要設計/06_data設計方針/README.md) — lifecycle 単一経路（hot → warm → cold → archive_to_offline → purge）の設計思想
- [KEK / DEK rotation シナリオ](05_暗号化変更_KEK_DEK_rotation.md) — DEK revoke の詳細手順と暗号鍵管理の全体像
- [シナリオ index](README.md) — data 担当者シナリオ全体の構成と preservation_class 一覧
