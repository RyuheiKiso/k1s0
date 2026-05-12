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

## Trigger（発火条件）

KEK / DEK の定期ローテーション、または暗号アルゴリズム（AES-256-GCM から post-quantum への移行候補）の評価が必要になった時。

## 想定頻度 / 典型きっかけ

想定頻度: KEK rotation = 年次。典型きっかけ: 「年次 KEK shamir ceremony が完了し、新 KEK で全 DEK を再ラップする batch job を起動した」

## 主役 / 関与者

- 主役: data 担当者（シニア級）
- 関与: infra 担当者（KEK の Shamir ceremony が先行する。infra 担当者が KEK rotation を実施してから data 担当者が DEK rotation を行う）
- 関与: tier2 担当者（re-encrypted data の integration test 実行）
- 関与: dual reviewer（sign-off）

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

## 関連参照

- [data 設計方針](../../../03_概要設計/06_data設計方針/README.md) — 3 層暗号化（in-transit / at-rest / application-layer envelope）の設計思想
- [crypto-erase シナリオ](04_crypto_erase_archive_to_offline.md) — DEK revoke を活用した実質削除（crypto-shred）の手順
- [シナリオ index](README.md) — data 担当者シナリオ全体の構成と preservation_class 一覧
