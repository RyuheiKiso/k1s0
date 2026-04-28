# `tools/ci/actions/` — CI composite action

本ディレクトリは GitHub Actions の **composite action** を保持する。reusable workflow（`.github/workflows/_reusable-*.yml`）の内部実装として呼び出される共通部品で、言語 toolchain のセットアップ + cache 規約の集約を担う。

## 関連設計

- [`docs/05_実装/30_CI_CD設計/15_monorepo_orchestration.md`](../../../docs/05_実装/30_CI_CD設計/15_monorepo_orchestration.md)（IMP-CI-MR-006）
- [`docs/05_実装/30_CI_CD設計/10_reusable_workflow/01_reusable_workflow設計.md`](../../../docs/05_実装/30_CI_CD設計/10_reusable_workflow/01_reusable_workflow設計.md)（IMP-CI-RWF-021）

## 配置

```text
tools/ci/actions/
├── README.md           # 本ファイル
├── setup-rust/         # Rust toolchain + cargo cache
│   └── action.yml
├── setup-go/           # Go toolchain + module / build cache
│   └── action.yml
├── setup-dotnet/       # .NET SDK + NuGet cache
│   └── action.yml
└── setup-node/         # Node.js + pnpm + pnpm-store cache
    └── action.yml
```

## 利用規約（IMP-CI-RWF-021）

composite action は **reusable workflow からのみ呼び出す**。PR トリガ workflow から直接 `uses: ./tools/ci/actions/setup-rust` を呼ぶことは禁止する。理由:

- 呼出経路が階層化されることで「どの quality gate を通っているか」が明示される
- composite action の入力 / 出力契約が reusable workflow の input 規約と整合する
- composite action の変更が reusable workflow をスキップして PR に直接到達しない

呼出例（reusable workflow 内から）:

```yaml
# .github/workflows/_reusable-lint.yml の中で
- uses: ./tools/ci/actions/setup-rust
  with:
    components: rustfmt,clippy
    workspace: src/tier1/rust

- uses: ./tools/ci/actions/setup-go
  with:
    go-version: "1.22"
    workspace: src/tier1/go
```

## cache 規約（IMP-CI-MR-002）

各 composite action は **L1 cache**（job 内 dependency cache）を担当する。L2（job 間 artifact）と L3（main pre-warm）は呼出側 reusable workflow / scheduled job が担当。

| composite action | cache key 雛形 |
|---|---|
| setup-rust | `${{ runner.os }}-cargo-<workspace>-${{ hashFiles('**/Cargo.lock') }}` |
| setup-go | `${{ runner.os }}-gobuild-<workspace>-${{ hashFiles('**/go.sum') }}` |
| setup-dotnet | `${{ runner.os }}-nuget-<workspace>-${{ hashFiles('**/packages.lock.json', '**/*.csproj') }}` |
| setup-node | `${{ runner.os }}-pnpm-<workspace>-${{ hashFiles('**/pnpm-lock.yaml') }}` |

`<workspace>` を key に含めることで、`tier1-rust-dev` と `sdk-dev` の cache が衝突せず分離される。

## 入出力契約

各 action.yml の `inputs:` は workspace path を必須とする（cache key 計算と作業ディレクトリ指定の両方に使用）。`outputs:` は基本持たない（toolchain 配置と cache 設定で完結）。

## 拡張ロードマップ

リリース時点 + で追加予定の composite action:

| 名前 | 用途 | 配置予定 |
|---|---|---|
| `buf-lint-and-breaking` | buf lint + breaking を 1 step に集約 | リリース時点+ 1ヶ月 |
| `sbom-generate` | syft の呼出共通化 | リリース時点+ 1ヶ月 |
| `cosign-sign` | OIDC keyless 署名共通化 | リリース時点+ 1ヶ月 |
| `setup-buf` | buf CLI install + BSR plugin cache | リリース時点+ 1ヶ月 |

リリース時点 では reusable workflow に直接ステップを書いている部分があるが、リファクタは段階的に行う。

## 関連

- [`.github/workflows/`](../../../.github/workflows/) — reusable workflow / pr / renovate
- [`tools/ci/path-filter/filters.yaml`](../path-filter/filters.yaml) — path-filter 11 軸
