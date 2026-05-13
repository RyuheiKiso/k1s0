---
id: plan.security.scenario_cosign_supply_chain_admission_reject
axis: security
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.security.security_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [B, D, E]
  proof_classes: []
---

# cosign / supply chain admission 拒否対応

## 一文方針

Harbor admission policy（`block-on-unsigned-image`）または Kyverno admission controller が cosign verify fail で image deploy を拒否した時に、`v1_supply_chain_compromise` playbook の contain → eradicate → recover を実施し、cosign 再署名 + Rekor inclusion proof の再取得で admission を通過させ、`provenance_attestation.lock.yaml` を更新してクローズする。

> 朝 11 時、Argo CD の deploy が Kyverno admission `DENY` で止まったという通知が Mattermost `#ci-alerts` に届く。security 担当者（シニア級）が Harbor access log を開き、cosign 署名が欠落した image digest を特定する。Tekton pipeline の signing step でタイムアウトが発生したことを確認し、infra 担当者に quarantine 操作を依頼しながら cosign verify コマンドを手元で実行し始める。

## ペルソナ要約

主役: security 担当者（シニア級）、目的: cosign verify が失敗した未署名 image を Harbor で quarantine し production への混入を防ぐ

## 現状業務での痛み

- cosign verify なしで未署名 image が production に入り、supply chain attack の踏み台になるリスクがある
- image の署名状況が把握できず、どの image が検証済みかが不明確
- 未署名 image の quarantine 手順が不明確で、発見から対処まで時間がかかる

## k1s0 でこう変わる

- Kyverno admission policy が cosign verify を強制し、未署名 image の production 混入が物理的に不可能になる
- Harbor が全 image の署名状況を管理し、未署名 image が即時 quarantine される
- cosign verify fail 時の runbook が Backstage TechDocs に定義され、発見から quarantine までが即時実行できる

## Trigger（発火条件）

Harbor admission policy または Kyverno `cosign-verify-policy` が `DENY` を返した時、または Tekton Chains の attestation が欠落した image が CI で検知された時。

## 想定頻度 / 典型きっかけ

想定頻度: イベント駆動（月 1-3 件）。典型きっかけ: 「Tekton pipeline の signing step でタイムアウトが発生し、cosign 署名が欠落したまま image が Harbor に push された。Kyverno admission が production namespace への deploy を拒否した」

## 主役 / 関与者

- 主役: security 担当者（シニア級）
- 関与: infra 担当者（Harbor quarantine 操作 / Kyverno policy 確認）
- 関与: tier1 担当者（Tekton pipeline の signing step 修正）

| 役割 | 級 | 主に居る場所 | 朝最初に見る画面 | このシナリオでの主要動作 |
|---|---|---|---|---|
| 主役（security）| シニア | 本社 IT 室 / リモート | Mattermost #ci-alerts / provenance_attestation.lock.yaml | root cause 特定・cosign verify 確認・再署名 pipeline 修正指示・admission 通過確認 |
| 関与（infra）| シニア〜ミドル | 本社 IT 室 / リモート | Harbor dashboard / Kyverno audit log | quarantine 操作・Kyverno policy 確認・Rekor publish 確認 |
| 関与（tier1）| シニア〜ミドル | 本社 IT 室 / リモート | Tekton pipeline log | signing step タイムアウト修正・pipeline 再実行・attestation 生成確認 |

## 個人 KPI / 達成感

- 未署名 image の production 混入 0 件が admission policy で定量確認でき、supply chain 安全性の達成感を得られる
- cosign verify fail から quarantine までの MTTR が閾値以内であることを確認でき、対応速度の改善を実感できる

## 工数 / 関与人数 / コスト感

- 工数: 数時間〜半日（quarantine 実行 30min + 影響調査 2h + 対処 4h）
- 関与人数: 3〜4 名（security 担当者・infra 担当者・tier1 担当者・dual reviewer）
- コスト感: 低。admission policy が自動防御するため quarantine 作業は例外対応のみになる

## 前提

- Harbor の `block-on-unsigned-image` admission policy が production namespace に active
- Kyverno `cosign-verify-policy` が cluster に適用済み
- Tekton Chains が pipeline に統合済みで、全 build artifact に `v1_runtime_image_release` の attestation が付与される設計
- `provenance_attestation.lock.yaml` が build artifact として存在し、artifact ↔ attestation の双方向 lock が CI で管理済み

## 流れ

1. **root cause 特定**: Kyverno audit log または Harbor access log から拒否された image digest と拒否理由（署名欠落 / 署名不正 / Rekor inclusion proof なし）を特定する
2. **harbor quarantine**: 該当 image を Harbor で quarantine state に設定し、他の namespace / cluster への pull を物理 block する（`harbor image quarantine --image <digest>`）
3. **cosign verify を実行して状態確認**: `cosign verify --key openbao://transit/signing-key <image-digest>` で verify の具体的 error（missing signature / invalid signature / no Rekor entry）を記録する
4. **署名 pipeline の修正**:
   - Tekton の signing step タイムアウトなら: `timeout` 設定を延長し、signing step を独立 stage に分離する PR を起票する
   - signing key の rotation overdue なら: シナリオ 06（KEK shamir ceremony）の signing_key rotation パートを先行して実施する
