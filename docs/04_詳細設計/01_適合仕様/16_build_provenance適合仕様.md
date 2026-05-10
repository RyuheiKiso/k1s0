---
id: detail.security.build_provenance_conformance
axis: security
phase: detail
kind: conformance_spec
status: draft
depends_on:
  - arch.security.security_index
  - arch.security.threat_model_policy
  - arch.security.boundary_control_policy
  - arch.security.vulnerability_management_policy
  - detail.security.threat_model_conformance
  - detail.security.security_enforcement
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
    - v1_refinement_proof
lock_artifacts:
  - artifact_inventory.lock.yaml
  - provenance_attestation.lock.yaml
  - reproducibility_matrix.lock.yaml
---

# security build provenance 適合仕様（v1）

## 一文定義
- build_provenance 適合仕様は、5 build_provenance_class × 5 phase の cross-product として完全列挙される build_loop catalog の各 cell に対し、5 enforcement orchestrator のいずれかを必ず物理 pointer として bind し、全 13 軸 + meta + 自軸の defense-in-depth 層 D / E に bind 済みであることを 5 層 defense-in-depth + 13 項 CI 不変条件 + 13 件 stress test pack で物理 enforce する supply chain integrity meta-axis である。

## 位置づけ
- [採用 OSS 一覧](../../02_要件定義/04_技術選定/01_OSS採用一覧.md) の Harbor / Trivy / Cosign（Sigstore）/ Syft / Grype / Kyverno / cert-manager / OpenBao / GitHub Actions self-hosted runner / Tekton（Tekton Chains 含む）/ Bazel / Nix / Witness / Gitsign / Buildkit、5 階層全 artifact 表面、infra security 方針、[境界制御方針](../../03_概要設計/07_security設計方針/02_境界制御方針.md)（v1_build_time surface / runner ephemeral identity）、[脆弱性管理方針](../../03_概要設計/07_security設計方針/04_脆弱性管理方針.md)（SBOM 運用 / Trivy + Grype）、[脅威モデル適合仕様](15_脅威モデル適合仕様.md)（v1_supply_chain actor の primary defense_layer）、tier1/11 鍵管理（HSM PKCS#11 / per-job ephemeral signing key）、tier1/14 OSS lifecycle が散在的に宣言する build hermeticity / artifact signing chain / dependency lock scope / rebuild determinism / supply chain transparency を、機械可読な単一の真として一箇所に固定する仕様書。
- security 軸が物理的に唯一の build provenance facade として振る舞う
- 17 軸 + meta（00 軸登録）の 17 軸目として、tier1〜client / infra / data / test の各軸の build artifact 投影面を統合する

## 設計原則
- **build_provenance_class は bundle である**: yaml に書く `security.build_provenance.*` attribute のうち、build_provenance_class 1 つの値が他 5 dimension（hermetic_scope / artifact_signing_chain / dependency_lock_scope / rebuild_determinism / supply_chain_transparency）を一意に導出する
- **dimension override は禁止する**: 「v1_runtime_image_release の rebuild_determinism を best_effort に下げたい」要望は新 class を切ることで表現
- **dead spec を CI で殺す**: classes.yaml に登録された全 class、artifact_inventory に列挙された全 artifact、provenance_attestation に列挙された全 predicate、reproducibility_matrix に列挙された全 hash entry が参照されなくなった時点で CI fail。Harbor catalog のうち build_provenance_class 注釈が無い image は admission deny by default
- **5 種類 enforcement orchestrator を全 build_provenance_class に必須化する**: tekton_chains_attestor / bazel_nix_hermetic_runner / reproducibility_consensus_verifier / cosign_rekor_witness_chain / kyverno_admission_verifier
- **build reproducibility を物理 property とする**: 「同 input source git commit + 同 builder image digest + 同 dependency lock hash → 同 output artifact bit hash」を reproducibility_consensus_verifier で複数 builder（Tekton + GitHub Actions self-hosted + Bazel remote-exec + Nix flake build の 4 経路）から並列実行し、N/N 全 builder で sha256 が一致することを CI で必ず検証する

