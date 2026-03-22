# service tier Git ワークフロー

Git の基本操作からブランチ戦略、PR の出し方、コンフリクト解消まで、service tier の開発で必要な Git ワークフローを具体的なコマンド付きで説明する。

---

## ブランチ戦略

k1s0 では **main ブランチ** と **feature ブランチ** の 2 種類を使うシンプルな戦略を採用している。

```
main ─────●────●────●────●─────── 常にデプロイ可能な状態
            \       /    \      /
             ●───●        ●──●     feature ブランチ（作業用）
```

| ブランチ | 役割 | マージ条件 |
| --- | --- | --- |
| `main` | 本番に近い状態を維持する。CI が通り、レビュー済みのコードだけがここに入る | -- |
| `feat/*`, `fix/*` 等 | 機能追加・バグ修正などの作業用ブランチ | PR のレビュー承認 + CI 通過 |

### ブランチ命名規則

```
{種類}/{サービス名}/{簡潔な説明}
```

| 種類 | 用途 | 例 |
| --- | --- | --- |
| `feat` | 新機能追加 | `feat/task/add-status-filter` |
| `fix` | バグ修正 | `fix/task/status-transition-error` |
| `refactor` | リファクタリング（動作は変えない） | `refactor/task/extract-usecase` |
| `docs` | ドキュメント修正 | `docs/task/update-api-spec` |
| `test` | テスト追加・修正 | `test/task/add-integration-tests` |
| `chore` | 雑務（依存更新、設定変更等） | `chore/task/update-dependencies` |

**ポイント**: サービス名を含めることで、どのサービスの変更かが一目でわかる。

---

## コミットメッセージの書き方

