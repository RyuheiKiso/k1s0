---
id: plan.security.scenario_secret_compromise_drill_kek_ceremony
axis: security
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.security.security_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [C, D, E]
  proof_classes: []
---

# secret_compromise_drill / KEK shamir ceremony

## 一文方針

四半期ごとの `v1_secret_compromise_drill`（secret exposure の revoke → re-issue フロー ≤ 30 分の検証）と、同 ceremony 内での KEK Shamir M-of-N threshold 再構築（OpenBao Transit + HSM PKCS#11）を一連で実施し、`drill_progress.lock.yaml` の green 更新と `rotation_progress.lock.yaml` の整合確認でクローズする。

## Trigger（発火条件）

四半期 ceremony 到来時（1 月 / 4 月 / 7 月 / 10 月の第 1 週、シナリオ 04 live drill と同月になる場合は別週に分離する）、または `v1_secret_exposure` incident class の疑義が発生し緊急 ceremony が必要になった時。

## 想定頻度 / 典型きっかけ

想定頻度: 四半期（定期）+ 緊急（随時）。典型きっかけ: 「四半期 ceremony。今回は KEK shamir custodian 3 名が staging HSM に集まり M-of-3 で share を再構築し、OpenBao Transit の key version を bump してから 8 secret class 全件の rotation cadence を確認する」

## 主役 / 関与者

- 主役: security 担当者（シニア級、ceremony リード）
- 必須参加: KEK shamir custodian 全員（M-of-N の M 名、v1 は M=2 N=3）
- 関与: data 担当者（DEK rotation のタイミング調整、シナリオ依存関係参照）
- 立会（記録）: 第三者 witness（ops 担当者または formal 担当者、dual cosign 署名のため）

## 前提

- KEK shamir share が `v1_kek` secret class として OpenBao Transit + HSM PKCS#11-backed に格納済み（share は memory only、disk persist 禁止）
- `rotation_progress.lock.yaml` が存在し、前回の `last_rotated_at` が 90 日以内
- staging HSM 環境が production HSM と同等の物理セキュリティ（PKCS#11 destroy 機能）を持つ
- data 担当者がシナリオ 06（KEK rotation）→ data-05（DEK rotation）の順序依存を確認済み

## 流れ

1. ceremony リード（security 担当者）が Mattermost `#security-ceremony` channel を開設し、custodian 全員 + witness の参加を確認する
2. **secret_compromise_drill パート（≤ 30 分の success criteria 計測開始）**:
   - テスト用 secret（`v1_api_key` class の test-only instance）を意図的に compromise 状態にシミュレートする（OpenBao dev namespace で token を意図的に外部 channel に書く）
   - 5 検知経路（gitleaks / Trivy / Falco / Tetragon / audit_event）のいずれかが detect するまでの時間を記録する
   - 検知後、OpenBao で該当 secret class 全 lease を即時 revoke する（`vault lease revoke -prefix secret/test-compromise`）
   - External Secrets Operator に rotate された新 secret を再配信し、application が Kubernetes Secret reload で吸収するまでの時間を計測する
   - 計測結果が ≤ 30 分であることを success criteria として確認する
3. **KEK shamir ceremony パート**:
   - custodian M 名が各自の HSM に登録済みの shamir share を提示する（物理 authentication）
   - OpenBao Transit の key version を bump する（`vault write -f transit/keys/kek/rotate`）
   - 旧 key version を min_decryption_version のみ残し（DEK の再暗号化が完了するまで）、新 key version での re-wrap をスケジュールする
   - HSM PKCS#11 destroy で旧 share を物理 zeroize する（`pkcs11-tool --delete-object --type privkey --id <old-share-id>`）
   - 新 share を M-of-N で再分配し、custodian 全員の物理 authentication で確認する
4. witness（ops または formal 担当者）が ceremony の全工程に cosign dual sign-off を実施する（`cosign sign --key openbao://transit/ceremony-signing-key ceremony_record.json`）
5. `rotation_progress.lock.yaml` の `v1_kek.last_rotated_at` を Tekton job が自動更新する
6. data 担当者に KEK rotation 完了を通知し、DEK re-wrap スケジュールを共有する（DEK rotation のタイミング調整）

## 関連適合仕様 / 関連 OSS

- 鍵管理適合仕様: [../../../04_詳細設計/01_適合仕様/05_鍵管理適合仕様.md](../../../04_詳細設計/01_適合仕様/05_鍵管理適合仕様.md)
- security 強制機構: [../../../04_詳細設計/02_強制機構/06_security強制機構.md](../../../04_詳細設計/02_強制機構/06_security強制機構.md)
- 関連 OSS: OpenBao Transit / HSM PKCS#11 / Cosign / External Secrets Operator / Tekton

## 期待結果 / 観測指標

- secret_compromise_drill: revoke → re-issue が ≤ 30 分（時刻を audit_event で記録）
- KEK shamir ceremony: 旧 share が HSM PKCS#11 destroy で物理 zeroize 完了（`pkcs11-tool --list-objects` で旧 share が不在）
- `rotation_progress.lock.yaml` の `v1_kek.last_rotated_at` が当日日付で更新済み
- dual cosign signature + witness attestation が ceremony record に記録済み
- `drill_progress.lock.yaml` の `v1_secret_compromise_drill.last_green_at` が更新済み

## 失敗時の挙動 / escalation

- **revoke → re-issue が 30 分超過**: success criteria fail。`drill_progress.lock.yaml` を `red` で記録し、postmortem PR を即日起票する（action item: OpenBao lease revoke の自動化スクリプト整備）。release_gate が drill_overdue_count を cap まで consumed するまでの間に修正を完了する
- **custodian が M 名揃わない（物理的不在）**: ceremony を中止して翌週に再スケジュールする。cadence 違反（90 日超）が迫る場合は tech lead + L3 escalation を経由して例外 ceremony（M を一時的に 1 名減じた threshold での reconstruction）の承認を得る。全工程の audit_event 記録は mandatory
- **HSM PKCS#11 destroy が失敗（device error）**: 旧 share を OpenBao 側で access 禁止 state にしてから HSM ベンダーに修理依頼。物理 zeroize が完了するまで `rotation_progress.lock.yaml` は pending 状態とし、新 KEK への切替を保留する

## 関連参照

- [security 担当者シナリオ index](./README.md) — security 担当者シナリオ全体の構成と主要分類一覧
- [鍵管理適合仕様](../../../04_詳細設計/01_適合仕様/05_鍵管理適合仕様.md) — KEK shamir M-of-N の structural spec
- [security 設計方針: 秘密管理方針](../../../03_概要設計/07_security設計方針/03_秘密管理方針.md) — 8 secret class の rotation cadence 定義
- [data 担当者シナリオ: 暗号化変更 KEK DEK rotation](../05_data担当者シナリオ/05_暗号化変更_KEK_DEK_rotation.md) — data 視点の DEK rotation（本シナリオとの依存関係）
