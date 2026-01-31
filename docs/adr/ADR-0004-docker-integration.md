# ADR-0004: Docker 統合コマンドの導入

## ステータス

承認済み（Accepted）

## 日付

2025-01-15

## コンテキスト

k1s0 で生成されたサービスは Docker コンテナとしてデプロイされることを前提としている。しかし、Docker イメージのビルドや docker compose による開発環境の起動は各チームが個別に行っており、以下の課題があった：

- Dockerfile の記述が各チームでばらばら
- docker compose の設定が統一されていない
- ビルド手順がドキュメント化されていない場合がある

## 決定

k1s0 CLI に `docker` サブコマンドを追加し、以下の操作を統一的に提供する：

- `k1s0 docker build`: テンプレートに沿った Docker イメージのビルド
- `k1s0 docker compose up/down/logs`: docker compose による開発環境の管理
- `k1s0 docker status`: コンテナ状態の確認

### 採用理由

- テンプレートで生成される Dockerfile/docker-compose.yml と CLI コマンドを一体化することで、開発体験を統一できる
- プロキシ設定（`--http-proxy`）などの共通オプションを CLI 側で吸収できる

## 帰結

### 正の帰結

- Docker 操作の統一化により、チーム間のばらつきが解消される
- CI/CD パイプラインとの連携が容易になる

### 負の帰結

- Docker / docker compose がインストールされていない環境では使用できない（`k1s0 doctor` で検出可能）

## 関連ドキュメント

- [CLI 設計書 - docker コマンド](../design/cli/commands-docker.md)
- [ADR-0001](ADR-0001-scope-and-prerequisites.md) - 実装スコープと前提