[Conventional Commits](https://www.conventionalcommits.org/) の形式に従う。

```
{種類}({スコープ}): {変更内容の要約}

{本文（任意）: なぜこの変更が必要かを説明}
```

### 例

```
feat(task): タスクステータス更新APIを追加

UpdateTaskStatusのgRPCエンドポイントを追加。
ステータス遷移のバリデーションはドメイン層で実装。
```

```
fix(task-react): ステータスフィルターが初期化されない問題を修正
```

```
test(task): タスク作成ユースケースの単体テストを追加
```

### 種類一覧

| 種類 | 用途 |
| --- | --- |
| `feat` | 新機能 |
| `fix` | バグ修正 |
| `refactor` | リファクタリング |
| `test` | テスト追加・修正 |
| `docs` | ドキュメント |
| `chore` | 依存更新、設定変更等 |
| `style` | フォーマット修正（動作に影響しない） |
| `perf` | パフォーマンス改善 |

### 守るべきルール

- 1 行目は **50 文字以内**（日本語の場合は目安として 25 文字程度）
- 1 行目は動詞で始める（「追加」「修正」「削除」など）
- 本文は 1 行空けてから書く
- 「何を変えたか」よりも「なぜ変えたか」を重視する

---

## 日常の作業フロー

### 1. リポジトリのクローン（初回のみ）

```bash
# k1s0 CLI を使う場合（推奨）
$ k1s0
# メインメニュー → プロジェクト設定 → sparse-checkout 設定
# 自分の担当サービスを選択すると、必要な部分だけクローンされる

# 手動の場合
$ git clone --filter=blob:none --sparse https://github.com/{org}/k1s0.git
$ cd k1s0
$ git sparse-checkout set regions/service/task regions/system/library
```

### 2. 最新の main を取得

```bash
$ git checkout main
$ git pull origin main
```

### 3. feature ブランチを作成

```bash
$ git checkout -b feat/task/add-status-update-api
```

### 4. 実装・テスト

コードを書いて、テストを通す。

```bash
# Rust サーバーのテスト
$ cd regions/service/task/server/rust/task
$ cargo test

# React クライアントのテスト
$ cd regions/service/task/client/react/task
$ pnpm test
```

### 5. 変更をステージング

```bash
# 変更内容を確認
$ git status
$ git diff

# 特定のファイルをステージング（推奨）
$ git add regions/service/task/server/rust/task/src/domain/model/task.rs
$ git add regions/service/task/server/rust/task/src/application/usecase/update_status.rs

# まとめてステージングする場合
$ git add regions/service/task/
```

### 6. コミット

```bash
$ git commit -m "feat(task): タスクステータス更新APIを追加"
```

### 7. リモートにプッシュ

```bash
$ git push origin feat/task/add-status-update-api
```

### 8. PR を作成

GitHub の画面、または `gh` CLI で作成する（後述）。

---

## PR（Pull Request）の出し方

### GitHub CLI での作成

```bash
$ gh pr create --title "feat(task): タスクステータス更新APIを追加" --body "$(cat <<'EOF'
## 概要
タスクのステータスを更新するgRPC APIを追加しました。

## 変更内容
- domain層: TaskStatusの遷移バリデーション追加
- usecase層: UpdateTaskStatusUsecase追加
- infrastructure層: TaskRepositoryにupdate_status実装
- presentation層: gRPCハンドラー追加
- テスト: 各層の単体テスト追加

## テスト方法
- `cargo test` で全テスト通過を確認
- ローカルでgRPCurlを使って動作確認済み

## 影響範囲
- taskサービスのサーバーのみ
- 既存APIへの影響なし
EOF
)"
```

### PR に書くべき内容

| セクション | 内容 |
| --- | --- |
| 概要 | この PR で何を実現するか（1-2 文） |
| 変更内容 | 具体的に何を変えたか（箇条書き） |
| テスト方法 | どうやって動作を確認したか |
| 影響範囲 | 他のサービスやコンポーネントへの影響 |
| スクリーンショット | UI 変更がある場合はビフォー/アフター |
| 関連 Issue | 関連する Issue 番号（例: `Closes #123`） |

---

## レビュー指摘への対応

### 指摘を受けたら

1. **すべてのコメントを読む**（返信する前にすべて把握する）
2. **修正する**
3. **新しいコミットとして追加する**（既存コミットを amend しない）
4. **レビュアーに返信する**（何をどう修正したか簡潔に説明）
5. **再レビューを依頼する**

```bash
# 修正をコミット
$ git add regions/service/task/server/rust/task/src/domain/model/task.rs
$ git commit -m "fix(task): レビュー指摘対応 - ステータス遷移の検証を強化"

# プッシュ
$ git push origin feat/task/add-status-update-api
```

### 指摘に同意しない場合

- 理由を説明して議論する（感情的にならない）
- チームの判断を尊重する

---

## コンフリクト解消の手順

コンフリクト（競合）は、同じファイルの同じ箇所を複数人が変更した場合に発生する。

### 手順

```bash
# 1. main の最新を取得
$ git checkout main
$ git pull origin main

# 2. 自分のブランチに戻る
$ git checkout feat/order/add-status-update-api

# 3. main をマージ（ここでコンフリクトが発生する場合がある）
$ git merge main
```

### コンフリクトが発生した場合

ファイル内に以下のようなマーカーが入る。

```
<<<<<<< HEAD
// 自分の変更
pub fn update_status(&mut self, new_status: TaskStatus) -> Result<(), DomainError> {
=======
// main側の変更
pub fn change_status(&mut self, new_status: TaskStatus) -> Result<(), DomainError> {
>>>>>>> main
```

**解消手順**:

1. コンフリクトが起きたファイルを開く
2. `<<<<<<<`、`=======`、`>>>>>>>` のマーカーを確認
3. どちらの変更を残すか（あるいは両方を統合するか）判断して編集
4. マーカーをすべて削除
5. テストを実行して動作確認

```bash
# 4. コンフリクトを解消したファイルをステージング
$ git add regions/service/task/server/rust/task/src/domain/model/task.rs

# 5. マージコミットを作成
$ git commit -m "merge: mainブランチをマージしコンフリクトを解消"

# 6. プッシュ
$ git push origin feat/task/add-status-update-api
```

**迷ったら**: 変更した本人に相談する。自分で判断できない場合は「間違って直すより、聞いて正しく直す」方が安全。

---

## sparse-checkout の使い方

k1s0 はモノリポのため、全ファイルをチェックアウトすると非常に大きくなる。sparse-checkout を使って、自分に必要な部分だけを取得する。

### k1s0 CLI での設定（推奨）

```bash
$ k1s0
# メインメニュー → プロジェクト設定 → sparse-checkout 設定
# 担当サービス（例: task）を選択
# → 自動的に以下がチェックアウトされる:
#   - regions/service/task/           （自分のサービス）
#   - regions/system/library/         （system tier ライブラリ）
#   - regions/business/{領域名}/      （所属するbusiness tier）
```

### 手動での設定

```bash
# 現在のsparse-checkout設定を確認
$ git sparse-checkout list

# チェックアウト対象を追加
$ git sparse-checkout add regions/service/board

# チェックアウト対象を再設定（上書き）
$ git sparse-checkout set \
  regions/service/task \
  regions/system/library \
  regions/business/taskmanagement
```

### 何をチェックアウトすべきか

| 対象 | パス | 理由 |
| --- | --- | --- |
| 自分のサービス | `regions/service/{サービス名}/` | 開発対象 |
| system ライブラリ | `regions/system/library/` | ほぼ全サービスが依存 |
| 所属する business tier | `regions/business/{領域名}/` | 領域共通機能の利用 |
| 他サービス（任意） | `regions/service/{別サービス名}/` | 参考にしたい場合のみ |

---

## やってはいけないこと

### 絶対にやらない

| やってはいけないこと | 理由 | 代わりにどうするか |
| --- | --- | --- |
| `git push --force` を main に実行 | 他の全員の作業が壊れる | feature ブランチでのみ、必要に応じて `--force-with-lease` を使う |
| main ブランチに直接コミット | レビューなしのコードが入る | 必ず feature ブランチ → PR → レビューの流れで |
| `.env` や認証情報をコミット | 機密情報が漏洩する | `.gitignore` に追加し、環境変数や Vault で管理する |
| 巨大なバイナリファイルをコミット | リポジトリが肥大化する | Git LFS を使うか、別の方法で管理する |
| 他サービスのコードを勝手に変更 | 担当者の知らないところで壊れる | 担当者に相談してから変更する |

### できれば避ける

| 避けたいこと | 理由 | 代わりにどうするか |
| --- | --- | --- |
| 1 つの PR に大量の変更を入れる | レビューが困難になる | 機能ごとに小さな PR に分割する |
| コミットメッセージが雑（「修正」だけ等） | 後から変更理由がわからなくなる | Conventional Commits に従い具体的に書く |
| 長期間マージしないブランチ | コンフリクトが増える | こまめに main をマージするか、小さな PR で早めにマージする |
| `git add .` や `git add -A` の多用 | 不要なファイルが混入するリスク | 変更したファイルを個別に指定して add する |

---

## 関連ドキュメント

- [CI/CD 設計](../../infrastructure/cicd/CI-CD設計.md) -- GitHub Actions パイプライン設計の詳細
- [CICD テンプレート](../../templates/infrastructure/CICD.md) -- CI/CD ひな形仕様
- [コーディング規約](../../architecture/conventions/コーディング規約.md) -- lint ルールの詳細
- [CLIフロー](../../cli/flow/CLIフロー.md) -- k1s0 CLI の操作フロー
