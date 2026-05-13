---
id: env.tier2.tier2_oss_install
axis: tier2
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.tier2.tier2_repository_acquisition
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# 主要 OSS 導入

## 一文方針

- ドメインイベント生成器 / 決定表 lint ツール / workflow 定義 lint / Buf + Apicurio を導入し、各ツールが起動することを本ページの検収とする。

## Buf CLI + Apicurio（tier1 と共通）

```bash
go install github.com/bufbuild/buf/cmd/buf@latest
buf --version

docker pull apicurio/apicurio-registry-mem:latest
```

## ドメインイベント生成器

Domain Event スキーマ（Avro）から各言語のコードを生成する。

```bash
# avro-tools（Java が必要）
docker pull apache/avro-tools:latest
# または
# rust-avro-codegen
cargo install avro-codegen
```

## 決定表 lint ツール

業務ルールを表形式で定義した決定表の整合性を検証する。

```bash
# FEEL / DMN ベースの決定表検証（camunda dmn CLI）
npm install -g @camunda/dmn-js-cli
# または
docker pull camunda/camunda-bpm-platform
```

## workflow 定義 lint

BPMN / 独自 workflow 定義の lint ツール。

```bash
# bpmnlint
npm install -g bpmnlint
bpmnlint --version
```

## PostgreSQL クライアント

```bash
sudo apt install -y postgresql-client
psql --version
```

## Kafka CLI

```bash
# kafka-tools（Docker 経由）
docker pull confluentinc/cp-kafka:latest
# または
sudo apt install -y kafkacat
```

## 検収コマンド

```bash
buf --version
bpmnlint --version
psql --version
docker images | grep -E "apicurio|kafka"
```

## 関連参照

- [04_リポジトリ取得手順](04_リポジトリ取得手順.md)
- [06_開発エディタIDE設定](06_開発エディタIDE設定.md)
- [07_テスト検証環境](07_テスト検証環境.md)
