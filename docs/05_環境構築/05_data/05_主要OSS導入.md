---
id: env.data.data_oss_install
axis: data
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.data.data_repository_acquisition
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# 主要 OSS 導入

## 一文方針

- `docker compose up` で postgres / kafka / clickhouse / apicurio / valkey の 5 サービスを同時起動し、全て Healthy 状態になることを主要 OSS 導入の検収条件とする。

## docker-compose.yml の雛形

以下の `docker-compose.yml` を作業ディレクトリに配置する（リポジトリには `src/data/local/docker-compose.yml` として管理する）。

```yaml
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: k1s0
      POSTGRES_PASSWORD: k1s0pass
      POSTGRES_DB: k1s0db
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U k1s0"]
      interval: 10s
      timeout: 5s
      retries: 5

  kafka:
    image: confluentinc/cp-kafka:7.6.0
    environment:
      KAFKA_NODE_ID: 1
      KAFKA_PROCESS_ROLES: broker,controller
      KAFKA_LISTENERS: PLAINTEXT://0.0.0.0:9092,CONTROLLER://0.0.0.0:9093
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://localhost:9092
      KAFKA_CONTROLLER_QUORUM_VOTERS: 1@localhost:9093
      KAFKA_CONTROLLER_LISTENER_NAMES: CONTROLLER
      CLUSTER_ID: k1s0-local-cluster
    ports:
      - "9092:9092"
    healthcheck:
      test: ["CMD", "kafka-topics", "--bootstrap-server", "localhost:9092", "--list"]
      interval: 20s
      timeout: 10s
      retries: 5

  clickhouse:
    image: clickhouse/clickhouse-server:24.3
    ports:
      - "8123:8123"
      - "9000:9000"
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:8123/ping"]
      interval: 10s
      timeout: 5s
      retries: 5

  apicurio:
    image: apicurio/apicurio-registry-mem:2.5.0.Final
    ports:
      - "8080:8080"
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:8080/health/ready"]
      interval: 10s
      timeout: 5s
      retries: 10

  valkey:
    image: valkey/valkey:7.2-alpine
    ports:
      - "6379:6379"
    healthcheck:
      test: ["CMD", "valkey-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5
```

## 5 サービスの起動

```bash
# 上記 docker-compose.yml があるディレクトリで実行
docker compose up -d

# 全サービスが Healthy になるまで確認
docker compose ps
```

## 接続確認

```bash
# PostgreSQL
psql -h localhost -U k1s0 -d k1s0db -c "SELECT version();"

# Kafka
kafka-topics.sh --bootstrap-server localhost:9092 --list

# ClickHouse
clickhouse-client --host localhost --query "SELECT version()"

# Apicurio
curl -s http://localhost:8080/apis/registry/v2/system/info | python3 -m json.tool

# Valkey
redis-cli -p 6379 ping
```

## Rook+Ceph について

Rook+Ceph は kind cluster 上でのみ動作する。ローカル docker compose 環境では代替として MinIO（S3 互換）を使って backup 先ストレージを模倣する。kind cluster の設定は infra 軸エンジニアと協力して構築する。

## 検収コマンド

```bash
docker compose ps
# 5 サービス全て Status: healthy であること
```

## 関連参照

- [04_リポジトリ取得手順](04_リポジトリ取得手順.md)
- [06_開発エディタIDE設定](06_開発エディタIDE設定.md)
- [07_テスト検証環境](07_テスト検証環境.md)
- [11_軸固有環境設定](11_軸固有環境設定.md)