5. **image の再 build と再署名**:
   - 問題が image ではなく signing step のみであれば、同 git commit で Tekton pipeline を再実行する（`tkn pipeline start security-resign --param image-digest=<digest>`）
   - 再 build が必要な場合は hermetic sandbox（`RUN --network=none`）で再実行し、bit_for_bit_ci_verified を 3 builder で通過させる
6. **cosign 再署名と Rekor への publish**: `cosign sign --key openbao://transit/signing-key <new-image-digest>` → Sigstore Rekor に inclusion proof が記録されたことを `cosign verify` で確認する
7. **Witness multi-attestation**: Tekton Chains が新 image に provenance attestation を追加し、`provenance_attestation.lock.yaml` が Tekton job で更新されたことを確認する
8. **admission 通過の確認**: Harbor quarantine を解除し、`kubectl apply` を再実行して Kyverno が `ALLOW` を返すことを確認する

## Timeline

| T+ | actor | action | 通知例 |
|---|---|---|---|
| 0 | security 担当者 | cosign verify fail alert を受信し Harbor で未署名 image を quarantine | `cosign fail alert / image quarantine 実行 / T+0` |
| 30分 | security 担当者 | 未署名 image の影響範囲を調査し production への混入がないことを確認 | `影響範囲調査完了 / production 混入なし確認` |
| 2h | security 担当者 + tier1 担当者 | image に署名し cosign verify を通過させる | `image 署名完了 / cosign verify green` |
| 4h | dual reviewer | 署名設定と admission policy を確認し sign-off | `sign-off 完了` |

## 業界 9 業務との紐付け

cosign admission は全 9 業務の build artifact に対して基盤として機能するため、admission 拒否はすべての業務の deploy を止め得る。特に影響度が高い業務:

- **FA 生産指示**: 制御系 binary の unsigned image が admission で拒否されると設備制御が停止する。admission 拒否対応の迅速さが FA 業務継続に直結する最高優先度の業務。
- **SCADA テレメトリ収集**: テレメトリ収集 binary の admission 拒否はデータ欠損に繋がり、設備状態の可視性が失われる。cosign 再署名完了まで業務継続不可。
- **警報配信**: 警報配信コンポーネントの admission 拒否は警報受信不能に繋がり、可用性 SLO に直接影響する。

## 関連適合仕様 / 関連 OSS

- build_provenance 適合仕様: [../../../04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md](../../../04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md)
- security 強制機構: [../../../04_詳細設計/02_強制機構/06_security強制機構.md](../../../04_詳細設計/02_強制機構/06_security強制機構.md)
- 関連 OSS: Cosign / Sigstore Rekor / Tekton Chains / Harbor / Kyverno / OpenBao Transit / Witness

## 期待結果 / 観測指標

- `cosign verify --key openbao://transit/signing-key <image-digest>` が pass（exit 0）
- Rekor に inclusion proof が記録済み（`rekor-cli get --uuid <entry-uuid>`）
- Kyverno `cosign-verify-policy` が `ALLOW` を返す
- `provenance_attestation.lock.yaml` が新 image digest で更新済み
- Harbor quarantine が解除済みで production namespace への deploy が通過

## 失敗時の挙動 / escalation

- **signing key が compromise されている疑義（invalid signature / unknown key）**: 即時シナリオ 06（secret_compromise_drill / KEK ceremony）の緊急パートを発火。signing key 全 version を revoke してから re-issue し、`v1_signing_key` の `rotation_progress.lock.yaml` を更新する
- **Tekton Chains が繰り返し失敗する（attestation 生成 bug）**: staging での再現を確認してから Tekton Chains の upgrade / patch を tier1 担当者と共同で実施する。production への deploy は Tekton Chains が green になるまで全体保留（release_gate AND-gate）
- **Rekor への publish が失敗（Sigstore インフラ障害）**: private cosign attestation archive（Harbor 内）のみで一時的に代替し、Rekor が復旧次第 backfill する。この間の audit_event に `rekor_unavailable=true` を付与し、監査時に区別可能にする

## 失敗パターン (anti-pattern)

- quarantine なしの調査: 未署名 image を production で稼働させたまま調査すると supply chain attack のリスクが継続する
- 例外的な admission bypass: Kyverno policy を一時無効化すると他の未署名 image も混入するリスクが生じる

## 関連参照

- [security 担当者シナリオ index](./README.md) — security 担当者シナリオ全体の構成と主要分類一覧
- [build_provenance 適合仕様](../../../04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md) — 5 enforcement orchestrator の詳細と admission verifier
- [security 設計方針: インシデント対応方針](../../../03_概要設計/07_security設計方針/07_インシデント対応方針.md) — `v1_supply_chain_compromise` の contain action
- [security 担当者シナリオ: CVE CRITICAL incident 対応](./07_CVE_CRITICAL_incident対応.md) — CVE 起因の rebuild 後に本シナリオの admission 通過を確認する
