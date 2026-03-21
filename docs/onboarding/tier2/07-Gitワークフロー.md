# Git ワークフロー

このドキュメントでは、k1s0 プロジェクトでの Git の使い方を、具体的なコマンド付きで解説します。Git に慣れていない方でもステップに沿って進められるようになっています。

---

## ブランチ戦略

k1s0 では **main + feature branches** のシンプルなブランチ戦略を採用しています。

```
main（本番用ブランチ。常に動く状態を保つ）
 ├── feat/add-category-description   ← 機能追加ブランチ
 ├── fix/category-validation-error   ← バグ修正ブランチ
 └── chore/update-dependencies       ← 雑務ブランチ
```

- **main**: 本番環境にデプロイされるブランチ。直接コミットは禁止。
- **feature branches**: 各タスク（機能追加、バグ修正など）ごとに main から分岐して作る作業ブランチ。作業が終わったら PR（Pull Request）を出して main にマージする。

---

## ブランチ命名規則

ブランチ名は以下の形式で付けます。何のための変更かが一目でわかるようにしましょう。

| 接頭辞 | 用途 | 例 |
| --- | --- | --- |
| `feat/` | 新しい機能の追加 | `feat/add-category-description-api` |
| `fix/` | バグの修正 | `fix/category-not-found-error` |
| `chore/` | 依存ライブラリの更新、CI設定変更など | `chore/update-sqlx-version` |
| `docs/` | ドキュメントの追加・修正 | `docs/add-api-usage-guide` |
| `refactor/` | 動作を変えないコード改善 | `refactor/simplify-category-usecase` |
| `test/` | テストの追加・修正 | `test/add-category-handler-tests` |

ルール:
- 英語の小文字とハイフン（`-`）を使う
- 日本語は使わない
- 短く、何の変更かわかる名前にする

---

## コミットメッセージの書き方

k1s0 では **Conventional Commits** の形式を使います。

### 基本形式

```
<種類>: <変更内容の要約>

<本文（任意）>
```

### 種類一覧

| 種類 | 意味 | 例 |
| --- | --- | --- |
| `feat:` | 新機能の追加 | `feat: カテゴリ説明文の更新APIを追加` |
| `fix:` | バグ修正 | `fix: カテゴリコードが空の場合のバリデーション追加` |
| `chore:` | 雑務（依存更新、設定変更等） | `chore: sqlx を 0.8.1 に更新` |
| `docs:` | ドキュメントの変更 | `docs: APIエンドポイントの説明を追加` |
| `refactor:` | 動作を変えないコード改善 | `refactor: カテゴリリポジトリのクエリを整理` |
| `test:` | テストの追加・修正 | `test: ManageCategoriesUseCaseのテストを追加` |

### 良い例・悪い例

```
# 良い例 --- 何を・なぜ変更したかがわかる
feat: カテゴリの説明文を更新するAPIを追加
fix: 存在しないカテゴリを更新しようとした時にpanicする問題を修正

# 悪い例 --- 何が変わったかわからない
修正
update
WIP
あとで直す
```

---

## 日常の作業フロー

### 1. リポジトリのクローン（初回のみ）

```bash
# リポジトリをクローン
git clone https://github.com/your-org/k1s0.git
cd k1s0
```

### 2. 最新の main を取得

```bash
# main ブランチに移動して最新を取得
git checkout main
git pull origin main
```

### 3. 作業ブランチを作成

```bash
# main から新しいブランチを作成して移動
git checkout -b feat/add-category-description-api
```

ここで `feat/add-category-description-api` の部分は、自分のタスクに合った名前にしてください。

### 4. 実装する

コードを編集します。こまめにコミットしましょう（1つの論理的なまとまりごとに）。

### 5. 変更をステージングしてコミット

```bash
# 変更されたファイルを確認
git status

# 変更内容を確認（何が変わったか確認する癖をつけよう）
git diff

# 特定のファイルをステージングに追加
git add src/domain/entity/master_category.rs
git add src/usecase/manage_categories.rs

# コミット
git commit -m "feat: カテゴリの説明文を更新するAPIを追加"
```

> **注意**: `git add .` や `git add -A` は避けてください。不要なファイル（`.env` や一時ファイル）を誤ってコミットする原因になります。追加するファイルは名前で指定しましょう。

### 6. リモートにプッシュ

```bash
# 初回プッシュ（-u でリモートブランチとの紐付けを行う）
git push -u origin feat/add-category-description-api

# 2回目以降
git push
```

### 7. PR（Pull Request）を作成

GitHub の Web UI またはコマンドラインで PR を作成します。

---

## PR（Pull Request）の出し方

### PR に書くべき内容

```markdown
## 概要
カテゴリの説明文を更新する gRPC API を追加した。

## 変更内容
- proto 定義に UpdateCategoryDescription RPC を追加
- usecase 層に update_category_description メソッドを追加
- adapter 層に gRPC ハンドラーを追加
- 単体テストを追加

## テスト方法
- `cargo test` で全テスト通過を確認
- docker-compose で起動し、grpcurl で動作確認済み

## 関連 Issue
- #123
```

### コマンドラインでの PR 作成（gh CLI を使う場合）

