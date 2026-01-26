
# k1s0 構想（正式版）

本ドキュメントは、k1s0 の framework / templates / CLI を含むモノレポ全体の「規約・方針・運用の固定」を定義する。

## 位置づけ

- この文書は「方針（何を固定するか）」を定める。実装詳細はテンプレ・コード・ADR・運用手順に分離する。
- 規約はドキュメントだけで終わらせず、テンプレ/CLI/CI により自動検知して逸脱を落とす。

## 用語

- framework: 共通部品（crate/ライブラリ）および共通マイクロサービス（auth/config/endpoint 等）
- template: 新規サービス生成の雛形（ディレクトリ構造・必須ファイル・設定・CI 等）
- CLI: 生成・導入・lint・upgrade を担うコマンド群（例: `k1s0 init`, `k1s0 new-feature`, `k1s0 lint`, `k1s0 upgrade`）

## キーワード規約

文中の MUST/SHOULD/MAY は RFC 2119 相当で解釈する。

---


