# ADR-0115: DB マイグレーション実行戦略（外部実行 vs 起動時自動実行）

## ステータス

承認済み

## コンテキスト

k1s0 プロジェクトでは、データベーススキーマの管理方法がサービスによって異なっており、外部監査（ARCH-002, MED-007）で以下の問題が指摘された:

1. **3 種類の混在方式**:
   - **sqlx migrate!()**: 起動時に自動実行（event-store 等の一部サービス）
   - **init-db SQL**: PostgreSQL の `docker-entrypoint-initdb.d` でコンテナ初期化時に実行（auth, config, saga 等の init-db 管理サービス）
   - **just migrate-all**: 外部 CLI から手動実行（task, board, activity 等の service tier）

2. **CRIT-004**: マイグレーションが自動実行されないため、新規環境構築時にサービスが DB スキーマ不整合でクラッシュする
3. **人的オペレーション依存**: `just migrate-all` を忘れると起動エラーになる

## 決定

以下の方針を採用する:

### 開発環境（docker-compose）

`just local-up-dev` に 2 フェーズ起動を実装済み:
1. Phase 1: infra サービス起動 + healthcheck 通過を待機
2. Phase 2: `just migrate-all` を自動実行してからアプリケーションサービスを起動

```bash
# local-up-dev が内部で実行するフロー
just local-up           # Phase 1: infra 起動
sleep 5                  # DB 安定化待機
just migrate-all         # Phase 2: 全マイグレーション実行
just local-up-system     # Phase 3: system アプリ起動
```

### 本番環境（Kubernetes）

Init Container パターンを採用する（実装予定）:

```yaml
# Deployment.spec.initContainers の例
initContainers:
  - name: migrate
    image: ghcr.io/k1s0/sqlx-migrate:latest
    command: ["sqlx", "migrate", "run"]
    env:
      - name: DATABASE_URL
        valueFrom:
          secretKeyRef:
            name: db-secret
            key: url
```

### 長期方針: 起動時自動実行への移行

新規サービスでは `sqlx::migrate!().run(&pool).await?` を起動時に実行することを推奨する。

```rust
// 推奨パターン（新規サービス）
let pool = sqlx::PgPool::connect(&database_url).await?;
sqlx::migrate!("./migrations").run(&pool).await
    .context("DB マイグレーション実行に失敗")?;
```

ただし、既存サービスへの適用は移行リスクを伴うため段階的に行う。

## 理由

1. **移行リスクの局限**: 全サービスを一斉に起動時自動実行に切り替えると、マイグレーション失敗がサービス起動失敗に直結する。既存サービスは現状維持が安全
2. **Init Container の堅牢性**: K8s では Init Container が完了するまで Pod が起動しないため、スキーマ未適用状態でサービスが起動するリスクを排除できる
3. **docker-compose での 2 フェーズ**: `just local-up-dev` の自動化により人的オペレーションの漏れを防止する（CRIT-004 対応）

## 影響

**ポジティブな影響**:
- `just local-up-dev` により新規環境構築時の手動操作が不要になる
- K8s では Init Container により確実なマイグレーション実行が保証される
- 新規サービスは起動時自動実行で実装することで一貫性が向上する

**ネガティブな影響・トレードオフ**:
- 3 種類の混在方式は短期的に継続する（既存サービスへの影響を最小化するため）
- `just migrate-all` を直接実行した場合は依然として手動操作が必要
- Init Container の実装は全サービスの Helm Chart 更新が必要（工数大）

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 全サービス起動時自動実行 | 全 Rust サービスで `sqlx::migrate!().run()` を追加 | 既存サービスへの変更が多く、移行リスクが高い。段階的移行を優先 |
| マイグレーション専用サービス | マイグレーションのみを実行する docker-compose サービスを追加 | Init Container の K8s パターンと二重管理になる |
| FlywayDB / Liquibase | Java ベースのマイグレーションツールに切り替え | Rust/sqlx エコシステムとの親和性が低い |

## 参考

- [`docs/infrastructure/docker/docker-compose設計.md`](../../../infrastructure/docker/docker-compose設計.md) — migrate-all フロー説明
- [`justfile`](../../../../justfile) — `local-up-dev`・`migrate-all` の実装
- ADR-0068 — readyz チェックの統一方針（マイグレーション前後の readyz 挙動に関連）

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-06 | 初版作成（ARCH-002, MED-007, CRIT-004 外部監査対応） | kiso ryuhei |
