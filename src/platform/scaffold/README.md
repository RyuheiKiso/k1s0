# k1s0-scaffold

Backstage Software Template v1beta3 互換の Scaffold CLI（Rust）。tier2 / tier3 の新規サービス雛形を 1 コマンドで展開する。

## 設計正典

- [`docs/05_実装/20_コード生成設計/30_Scaffold_CLI/01_Scaffold_CLI設計.md`](../../../docs/05_実装/20_コード生成設計/30_Scaffold_CLI/01_Scaffold_CLI設計.md) — IMP-CODEGEN-SCF-030〜037
- [`docs/05_実装/50_開発者体験設計/30_Scaffold_CLI運用/01_Scaffold_CLI運用.md`](../../../docs/05_実装/50_開発者体験設計/30_Scaffold_CLI運用/01_Scaffold_CLI運用.md) — DX 運用設計
- [`docs/02_構想設計/adr/ADR-DEV-001-paved-road.md`](../../../docs/02_構想設計/adr/ADR-DEV-001-paved-road.md) — Paved Road 思想

## 役割

- 採用組織の開発者が `k1s0-scaffold new tier2-go-service --name foo --owner @k1s0/payment` を実行すると、tier2 Go ドメインサービスの最小セット（go.mod / cmd / internal / Dockerfile / catalog-info.yaml）が一式生成される
- CLI 経路と Backstage UI 経路（custom action `k1s0:scaffold-engine`）が同一の engine（`src/lib.rs`）を呼び、生成結果のバイト一致を保証する
- 4 テンプレート対応:
  - [`src/tier2/templates/tier2-go-service/`](../../tier2/templates/tier2-go-service/)
  - [`src/tier2/templates/tier2-dotnet-service/`](../../tier2/templates/tier2-dotnet-service/)
  - [`src/tier3/templates/tier3-bff/`](../../tier3/templates/tier3-bff/)
  - [`src/tier3/templates/tier3-web/`](../../tier3/templates/tier3-web/)

## サブコマンド契約

| サブコマンド | 動作 |
|---|---|
| `k1s0-scaffold list` | テンプレート一覧 |
| `k1s0-scaffold new <template> --name <n> --owner <o> [--namespace <ns>] [--system <s>]` | 雛形生成 |
| `k1s0-scaffold new <template> --input <json> --dry-run` | 入力 JSON で生成、ファイル出力なしで diff のみ stdout（CI / golden test 用） |

## ビルド

```bash
cd src/platform/scaffold
cargo build --release
# 単一静的バイナリが target/release/k1s0-scaffold に出力される
```

GitHub Releases で multi-arch（linux-x64 / linux-arm64 / darwin-arm64 / windows-x64）配布する想定。

## テンプレート変数

最小セットは 5 + 1 の 6 変数:

| 変数 | 用途 |
|---|---|
| `name` | サービス名（kebab-case） |
| `owner` | 所有チーム |
| `system` | 所属サブシステム |
| `description` | 概要説明 |
| `namespace` | .NET ルート名前空間（tier2-dotnet-service のみ） |
| `tier` / `language` | テンプレート種別が `template.yaml` で固定指定（catalog 自動付与） |

## golden snapshot 検証

`tests/golden/scaffold/<template-name>/` に固定入力での出力期待値を保存する。CI で `tools/codegen/scaffold/run.sh --check` 相当が `compare-outputs.sh` を呼んで diff 検出する（IMP-CODEGEN-POL-005）。

## アーキテクチャ

- `src/main.rs` — CLI（clap）
- `src/lib.rs` — engine 公開 API（Backstage custom action 経路はこちら）
- `src/template.rs` — `template.yaml` パース（serde_yaml）
- `src/engine.rs` — Handlebars テンプレ展開 + walkdir 再帰
- `src/error.rs` — 統一エラー型（thiserror）
