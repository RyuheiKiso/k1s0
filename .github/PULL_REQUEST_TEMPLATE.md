# PR概要

<!-- 1〜3 箇条書きで変更内容を要約する -->

## Summary

-
-
-

## Test plan

<!-- 変更を検証するためのテスト手順を箇条書きで記載する -->

- [ ] `cargo build --workspace` が clean
- [ ] `cargo test --workspace` が pass
- [ ] `python3 -m tools.lock_yaml_generator.cli all` が OK
- [ ] `release_gate_status: green` / `red_count: 0` / `yellow_count: 0` を確認

## Dual sign-off チェックボックス

<!-- k1s0 dual sign-off 規約: LLM 単独 sign-off 禁止 -->
<!-- 参照: src/_meta/dual_review/README.md + CLAUDE.md -->

### Human cosign

- [ ] **human_cosign**: 実装者本人が `cosign sign-blob` で署名済み
  - cosign signed by: @<!-- GitHub username -->
  - evidence type: <!-- cosign_sign_blob / gpg_sign -->
  - fingerprint: <!-- OpenBao Transit cosign_key の fingerprint -->

### AI static analysis evidence

- [ ] **ai_cargo_clippy**: `cargo clippy -D warnings` pass (CI job: `ai-cargo-clippy`)
- [ ] **ai_semgrep**: `semgrep --strict` pass (CI job: `ai-semgrep`)
- [ ] **ai_release_gate**: `release_gate_status: green` (CI job: `check-release-gate`)

### Phase gate

- [ ] 対応 Phase の exit criteria (`release_gate.lock.yaml` の対応 cell が green) を満たす
- [ ] `src/_meta/dual_review/v1_implementation_review.dual_review.lock.yaml` の対応 phase entry を更新済み

---

🤖 Generated with [Claude Code](https://claude.com/claude-code)
