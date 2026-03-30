# ADR-0060: saga-rust 専用データベース（k1s0_saga）への分離

## ステータス

承認済み（2026-03-30）

## コンテキスト

saga-rust と master-maintenance は両サービスとも `k1s0_system` データベースを使用していた。
sqlx は初回マイグレーション実行時に対象データベースの public スキーマ内に `_sqlx_migrations` テーブルを作成する。
2つのサービスが同一データベースを共有すると、次の問題が発生する。

- `_sqlx_migrations` テーブルへの同時書き込みによるマイグレーション競合
- saga のスキーマ変更が master-maintenance のテーブルと同一トランザクション境界内に存在する
- 本番デプロイ時のローリングアップデートで `_sqlx_migrations` 整合性が破壊されるリスク

外部技術監査（CRIT-09）でこの問題が指摘された。

## 決定

saga-rust 用の専用データベース `k1s0_saga` を作成し、saga のマイグレーションを独立させる。

### 変更ファイル

| ファイル | 変更内容 |
|--------|---------|
| `infra/docker/init-db/01-create-databases.sql` | `CREATE DATABASE k1s0_saga;` を追加し、k1s0 ユーザーへの GRANT を付与 |
| `infra/docker/init-db/04-saga-schema.sql` | 接続先を `\connect k1s0_saga` に変更 |
| `regions/system/server/rust/saga/config/config.docker.yaml` | `database.name: "k1s0_saga"` に変更 |
| `infra/helm/services/system/saga/values.yaml` | DB 名コメントを `k1s0_saga` に更新 |

## 理由

1. **マイグレーション独立性**: `_sqlx_migrations` テーブルが専用 DB に分離されることで、他サービスのマイグレーション状態と競合しない
2. **最小権限の原則**: saga サービスは saga 関連テーブルのみにアクセスすべきであり、system 全体の DB を共有するのは過剰な権限
3. **障害分離**: saga の DB 障害が master-maintenance やその他のサービスに伝播しない
4. **設計一貫性**: 他サービス（workflow, scheduler 等）はそれぞれ専用 DB を持っており、saga のみ共有 DB を使用するのは設計上の例外だった

## 影響

**ポジティブな影響**:

- マイグレーション競合リスクが解消される
- saga DB へのスキーマ変更が他サービスに影響しない
- サービス単位の DB 障害分離が実現できる

**ネガティブな影響・トレードオフ**:

- 新規データベース追加のため、init-db スクリプトの更新が必要（対応済み）
- 本番環境の PostgreSQL に `k1s0_saga` データベースを事前作成するオペレーションが必要
- 既存の `k1s0_system` 上に saga テーブルが存在する場合は、データマイグレーションが必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| スキーマ分離 | k1s0_system 内で別スキーマ（saga_schema）を使用 | sqlx の接続文字列は database 単位であり、スキーマを変える場合は `search_path` の動的変更が必要になり複雑 |
| 現状維持 | k1s0_system を継続使用 | CRIT-09 監査指摘の根本原因が未対応のまま残る。マイグレーション競合リスクが継続する |
| テーブルプレフィックス分離 | `saga_` プレフィックスで同一 DB 内に共存 | `_sqlx_migrations` テーブルの競合は解消されない |

## 参考

- [ADR-0007: saga 補償トランザクション](./0007-saga-compensation-inventory-reservations.md)
- `infra/docker/init-db/` — init-db スクリプト群
- CRIT-09 外部技術監査指摘事項（2026-03-30）

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-30 | 初版作成（CRIT-09 監査対応） | - |
