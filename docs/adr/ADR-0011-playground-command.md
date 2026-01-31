# ADR-0011: Playground コマンドの導入

## ステータス

承認済み（Accepted）

## 日付

2025-01-20

## コンテキスト

k1s0 の新規ユーザーがテンプレートの動作を確認するには、`new-feature` でプロジェクトを生成し、依存関係をセットアップし、ビルド・実行する必要があった。この手順は初学者にとってハードルが高く、テンプレートの評価や学習が困難であった。

## 決定

k1s0 CLI に `playground` サブコマンドを追加し、サンプルコード付きの一時的な開発環境を即座に起動できるようにする：

- `k1s0 playground start --type <type>`: テンプレートタイプを指定して playground を起動
- `k1s0 playground stop`: playground 環境を停止・削除
- `k1s0 playground status`: 実行中の playground を確認
- `k1s0 playground list`: 利用可能なテンプレートを一覧表示

### 対応テンプレート

- backend-rust, backend-go, backend-csharp, backend-python
- frontend-react, frontend-flutter

### モード

- `standalone`: 単体での起動（デフォルト）
- `local`: docker compose による関連サービス込みの起動

### 採用理由

- 新規ユーザーのオンボーディングを大幅に短縮できる
- テンプレートの動作確認を低コストで行える

## 帰結

### 正の帰結

- 学習コストの低減
- テンプレートの品質検証が容易になる

### 負の帰結

- playground テンプレートの保守コストが追加される
- Docker が必要（`local` モード時）

## 関連ドキュメント

- [CLI 設計書 - playground コマンド](../design/cli/commands-playground.md)
- [ADR-0004](ADR-0004-docker-integration.md) - Docker 統合
