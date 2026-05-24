---
id: plan.data.scenario_encryption_kek_dek_rotation
axis: data
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.data.data_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [D, E]
  proof_classes: []
---

# KEK / DEK rotation

## 一文方針

[DEK / KEK 階層](../../../04_詳細設計/01_適合仕様/05_鍵管理適合仕様.md) の定期ローテーションを 3 層暗号化の構造を崩さずに完結させ、re-encryption 100% 完了後に旧 DEK を revoke し、tier2 integration test で復号正常を確認してから dual reviewer sign-off まで到達する。

> 月末の朝 10 時、本社 IT 室の data 担当者（シニア級）が `encryption_rotation.lock.yaml` を確認し、infra 担当者から年次 KEK shamir ceremony 完了の Mattermost 通知を確認する。手元には Argo CronWorkflow の管理画面と OpenBao の vault 操作端末、Mattermost 越しに infra 担当者・tier2 担当者・dual reviewer がいる。

## ペルソナ要約

主役: data 担当者（シニア級）、目的: KEK / DEK rotation を定期 cadence で実施し長期残存 KEK によるリスクを排除する

## 現状業務での痛み

- rotation 未実施の KEK が長期残存し、鍵漏洩時の影響範囲が時間とともに拡大する
- rotation スケジュールが文書管理で属人化し、担当者が変わると rotation が長期間未実施になる
- rotation 後のデータ再暗号化が手動で、rotation 完了の確認に時間がかかる

## k1s0 でこう変わる

- rotation.lock.yaml が rotation スケジュールを管理し、期限超過が CI で自動検知される
- OpenBao の auto-rotation が KEK の定期更新を自動実行し、属人化を排除する
- rotation 後の DEK 再暗号化が自動実行され、rotation 完了の CI validation で全データの再暗号化を確認できる

## Trigger（発火条件）

KEK / DEK の定期ローテーション、または暗号アルゴリズム（AES-256-GCM から post-quantum への移行候補）の評価が必要になった時。

## 想定頻度 / 典型きっかけ

想定頻度: KEK rotation = 年次。典型きっかけ: 「年次 KEK shamir ceremony が完了し、新 KEK で全 DEK を再ラップする batch job を起動した」

## 主役 / 関与者

- 主役: data 担当者（シニア級）
- 関与: infra 担当者（KEK の Shamir ceremony が先行する。infra 担当者が KEK rotation を実施してから data 担当者が DEK rotation を行う）
- 関与: tier2 担当者（re-encrypted data の integration test 実行）
- 関与: dual reviewer（sign-off）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（data）| シニア | 本社 IT 室 | `encryption_rotation.lock.yaml` / Argo CronWorkflow | DEK rotation 実施・re-encryption 進捗管理・旧 DEK revoke |
| 関与（infra）| シニア | 本社 IT 室 / リモート | OpenBao 管理画面 / Argo CD | KEK shamir ceremony 実施（先行）・OpenBao 状態管理 |
| 関与（tier2）| ミドル〜シニア | 本社 / リモート | Backstage TechDocs | re-encrypted data の integration test 実行 |
| 承認（dual reviewer）| シニア | 本社 / リモート | Mattermost `#data-ops` | sign-off レビュー |

## 個人 KPI / 達成感

- KEK 残存期間が rotation.lock.yaml で定量管理され、リスク削減の達成感を継続的に得られる
- rotation 完了率 100% の達成を lock.yaml で確認できるようになり、鍵管理品質向上を実感できる

## 工数 / 関与人数 / コスト感

- 工数: 半日〜1 日/rotation（OpenBao 設定確認 1h + rotation 実施 2h + 再暗号化確認 1h）
- 関与人数: 3 名（data 担当者・security 担当者・dual reviewer）
- コスト感: 低。OpenBao auto-rotation により継続運用コストが監視のみになる

## 前提

- 3 層暗号化が確立済み: in-transit（mTLS）/ at-rest（AES-256-GCM L1+）/ application-layer envelope
- KEK は OpenBao に保管済み。Shamir's Secret Sharing で分散管理（infra 担当者シナリオ参照）
- DEK は envelope 暗号化で各 record に付属
- Argo CronWorkflow で batch job の実行基盤が整備済み
- `encryption_rotation.lock.yaml` が存在し、rotation 進捗の記録フォーマットが定義済み

## 流れ

