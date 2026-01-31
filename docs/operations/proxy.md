# プロキシ環境での Docker 使用方法

企業ネットワーク等でプロキシを経由する環境での Docker ビルド・実行方法を説明する。

## 1. Docker ビルド時のプロキシ設定

### k1s0 CLI を使用する場合

```bash
k1s0 docker build --http-proxy http://proxy.example.com:8080 --https-proxy http://proxy.example.com:8080
```

### docker build を直接使用する場合

```bash
docker build \
  --build-arg HTTP_PROXY=http://proxy.example.com:8080 \
  --build-arg HTTPS_PROXY=http://proxy.example.com:8080 \
  --build-arg NO_PROXY=localhost,127.0.0.1 \
  -t my-app:latest .
```

## 2. Docker Compose でのプロキシ設定

`docker-compose.yml` の `build.args` にプロキシを追加する:

```yaml
services:
  app:
    build:
      context: .
      args:
        HTTP_PROXY: http://proxy.example.com:8080
        HTTPS_PROXY: http://proxy.example.com:8080
        NO_PROXY: localhost,127.0.0.1,db,redis
```

**注意:** `NO_PROXY` に compose 内の他サービス名（`db`, `redis` 等）を含めること。

## 3. Dockerfile のプロキシ対応

k1s0 が生成する全 Dockerfile には以下の `ARG` が宣言済み:

```dockerfile
ARG HTTP_PROXY
ARG HTTPS_PROXY
ARG NO_PROXY
```

ビルド時に `--build-arg` で渡された値が自動的に使用される。ランタイムには引き継がれない（セキュリティ上の理由）。

## 4. Docker デーモンのプロキシ設定

Docker デーモン自体にプロキシを設定する場合は、Docker Desktop の設定画面または `/etc/docker/daemon.json` で設定する。

### Linux

```bash
sudo mkdir -p /etc/systemd/system/docker.service.d
sudo tee /etc/systemd/system/docker.service.d/proxy.conf <<EOF
[Service]
Environment="HTTP_PROXY=http://proxy.example.com:8080"
Environment="HTTPS_PROXY=http://proxy.example.com:8080"
Environment="NO_PROXY=localhost,127.0.0.1"
EOF
sudo systemctl daemon-reload
sudo systemctl restart docker
```

### Windows / macOS

Docker Desktop → Settings → Resources → Proxies で設定。

## 関連ドキュメント

- [CLI 設計](../design/cli/): docker コマンドの詳細
- [テンプレート設計](../design/template/): Dockerfile テンプレート仕様
