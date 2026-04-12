# sqlx-migrate: 基本

- 対象読者: SQL データベースの基本操作を理解している Rust 開発者
- 学習目標: sqlx-migrate の仕組みを理解し、Rust プロジェクトでデータベースマイグレーションを管理できるようになる
- 所要時間: 約 30 分
- 対象バージョン: SQLx 0.8
- 最終更新日: 2026-04-12

## 1. このドキュメントで学べること

- sqlx-migrate が解決する課題と仕組みを説明できる
- sqlx-cli を使ってマイグレーションの作成・適用・ロールバックができる
- `sqlx::migrate!()` マクロでアプリケーション起動時にマイグレーションを自動適用できる
- `#[sqlx::test]` でテスト用データベースを自動プロビジョニングできる

## 2. 前提知識

- SQL の基礎（CREATE TABLE、ALTER TABLE 等）
- Rust の基本文法と Cargo の使い方（[Rust: 基本](../language/rust_basics.md)）
- 非同期プログラミング（async/await）の基礎概念

## 3. 概要

データベースマイグレーションとは、データベーススキーマの変更（テーブル作成・カラム追加・インデックス変更など）をバージョン管理する仕組みである。コードの変更は Git で追跡できるが、データベースの変更はスクリプトとして明示的に管理しなければ、環境間の差異やデプロイ時の不整合を招く。

sqlx-migrate は Rust の非同期 SQL ツールキット SQLx が提供するマイグレーション機能である。CLI ツール（sqlx-cli）とプログラム API（`sqlx::migrate::Migrator`、`sqlx::migrate!()` マクロ）の 2 つの方法でマイグレーションを管理できる。PostgreSQL・MySQL・SQLite に対応している。

## 4. 用語の整理

| 用語 | 説明 |
|------|------|
| マイグレーション | データベーススキーマの変更を SQL スクリプトとして管理する仕組み |
| sqlx-cli | SQLx 用のコマンドラインツール。マイグレーションの作成・適用・ロールバックを行う |
| Migrator | マイグレーションをプログラムから実行するための構造体 |
| `sqlx::migrate!()` | マイグレーションファイルをコンパイル時にバイナリへ埋め込むマクロ |
| _sqlx_migrations | データベース内に自動作成される、適用済みマイグレーションの記録テーブル |
| 可逆マイグレーション | up（適用）と down（取消）の両方のスクリプトを持つマイグレーション |
| MigrationSource | マイグレーションファイルの読込元を抽象化するトレイト |

## 5. 仕組み・アーキテクチャ

sqlx-migrate は「マイグレーションファイル」と「データベースの適用履歴テーブル」を照合し、未適用のマイグレーションを順番に実行する。CLI とプログラム API の両方からこの仕組みを利用できる。

![sqlx-migrate アーキテクチャ](./img/sqlx-migrate_basics_architecture.svg)

マイグレーションファイルは `<タイムスタンプ>_<説明>.sql` の命名規則に従う。適用済みのマイグレーションはデータベース内の `_sqlx_migrations` テーブルにハッシュ値とともに記録され、同じマイグレーションが二重に適用されることを防ぐ。ハッシュが変更された場合はエラーとなり、適用済みスクリプトの改変を検知できる。

## 6. 環境構築

### 6.1 必要なもの

- Rust ツールチェイン（rustup 経由）
- 対応データベース（PostgreSQL / MySQL / SQLite のいずれか）
- sqlx-cli（マイグレーション管理用 CLI）

### 6.2 セットアップ手順

```bash
# sqlx-cli をインストールする（PostgreSQL 向けの例）
cargo install sqlx-cli --features postgres

# DATABASE_URL 環境変数を設定する
export DATABASE_URL="postgres://user:password@localhost/mydb"

# データベースを作成する
sqlx db create
```

### 6.3 動作確認

```bash
# sqlx-cli のバージョンを確認する
sqlx --version

# マイグレーション情報を表示する（初回は空）
sqlx migrate info
```

## 7. 基本の使い方

マイグレーションの作成から適用までの基本的なワークフローを示す。

![マイグレーションライフサイクル](./img/sqlx-migrate_basics_lifecycle.svg)

### 7.1 マイグレーションの作成と適用

```bash
# マイグレーションファイルを作成する
sqlx migrate add create_users_table

# 上記コマンドにより以下が生成される
# migrations/20260412000000_create_users_table.sql
```

生成されたファイルに SQL を記述する。

```sql
-- ユーザーテーブルを作成するマイグレーション
-- 主キーとして UUID を使用する
CREATE TABLE users (
    -- ユーザーの一意識別子
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- ユーザー名（一意制約付き）
    username VARCHAR(255) NOT NULL UNIQUE,
    -- メールアドレス（一意制約付き）
    email VARCHAR(255) NOT NULL UNIQUE,
    -- レコード作成日時
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

```bash
# マイグレーションを適用する
sqlx migrate run
# Applied 20260412000000/migrate create_users_table (12.5ms)

