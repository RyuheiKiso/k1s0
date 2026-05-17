# tier1 コーディングポリシー

全軸共通ルールは `src/CLAUDE.md` を参照。本ファイルは tier1 固有の制約のみ記述する。

## 言語・workspace

- **Server**: Rust stable
- **Operator**: Go 1.22+（controller-runtime / kubebuilder）
- **Library**: Rust / C# (.NET 8+) / Go / TypeScript（4 言語等価強度）
- **Cargo workspace root**: `src/tier1/Cargo.toml`
- **go.work root**: `src/tier1/go.work`
- **Buf workspace**: `src/tier1/schema/buf.work.yaml`
- **lock.yaml 配置先**: `src/tier1/lock/`（手書き禁止）

設計パターン・モジュール構成の詳細は `docs/03_概要設計/02_tier1設計方針/README.md` を単一の真とする。

## コーディング制約

### proto / codegen

- `.proto` が単一の真 → 手書き SDK / 手書き生成コード diff 禁止（diff 検出で CI fail）
- `buf generate` の出力を手書きで編集禁止（`generated/` は codegen 出力専用）
- proto の Buf workspace は `src/tier1/schema/buf.work.yaml` に集約（root 配置禁止）

### 必須 annotation

全 RPC method に以下の annotation が必須（欠落で CI fail）:

```protobuf
option (tier1.bidi.conformance_class) = ...;
option (tier1.auth.auth_class) = ...;
option (tier1.slo.slo_class) = ...;
option (tier1.quota.quota_class) = ...;
option (tier1.observability.signal_class) = ...;
// PII フィールドには必須:
option (tier1.pii.field_pii) = true;
option (tier1.pii.redaction_class) = ...;
```

### 公開 API の型制約

- 公開シグネチャに生 key bytes / 生 access_token を露出禁止 → `KeyHandle` / `AuthContext` を必須型として使う
- L3/L2*/L1+ カテゴリの OSS 型を公開 API シグネチャに露出禁止（`cargo-deny` / `depguard` / `BannedApiAnalyzers` で CI fail）
- `capabilities.lock.yaml` との双方向 lock 違反禁止（追加した API は capabilities に反映する）

### lint ツール

| 言語 | ツール |
|---|---|
| Rust | `cargo-deny [bans]` + `clippy` + `cargo public-api` snapshot drift 禁止 |
| Go | `golangci-lint depguard` + `go-apidiff` + 自製 `go/analysis` |
| C# | `BannedApiAnalyzers` + Central Package Management + `PublicApiAnalyzers` + `K1s0Analyzer` |
| TypeScript | `eslint-plugin-import no-restricted-imports` + `eslint-plugin-boundaries` + `api-extractor` |

### ship blocker

- `dry_run.lock.yaml` の `last_green_at` > 365 日で CI fail（4 primary pair × 5 phase migration drill）

## 関連参照

- `docs/03_概要設計/02_tier1設計方針/README.md` — 言語・設計パターン
- `docs/04_詳細設計/02_強制機構/01_tier1強制機構.md` — CI fail 条件の詳細
- `docs/04_詳細設計/01_適合仕様/01_Bidi適合仕様.md` — Bidi conformance_class
- `docs/04_詳細設計/01_適合仕様/08_OSSライフサイクル適合仕様.md` — dry_run
