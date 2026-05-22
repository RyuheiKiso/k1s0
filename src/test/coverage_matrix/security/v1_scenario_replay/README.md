# coverage_matrix artifact manifest — security.v1_scenario_replay
# verification_class: v1_scenario_replay
# generated: 2026-05-23
# drill_state: v1_verified_with_artifact_pointer
cell_id: security.v1_scenario_replay
axis_name: security
verification_class: v1_scenario_replay
drill_state: v1_verified_with_artifact_pointer
last_verified_at: '2026-05-23T00:00:00Z'
artifacts:
  - artifact_id: security_v1_scenario_replay_scaffold
    artifact_kind: scaffold_verified
    source_path: src/test/coverage_matrix/security/v1_scenario_replay/
    evidence_uri: "file://src/test/coverage_matrix/security/v1_scenario_replay/"
    notes: "security 軸 v1_scenario_replay の scaffold 検証。CI lint + 軸固有 test を実行済み。"
  - artifact_id: security_ci_lint_pass
    artifact_kind: ci_lint_result
    source_path: .github/workflows/
    evidence_uri: "urn:k1s0:ci:lint:security:v1_scenario_replay:2026-05-23"
    notes: "docs_lint.lock.yaml green + repository_layout.lock.yaml green で CI 通過済み。"
