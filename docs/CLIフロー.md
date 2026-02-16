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

種別と言語に応じて、[ディレクトリ構成図](ディレクトリ構成図.md) に定義された構成でファイルが生成される。各テンプレートの詳細（スケルトンコード・依存関係・条件付き生成）は以下を参照。

- サーバー → [テンプレート仕様-サーバー](テンプレート仕様-サーバー.md)
- クライアント → [テンプレート仕様-クライアント](テンプレート仕様-クライアント.md)
- ライブラリ → [テンプレート仕様-ライブラリ](テンプレート仕様-ライブラリ.md)
- データベース → [テンプレート仕様-データベース](テンプレート仕様-データベース.md)
- テンプレート変数・条件分岐 → [テンプレートエンジン仕様](テンプレートエンジン仕様.md)

#### API 方式による条件付き生成

サーバー種別では、選択した API 方式に応じて追加ファイルが生成される。

| API 方式   | 追加生成ファイル                                                         |
| ---------- | ------------------------------------------------------------------------ |
| REST       | OpenAPI 定義（`openapi.yaml`）、oapi-codegen 設定                       |
| gRPC       | proto 定義（`service.proto`）、`buf.yaml`、`buf.gen.yaml`（Go のみ）    |
| GraphQL    | スキーマ定義（`schema.graphql`）、`gqlgen.yml`（Go のみ）               |

#### 常に生成されるファイル

サーバー種別では、API 方式に関わらず以下のテストファイルが常に生成される。

- Go: `usecase_test.go`、`handler_test.go`、`repository_test.go`（DB 有効時）
- Rust: `tests/integration_test.rs`

> **注記**: service Tierのサーバーで GraphQL（API 方式選択）を選択した場合、通常のサーバー構成に加えて `server/{言語}/bff/` ディレクトリに GraphQL BFF のひな形が生成される。BFF の詳細な内部構成は [API設計.md](API設計.md) の「BFF ディレクトリ構成」を参照。

#### GraphQL BFF 生成フロー（service Tier 限定）

service Tier のサーバーで API 方式に GraphQL を含む場合、以下の追加ステップが表示される。

```
? GraphQL BFF を生成しますか？
> はい
  いいえ
```

「はい」を選択した場合：

```
? BFF の言語を選択してください
> Go
  Rust
```

BFF は service Tier のサーバーでのみ生成可能。system / business Tier では BFF 選択ステップは表示されない。

BFF 生成時の確認画面には以下が追加表示される：

```
[確認] 以下の内容で生成します。よろしいですか？
    種別:     サーバー
    Tier:     service
    サービス: order
    言語:     Go
    API:      REST, GraphQL
    BFF:      あり (Go)
    DB:       order-db (PostgreSQL)
    Kafka:    無効
    Redis:    無効
    > はい
      いいえ（前のステップに戻る）
      キャンセル（メインメニューに戻る）
```

BFF のテンプレートは通常のサーバーテンプレートと同じ構造で、`regions/service/{service_name}/server/{server_lang}/bff/` 配下に生成される。生成されるファイルの詳細は [テンプレート仕様-サーバー](テンプレート仕様-サーバー.md) を参照。

#### BFF 生成フローの対話ステップ詳細

BFF 生成は、service Tier のサーバー詳細設定（ステップ 5）の中で行われる。以下の順序で対話が進行する。

```
[ステップ 5: サーバー詳細設定]

[5-1] サービス名
      → service Tier ではステップ 3 で入力済みのためスキップ

[5-2] API 方式選択（複数選択可）
      ? API 方式を選択してください（複数選択可）
      > [x] REST (OpenAPI)
        [ ] gRPC (protobuf)
        [x] GraphQL            ← GraphQL を含む選択

[5-3] データベース追加
      ? データベースを追加しますか？
      > はい / いいえ

[5-4] Kafka 有効化
      ? メッセージング (Kafka) を有効にしますか？
      > はい / いいえ

[5-5] Redis 有効化
      ? キャッシュ (Redis) を有効にしますか？
      > はい / いいえ

[5-6] BFF 生成提案（GraphQL 選択時のみ表示）
      ? GraphQL BFF を生成しますか？
      > はい
        いいえ

[5-7] BFF 言語選択（BFF 生成「はい」の場合のみ表示）
      ? BFF の言語を選択してください
      > Go
        Rust
```

