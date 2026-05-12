---
id: plan.data.scenario_archive_restore
axis: data
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.data.data_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [C, E]
  proof_classes: []
---

# archive_to_offline 復元

## 一文方針

archive_to_offline に送出済みのデータを litigation hold 対応・監査要求・障害調査のため online に復元し、integrity 確認・audit hash chain への記録・dual reviewer sign-off を完結させてから業務利用可能な状態で提供する。

> 午前 11 時、本社 IT 室の data 担当者（シニア級）が Mattermost `#data-ops` で法務部門からの「監査機関より 3 年前の受注データ参照要求が届いた」メッセージを確認する。手元には `data_lifecycle.lock.yaml` と OpenBao 管理画面、Mattermost 越しに security 担当者・infra 担当者・法務 / コンプライアンス担当者・dual reviewer がいる。

## Trigger（発火条件）

法務部門からの litigation hold 通知、外部監査人からの特定期間データ参照要求、または過去データを用いた障害再現調査依頼が届いた時。

## 想定頻度 / 典型きっかけ

想定頻度: 不定期（年間 0-3 件程度）。典型きっかけ: 「監査機関から 3 年前の受注データの参照要求があり、archive_to_offline 済みの cold data を online に復元して外部監査人に提供する必要が生じた」「製造ライン障害の根本原因調査のため 1 年前の SCADA テレメトリを復元して tier2 担当者が分析できる状態にする必要が生じた」

## 主役 / 関与者

- 主役: data 担当者（シニア級）
- 関与: security 担当者（DEK の alive 確認 / 復元承認）
- 関与: infra 担当者（offline media からの転送経路確保）
- 関与: 法務 / コンプライアンス担当者（litigation hold の適法性確認）
- 承認: dual reviewer（data 担当者 2 名、PR author 不可）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（data）| シニア | 本社 IT 室 | `data_lifecycle.lock.yaml` / Mattermost `#data-ops` | 復元要求確認・DEK alive 確認要請・転送実施・audit emit・lock.yaml 記録 |
| 関与（security）| シニア | 本社 / リモート | OpenBao 管理画面 | DEK alive 確認・復元承認 |
| 関与（infra）| シニア | 本社 IT 室 / リモート | Argo CD / Kyverno | offline media からの転送経路確保 |
| 関与（法務 / コンプライアンス）| — | 本社 | Mattermost `#legal` | litigation hold の適法性確認 |
| 承認（dual reviewer）| シニア | 本社 / リモート | Mattermost `#data-ops` | 変更 PR sign-off（data 担当者 2 名、author 不可） |

## 前提

- 対象データの DEK が OpenBao で revoke されていない（crypto-erase 済みデータは復元不可能）
- `data_lifecycle.lock.yaml` に archive_to_offline 操作の記録が存在し、offline media の場所が特定可能
- audit hash chain が稼働しており、復元操作の emit が保証済み
- 復元先の online ストレージ（Rook+Ceph cold tier）に十分な容量がある
- **crypto-erase 済みデータは物理的に復元不可能**: DEK revoke 後の data は暗号化されたまま DEK が存在しないため復号できない。crypto-erase と archive_to_offline は別操作（[04_crypto_erase_archive_to_offline.md](./04_crypto_erase_archive_to_offline.md) 参照）

## 流れ

1. 要求内容を確認する（対象テナント / 期間 / データ種別 / 利用目的 / 提供先）
2. security 担当者に DEK の alive 状態を確認してもらう（OpenBao で対象 DEK が revoke されていないこと）
   - DEK が revoke 済みの場合: 復元不可として要求元に通知し、法務担当者に説明する
3. `data_lifecycle.lock.yaml` で対象データの offline media 保管場所（tape / S3 Glacier 等）を確認する
4. infra 担当者と協力して offline media から Rook+Ceph cold tier への転送経路を確保する
5. データ転送を実行する
   - 転送完了後に checksum を確認する（archive_to_offline 時に記録した checksum と比較）
   - integrity 確認が失敗した場合は転送を中断し escalation する（手順「失敗時の挙動」参照）