## v1 build_provenance_class セット（5 class）

| class | hermetic_scope | artifact_signing_chain | dependency_lock_scope | rebuild_determinism | supply_chain_transparency |
|---|---|---|---|---|---|
| `v1_runtime_image_release` | codegen_compile_link_image_attest | cosign_rekor_witness | sbom_attested_mirror | bit_for_bit_ci_verified | rekor_plus_private_archive |
| `v1_library_sdk_release` | codegen_compile_link_image_attest | cosign_rekor_witness_multiparty | nix_flake_pinned | nix_flake_bit_identical | rekor_plus_private_archive_plus_reviewer_attest |
| `v1_cli_or_oci_artifact_release` | codegen_compile_link_image_attest | cosign_rekor_witness | sbom_attested_mirror | bit_for_bit_ci_verified | rekor_plus_private_archive |
| `v1_thin_business_api_artifact` | codegen_only | cosign_rfc3161_timestamp | hash_pinned_mirror | source_date_epoch_pinned | rekor_public |
| `v1_dev_or_canary_artifact` | codegen_compile | cosign_keyless | lockfile_only | best_effort | log_only |

### 各 class の不変条件と典型用途

#### v1_runtime_image_release
- 用途: tier1 / tier2 server image / Companion sidecar image / infra runner image / ops-edge cluster image。production deploy 経路の本命
- hermetic_scope: codegen → compile → link → image build → attestation の全段で `RUN --network=none`（Buildkit）+ Bazel sandbox + Nix sandbox の三重 sandbox。internal mirror（Athens / Verdaccio / Harbor proxy / cargo registry proxy / NuGet feed）のみ通過、Internet 直接 fetch 物理不可能
- artifact_signing_chain: cosign keyless（GitHub Actions OIDC → OpenBao → Cosign Fulcio）+ Rekor inclusion proof + Witness multi-attestation
- dependency_lock_scope: sbom_attested_mirror。Syft 生成 SPDX + CycloneDX 両 format が SBOM として attached
- rebuild_determinism: bit_for_bit_ci_verified。同 input → 同 output の sha256 一致を Tekton + GitHub Actions + Bazel remote-exec の 3 builder で並行検証、3/3 一致で green
- supply_chain_transparency: rekor_plus_private_archive。Sigstore Rekor public log + Harbor 内 private cosign attestation archive 双方に publish
- rotation_cadence_hours: 24（per-job ephemeral signing key）

#### v1_library_sdk_release
- 用途: 9 言語 Library SDK（dotnet8 / java21 / node20 / rust / go122 / python312 / ruby33 / dotnet_framework_4_6_2_plus / browser TypeScript / tauri rust+webview）。end-user の依存先として最も改竄影響が大きい
- hermetic_scope: Nix flake で全 dependency の hash 固定 + Bazel sandbox で言語横断 hermetic + Buildkit で OS 側 network egress 物理不可能
- artifact_signing_chain: cosign_rekor_witness_multiparty。GitHub Actions runner + Tekton Chains + 独立 isolated builder の 3 attestor が独立 OIDC keyless signing、`reviewer_attest_signers_min: 2` で人間 reviewer attestation も in-toto link として付加
- dependency_lock_scope: nix_flake_pinned。Nix flake input の sha256 / sri hash が flake.lock で純粋関数的に固定
- rebuild_determinism: nix_flake_bit_identical。`consensus_builders_min: 3` で N/N 全 builder で nix build 結果 sha256 一致
- supply_chain_transparency: rekor_plus_private_archive_plus_reviewer_attest。Rekor public log + Harbor private archive + reviewer 2 名以上の人間 attestation を 3 重 publish
- SLSA L4 schema 必須（`reproducible: true` / `hermetic: true` / `parameterless: true`）

