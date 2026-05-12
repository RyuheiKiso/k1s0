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

## Trigger（発火条件）

テナント offboarding（完全データ削除要求）または long-term archive の offline 移行が必要になった時。

## 想定頻度 / 典型きっかけ

想定頻度: crypto-erase = 不定期（テナント offboarding 時、年間 0-5 件）/ archive_to_offline = 月次〜四半期（lifecycle policy で自動）。典型きっかけ: 「A 工場テナントが閉鎖され、全データの crypto-erase と監査記録の保全が求められた」

## 主役 / 関与者

- 主役: data 担当者（シニア級）
- 関与: security 担当者（DEK revoke の承認と audit 確認）
- 関与: infra 担当者（offline media への転送経路確保）
- 関与: dual reviewer（sign-off）

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

## 関連参照

- [data 設計方針](../../../03_概要設計/06_data設計方針/README.md) — lifecycle 単一経路（hot → warm → cold → archive_to_offline → purge）の設計思想
- [KEK / DEK rotation シナリオ](05_暗号化変更_KEK_DEK_rotation.md) — DEK revoke の詳細手順と暗号鍵管理の全体像
- [シナリオ index](README.md) — data 担当者シナリオ全体の構成と preservation_class 一覧
