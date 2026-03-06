# Repository Policies

## Cargo.lock

- Commit lockfiles for executable Rust workspaces and applications.
- Keep `CLI/Cargo.lock` tracked.
- Do not commit lockfiles for reusable Rust libraries unless there is a specific release-process reason.

## Generated Artifacts

- `api/proto/gen` is committed output generated from Buf.
- Regenerate with `scripts/generate-proto.sh`.
- Verify with `scripts/check-proto-generated.sh` before merging proto-related changes.
