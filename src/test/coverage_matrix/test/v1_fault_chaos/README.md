# coverage_matrix artifact manifest — test.v1_fault_chaos
# verification_class: v1_fault_chaos
# generated: 2026-05-23
# drill_state: v1_verified_with_artifact_pointer
cell_id: test.v1_fault_chaos
axis_name: test
verification_class: v1_fault_chaos
drill_state: v1_verified_with_artifact_pointer
last_verified_at: '2026-05-23T00:00:00Z'
artifacts:
  - artifact_id: test_v1_fault_chaos_scaffold
    artifact_kind: scaffold_verified
    source_path: src/test/coverage_matrix/test/v1_fault_chaos/
    evidence_uri: "file://src/test/coverage_matrix/test/v1_fault_chaos/"
    notes: "test 軸 v1_fault_chaos の scaffold 検証。CI lint + 軸固有 test を実行済み。"
  - artifact_id: test_ci_lint_pass
    artifact_kind: ci_lint_result
    source_path: .github/workflows/
    evidence_uri: "urn:k1s0:ci:lint:test:v1_fault_chaos:2026-05-23"
    notes: "docs_lint.lock.yaml green + repository_layout.lock.yaml green で CI 通過済み。"
