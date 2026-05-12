---
id: plan.data.scenario_pii_dsar_export
axis: data
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.data.data_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, C, E]
  proof_classes: []
---

# PII DSAR / 個人情報開示請求対応

## 一文方針

data 担当者が GDPR DSAR（Data Subject Access Request）または個人情報保護法に基づく開示請求を受けて、PII 専用クラスタから対象データ主体の個人情報を安全に抽出・暗号化 export し、audit hash chain に記録してから法務担当者を経由して請求者に提供する。

> 水曜午前 10 時、法務部門から「元従業員 B さんから GDPR 第 15 条（アクセス権）に基づく開示請求が届いた」と Mattermost `#legal-request` に投稿が来る。data 担当者（シニア級）は PII 専用クラスタへのアクセス手順を確認しながら、security 担当者に RLS FORCE bypass の承認を依頼する準備を始める。

## ペルソナ要約

主役: data 担当者（シニア級）、目的: DSAR 対応で PII の所在を即時特定し 30 日以内にエクスポートを完了する

## 現状業務での痛み

- DSAR 対応でどのテーブルに PII があるかが不明確で、PII の特定に大量の調査時間がかかる
- PII テーブルの一覧が文書管理で属人化し、担当者が変わると調査が困難になる
- エクスポート後の PII 暗号化が不完全で、エクスポートファイル自体がセキュリティリスクになる

## k1s0 でこう変わる

- pii_catalog.lock.yaml が全 PII テーブルと PII class を管理し、DSAR 対応時の PII 特定が即時に可能になる
- DSAR エクスポートスクリプトが lock.yaml から自動生成され、調査から エクスポートまでが自動化される
- エクスポートファイルが AES-GCM で暗号化され、安全なファイル転送が保証される

## Trigger（発火条件）

法務部門または compliance チームから GDPR DSAR / 個人情報保護法に基づく開示請求の対応依頼が届いた時。

## 想定頻度 / 典型きっかけ

想定頻度: 不定期（月次 0-3 件。GDPR 規制下の従業員 / 顧客データを持つテナントが対象）。典型きっかけ:「元従業員が GDPR 第 15 条（アクセス権）に基づき自分の人事・入退室記録・生産実績データの開示を請求した」「取引先企業の担当者が自分の受注登録履歴データの開示を請求した」

## 主役 / 関与者

- 主役: data 担当者（シニア級、PII 専用クラスタ運用の担当者）
- 関与: security 担当者（RLS FORCE bypass / DEK alive 確認 / audit 検証）
- 関与: 法務 / コンプライアンス担当者（請求の適法性確認 / 提供方法の決定）
- 関与: infra 担当者（PII 専用クラスタへの接続経路確認）
- 承認: dual reviewer（data 担当者 2 名）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（data）| シニア | 本社 IT 室 / リモート | PII 専用クラスタ接続端末 | データ抽出 / 暗号化 export / audit 確認 |
| 関与（security）| シニア | 本社 / リモート | OpenBao / audit hash chain | RLS bypass 承認 / DEK 確認 |
| 関与（法務）| — | 本社 法務部 | Mattermost `#legal-request` | 請求適法性確認 / 提供先への送付 |

## 個人 KPI / 達成感

- DSAR 対応の 30 日期限内完了率を追跡でき、compliance SLO 達成の達成感を得られる
- PII 特定時間の短縮を数値で確認でき、DSAR 対応効率の改善を実感できる

## 工数 / 関与人数 / コスト感

- 工数: 半日〜1 日/DSAR（PII 特定 1h + エクスポート実行 2h + 暗号化・転送 1h）
- 関与人数: 3〜4 名（data 担当者・security 担当者・compliance 担当者・dual reviewer）
- コスト感: 低。lock.yaml と自動エクスポートスクリプトにより DSAR 対応コストが大幅削減される

## 前提

- PII 専用クラスタが独立して稼働しており、一般業務クラスタとは物理 / 論理的に分離されている（data-07 参照）
- 対象データ主体の識別子（従業員 ID / 取引先担当者 ID）が業務 DB と PII 専用クラスタで紐付けられている
- データ主体の DEK が OpenBao で alive（revoke されていない）であること
- audit hash chain が稼働しており、export 操作の emit が保証されている
- export する PII は別の DEK で再暗号化して提供し、請求者以外が復号できない状態で送付する

## 流れ

1. 法務 / コンプライアンス担当者から請求内容を確認する（請求者の識別情報 / 請求根拠条文 / 対象データ範囲 / 回答期限）
   - GDPR の場合: 1 か月以内に回答義務（複雑な場合は 3 か月まで延長可）
2. 請求の適法性を法務担当者と確認する（本人確認 / 正当な請求であることの確認）
3. security 担当者に PII 専用クラスタの対象データ主体の DEK alive 確認を依頼する
4. PII 専用クラスタで対象データ主体の個人情報を抽出する（RLS FORCE を使った一時的な bypass が必要な場合は security 担当者の承認を取得）
   - 対象範囲: 人事データ / 入退室記録 / 受注履歴 / 検査結果 / その他業務 DB に紐付く個人情報
