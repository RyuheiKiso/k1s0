# ADR-0095: CLI テンプレートバージョンのワークスペース同期ポリシー

## ステータス

承認済み

## コンテキスト

`CLI/crates/k1s0-cli/templates/` 配下の Rust / Go テンプレートが、実際のワークスペース（`regions/system/Cargo.toml`）で管理される依存バージョンと乖離していた。

乖離が発生していた依存の一覧（外部監査 C-006, H-012, L-003 で指摘）:

| テンプレートファイル | 依存 | テンプレート版 | ワークスペース版 |
|---|---|---|---|
| `server/rust/Cargo.toml.tera` | axum | 0.7 | 0.8 |
| `server/rust/Cargo.toml.tera` | opentelemetry | 0.24 | 0.27 |
| `server/rust/Cargo.toml.tera` | opentelemetry-otlp | 0.17 | 0.27 |
| `server/rust/Cargo.toml.tera` | opentelemetry_sdk | 0.24 | 0.27 |
| `server/rust/Cargo.toml.tera` | utoipa-swagger-ui | 8 | 9 |
| `server/rust/Cargo.toml.tera` | mockall | 0.13 | 0.14 |
| `bff/rust/Cargo.toml.tera` | actix-web | 4 | 使用禁止（axum 0.8 に移行済み） |
| `bff/rust/main.rs.tera` | actix-web パターン | HttpServer::new | axum Router/serve |
| `bff/go/go.mod.tera` | gqlgen | v0.17.45 | v0.17.55 |
| `bff/go/go.mod.tera` | gqlparser/v2 | v2.5.11 | v2.5.19 |

テンプレートから生成されたサービスが古いバージョンを参照した場合、以下のリスクが生じる:

- axum 0.7 と 0.8 は API が非互換（Router 構築方法・State 抽出等）であり、ワークスペースライブラリとのリンクに失敗する。
- opentelemetry 0.24 → 0.27 は破壊的変更を含み、テレメトリ初期化が実行時にパニックする可能性がある。
- BFF テンプレートが actix-web を使用している場合、ワークスペースが axum に統一されているため依存ツリーが重複し、バイナリサイズと compile time が増大する。
- gqlgen のバージョン相違は server/bff 間でコード生成スキーマの互換性を損なう。

## 決定

1. **Rust server テンプレート** (`server/rust/Cargo.toml.tera`) の依存バージョンをワークスペース (`regions/system/Cargo.toml`) と完全に一致させる。
2. **Rust BFF テンプレート** (`bff/rust/`) を actix-web から axum 0.8 へ移行し、`main.rs.tera` も axum の `Router`/`axum::serve` パターンに書き換える。
3. **Go BFF テンプレート** (`bff/go/go.mod.tera`) の gqlgen を `v0.17.55`、gqlparser を `v2.5.19` に更新し server テンプレートと統一する。
4. **今後のポリシー**: ワークスペースの依存バージョンを更新する際は、対応するテンプレートを同時に更新する。PRレビューチェックリストに「テンプレートバージョン同期確認」を追加する。

## 理由

テンプレートはワークスペース上で実際に動作するサービスのひな型であるため、バージョンが一致していなければ生成直後にビルドが失敗する。ワークスペースと同一バージョンを維持することで、生成されたコードがそのままビルド・テスト可能な状態を保証できる。

actix-web → axum 移行は、ワークスペース全体が既に axum に統一済みであり、BFF のみ actix-web を残すことで二重依存・API 不整合・学習コスト増大が生じるため、早期に統一する。

## 影響

**ポジティブな影響**:

- テンプレートから生成したサービスが即座にビルド・実行可能になる。
- ワークスペース全体で HTTP フレームワーク・オブザーバビリティスタックが axum / opentelemetry 0.27 に統一される。
- 外部監査指摘（C-006 CRITICAL / H-012 HIGH / L-003 LOW）が解消される。

**ネガティブな影響・トレードオフ**:

- 既存の actix-web ベース BFF を既に生成している場合、手動移行が必要になる。

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| テンプレートにバージョン変数を外出し | Tera 変数でバージョンを制御 | バージョン管理が二重化し、同期漏れリスクが増える |
| actix-web を BFF のみ維持 | BFF は actix-web 継続 | ワークスペースの統一方針に反し依存ツリーが膨張する |
| 自動同期 CI スクリプト | workspace Cargo.toml からテンプレートを自動生成 | 対応コストが大きく即時解消が難しい |

## 参考

- [ADR-0041: Rust BFF フレームワーク統一](0041-rust-bff-framework-unification.md)
- [ADR-0044: buf validate Phase1](0044-buf-validate-phase1.md)
- `regions/system/Cargo.toml` — ワークスペース依存定義の正規ソース
- `CLI/crates/k1s0-cli/templates/` — テンプレート群

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-04 | 初版作成（外部監査 C-006/H-012/L-003 対応） | @kiso ryuhei |