6. DEK を使って復号確認（データが正常に読み取れることをサンプル確認する）
7. audit hash chain に復元操作を emit する（対象 tenant / データ範囲 / 要求元 / 目的を記録）
8. 復元データへのアクセス権を要求元（外部監査人 / 業務担当者）に付与する（litigation hold の場合は read-only 権限のみ）
9. `data_lifecycle.lock.yaml` に復元操作の記録を追記する（復元日時 / 要求元 / 目的 / 提供先 / 削除予定日）
10. dual reviewer sign-off を取得する
11. 利用期間終了後: 復元した online copy を crypto-erase して cold tier から削除し `data_lifecycle.lock.yaml` に記録する（restore はあくまで一時的な online 化）

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（data は全業務の PostgreSQL / Kafka / ClickHouse の永続化基盤を担うため）。特に影響度が高い 2 業務:

- **品質検査（GMP 監査）**: 医薬品 GMP 規制の監査要求に応じて検査記録を archive から復元するケースが最多であり、7 年保持期間の archive_to_offline データの integrity が監査に直結する。
- **受注管理（litigation hold）**: 法的紛争時の受注データ参照要求（litigation hold）への対応が典型例であり、DEK の alive 確認と read-only 権限付与の正確な手順が法的リスク管理の根幹となる。

## 関連適合仕様 / 関連 OSS

- データ保全適合仕様: [../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)
- data 強制機構: [../../../04_詳細設計/02_強制機構/05_data強制機構.md](../../../04_詳細設計/02_強制機構/05_data強制機構.md)
- 関連 OSS: OpenBao（DEK 管理）/ Rook+Ceph（online ストレージ）/ audit hash chain（操作の証跡）

## 期待結果 / 観測指標

- artifact: `data_lifecycle.lock.yaml` に復元操作エントリが追記済み（対象期間 / 要求元 / 目的 / 提供先 / 削除予定日）
- audit: audit hash chain に復元操作が emit 済みであることを security 担当者が確認
- integrity: 転送後 checksum が archive_to_offline 時と一致（`sha256sum` で照合）
- access: 要求元（外部監査人 / 業務担当者）が対象データを read できることを確認
- sign-off: dual reviewer（data 担当者 2 名）sign-off 完了

## 失敗時の挙動 / escalation

- **DEK が revoke 済みで復元不可能**: 要求元に復元不可を通知し、法務 / コンプライアンス担当者と対応を協議する。audit hash chain に revoke 済みのため復元不可である旨を記録する（**SLA: 2h 以内**）。
- **checksum 不一致（integrity 違反）**: 転送を中断し、data 担当者 + security 担当者 + infra 担当者で Mattermost `#data-incident` に集合（**SLA: 30 分以内**）。offline media の破損を確認し、冗長コピー（tape 複製 / Glacier redundant copy）から再転送を試みる。**postmortem 期限: 3 営業日以内**。
- **audit hash chain への emit 失敗**: 復元操作を一時停止し、security / ops 担当者に即時 escalate（**SLA: 1h 以内**）。Backstage runbook `audit-chain-integrity-check` を参照。復元手続きはaudit emit が保証されるまで完結としない。
- **offline media の所在が不明**: `data_lifecycle.lock.yaml` に archive 記録がない場合は infra 担当者と共同で media 台帳を確認する（**SLA: 4h 以内**）。台帳に記録がない場合は compliance incident として法務担当者に報告する。

## 関連参照

- [data 担当者シナリオ index](./README.md) — data 担当者シナリオ全体の構成と 5 preservation_class 一覧
- [crypto-erase / archive_to_offline](./04_crypto_erase_archive_to_offline.md) — archive_to_offline 操作（本シナリオの逆方向）の詳細手順
- [restore drill](./03_restore_drill.md) — 定期的な restore drill シナリオ（本番 restore と手順共通）
- [KEK / DEK rotation](./05_暗号化変更_KEK_DEK_rotation.md) — DEK alive 確認に関連する鍵管理の全体像
- [データ保全適合仕様](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md) — lifecycle 単一経路と archive 管理の設計根拠
