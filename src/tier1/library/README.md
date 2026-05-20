# tier1/library

4 言語等価強度の SDK（将来実装）。

| 言語 | ディレクトリ |
|---|---|
| Rust | `rust/` |
| C# (.NET 8+) | `csharp/` |
| Go | `go/` |
| TypeScript | `typescript/` |

設計方針は `docs/03_概要設計/02_tier1設計方針/README.md` を参照。

## 移動済みコンポーネント

- `.NET Framework 4.6.2+` 互換ラッパー（`AuthContextCompat` / `KeyHandleCompat`）は `src/tier3/csharp/legacy/tier1_compat/` に移動済み。
