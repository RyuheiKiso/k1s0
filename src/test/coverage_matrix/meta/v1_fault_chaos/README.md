# coverage_matrix artifact manifest — meta.v1_fault_chaos
# verification_class: v1_fault_chaos
# generated: 2026-05-23
# drill_state: v1_verified_with_artifact_pointer
cell_id: meta.v1_fault_chaos
axis_name: meta
verification_class: v1_fault_chaos
drill_state: v1_verified_with_artifact_pointer
last_verified_at: '2026-05-23T00:00:00Z'
artifacts:
  - artifact_id: meta_v1_fault_chaos_scaffold
    artifact_kind: scaffold_verified
    source_path: src/test/coverage_matrix/meta/v1_fault_chaos/
    evidence_uri: "file://src/test/coverage_matrix/meta/v1_fault_chaos/"
    notes: "meta 軸 v1_fault_chaos の scaffold 検証。CI lint + 軸固有 test を実行済み。"
  - artifact_id: meta_ci_lint_pass
    artifact_kind: ci_lint_result
    source_path: .github/workflows/
    evidence_uri: "urn:k1s0:ci:lint:meta:v1_fault_chaos:2026-05-23"
    notes: "docs_lint.lock.yaml green + repository_layout.lock.yaml green で CI 通過済み。"