# 適用状態を確認する
sqlx migrate info
```

## 8. ステップアップ

### 8.1 可逆マイグレーション

`-r` フラグを付けると、up（適用）と down（取消）の 2 つのファイルが生成される。

```bash
# 可逆マイグレーションを作成する
sqlx migrate add -r add_posts_table
# Creating migrations/20260412000001_add_posts_table.up.sql
# Creating migrations/20260412000001_add_posts_table.down.sql

# ロールバックする（直近の 1 件を取り消す）
sqlx migrate revert
```

### 8.2 プログラムからのマイグレーション実行

`sqlx::migrate!()` マクロを使うと、マイグレーションファイルをコンパイル時にバイナリへ埋め込める。アプリケーション起動時に自動でマイグレーションを適用する用途に有効である。

```rust
// sqlx::migrate!() マクロを使ったマイグレーション実行の例
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // データベース接続プールを作成する
    let pool = PgPool::connect("postgres://user:pass@localhost/mydb").await?;
    // マイグレーションをバイナリに埋め込み実行する
    sqlx::migrate!("./migrations").run(&pool).await?;
    // アプリケーションのメインロジックを開始する
    println!("マイグレーション完了");
    Ok(())
}
```

### 8.3 テストでのマイグレーション活用

`#[sqlx::test]` 属性を使うと、テストごとに一時データベースを自動作成し、マイグレーションを適用した状態でテストを実行できる。

```rust
// #[sqlx::test] によるテスト用データベース自動プロビジョニングの例
use sqlx::{PgPool, Row};

// テスト用データベースを自動作成しマイグレーションを適用する
#[sqlx::test]
async fn test_user_insert(pool: PgPool) -> sqlx::Result<()> {
    // ユーザーを挿入する
    sqlx::query("INSERT INTO users (username, email) VALUES ($1, $2)")
        .bind("testuser")
        .bind("test@example.com")
        .execute(&pool)
        .await?;
    // 挿入したユーザーを取得する
    let row = sqlx::query("SELECT username FROM users WHERE email = $1")
        .bind("test@example.com")
        .fetch_one(&pool)
        .await?;
    // ユーザー名を検証する
    assert_eq!(row.get::<String, _>("username"), "testuser");
    Ok(())
}
```

## 9. よくある落とし穴

- **DATABASE_URL 未設定**: sqlx-cli は `DATABASE_URL` 環境変数または `.env` ファイルを参照する。未設定だとコマンドが失敗する
- **適用済みスクリプトの編集**: 一度適用したマイグレーションファイルを変更するとハッシュ不一致エラーになる。修正は新しいマイグレーションで行うこと
- **マイグレーション順序の衝突**: 複数人が同時にマイグレーションを作成するとタイムスタンプが近接し、意図しない適用順序になることがある。PR マージ前に順序を確認すること
- **down スクリプトの未記述**: 可逆マイグレーション（`-r`）で作成した down.sql を空のまま放置すると、ロールバック時に何も実行されず不整合が生じる
- **migrate feature の有効化忘れ**: `sqlx::migrate!()` や `#[sqlx::test]` には Cargo.toml で `migrate` フィーチャーの有効化が必要である

## 10. ベストプラクティス

- マイグレーション名は変更内容を明確に表すものにする（例: `add_email_index_to_users`）
- 本番環境では可逆マイグレーション（`-r`）を使い、ロールバック手段を確保する
- 1 つのマイグレーションには 1 つの論理的変更のみを含める
- CI/CD パイプラインで `sqlx migrate run` を自動実行し、デプロイと同期させる
- テストでは `#[sqlx::test]` を活用し、テストごとにクリーンなデータベース状態を保証する

## 11. 演習問題

1. `sqlx migrate add` で `create_products_table` マイグレーションを作成し、id・name・price カラムを持つテーブルを定義して適用せよ
2. 可逆マイグレーションで `add_category_to_products` を作成し、up.sql でカラム追加、down.sql でカラム削除を記述せよ。適用後にロールバックして動作を確認せよ
3. `sqlx::migrate!()` マクロを使い、アプリケーション起動時にマイグレーションを自動適用するコードを書け

## 12. さらに学ぶには

- SQLx 公式リポジトリ: https://github.com/launchbadge/sqlx
- sqlx-cli README: https://github.com/launchbadge/sqlx/tree/main/sqlx-cli
- 関連 Knowledge: [PostgreSQL: 基本](../infra/postgresql_basics.md)
- 関連 Knowledge: [Testcontainers: 基本](testcontainers_basics.md)

## 13. 参考資料

- SQLx GitHub リポジトリ: https://github.com/launchbadge/sqlx
- SQLx クレートドキュメント: https://docs.rs/sqlx/
- sqlx-cli README: https://github.com/launchbadge/sqlx/tree/main/sqlx-cli
- SQLx マイグレーション API ドキュメント: https://docs.rs/sqlx/latest/sqlx/migrate/
