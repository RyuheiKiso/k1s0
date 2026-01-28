# アーキテクチャドキュメント

本ディレクトリには、k1s0 システムのアーキテクチャ設計ドキュメントを格納する。

## ドキュメント一覧

| ドキュメント | 説明 |
|-------------|------|
| [overview.md](./overview.md) | システム全体像、コンポーネント構成、技術スタック |
| [clean-architecture.md](./clean-architecture.md) | Clean Architecture の原則と k1s0 での適用方法 |
| [tier-system.md](./tier-system.md) | Tier システム（Tier1/2/3）と依存ルール |
| [service-mesh.md](./service-mesh.md) | gRPC サービス間通信、認証・認可、観測性 |

## 図

`diagrams/` ディレクトリには、各ドキュメントで使用する図（Mermaid 等）を格納する。

## 関連ドキュメント

- [ADR 一覧](../adr/README.md): アーキテクチャ上の重要な決定
- [規約一覧](../conventions/README.md): 開発規約
- [設計ドキュメント](../design/README.md): 各コンポーネントの詳細設計
