# tier2-go-service テンプレート

tier2 ドメイン共通業務ロジックの新規 Go サービスを生成する Backstage Software Template。`k1s0-scaffold` CLI と Backstage UI の両経路から呼び出し可能（IMP-CODEGEN-SCF-030 / 031）。

## 利用方法

### CLI

```bash
k1s0-scaffold new tier2-go-service \
  --name example-svc \
  --owner @k1s0/payment \
  --system k1s0
```

### Backstage UI

`/create` から「Tier2 Go Service」を選択し、フォームに `name` / `owner` / `system` を入力。

## 生成内容

- `{{name}}/go.mod` — module: `github.com/<owner>/{{name}}`
- `{{name}}/cmd/{{name}}/main.go` — k1s0 SDK State.Save サンプル
- `{{name}}/internal/service.go` — domain logic placeholder
- `{{name}}/Dockerfile` — distroless multi-stage build
- `{{name}}/catalog-info.yaml` — Backstage Component（`spec.dependsOn` に SDK 自動付与）
- `{{name}}/.k1s0/template-version` — テンプレートバージョン（migration 用 metadata）

## テンプレート変数

| 変数 | 用途 | 既定値 |
|---|---|---|
| `name` | サービス名（kebab-case） | （必須） |
| `owner` | 所有チーム（`@k1s0/<team>`） | （必須） |
| `system` | サブシステム | `k1s0` |
| `description` | 概要説明 | `tier2 Go ドメインサービス` |
| `tier` | tier 識別（catalog 自動付与） | `tier2`（固定） |
| `language` | 言語識別（catalog 自動付与） | `go`（固定） |

## 関連設計

- [`docs/05_実装/20_コード生成設計/30_Scaffold_CLI/01_Scaffold_CLI設計.md`](../../../../docs/05_実装/20_コード生成設計/30_Scaffold_CLI/01_Scaffold_CLI設計.md)
- [`examples/tier2-go-service/`](../../../../examples/tier2-go-service/) — テンプレート展開後の典型形
