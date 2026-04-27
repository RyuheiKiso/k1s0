# src/contracts — k1s0 single source of truth（Protobuf）

本ディレクトリは tier1 公開 12 API と内部 gRPC を Protobuf で定義する。
**契約は手書き、SDK は生成物**。`buf generate` で 4 言語 SDK（.NET / Go / Rust / TypeScript）を自動生成する。
詳細設計は [`docs/05_実装/00_ディレクトリ設計/30_共通契約レイアウト/`](../../docs/05_実装/00_ディレクトリ設計/30_共通契約レイアウト/)。

## 配置構造

```text
contracts/
├── buf.yaml                          # 2 module 構成（tier1 / internal）
├── buf.lock                          # 依存固定
├── tier1/                            # 公開 12 API（tier2 / tier3 から見える）
│   └── k1s0/tier1/<api>/v1/*.proto
└── internal/                         # tier1 内部 gRPC（Go ↔ Rust core）
    └── k1s0/internal/<comp>/v1/*.proto
```

## 公開 12 API（`tier1/k1s0/tier1/<api>/v1/`）

| API | 主な動詞 | RPC 数 |
|---|---|---|
| `state` | Get / Save / Delete / GetBulk / SaveBulk | 5 |
| `pubsub` | Publish / Subscribe（streaming）/ DeleteSubscription | 3 |
| `serviceinvoke` | Invoke / InvokeStream（streaming） | 2 |
| `secrets` | Get / GetBulk / Rotate | 3 |
| `binding` | Invoke | 1 |
| `workflow` | Start / Signal / Query / Cancel / Terminate / GetStatus | 6 |
| `log` | Send（streaming）/ SendBatch | 2 |
| `telemetry` | EmitMetric / EmitSpan | 2 |
| `decision` | Evaluate / BatchEvaluate / RegisterRule / ListVersions / GetRule | 5 |
| `audit` | Record / Query | 2 |
| `pii` | Classify / Mask | 2 |
| `feature` | EvaluateBoolean / String / Number / Object / Refresh / RegisterFlag / GetFlag / ListFlags | 7（公開）+ 3（admin） |
| `health` | Check / Watch（streaming） | 2（gRPC 標準） |

合計 43 RPC（health 除く）/ 47 RPC（health 含む）。共通型は `tier1/k1s0/tier1/common/v1/common.proto` に集約
（`TenantContext` / `ErrorDetail` / `K1s0ErrorCategory`）。

## 内部 gRPC（`internal/k1s0/internal/<comp>/v1/`）

| パッケージ | 主題 | 設計 |
|---|---|---|
| `errors.v1` | `ErrorDetail` + 6 区分 `ErrorCategory` | DS-SW-IIF-004 |
| `audit.v1.AuditService` | `AppendHash` / `VerifyChain`（Server Streaming） | DS-SW-IIF-005 |
| `decision.v1.DecisionService` | `EvaluateDecision`（collect_trace 対応） | DS-SW-IIF-006 |
| `pii.v1.PiiMaskService` | `MaskPii` / `MaskPiiBatch`（p99 < 3ms 目標） | DS-SW-IIF-007 |

## 生成パス

`buf generate` の出力先（docs 正典）:

| 言語 | 出力先 |
|---|---|
| Go（公開） | `src/sdk/go/proto/v1/k1s0/tier1/<api>/v1/` |
| Rust（公開） | `src/sdk/rust/crates/k1s0-sdk-proto/src/gen/v1/` |
| TypeScript（公開） | `src/sdk/typescript/src/proto/k1s0/tier1/<api>/v1/` |
| .NET（公開） | `src/sdk/dotnet/src/K1s0.Sdk.Proto/Generated/` |
| Go（内部） | `src/tier1/go/internal/proto/v1/` |
| Rust（内部） | `src/tier1/rust/crates/proto-gen/src/` |

## ゲート

- `buf lint` / `buf format` / `buf breaking`（PR で 3 種すべて通過必須）
- IDL 命名規約と STANDARD lint の衝突箇所は `buf.yaml` の `lint.except` で除外（IDL 正典を優先）

## 関連設計

- [ADR-TIER1-002](../../docs/02_構想設計/adr/ADR-TIER1-002-protobuf-grpc.md) — Protobuf gRPC 内部通信
- [ADR-TIER1-003](../../docs/02_構想設計/adr/ADR-TIER1-003-language-invisibility.md) — 内部言語の不可視化
- [docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/](../../docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/) — IDL 正典
