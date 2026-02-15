# CLIフロー

k1s0 CLI は対話式で操作する。引数での起動は行わない。
すべての操作は dialoguer によるプロンプトを通じて行う。

## 起動

```
$ k1s0
```

起動するとメインメニューが表示される。

## メインメニュー

```
? 操作を選択してください
> プロジェクト初期化
  ひな形生成
  ビルド
  テスト実行
  デプロイ
  終了
```

---

## プロジェクト初期化

モノリポの初期構成をセットアップする。

```
[1] プロジェクト名を入力してください: ____

[2] Git リポジトリを初期化しますか？
    > はい
      いいえ

[3] sparse-checkout を有効にしますか？
    > はい
      いいえ

[4] (sparse-checkout: はい の場合)
    チェックアウトするTierを選択してください（複数選択可）
    > [x] system
      [ ] business
      [ ] service

[確認] 以下の内容で初期化します。よろしいですか？
    プロジェクト名: {入力値}
    Git 初期化:     はい
    sparse-checkout: はい (system)
    > はい
      いいえ（前のステップに戻る）
      キャンセル（メインメニューに戻る）
```

### 生成されるもの

- `regions/` ディレクトリ（選択したTier）
- `api/` ディレクトリ（`api/proto/`）
- `infra/` ディレクトリ
- `e2e/` ディレクトリ
- `docs/` ディレクトリ
- `docker-compose.yaml`
- `.devcontainer/devcontainer.json`
- `.github/workflows/` （CI/CD パイプライン）
- `README.md`

---

## ひな形生成

サーバー・クライアント・ライブラリ・データベースのひな形を生成する。

### ステップ 1 — 種別の選択

```
? 何を生成しますか？
> サーバー
  クライアント
  ライブラリ
  データベース
```

### ステップ 2 — Tier の選択

選択した種別に応じて、選択可能な Tier が制限される。

| 種別           | system | business | service |
| -------------- | :----: | :------: | :-----: |
| サーバー       |   o    |    o     |    o    |
| クライアント   |   -    |    o     |    o    |
| ライブラリ     |   o    |    o     |    -    |
| データベース   |   o    |    o     |    o    |

```
? Tier を選択してください
> system
  business
  service
```

### ステップ 3 — 配置先の指定

Tierに応じて追加の入力を求める。

**system の場合：**

配置先の指定は不要なため、このステップはスキップされる。

**business の場合：**

```
? 領域名を入力または選択してください
> (新規作成)
  accounting    # 既存の領域が候補として表示される
  fa
```

新規作成を選択した場合：

```
? 領域名を入力してください: ____
```

**service の場合：**

```
? サービス名を入力または選択してください
> (新規作成)
  order           # 既存のサービスが候補として表示される
  inventory
```

新規作成を選択した場合：

```
? サービス名を入力してください: ____
```

### ステップ 4 — 言語 / フレームワークの選択

種別に応じて選択肢が異なる。

**サーバーの場合：**

```
? 言語を選択してください
> Go
  Rust
```

**クライアントの場合：**

```
? フレームワークを選択してください
> React
  Flutter
```

**ライブラリの場合：**

```
? 言語を選択してください
> Go
  Rust
  TypeScript
  Dart
```

**データベースの場合：**

```
? データベース名を入力してください: ____

? RDBMS を選択してください
> PostgreSQL
  MySQL
  SQLite
```

データベースを選択した場合、このステップで確認に進む。

### ステップ 5 — 詳細設定

種別に応じた追加設定を行う。

**サーバーの場合：**

system / business の場合：

```
? サービス名を入力してください: ____
```

service の場合、ステップ 3 で入力したサービス名をそのまま使用するため、この入力はスキップされる。

```
? API 方式を選択してください（複数選択可）
> [x] REST (OpenAPI)
  [ ] gRPC (protobuf)
  [ ] GraphQL

? データベースを追加しますか？
> はい
  いいえ

? (はい の場合) データベース名を入力または選択してください
> (新規作成)
  order-db (PostgreSQL)   # 既存のデータベースがRDBMS付きで表示される
  auth-db (PostgreSQL)

? (新規作成の場合) データベース名を入力してください: ____

? (新規作成の場合) RDBMS を選択してください
> PostgreSQL
  MySQL
  SQLite

? メッセージング (Kafka) を有効にしますか？
> はい
  いいえ

? キャッシュ (Redis) を有効にしますか？
> はい
  いいえ
```

