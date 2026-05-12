---
id: plan.security.scenario_build_provenance_class_add_change
axis: security
phase: plan
kind: plan_doc
status: draft
depends_on:
  - arch.security.security_index
  - req.team.tier_engineer_requirement
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
    - v1_refinement_proof
---

# build_provenance_class 追加 / 5 phase enforcement 変更

## 一文方針

新規 build_provenance_class の追加または既存 class の 5 phase enforcement 変更が必要になった時に、`artifact_inventory.lock.yaml` + `provenance_attestation.lock.yaml` + `reproducibility_matrix.lock.yaml` の 3 lock artifact を build script 経由で更新し、5 enforcement orchestrator のすべてに新 class が bind 済みであることを CI で物理確認してクローズする。

## Trigger（発火条件）

新規 artifact 種別（例: Tauri sidecar executable / WASM module）が出現し既存 5 class に収まらない時、または SLSA L3+ の upstream spec 変更で 5 phase enforcement の pointer 修正が必要になった時。

## 想定頻度 / 典型きっかけ

想定頻度: 不定期（年 1-2 件）。典型きっかけ: 「Tauri Companion sidecar の .exe を Windows 向け MDM 配布するにあたり、`v1_runtime_image_release`（OCI image 前提）では artifact_signing_chain が合わない。新 class `v1_desktop_exe_release` として `artifact_signing_chain=authenticode_plus_cosign` を持つ class bundle を追加する」

## 主役 / 関与者

- 主役: security 担当者（シニア級）
- 関与: tier1 担当者（Tekton pipeline の新 build step 追加 / Bazel target 追加）
- 関与: infra 担当者（Harbor / Kyverno admission verifier の設定拡張）
- 連絡: tier1 担当者シナリオ 09（supply chain 障害対応）との overlap 確認

## 前提

- 5 build_provenance_class の class bundle 定義が `classes.yaml`（build artifact）に存在し、dimension override 禁止のルールが CI で enforce 済み
- 3 lock artifact（`artifact_inventory.lock.yaml` / `provenance_attestation.lock.yaml` / `reproducibility_matrix.lock.yaml`）が build script の自動生成物として管理済み
- 5 enforcement orchestrator（tekton_chains_attestor / bazel_nix_hermetic_runner / reproducibility_consensus_verifier / cosign_rekor_witness_chain / kyverno_admission_verifier）が新 class をサポートするよう拡張可能な設計である

## 流れ

1. **新 class 定義**: `classes.yaml` に新 class entry を追加する。5 dimension（hermetic_scope / artifact_signing_chain / dependency_lock_scope / rebuild_determinism / supply_chain_transparency）を class bundle として一括宣言する。dimension override 禁止のため、全 dimension を新 class で定義し直す
2. **build script 更新**: `tools/lock_yaml_generator/build_provenance_gen` スクリプトに新 class を認識させ、3 lock artifact を再生成する。生成後に `artifact_inventory.lock.yaml` の dead entry check（参照消失 CI fail）が green であることを確認する
3. **Tekton pipeline 拡張**: 新 artifact 種別に対応する build step（signing / attestation）を Tekton pipeline config に追加する。`v1_desktop_exe_release` の場合: Authenticode signing（Windows SDK）+ cosign keyless の dual sign
4. **reproducibility_consensus_verifier 拡張**: 新 class の rebuild determinism（`bit_for_bit_ci_verified` または `source_date_epoch_pinned`）が要求する builder 数（N-of-N）を `reproducibility_matrix.lock.yaml` に宣言し、Tekton + GitHub Actions の双方で hash 一致チェックを追加する
5. **Kyverno admission verifier 拡張**: Harbor の image policy または admission webhook に新 artifact 種別の署名検証条件（cosign verify key + Rekor entry or Authenticode thumbprint + cosign）を追加する
6. staging 環境で新 artifact を実際に build し、5 phase（build → attest → verify → publish → monitor）を通過させる
7. CI の build_provenance coverage check（全 artifact が何らかの class に bind されているか）が green であることを確認する
8. security 担当者 2 名の dual reviewer sign-off を取得し、3 lock artifact の PR を merge する
9. tier1 担当者に build_provenance_class 変更完了を通知し、SBOM 月次トリアージ（シナリオ 02）の対象 class リストを更新する

## 関連適合仕様 / 関連 OSS

- build_provenance 適合仕様: [../../../04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md](../../../04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md)
- 関連 OSS: Tekton Chains / Cosign / Sigstore Rekor / Bazel / Nix / Kyverno / Harbor / Witness

## 期待結果 / 観測指標

- `classes.yaml` に新 class が 5 dimension 全宣言済み
- 3 lock artifact が build script 経由で再生成され、dead entry がゼロ
- 新 artifact が 5 phase（build → attest → verify → publish → monitor）を staging で通過
- CI の build_provenance coverage check が green
- dual reviewer sign-off 記録済み

## 失敗時の挙動 / escalation

- **reproducibility_consensus_verifier で hash 不一致（non-determinism）**: 新 class の hermetic_scope 設定が不十分。Nix flake / Bazel sandbox の設定を強化し、Internet fetch 経路を内部 mirror に限定する。non-determinism が残る間は `rebuild_determinism=best_effort` の class へ降格させ、本番用 class には使用しない
- **dead entry が CI fail になった（既存 artifact が新 class に移行されていない）**: 影響 artifact の owner（tier1 / infra 担当者）に連絡し、`artifact_inventory.lock.yaml` の entry を新 class に更新する PR を同 sprint 内で merge する
- **Kyverno admission verifier が新 artifact 形式を拒否し続ける（設定 bug）**: admission webhook を dry-run mode に一時切替（`kyverno.io/policy-action: audit`）して新設定を検証してから enforce に戻す。production への新 artifact deploy は dry-run が green になるまで保留

## 関連参照

- [security 担当者シナリオ index](./README.md) — security 担当者シナリオ全体の構成と主要分類一覧
- [build_provenance 適合仕様](../../../04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md) — 5 class × 5 phase × 5 enforcement orchestrator の structural spec
- [security 担当者シナリオ: SBOM CVE 月次トリアージ](./02_SBOM_CVE_supply_chain月次トリアージ.md) — build_provenance_class 変更後の SBOM 対象 class 更新
- [tier1 担当者シナリオ: supply chain 障害対応](../01_tier1担当者シナリオ/09_supply_chain障害対応.md) — build_provenance_class 変更時の tier1 側連動