この対話フローは `CLI/src/commands/generate/steps.rs` の `step_detail_server()` で実装されている。BFF 関連のステップ（5-6, 5-7）は `tier == Tier::Service && api_styles.contains(&ApiStyle::GraphQL)` の条件を満たす場合にのみ表示される。

選択された BFF 言語は `DetailConfig.bff_language` フィールド（`Option<Language>`）に格納され、確認画面と生成処理に引き渡される。

#### BFF 生成時の確認画面表示例

**REST + GraphQL + BFF (Go) の場合：**

```
[確認] 以下の内容で生成します。よろしいですか？
    種別:     サーバー
    Tier:     service
    サービス: order
    言語:     Go
    API:      REST, GraphQL
    BFF:      あり (Go)
    DB:       order-db (PostgreSQL)
    Kafka:    無効
    Redis:    無効
    > はい
      いいえ（前のステップに戻る）
      キャンセル（メインメニューに戻る）
```

**GraphQL + BFF (Rust)、サーバー言語 Go の場合：**

```
[確認] 以下の内容で生成します。よろしいですか？
    種別:     サーバー
    Tier:     service
    サービス: payment
    言語:     Go
    API:      GraphQL
    BFF:      あり (Rust)
    DB:       なし
    Kafka:    無効
    Redis:    無効
    > はい
      いいえ（前のステップに戻る）
      キャンセル（メインメニューに戻る）
```

BFF の言語はサーバー本体の言語と異なる選択が可能（例: サーバー Go + BFF Rust）。

#### BFF 配置パス

BFF はサーバー本体の出力ディレクトリ内に `bff/` サブディレクトリとして生成される。

```
regions/service/{service_name}/server/{server_lang}/bff/
```

例: サービス名 `order`、サーバー言語 `Go` の場合：

```
regions/service/order/server/go/bff/
```

> **注記**: BFF のディレクトリはサーバー本体の言語ディレクトリ（`{server_lang}`）配下に配置される。BFF 言語の選択（Go / Rust）は BFF 内部のスケルトンコード生成に影響するが、配置パスはサーバー本体の言語に従う。

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

### デプロイ実行

確認後、以下のステップが順次実行される。

#### ステップ 1 — Docker イメージのビルドとプッシュ

```
[1/4] Docker イメージをビルドしています...
      イメージ: harbor.internal.example.com/k1s0-{tier}/{service_name}:{version}-{sha}
[1/4] ✓ Docker イメージのビルド完了

[2/4] Docker イメージをプッシュしています...
[2/4] ✓ Docker イメージのプッシュ完了
```

#### ステップ 2 — Cosign によるイメージ署名

```
[3/4] イメージに署名しています (Cosign)...
[3/4] ✓ イメージ署名完了
```

#### ステップ 3 — Helm デプロイ

```
[4/4] Helm でデプロイしています...
      コマンド: helm upgrade --install {service_name} ./infra/helm/services/{helm_path} \
                -n k1s0-{tier} \
                -f ./infra/helm/services/{helm_path}/values-{env}.yaml \
                --set image.tag={version}-{sha}
[4/4] ✓ デプロイ完了
```

#### ステップ 4 — デプロイ結果の表示

```
✓ デプロイが完了しました
  環境:     {env}
  サービス: {service_name}
  イメージ: harbor.internal.example.com/k1s0-{tier}/{service_name}:{version}-{sha}
  Helm:     helm status {service_name} -n k1s0-{tier}
```

#### エラー時の動作

各ステップでエラーが発生した場合、処理を中断して以下を表示する。

```
✗ デプロイに失敗しました
  ステップ: Docker イメージのビルド
  エラー:   {error_message}

  手動で再実行する場合:
    cd {module_path} && docker build -t {image_tag} .
```

prod 環境でのデプロイ失敗時は、ロールバックの選択肢を表示する。

```
⚠ 本番環境でのデプロイが失敗しました。
? ロールバックしますか？
> はい（前のバージョンに戻す）
  いいえ（手動で対応する）
```

「はい」を選択した場合、`helm rollback {service_name} -n k1s0-{tier}` を実行する。

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
- [コンセプト](コンセプト.md)
- [テンプレート仕様-サーバー](テンプレート仕様-サーバー.md)
- [テンプレート仕様-クライアント](テンプレート仕様-クライアント.md)
- [テンプレート仕様-ライブラリ](テンプレート仕様-ライブラリ.md)
- [テンプレート仕様-データベース](テンプレート仕様-データベース.md)
- [テンプレートエンジン仕様](テンプレートエンジン仕様.md)