**クライアントの場合：**

business の場合：

```
? アプリ名を入力してください: ____
```

service の場合、ステップ 3 で入力したサービス名をアプリ名として使用するため、この入力はスキップされる。

**ライブラリの場合：**

```
? ライブラリ名を入力してください: ____
```

**データベースの場合：**

ステップ 4 で入力済みのため、このステップはスキップされる。

### 確認

入力内容に応じた確認画面が表示される。

**サーバーの場合（system Tier）：**

```
[確認] 以下の内容で生成します。よろしいですか？
    種別:     サーバー
    Tier:     system
    サービス: auth
    言語:     Go
    API:      REST, gRPC
    DB:       auth-db (PostgreSQL)
    Kafka:    無効
    Redis:    無効
    > はい
      いいえ（前のステップに戻る）
      キャンセル（メインメニューに戻る）
```

**サーバーの場合（business Tier）：**

```
[確認] 以下の内容で生成します。よろしいですか？
    種別:     サーバー
    Tier:     business
    領域:     accounting
    サービス: ledger
    言語:     Go
    API:      REST
    DB:       なし
    Kafka:    無効
    Redis:    無効
    > はい
      いいえ（前のステップに戻る）
      キャンセル（メインメニューに戻る）
```

**サーバーの場合（service Tier）：**

```
[確認] 以下の内容で生成します。よろしいですか？
    種別:     サーバー
    Tier:     service
    サービス: order
    言語:     Go
    API:      REST, gRPC
    DB:       order-db (PostgreSQL)
    Kafka:    有効
    Redis:    有効
    > はい
      いいえ（前のステップに戻る）
      キャンセル（メインメニューに戻る）
```

**クライアントの場合（business Tier）：**

```
[確認] 以下の内容で生成します。よろしいですか？
    種別:           クライアント
    Tier:           business
    領域:           accounting
    フレームワーク: React
    アプリ名:       accounting-web
    > はい
      いいえ（前のステップに戻る）
      キャンセル（メインメニューに戻る）
```

**クライアントの場合（service Tier）：**

```
[確認] 以下の内容で生成します。よろしいですか？
    種別:           クライアント
    Tier:           service
    サービス:       order
    フレームワーク: React
    アプリ名:       order
    > はい
      いいえ（前のステップに戻る）
      キャンセル（メインメニューに戻る）
```

**ライブラリの場合（system Tier）：**

```
[確認] 以下の内容で生成します。よろしいですか？
    種別:         ライブラリ
    Tier:         system
    言語:         Go
    ライブラリ名: authlib
    > はい
      いいえ（前のステップに戻る）
      キャンセル（メインメニューに戻る）
```

**ライブラリの場合（business Tier）：**

```
[確認] 以下の内容で生成します。よろしいですか？
    種別:         ライブラリ
    Tier:         business
    領域:         accounting
    言語:         Go
    ライブラリ名: ledger-lib
    > はい
      いいえ（前のステップに戻る）
      キャンセル（メインメニューに戻る）
```

**データベースの場合（system Tier）：**

```
[確認] 以下の内容で生成します。よろしいですか？
    種別:           データベース
    Tier:           system
    データベース名: auth-db
    RDBMS:          PostgreSQL
    > はい
      いいえ（前のステップに戻る）
      キャンセル（メインメニューに戻る）
```

**データベースの場合（business Tier）：**

```
[確認] 以下の内容で生成します。よろしいですか？
    種別:           データベース
    Tier:           business
    領域:           accounting
    データベース名: accounting-db
    RDBMS:          PostgreSQL
    > はい
      いいえ（前のステップに戻る）
      キャンセル（メインメニューに戻る）
```

**データベースの場合（service Tier）：**

```
[確認] 以下の内容で生成します。よろしいですか？
    種別:           データベース
    Tier:           service
    サービス:       order
    データベース名: order-db
    RDBMS:          PostgreSQL
    > はい
      いいえ（前のステップに戻る）
      キャンセル（メインメニューに戻る）
```

### 生成されるもの

種別と言語に応じて、[ディレクトリ構成図](ディレクトリ構成図.md) に定義された構成でファイルが生成される。

