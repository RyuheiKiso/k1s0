# coverage_matrix artifact manifest — cross_slo.v1_type_invariant
# verification_class: v1_type_invariant
# generated: 2026-05-23
# drill_state: v1_verified_with_artifact_pointer
cell_id: cross_slo.v1_type_invariant
axis_name: cross_slo
verification_class: v1_type_invariant
drill_state: v1_verified_with_artifact_pointer
last_verified_at: '2026-05-23T00:00:00Z'
artifacts:
  - artifact_id: cross_slo_v1_type_invariant_scaffold
    artifact_kind: scaffold_verified
    source_path: src/test/coverage_matrix/cross_slo/v1_type_invariant/
    evidence_uri: "file://src/test/coverage_matrix/cross_slo/v1_type_invariant/"
    notes: "cross_slo 軸 v1_type_invariant の scaffold 検証。CI lint + 軸固有 test を実行済み。"
  - artifact_id: cross_slo_ci_lint_pass
    artifact_kind: ci_lint_result
    source_path: .github/workflows/
    evidence_uri: "urn:k1s0:ci:lint:cross_slo:v1_type_invariant:2026-05-23"
    notes: "docs_lint.lock.yaml green + repository_layout.lock.yaml green で CI 通過済み。"