#### v1_cli_or_oci_artifact_release
- 用途: Helm chart / OPA bundle / Apicurio subject bundle / CLI binary / Argo CD CMP / Kustomize plugin の OCI artifact 群
- hermetic_scope: Helm chart は values schema 込みで Bazel + Nix で build、OPA bundle は wasm compile を hermetic 化、CLI binary は Nix flake + cargo / go の reproducible build 経路
- artifact_signing_chain: v1_runtime_image_release と同等

#### v1_thin_business_api_artifact
- 用途: 12_client/15 の `v1_thin_business_api_only` distribution に対応する OpenAPI yaml only artifact。compile phase 不在のため hermetic は codegen のみ
- hermetic_scope: codegen_only。Buf workflow による .proto → OpenAPI yaml 派生のみ hermetic
- artifact_signing_chain: cosign_rfc3161_timestamp。yaml 単体の改竄防止に特化、RFC 3161 trusted timestamp authority + Cosign signature
- dependency_lock_scope: hash_pinned_mirror。proto schema 依存のみ
- rebuild_determinism: source_date_epoch_pinned。Buf codegen は `SOURCE_DATE_EPOCH = release tag commit timestamp` で deterministic 化
- 用途: レガシー .NET Framework 環境への OpenAPI 配布、tier1 Library を直接組み込めない言語向けの business API 経路

#### v1_dev_or_canary_artifact
- 用途: dev / canary / pre-release / e2e test 専用 artifact。production deploy 物理 reject
- hermetic_scope: codegen_compile。link / image build は best-effort
- artifact_signing_chain: cosign_keyless（OIDC keyless のみ、Witness multi-attestation 無し）
- **forbidden_in_production: true**。Kyverno admission policy `dcv_kyverno_blocks_dev_artifact_to_prod` で production namespace への deploy / pull を物理 block

## 5 enforcement orchestrator

### tekton_chains_attestor
- 担当: 全 5 class の `attestation` phase
- 主要 OSS: Tekton Chains（Apache 2.0）+ in-toto（Apache 2.0）+ Witness（Apache 2.0）
- 出力 artifact: `in-toto/slsaprovenance/v1.0` predicate / Witness attestation

### bazel_nix_hermetic_runner
- 担当: 全 production class の `hermetic_scope` 物理保証
- 主要 OSS: Bazel + Nix flake + Buildkit `RUN --network=none`
- bound parameter pin: `nix_flake.lock` / `MODULE.bazel.lock`

### reproducibility_consensus_verifier
- 担当: production class の `rebuild_determinism` 物理保証
- 出力: 複数 builder（Tekton + GitHub Actions + Bazel remote-exec + Nix flake build）の sha256 一致 verify

### cosign_rekor_witness_chain
- 担当: 全 class の `artifact_signing_chain` 物理保証
- 主要 OSS: Cosign / Rekor / Fulcio / Witness
- 出力 artifact: cosign signature + Rekor inclusion proof + Witness multi-attestation

### kyverno_admission_verifier
- 担当: 全 class の deploy 時 verify
- 主要 OSS: Kyverno（`verifyImages` + `verifySLSAProvenance`）

## 単一の真（5 yaml + supplementary）

### `classes.yaml`
- 5 build_provenance_class 定義

### `artifact_inventory.lock.yaml`（build artifact）
- Harbor catalog + 各 artifact の build_provenance_class 注釈
- 各 entry: `artifact_id` / `build_provenance_class` / `digest` / `build_completed_at` / `signing_chain` / `attestation_pointer` / `runner_image_self_attestation_uri`

### `provenance_attestation.lock.yaml`（build artifact）
- 全 artifact の in-toto / SLSA predicate 集約
- predicate type: `in-toto/slsaprovenance/v1.0` / `in-toto/spdx/v2.3` / `in-toto/cyclonedx/v1.5`

### `reproducibility_matrix.lock.yaml`（build artifact）
- 各 artifact について、複数 builder の sha256 結果集約
- consensus_status: `green` / `yellow` / `red`

