---
id: env.data.data_editor_ide
axis: data
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.data.data_oss_install
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# 開発エディタ / IDE 設定

## 一文方針

- VS Code に PostgreSQL / Kafka / ClickHouse の操作拡張を導入し、ローカル docker compose 環境への接続を確認できる状態を data 軸の IDE 検収条件とする。

## VS Code 拡張のインストール

```bash
code --install-extension ckolkman.vscode-postgres
code --install-extension redhat.vscode-yaml
code --install-extension jebbs.plantuml
code --install-extension bierner.markdown-preview-github-styles
```

| 拡張 | 用途 |
|---|---|
| vscode-postgres | PostgreSQL への接続・クエリ実行・スキーマ確認 |
| vscode-yaml | docker-compose.yml / k8s manifest の補完 |
| plantuml | ER 図の作成（データモデル設計） |
| markdown-preview-github-styles | docs レビュー |

## VS Code で PostgreSQL に接続

`ckolkman.vscode-postgres` 拡張の設定:

```json
{
  "host": "localhost",
  "user": "k1s0",
  "password": "k1s0pass",
  "port": 5432,
  "database": "k1s0db",
  "ssl": false
}
```

## DBeaver (オプション)

DBeaver Community Edition は PostgreSQL / ClickHouse / Kafka（Kafka Manager）を GUI で管理できる。Windows 側にインストールして WSL2 の localhost に接続する。

## Kafka UI (オプション)

Kafka の topic / consumer group を可視化するために `provectuslabs/kafka-ui` を docker compose に追加することを推奨する。

```yaml
  kafka-ui:
    image: provectuslabs/kafka-ui:latest
    ports:
      - "8888:8080"
    environment:
      KAFKA_CLUSTERS_0_NAME: local
      KAFKA_CLUSTERS_0_BOOTSTRAPSERVERS: kafka:9092
```

## 検収コマンド

```bash
code --list-extensions | grep -E "postgres|yaml"
# 2 拡張が一覧に含まれること
```

## 関連参照

- [05_主要OSS導入](05_主要OSS導入.md)
- [07_テスト検証環境](07_テスト検証環境.md)