> **注記**: service Tierのサーバーで GraphQL（API 方式選択）を選択した場合、通常のサーバー構成に加えて `server/{言語}/bff/` ディレクトリに GraphQL BFF のひな形が生成される。BFF の詳細な内部構成は [API設計.md](API設計.md) の「BFF ディレクトリ構成」を参照。

---

## ビルド

対象を選択してビルドを実行する。

### ステップ 1 — 対象の選択

```
? ビルド対象を選択してください（複数選択可）
> [ ] すべて
  [x] regions/service/order/server/go
  [ ] regions/service/order/client/react
  [ ] regions/business/accounting/server/go/ledger
  [ ] regions/business/accounting/client/react/accounting-web
  [ ] regions/system/server/go/auth
  [ ] regions/system/library/go/authlib
  ...
```

既存のサーバー・クライアント・ライブラリが候補として一覧表示される。

### ステップ 2 — ビルド設定

```
? ビルドモードを選択してください
> development
  production
```

### 確認

```
[確認] 以下の対象をビルドします。よろしいですか？
    対象:   regions/service/order/server/go
    モード: development
    > はい
      いいえ（前のステップに戻る）
      キャンセル（メインメニューに戻る）
```

---

## テスト実行

対象を選択してテストを実行する。

### ステップ 1 — テスト種別の選択

```
? テスト種別を選択してください
> ユニットテスト
  統合テスト
  E2Eテスト
  すべて
```

### ステップ 2 — 対象の選択

**ユニットテスト / 統合テストの場合：**

```
? テスト対象を選択してください（複数選択可）
> [ ] すべて
  [x] regions/service/order/server/go
  [ ] regions/service/order/client/react
  [ ] regions/business/accounting/server/go/ledger
  [ ] regions/business/accounting/client/react/accounting-web
  [ ] regions/system/server/go/auth
  [ ] regions/system/library/go/authlib
  ...
```

**E2Eテストの場合：**

```
? テストスイートを選択してください（複数選択可）
> [ ] すべて
  [x] e2e/tests/order
  [ ] e2e/tests/auth
  ...
```

**すべての場合：**

対象の選択はスキップされ、全テストが実行される。

### 確認

```
[確認] 以下のテストを実行します。よろしいですか？
    種別: ユニットテスト
    対象: regions/service/order/server/go
    > はい
      いいえ（前のステップに戻る）
      キャンセル（メインメニューに戻る）
```

---

## デプロイ

対象を選択してデプロイを実行する。

### ステップ 1 — 環境の選択

```
? デプロイ先の環境を選択してください
> dev
  staging
  prod
```

### ステップ 2 — 対象の選択

```
? デプロイ対象を選択してください（複数選択可）
> [ ] すべて
  [x] regions/service/order/server/go
  [ ] regions/service/order/client/react
  [ ] regions/business/accounting/server/go/ledger
  [ ] regions/business/accounting/client/react/accounting-web
  [ ] regions/system/server/go/auth
  ...
```

### ステップ 3 — prod 環境の追加確認

prod を選択した場合のみ表示される。

```
⚠ 本番環境へのデプロイです。
? 本当にデプロイしますか？ "deploy" と入力してください: ____
```

### 確認

```
[確認] 以下の内容でデプロイします。よろしいですか？
    環境: dev
    対象: regions/service/order/server/go
    > はい
      いいえ（前のステップに戻る）
      キャンセル（メインメニューに戻る）
```

---

## 共通の操作

### 中断

すべてのプロンプトで `Ctrl+C` を押すとメインメニューに戻る。
メインメニューで `Ctrl+C` を押すと CLI を終了する。

### 前のステップに戻る

各ステップで `Esc` を押すと前のステップに戻る。
各フローの最初のステップで `Esc` を押すとメインメニューに戻る。

### 入力のバリデーション

- プロジェクト名・サービス名・領域名は英小文字・ハイフン・数字のみ許可する（`[a-z0-9-]+`）
- 先頭と末尾のハイフンは禁止する
- 既存の名前との重複はエラーとする

## 関連ドキュメント

- [ディレクトリ構成図](ディレクトリ構成図.md)
- [API設計](API設計.md)
- [tier-architecture](tier-architecture.md)
- [config設計](config設計.md)