1. ローテーション対象を確認する: **KEK**（infra 担当者の KEK shamir ceremony が先行）/ **DEK**（data 担当者が実施）。KEK rotation が完了していることを確認してから DEK rotation を開始する
2. 新 DEK を生成し、新 KEK で wrap した新 envelope を全 record に書き込む（re-encryption）
3. re-encryption は batch job（Argo CronWorkflow）で実施する。進捗を `encryption_rotation.lock.yaml` に継続的に記録する
4. **旧 DEK と旧 envelope の purge**: re-encryption が 100% 完了したことを確認してから、古い DEK を OpenBao から revoke する
5. 動作確認: application layer で re-encrypted data が正常に復号できることを tier2 integration test で確認する
6. dual reviewer sign-off を得たうえで `encryption_rotation.lock.yaml` を完了状態に更新する

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | data 担当者 | rotation.lock.yaml で rotation cadence 到来を確認し OpenBao rotation を開始 | `KEK rotation 開始 / 対象: kek-XXXXXXXX / T+0` |
| 30分 | data 担当者 | 新 KEK が生成されたことを OpenBao で確認し DEK 再暗号化を開始 | `新 KEK 生成確認 / DEK 再暗号化開始` |
| 2h | data 担当者 | 全 DEK 再暗号化完了を CI validation で確認し lock.yaml に記録 | `再暗号化完了 / lock.yaml rotation 記録更新` |
| 1d | dual reviewer + security 担当者 | rotation 結果と lock.yaml を確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

全 9 業務に共通基盤として影響（data は全業務の PostgreSQL / Kafka / ClickHouse の永続化基盤を担うため）。特に影響度が高い 2 業務:

- **受注管理**: 受注データの暗号化鍵 rotation は取引記録の機密性維持に直結し、re-encryption 完了前後で受注処理の復号が途切れないことが RTO に関わる。
- **品質検査**: GMP 規制要件として暗号化の継続性が監査対象となるため、rotation の全フェーズが audit hash chain に記録されている必要がある。

## 関連適合仕様 / 関連 OSS

- 鍵管理適合仕様: [../../../04_詳細設計/01_適合仕様/05_鍵管理適合仕様.md](../../../04_詳細設計/01_適合仕様/05_鍵管理適合仕様.md)
- KEK shamir distribution: [../../../04_詳細設計/03_クロスカッティング適合仕様/02_KEK_shamir_distribution.md](../../../04_詳細設計/03_クロスカッティング適合仕様/02_KEK_shamir_distribution.md)
- 関連 OSS: OpenBao / Argo Workflows

## 期待結果 / 観測指標

- re-encryption が全 record に対して 100% 完了している
- 旧 DEK が OpenBao から revoke 済み
- tier2 integration test が全ケース pass（re-encrypted data の復号正常）
- `encryption_rotation.lock.yaml` が完了状態に更新済み
- dual reviewer 2 名の sign-off が記録済み

## 失敗時の挙動 / escalation

- **re-encryption batch job 中断**: `encryption_rotation.lock.yaml` の進捗記録を基に中断箇所から resume する。resume できない場合はジョブを再実行する（べき等性を確保した実装が前提）
- **integration test fail（復号不能）**: 旧 DEK の revoke を即時停止し、原因を特定する。旧 DEK が revoke 前であれば旧 envelope で読み取り可能な状態に戻せる
- **KEK rotation 未完了での DEK rotation 開始**: infra 担当者に KEK shamir ceremony の完了を確認してから再開する。誤って開始した場合は rotation を中止する
- **re-encryption batch job 途中失敗**: 部分的に新 DEK で書き直されたデータと旧 DEK のデータが混在する危険。data 担当者が即座に batch job を停止し、infra 担当者と ops 担当者に Mattermost `#data-incident` で報告（**SLA: 15 分以内**）。Backstage runbook `dek-rotation-incident` を参照。**postmortem 期限: 2 営業日以内**。
- **KEK revoke 後に復号不能な record が見つかった**: security 担当者に Mattermost `#security-incident` で即時報告（**SLA: 30 分以内**）。

## 失敗パターン (anti-pattern)

- rotation スケジュールの無期限延期: lock.yaml の cadence 超過は CI が ship blocker として検知する
- DEK 再暗号化確認の省略: rotation 後に再暗号化 CI validation を省略すると古い KEK で暗号化されたデータが残存する

## 関連参照

- [data 設計方針](../../../03_概要設計/06_data設計方針/README.md) — 3 層暗号化（in-transit / at-rest / application-layer envelope）の設計思想
- [crypto-erase シナリオ](04_crypto_erase_archive_to_offline.md) — DEK revoke を活用した実質削除（crypto-shred）の手順
- [シナリオ index](README.md) — data 担当者シナリオ全体の構成と preservation_class 一覧