```bash
gh pr create \
  --title "feat: カテゴリ説明文の更新APIを追加" \
  --body "## 概要
カテゴリの説明文を更新する gRPC API を追加した。

## テスト方法
cargo test で全テスト通過を確認済み"
```

### PR のルール

- PR はなるべく小さくする（目安: 変更ファイル数が10以下）
- タイトルはコミットメッセージと同じ Conventional Commits 形式で書く
- レビュアーを必ず指定する
- CI（自動テスト）が全て通っていることを確認してからレビューを依頼する

---

## レビュー指摘への対応方法

レビューで指摘をもらったら、以下の手順で対応します。

```bash
# 1. 指摘に基づいてコードを修正する
#    （エディタでファイルを編集）

# 2. 修正内容を確認
git diff

# 3. 修正したファイルをステージングしてコミット
git add src/usecase/manage_categories.rs
git commit -m "fix: レビュー指摘対応 - エラーメッセージを改善"

# 4. プッシュ（PR が自動的に更新される）
git push
```

レビュー指摘への対応コミットは別コミットとして追加します。force push でコミット履歴を書き換えるのは避けてください。

---

## コンフリクト解消の手順

自分のブランチと main で同じファイルの同じ箇所を編集した場合、コンフリクト（衝突）が発生します。

### 手順

```bash
# 1. main の最新を取得
git fetch origin main

# 2. main を自分のブランチに取り込む（マージ）
git merge origin/main
```

コンフリクトがある場合、以下のような表示がファイルに挿入されます:

```
<<<<<<< HEAD
// 自分の変更
pub description: Option<String>,
=======
// main 側の変更
pub description: String,
>>>>>>> origin/main
```

### 解消方法

1. エディタでコンフリクト箇所を開く
2. `<<<<<<<`、`=======`、`>>>>>>>` の行を削除し、正しいコードだけを残す
3. 保存する

```bash
# 3. コンフリクトを解消したファイルをステージング
git add src/domain/entity/master_category.rs

# 4. マージコミットを作成
git commit -m "merge: origin/mainとのコンフリクトを解消"

# 5. プッシュ
git push
```

> **ヒント**: コンフリクトが怖い場合は、こまめに `git merge origin/main` して main の変更を取り込むと、大きなコンフリクトを避けられます。

---

## sparse-checkout の使い方

k1s0 はモノリポのため、全てのコードをダウンロードすると非常に大きくなります。sparse-checkout を使うと、自分が必要な領域だけを手元に持ってこられます。

### k1s0 CLI を使う方法（推奨）

```bash
# k1s0 CLI の対話式ウィザードで sparse-checkout を設定する
k1s0 init
# または: k1s0 → よく使う操作 > プロジェクト初期化
```

### 手動で設定する方法

```bash
# 1. リポジトリをクローン（blobless clone で高速化）
git clone --filter=blob:none --sparse https://github.com/your-org/k1s0.git
cd k1s0

# 2. sparse-checkout を有効化（cone モード）
git sparse-checkout init --cone

# 3. 必要なディレクトリだけを追加
git sparse-checkout set \
  regions/system/ \
  regions/business/accounting/ \
  api/proto/ \
  docs/

# 4. 確認（指定したディレクトリだけが表示される）
ls regions/business/
```

### 領域を追加する場合

```bash
# 既存の設定を維持しつつ、新しい領域を追加
git sparse-checkout add regions/business/fa/
```

---

## やってはいけないこと

以下は重大な問題を引き起こす可能性があるため、絶対に行わないでください。

### main への直接プッシュ

```bash
# 絶対にやらないこと
git checkout main
git commit -m "ちょっとした修正"
git push origin main   # NG! 必ず PR を通す
```

### force push

```bash
# 絶対にやらないこと
git push --force       # NG! 他の人の作業を壊す可能性がある
git push -f            # NG! 同上
```

### 他の人のブランチを勝手にリベース

```bash
# やらないこと
git rebase main        # 共有ブランチでは危険
                       # git merge origin/main を使う
```

### 機密情報をコミット

```bash
# やらないこと
git add .env           # NG! パスワードや API キーが含まれている
git add credentials.json  # NG!
```

もし誤ってコミットしてしまった場合は、すぐにチームリーダーに報告してください。Git の履歴にはコミットが残るため、単にファイルを削除するだけでは不十分です。

---

## よく使う Git コマンド一覧

| コマンド | 説明 |
| --- | --- |
| `git status` | 変更されたファイルの一覧を表示 |
| `git diff` | 変更内容の差分を表示 |
| `git log --oneline -10` | 直近10件のコミットログを1行ずつ表示 |
| `git branch` | ローカルブランチの一覧を表示 |
| `git branch -a` | リモートブランチも含めた一覧を表示 |
| `git stash` | 作業中の変更を一時退避（ブランチを切り替えたい時に便利） |
| `git stash pop` | 退避した変更を復元 |
| `git fetch origin` | リモートの最新情報を取得（ローカルは変更しない） |
| `git checkout -- <file>` | ファイルの変更を取り消す（未コミットの変更を破棄） |

---

## 関連ドキュメント

- [CI/CD 設計](../../infrastructure/cicd/CI-CD設計.md)
- [コーディング規約](../../architecture/conventions/コーディング規約.md)
