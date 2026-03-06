# Proto Generation

Source of truth:
- `api/proto/**/*.proto`
- `api/proto/buf.yaml`
- `api/proto/buf.gen.yaml`

Committed generated artifacts:
- `api/proto/gen/go`
- `api/proto/gen/rust`
- `api/proto/gen/ts`

Commands:
```bash
./scripts/generate-proto.sh
./scripts/check-proto-generated.sh
```

Policy:
- Update generated code in the same change as any `.proto` or Buf template change.
- Do not hand-edit files under `api/proto/gen`.
- CI treats diffs under `api/proto/gen` after regeneration as a failure.
- The repository root `gen/` directory is not used and should remain absent.
