# tools — 横断ツール（local-stack / devcontainer / codegen / sparse / git-hooks / ci）

開発者体験（DX）と CI を支える横断ツール群。
詳細設計は [`docs/05_実装/00_ディレクトリ設計/70_共通資産/01_tools配置.md`](../docs/05_実装/00_ディレクトリ設計/70_共通資産/01_tools配置.md)。

## 配置

```text
tools/
├── README.md                                       # 本ファイル
├── _link_check.py                                  # docs 横断リンクチェッカ
├── _link_fix.py                                    # docs リンク自動修正
├── _export_svg.py                                  # drawio → SVG 一括 export
├── local-stack/                                    # kind ベース本番再現スタック（IMP-DEV-POL-006）
│   ├── up.sh / down.sh / status.sh
│   ├── kind-cluster.yaml
│   └── manifests/{20..95}_*/                       # 17 レイヤ namespace yaml
├── devcontainer/                                   # 10 役 Dev Container プロファイル
│   ├── postCreate.sh / doctor.sh / README.md
│   └── profiles/{tier1-rust-dev, tier2-dev, ...}/
├── codegen/                                        # buf / openapi / grpc-docs 生成ラッパ
│   ├── buf/run.sh                                  # buf generate + lint + breaking
│   ├── openapi/run.sh                              # proto → OpenAPI v2 export
│   └── grpc-docs/run.sh                            # proto → gRPC reference Markdown
├── sparse/                                         # sparse-checkout 10 役 cone 定義
│   ├── checkout-role.sh / verify.sh
│   └── cones/{tier1-rust-dev, ...}.txt
├── git-hooks/                                      # 自作 pre-commit hook
│   ├── japanese-header-guard.py                    # 日本語ヘッダコメント強制（src/CLAUDE.md）
│   ├── file-length-guard.py                        # 1 ファイル 500 行以内強制
│   ├── drawio-svg-staleness.sh                     # drawio 更新後 SVG 未 export 検知
│   └── link-check-wrapper.py                       # docs リンクチェック wrapper
├── ci/                                             # 内製 CI linter
│   ├── go-dep-check/                               # Go 側 依存方向 linter（独立 go.mod）
│   └── rust-dep-check/                             # Rust 側 依存方向 linter
└── migration/                                      # .NET Framework → .NET 8 移行支援
```

## ローカル開発の起点

```sh
# 役割を選んで sparse-checkout
./tools/sparse/checkout-role.sh tier1-rust-dev

# kind ベースのインフラを起動
./tools/local-stack/up.sh

# 自作 CLI / コード生成
./tools/codegen/buf/run.sh                  # contracts → 4 SDK 再生成
./tools/codegen/openapi/run.sh --check      # OpenAPI 差分チェック
```

## CI ゲート連携

| Hook / linter | 役割 | 失敗時の動作 |
|---|---|---|
| `japanese-header-guard.py` | 日本語ヘッダコメント | pre-commit で reject |
| `file-length-guard.py` | 500 行制限 | pre-commit で reject |
| `drawio-svg-staleness.sh` | drawio 更新時 SVG 自動 export | pre-commit で reject |
| `go-dep-check` | tier 越境 import 禁止 | CI で reject |
| `rust-dep-check` | Cargo.toml の path 依存違反 | CI で reject |

## 関連設計

- [docs/05_実装/00_ディレクトリ設計/70_共通資産/01_tools配置.md](../docs/05_実装/00_ディレクトリ設計/70_共通資産/01_tools配置.md)
- [ADR-DEV-001](../docs/02_構想設計/adr/ADR-DEV-001-paved-road.md) — Paved Road
- [IMP-DEV-POL-006](../docs/05_実装/) — local-stack ベース DX
- [IMP-DIR-ROOT-002](../docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md) — 依存方向強制
