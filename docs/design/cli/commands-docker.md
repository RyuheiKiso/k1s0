# docker コマンド (v0.2.3)

← [CLI 設計書](./)

## 目的

Docker イメージのビルドと docker-compose によるローカル開発環境の操作を支援する。

## サブコマンド

### docker build

```bash
# 基本（タグは manifest から自動生成: {feature_name}:{template_version}）
k1s0 docker build

# カスタムタグ
k1s0 docker build --tag my-app:latest

# キャッシュ無効化
k1s0 docker build --no-cache

# プロキシ指定
k1s0 docker build --http-proxy http://proxy:8080 --https-proxy https://proxy:8080
```

### docker compose

```bash
# サービス起動（バックグラウンド + ビルド）
k1s0 docker compose up -d --build

# サービス停止（ボリューム含む）
k1s0 docker compose down -v

# ログ表示（フォロー）
k1s0 docker compose logs -f

# 特定サービスのログ
k1s0 docker compose logs app
```

### docker status

```bash
# コンテナ状態を表示
k1s0 docker status

# JSON 出力
k1s0 docker status --json
```

## エラーハンドリング

| エラー | 復旧提案 |
|--------|---------|
| Docker 未インストール | `https://docs.docker.com/get-docker/` を案内 |
| Docker Compose v2 なし | Docker Desktop またはプラグイン追加を案内 |
| Dockerfile なし | `k1s0 new-feature` で生成を案内 |
| docker-compose.yml なし | `k1s0 new-feature` で生成を案内 |
