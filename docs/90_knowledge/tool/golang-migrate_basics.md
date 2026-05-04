# golang-migrate: 基本

- 対象読者: データベースを使うアプリケーションを開発する Go エンジニア
- 学習目標: golang-migrate を使ってデータベーススキーマのバージョン管理と安全な変更適用ができるようになる
- 所要時間: 約 30 分
- 対象バージョン: golang-migrate v4.18.x
- 最終更新日: 2026-04-12

## 1. このドキュメントで学べること

- golang-migrate が解決する課題を説明できる
- CLI でマイグレーションファイルを作成・適用・ロールバックできる
- Go アプリケーション内からライブラリとしてマイグレーションを実行できる
- dirty 状態の原因を理解し、復旧できる

## 2. 前提知識

- SQL の基本（CREATE TABLE, ALTER TABLE 等）
- Go の基本的な文法と `go install` の使い方
- データベース（PostgreSQL 等）への接続経験

## 3. 概要

golang-migrate は、データベーススキーマの変更をバージョン管理するためのツールである。Go で実装されており、CLI ツールとしても Go ライブラリとしても利用できる。

アプリケーション開発では、テーブルの追加やカラムの変更が頻繁に発生する。これらの変更を手動で SQL 実行していると、環境ごとにスキーマが食い違う問題が起きる。golang-migrate は各変更を番号付きの SQL ファイル（マイグレーション）として管理し、どの環境でも同じ順序で適用できる仕組みを提供する。

## 4. 用語の整理

| 用語 | 説明 |
|------|------|
| マイグレーション | データベーススキーマの 1 つの変更単位。up（適用）と down（戻し）のペアで構成される |
| up | スキーマ変更を適用する方向の SQL。テーブル作成やカラム追加など |
| down | スキーマ変更を元に戻す方向の SQL。up の逆操作を記述する |
| version | マイグレーションの連番。schema_migrations テーブルで現在のバージョンを追跡する |
| dirty | マイグレーション実行中にエラーが発生し、中途半端な状態になったことを示すフラグ |
| Source ドライバ | マイグレーションファイルの読み込み元（ファイルシステム、GitHub、S3 等） |
| Database ドライバ | マイグレーション先のデータベース（PostgreSQL、MySQL、SQLite 等） |

## 5. 仕組み・アーキテクチャ

golang-migrate は「Source ドライバ」「マイグレーションエンジン」「Database ドライバ」の 3 層で構成される。Source ドライバがマイグレーションファイルを読み込み、エンジンがバージョン順に実行し、Database ドライバが対象データベースへ SQL を発行する。

![golang-migrate アーキテクチャ](./img/golang-migrate_basics_architecture.svg)

マイグレーションの作成から適用、ロールバック、エラー復旧までのワークフローは以下のとおりである。

![マイグレーション ワークフロー](./img/golang-migrate_basics_workflow.svg)

## 6. 環境構築

### 6.1 必要なもの

- Go 1.21 以上
- 対象データベース（PostgreSQL、MySQL、SQLite 等）
- ターミナル

### 6.2 セットアップ手順

```bash
# golang-migrate CLI をインストールする
go install -tags 'postgres' github.com/golang-migrate/migrate/v4/cmd/migrate@latest

# MySQL を使う場合はタグを変更する
# go install -tags 'mysql' github.com/golang-migrate/migrate/v4/cmd/migrate@latest
```

### 6.3 動作確認

```bash
# バージョンを確認する
migrate -version
```

## 7. 基本の使い方

### マイグレーションファイルの作成

```bash
# マイグレーションファイルのペアを生成する（-seq で連番、-ext で拡張子を指定する）
migrate create -ext sql -dir db/migrations -seq create_users_table
```

上記コマンドで以下の 2 ファイルが生成される。

- `db/migrations/000001_create_users_table.up.sql`（適用用）
- `db/migrations/000001_create_users_table.down.sql`（戻し用）

```sql
-- 000001_create_users_table.up.sql: ユーザーテーブルを作成するマイグレーション

-- ユーザーテーブルを作成する
CREATE TABLE users (
    -- 主キーを自動採番で定義する
    id SERIAL PRIMARY KEY,
    -- メールアドレスカラムを一意制約付きで定義する
    email VARCHAR(255) NOT NULL UNIQUE,
    -- 作成日時カラムをデフォルト値付きで定義する
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
```

```sql
-- 000001_create_users_table.down.sql: ユーザーテーブルを削除するロールバック

-- ユーザーテーブルを削除する
DROP TABLE IF EXISTS users;
```

### マイグレーションの実行