### supplementary
- `cosign_keys.lock.yaml`（手書き＋ generator validate、cosign signing key の version pin）
- `mirror_inventory.lock.yaml`（手書き＋ generator validate、Athens / Verdaccio / Harbor proxy / cargo registry / NuGet feed の inventory）
- `runner_image.lock.yaml`（runner image の self-attestation pin）

## 5 層 defense-in-depth

### 層 A: compile
- Tekton Chains が `in-toto/slsaprovenance/v1.0` predicate を emit
- Buf workflow codegen の `in-toto/link` metadata と統合

### 層 B: lint
- Conftest custom rule の `dcv_pr_blocks_*`
- GitHub Actions Required Status Check（cosign verify / SBOM attach / Trivy scan / reproducibility consensus）

### 層 C: chaos test
- Litmus で attestation tampering 注入
- consensus failure を property test
- runner image self-attestation 改竄試験

### 層 D: runtime
- Kyverno admission policy の `dcv_kyverno_blocks_*`
- `verifyImages` + `verifySLSAProvenance`

### 層 E: 物理
- Linux user namespace + seccomp + cgroup v2
- Bazel sandbox + Nix sandbox の二重 sandbox
- Rekor Merkle tree immutable log
- HSM PKCS#11 transit
- kube-bench CIS / NSA/CISA Hardening

## CI 不変条件（13 項）

| # | 不変条件 |
|---|---|
| 不変 1 | `classes.yaml` の 5 class enum が version 別に purely additive |
| 不変 2 | `artifact_inventory.lock.yaml` が build artifact、手書き diff = 0 |
| 不変 3 | Harbor catalog の全 image が `build_provenance_class` 注釈持ち |
| 不変 4 | 全 production artifact が cosign signed + SBOM attached |
| 不変 5 | `provenance_attestation.lock.yaml` の全 predicate が schema 適合 |
| 不変 6 | `reproducibility_matrix.lock.yaml` の全 production artifact が consensus_status=green |
| 不変 7 | runner image self-attestation が daily 検証 green |
| 不変 8 | per-job ephemeral signing key の rotation cadence 24h 内 |
| 不変 9 | mirror_inventory のうち Internet 直接 fetch 経路ゼロ |
| 不変 10 | dev_or_canary_artifact が production namespace への deploy / pull で admission block |
| 不変 11 | reviewer attestation （v1_library_sdk_release）の signers_min: 2 完備 |
| 不変 12 | Rekor inclusion proof の online verify green |
| 不変 13 | 13 軸 + meta + security 軸の各 spec の build artifact と本層の `artifact_inventory` / `provenance_attestation` が双方向 lock |

## 1.0.0 ship blocker
- 5 build_provenance_class 全てに対し全 production artifact が `attestation` phase 完了
- `artifact_inventory.lock.yaml` の全 entry が `signing_chain` complete + `attestation_pointer` resolvable
- `reproducibility_matrix.lock.yaml` の全 production artifact が consensus green
- runner image self-attestation の daily check green 維持
- 18 軸との双方向 lock 整合（drift ゼロ）

## 採用しない設計
- 商用 SLSA tooling: 採用しない
- 単一 builder のみによる build: 禁止、production class は 3 builder consensus 必須
- cosign signing key の long-lived: 禁止、per-job ephemeral 24h rotation
- mirror なしの Internet 直接 fetch: 禁止、admission block
- runner image の self-attestation 省略: 禁止
- LLM 単独 sign-off: 禁止（v1_library_sdk_release の reviewer attestation には人間 2 名必須）

## 関連参照
- [security 設計方針 index](../../03_概要設計/07_security設計方針/README.md)
- [脅威モデル適合仕様](15_脅威モデル適合仕様.md)
- [security 強制機構](../02_強制機構/06_security強制機構.md)
- [脆弱性管理方針](../../03_概要設計/07_security設計方針/04_脆弱性管理方針.md)
- [境界制御方針](../../03_概要設計/07_security設計方針/02_境界制御方針.md)
- [release_gate 体系](../05_lock_yaml体系/03_release_gate体系.md)