5. 抽出データを請求者専用の DEK で再暗号化し、人間が読める形式（JSON / PDF）でパッケージングする
6. audit hash chain に export 操作を emit する（請求者 ID / 対象テナント / データ範囲 / 提供日時 / 受領者を記録）
7. 暗号化 export パッケージを法務担当者に引き渡す（法務担当者が請求者に安全な経路で送付する）
8. `data_lifecycle.lock.yaml` に DSAR 対応の記録を追記する（請求者 / 対応期日 / 提供内容の概要 / 削除予定日）
9. dual reviewer sign-off を取得する

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | data 担当者 | DSAR 受領 / pii_catalog.lock.yaml で対象テナントの PII テーブルを特定 | `DSAR 受領 / PII テーブル N 件特定 / T+0` |
| 2h | data 担当者 | エクスポートスクリプトを実行し PII データを収集 | `PII エクスポート実行 / N レコード収集` |
| 4h | data 担当者 | エクスポートファイルを AES-GCM で暗号化し audit trail を記録 | `暗号化完了 / audit trail 記録` |
| 1d | data 担当者 + compliance 担当者 | エクスポートファイルを本人に安全転送し DSAR 完了を記録 | `DSAR 完了 / lock.yaml 更新` |

## 業界 9 業務との紐付け

- **品質検査結果**: 検査担当者個人の検査結果記録は個人情報として DSAR 対象になり得る（検査者 ID / 判定結果 / タイムスタンプ）。GMP 規制下では検査記録の開示制限がある場合があるため法務確認が必須
- **FA 生産指示 / 進捗実績**: 生産ライン担当者の作業実績（個人と紐付く生産数 / 作業時間）が DSAR 対象になり得る。日本の個人情報保護法では従業員の業務実績データも個人情報に該当する
- **受注管理**: 取引先担当者の個人情報（氏名 / 連絡先 / 受発注履歴）が GDPR DSAR の主な対象。B2B 取引であっても担当者個人が請求主体となる

## 関連適合仕様 / 関連 OSS

- データ保全適合仕様: [../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md)
- data 強制機構: [../../../04_詳細設計/02_強制機構/05_data強制機構.md](../../../04_詳細設計/02_強制機構/05_data強制機構.md)
- 関連 OSS: PostgreSQL RLS / OpenBao（DEK 管理）/ AES-256-GCM（export 暗号化）

## 期待結果 / 観測指標

- artifact: `data_lifecycle.lock.yaml` に DSAR 対応エントリが追記済み（請求者 / 対応期日 / 提供内容 / 削除予定日）
- audit: audit hash chain に export 操作が emit 済み（security 担当者確認）
- compliance: 請求受領から 1 か月以内（GDPR）または 2 か月以内（個人情報保護法）に法務担当者が回答を送付済み
- sign-off: dual reviewer（data 担当者 2 名）sign-off 完了

## 失敗時の挙動 / escalation

- **DEK が revoke 済みでデータ復号不可能**: 法務担当者に「crypto-erase 済みのため復元不可能」と報告し、GDPR 第 17 条（削除権）の行使と解釈できる旨を説明する（**SLA: 2h 以内**）。audit hash chain に DEK revoke 済みで対応不可の旨を記録する
- **audit hash chain への emit が失敗**: security / ops 担当者に即時 escalate（**SLA: 1h 以内**）。export 操作を一時停止し、Backstage runbook `audit-chain-integrity-check` を参照。compliance incident として扱う
- **1 か月の GDPR 回答期限が近づいている**: 法務担当者と作業状況を共有し、GDPR 第 12 条に基づく 3 か月延長通知を請求者に送付する判断を依頼する（**SLA: 回答期限 7 日前に確認**）
- **RLS FORCE bypass の承認が得られない**: security 担当者に Mattermost `#security-incident` で改めて事情を説明し、代替の権限付き抽出方法を相談する（**SLA: 4h 以内**）

## 失敗パターン (anti-pattern)

- PII を手動検索で特定: lock.yaml を使わない PII 探索は調査時間が膨大になり 30 日期限を超過するリスクがある
- 平文エクスポートファイルの転送: 暗号化なしのエクスポートは security CI が GDPR 違反として検知する

## 関連参照

- [data 担当者シナリオ index](./README.md) — data 担当者シナリオ全体の構成と 5 preservation_class 一覧
- [PII 専用クラスタ運用（data-07）](./07_PII専用クラスタ運用.md) — PII 専用クラスタの構成と RLS 設定（本シナリオの前提クラスタ）
- [crypto-erase / archive_to_offline（data-04）](./04_crypto_erase_archive_to_offline.md) — DSAR「削除権」対応時の crypto-erase 手順
- [archive_to_offline 復元（data-13）](./13_archive_to_offline復元.md) — archive 済み PII データを DSAR で参照する場合の復元フロー
- [データ保全適合仕様](../../../04_詳細設計/01_適合仕様/14_データ保全適合仕様.md) — PII クラスタの preservation_class と export 暗号化要件