```bash
# すべての未適用マイグレーションを適用する
migrate -source file://db/migrations -database "postgres://user:pass@localhost:5432/mydb?sslmode=disable" up

# 指定した数だけ適用する
migrate -source file://db/migrations -database "postgres://user:pass@localhost:5432/mydb?sslmode=disable" up 1

# 1 つ戻す
migrate -source file://db/migrations -database "postgres://user:pass@localhost:5432/mydb?sslmode=disable" down 1

# 現在のバージョンを確認する
migrate -source file://db/migrations -database "postgres://user:pass@localhost:5432/mydb?sslmode=disable" version
```

### Go ライブラリとしての使用

```go
// golang-migrate を Go アプリケーション内から実行するサンプル

package main

import (
 // ログ出力用パッケージをインポートする
 "log"

 // migrate 本体をインポートする
 "github.com/golang-migrate/migrate/v4"
 // PostgreSQL ドライバを副作用インポートで登録する
 _ "github.com/golang-migrate/migrate/v4/database/postgres"
 // ファイルソースドライバを副作用インポートで登録する
 _ "github.com/golang-migrate/migrate/v4/source/file"
)

func main() {
 // マイグレーションインスタンスを生成する（ソースとデータベースURLを指定する）
 m, err := migrate.New(
  "file://db/migrations",
  "postgres://user:pass@localhost:5432/mydb?sslmode=disable")
 if err != nil {
  // 初期化エラー時はログ出力して終了する
  log.Fatal(err)
 }
 // すべての未適用マイグレーションを実行する
 if err := m.Up(); err != nil && err != migrate.ErrNoChange {
  // 実行エラー時はログ出力して終了する
  log.Fatal(err)
 }
}
```

## 8. ステップアップ

### 8.1 環境変数による接続情報の管理

本番環境では接続文字列をコマンド引数に直接書かず、環境変数を使用する。

```bash
# 環境変数にデータベース URLを設定する
export DATABASE_URL="postgres://user:pass@localhost:5432/mydb?sslmode=disable"

# 環境変数を参照してマイグレーションを実行する
migrate -source file://db/migrations -database "$DATABASE_URL" up
```

### 8.2 トランザクション内での実行

PostgreSQL では、マイグレーション SQL を `BEGIN` / `COMMIT` で囲むことで、エラー時に自動ロールバックされる。これにより dirty 状態を防げる。

## 9. よくある落とし穴

- **dirty 状態で操作不能になる**: マイグレーション中にエラーが起きると dirty フラグが立ち、以降の操作が拒否される。`migrate force VERSION` で正しいバージョンを指定して復旧する
- **down マイグレーションの書き忘れ**: up だけ書いて down を空にすると、ロールバック時に何も起きず不整合が生じる。up と down は必ずペアで記述する
- **本番で `down -1` を安易に実行する**: down はデータ削除を伴う場合がある。本番環境では実行前に SQL の内容を必ず確認する
- **連番の衝突**: 複数人が同時にマイグレーションを作成すると番号が重複する。CI でファイル名の一意性を検証する仕組みを入れる

## 10. ベストプラクティス

- マイグレーションファイルは小さく保つ。1 ファイル = 1 つの変更に限定する
- down マイグレーションは必ず記述し、ロールバック可能な状態を維持する
- PostgreSQL では SQL をトランザクションで囲み、部分適用を防止する
- CI/CD パイプラインにマイグレーションの自動実行を組み込む
- マイグレーションファイルは一度適用したら編集しない。修正は新しいマイグレーションで行う

## 11. 演習問題

1. `migrate create` で `add_name_to_users` というマイグレーションを作成し、users テーブルに `name VARCHAR(100)` カラムを追加する up/down SQL を記述せよ
2. Go ライブラリとして migrate を使い、アプリケーション起動時に自動マイグレーションを実行するコードを書け
3. 意図的にエラーを含むマイグレーションを作成し、dirty 状態からの復旧手順を実践せよ

## 12. さらに学ぶには

- 公式リポジトリ: <https://github.com/golang-migrate/migrate>
- CLI リファレンス: <https://github.com/golang-migrate/migrate/tree/master/cmd/migrate>
- PostgreSQL チュートリアル: <https://github.com/golang-migrate/migrate/blob/master/database/postgres/TUTORIAL.md>
- 関連 Knowledge: PostgreSQL の基本は `../infra/postgresql_basics.md` を参照

## 13. 参考資料

- golang-migrate/migrate GitHub リポジトリ: <https://github.com/golang-migrate/migrate>
- Getting Started: <https://github.com/golang-migrate/migrate/blob/master/GETTING_STARTED.md>
- FAQ: <https://github.com/golang-migrate/migrate/blob/master/FAQ.md>
