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

> 朝 9 時、四半期 ceremony の当日。Mattermost `#security-ceremony` を security 担当者（シニア級、ceremony リード）が開設し、KEK shamir custodian 3 名と witness（ops 担当者）の参加を確認する。staging HSM 室に custodian が集まり、secret_compromise_drill パートの計測タイマーが動き始める。テスト用 secret の意図的 compromise シミュレートを準備しながら、30 分の success criteria を意識している。

## ペルソナ要約

主役: security 担当者（シニア級）、目的: KEK shamir ceremony を標準手順で実施し ceremony 参加者の変動による品質低下を防ぐ

## 現状業務での痛み

- ceremony 手順が文書管理で、ceremony 参加者が毎回異なると手順の解釈に差が生じる
- ceremony の実施記録が残らず、監査時に ceremony の適切な実施を証明できない
- shamir share の保管状況が把握できず、share 喪失リスクが管理されていない

## k1s0 でこう変わる

- ceremony 手順が Backstage TechDocs の runbook に定義され、誰が参加しても同一品質の ceremony が保証される
- ceremony 実施記録が ceremony.lock.yaml に記録され、監査時に証跡が即時提出できる
- shamir share の保管状況が lock.yaml で管理され、share 喪失リスクが定量的に把握できる

## Trigger（発火条件）

四半期 ceremony 到来時（1 月 / 4 月 / 7 月 / 10 月の第 1 週、シナリオ 04 live drill と同月になる場合は別週に分離する）、または `v1_secret_exposure` incident class の疑義が発生し緊急 ceremony が必要になった時。

## 想定頻度 / 典型きっかけ

想定頻度: 四半期（定期）+ 緊急（随時）。典型きっかけ: 「四半期 ceremony。今回は KEK shamir custodian 3 名が staging HSM に集まり M-of-3 で share を再構築し、OpenBao Transit の key version を bump してから 8 secret class 全件の rotation cadence を確認する」

## 主役 / 関与者

- 主役: security 担当者（シニア級、ceremony リード）
- 必須参加: KEK shamir custodian 全員（M-of-N の M 名、v1 は M=2 N=3）
- 関与: data 担当者（DEK rotation のタイミング調整、シナリオ依存関係参照）
- 立会（記録）: 第三者 witness（ops 担当者または formal 担当者、dual cosign 署名のため）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（security、ceremony リード）| シニア | staging HSM 室 / 本社 IT 室 | Mattermost #security-ceremony / rotation_progress.lock.yaml | ceremony 進行・drill 計測・KEK rotation 実施・dual sign 取得 |
| 必須参加（custodian）| シニア | staging HSM 室 | OpenBao HSM PKCS#11 console | shamir share 提示・物理 authentication・新 share 受領 |
| 立会（ops / formal）| シニア〜ミドル | staging HSM 室 / リモート | ceremony_record.json | cosign dual sign-off・ceremony 全工程の witness 記録 |
| 関与（data）| シニア〜ミドル | 本社 IT 室 / リモート | rotation_progress.lock.yaml | DEK re-wrap スケジュール調整・KEK 完了通知受領 |

## 個人 KPI / 達成感

- ceremony 実施の audit trail が 100% 記録されていることを確認でき、ceremony 品質の達成感を得られる
- ceremony cadence の達成率を lock.yaml で定量確認でき、鍵管理の継続的維持を実感できる

## 工数 / 関与人数 / コスト感

- 工数: 半日〜1 日（ceremony 準備 1h + 実施 2h + 記録 1h）
- 関与人数: 3〜5 名（security 担当者 2〜3 名・compliance 担当者・dual reviewer）
- コスト感: 低〜中。runbook と lock.yaml で手順が標準化されるため ceremony コストが安定する

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

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | security 担当者 | ceremony.lock.yaml で ceremony 参加者と手順を確認し ceremony を開始 | `KEK shamir ceremony 開始 / 参加者 N 名確認` |
| 30分 | security 担当者 + 参加者 | shamir share の検証と KEK ceremony を実施 | `ceremony 実施完了 / share 検証 OK` |
| 1h | security 担当者 | ceremony 記録を ceremony.lock.yaml に記録し audit trail を確認 | `ceremony 記録完了 / audit trail 確認` |
| 1d | dual reviewer | ceremony 記録と audit trail を確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

KEK shamir ceremony は全業務の暗号化基盤に影響するが、特に PII / business_data の保護が critical な業務に最も影響する:

- **受注**: 受注データは business_data / integrity が critical であり、KEK が compromise された場合は受注レコードの暗号化 DEK 全件が再 wrap 対象になる。ceremony での KEK rotation 完了は受注データ保護の根幹。
- **品質検査結果配信**: 検査データは PII / business_data として threat 対象であり、KEK shamir M-of-N threshold の維持が検査データの暗号保護の物理的な前提になる。
- **在庫最新値**: insider threat 対象の在庫データは DEK wrap に依存しており、KEK rotation が適切に行われることが在庫データの機密性継続を保証する。

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

## 失敗パターン (anti-pattern)

- 記録なしの ceremony: ceremony.lock.yaml への記録なしは監査時の証跡として認められない
- 参加者確認省略: 必要な参加者が揃っていない ceremony は ceremony gate CI が阻止する

## 関連参照

- [security 担当者シナリオ index](./README.md) — security 担当者シナリオ全体の構成と主要分類一覧
- [鍵管理適合仕様](../../../04_詳細設計/01_適合仕様/05_鍵管理適合仕様.md) — KEK shamir M-of-N の structural spec
- [security 設計方針: 秘密管理方針](../../../03_概要設計/07_security設計方針/03_秘密管理方針.md) — 8 secret class の rotation cadence 定義
- [data 担当者シナリオ: 暗号化変更 KEK DEK rotation](../05_data担当者シナリオ/05_暗号化変更_KEK_DEK_rotation.md) — data 視点の DEK rotation（本シナリオとの依存関係）
