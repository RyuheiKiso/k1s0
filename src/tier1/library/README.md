# tier1/library

4 言語等価強度の SDK。Bidi 適合仕様 3-layer split 規約に従い、全言語で 3 層に分離する。

| 言語 | ディレクトリ |
|---|---|
| Rust | `rust/` |
| C# (.NET 8+) | `csharp/` |
| Go | `go/` |
| TypeScript | `typescript/` |

設計方針は `docs/03_概要設計/02_tier1設計方針/README.md` を参照。

## 3-layer split 規約（Y-bidi-library-split）

全 4 言語は以下の 3 層に分離する。各層の責務は厳密に限定し、層をまたぐ依存は禁止する。

| 層 | 責務 | Rust | Go | C# | TypeScript |
|---|---|---|---|---|---|
| **Layer 1: SDK（公開 API）** | tier2 / tier3 / client が消費する公開 API | `crate::core` / `crate::backend` / `crate::frontend` | 各パッケージ（auth_context / keyhandle / cache 等） | `K1s0.Tier1` 名前空間（KeyHandle.cs / AuthContext.cs 等） | `index.ts` から re-export される型 |
| **Layer 2: Internal（server 内部 API）** | tier1 server 実装のみが使う内部 API（公開 API に含めない） | `pub(crate)` スコープ限定（`#[doc(hidden)]`） | `internal/` パッケージ（Go package internal 規則） | `K1s0.Tier1.Internal` 名前空間（`internal` アクセス修飾子） | export しない型（モジュール非公開） |
| **Layer 3: ProtoBridge（Buf codegen wrapper）** | buf generate 出力の proto 型 → Library 型変換のみ | `proto_bridge.rs`（`pub(crate)`） | `proto_bridge/` パッケージ | `K1s0.Tier1.ProtoBridge` 名前空間（`internal`） | `proto_bridge.ts`（index.ts 非 re-export） |

## 移動済みコンポーネント

- `.NET Framework 4.6.2+` 互換ラッパー（`AuthContextCompat` / `KeyHandleCompat`）は `src/tier3/csharp/legacy/tier1_compat/` に移動済み。
