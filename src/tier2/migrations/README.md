# tier2 migrations

tier2 の DB マイグレーションファイルを格納する。

## 実行方法

```bash
# sqlx CLI をインストールする
cargo install sqlx-cli --features postgres

# マイグレーションを実行する
sqlx migrate run \
  --database-url "postgres://k1s0:k1s0dev@localhost/tier2_dev" \
  --source src/tier2/migrations/
```

## ファイル一覧

| ファイル | 内容 |
|---|---|
| `0001_initial_schema.sql` | `domain_event` / `outbox_message` / `audit_event` テーブル作成 |
| `0002_rls_policies.sql` | テナント RLS ポリシー適用（RLS FORCE） |
| `0003_audit_hash_chain.sql` | audit hash chain 用カラム追加（`hash_digest` / `prev_digest`） |

## 開発環境の起動

```bash
# tier2 開発環境を起動する（PostgreSQL + Kafka + ClickHouse）
docker compose -f src/tier2/docker-compose.dev.yml up -d

# マイグレーションを実行する
sqlx migrate run \
  --database-url "postgres://k1s0:k1s0dev@localhost/tier2_dev" \
  --source src/tier2/migrations/
```
