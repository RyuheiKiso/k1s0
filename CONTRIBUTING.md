# Contributing to k1s0

k1s0 プロジェクトへの貢献を歓迎します。このドキュメントでは、ブランチ戦略・コミット規約・PRプロセス・コーディング規約について説明します。

## 目次

- [はじめに](#はじめに)
- [ブランチ戦略](#ブランチ戦略)
- [コミットメッセージ規約](#コミットメッセージ規約)
- [PRプロセス](#prプロセス)
- [コーディング規約](#コーディング規約)
- [セットアップ手順](#セットアップ手順)

---

## はじめに

- オンボーディングガイド: [`docs/onboarding/README.md`](docs/onboarding/README.md)
- 開発環境セットアップ: [`docs/infrastructure/devenv/windows-quickstart.md`](docs/infrastructure/devenv/windows-quickstart.md)

---

## ブランチ戦略

k1s0 では **main + feature branches** のシンプルなブランチ戦略を採用しています。

```
main（本番用ブランチ。常に動く状態を保つ）
 ├── feat/add-category-description   ← 機能追加ブランチ
 ├── fix/category-validation-error   ← バグ修正ブランチ
 └── chore/update-dependencies       ← 雑務ブランチ
```

| ブランチ | 役割 | マージ条件 |
| --- | --- | --- |
| `main` | 本番に近い状態を維持する。CI が通り、レビュー済みのコードだけがここに入る | -- |
| `feat/*`, `fix/*` 等 | 機能追加・バグ修正などの作業用ブランチ | PR のレビュー承認 + CI 通過 |

### ブランチ命名規則

ブランチ名は `{種類}/{サービス名}/{簡潔な説明}` の形式で付けます。

| 接頭辞 | 用途 | 例 |
| --- | --- | --- |
| `feat/` | 新しい機能の追加 | `feat/task/add-status-filter` |
| `fix/` | バグの修正 | `fix/task/total-calculation-error` |
| `chore/` | 依存ライブラリの更新、CI設定変更など | `chore/update-dependencies` |
| `docs/` | ドキュメントの追加・修正 | `docs/task/update-api-spec` |
| `refactor/` | 動作を変えないコード改善 | `refactor/task/extract-usecase` |
| `test/` | テストの追加・修正 | `test/task/add-integration-tests` |

ルール:
- 英語の小文字とハイフン（`-`）を使う
- 日本語は使わない
- 短く、何の変更かわかる名前にする

### やってはいけないこと

| やってはいけないこと | 理由 | 代わりにどうするか |
| --- | --- | --- |
| `git push --force` を main に実行 | 他の全員の作業が壊れる | feature ブランチでのみ、必要に応じて `--force-with-lease` を使う |
| main ブランチに直接コミット | レビューなしのコードが入る | 必ず feature ブランチ → PR → レビューの流れで |
| `.env` や認証情報をコミット | 機密情報が漏洩する | `.gitignore` に追加し、環境変数や Vault で管理する |

---

## コミットメッセージ規約

k1s0 では [Conventional Commits](https://www.conventionalcommits.org/) の形式を採用しています。

### 基本形式

```
{種類}({スコープ}): {変更内容の要約}

{本文（任意）: なぜこの変更が必要かを説明}
```

### 種類一覧

| 種類 | 意味 | 例 |
| --- | --- | --- |
| `feat` | 新機能の追加 | `feat(task): タスクステータス更新APIを追加` |
| `fix` | バグ修正 | `fix(task): ステータス遷移の検証を強化` |
| `chore` | 雑務（依存更新、設定変更等） | `chore: sqlx を 0.8.1 に更新` |
| `docs` | ドキュメントの変更 | `docs: APIエンドポイントの説明を追加` |
| `refactor` | 動作を変えないコード改善 | `refactor(task): カテゴリリポジトリのクエリを整理` |
| `test` | テストの追加・修正 | `test(task): タスク作成ユースケースの単体テストを追加` |
| `style` | フォーマット修正（動作に影響しない） | `style: rustfmt による整形` |
| `perf` | パフォーマンス改善 | `perf(search): クエリのインデックス最適化` |

### 守るべきルール

- 1 行目は **50 文字以内**（日本語の場合は目安として 25 文字程度）
- 本文は 1 行空けてから書く
- 「何を変えたか」よりも「なぜ変えたか」を重視する
- スコープは変更対象のサービス名・モジュール名を使う（例: `auth`, `task`, `bff-proxy`）
- スコープが全体に及ぶ場合は省略可（例: `chore: rustfmt で全体を整形`）

### 良い例・悪い例

```
# =============================================
# 良い例 --- 何を・なぜ変更したかがわかる
# =============================================

# 新機能: スコープ付き
feat(task): タスクステータス更新APIを追加

# バグ修正: 問題の内容が明確
fix(auth): 存在しないユーザーを更新しようとした時にpanicする問題を修正

# ドキュメント変更
docs(bff-proxy): ルーティング設計のセクションを追加

# リファクタリング: 動作への影響なし
refactor(tenant): テナントリポジトリのクエリビルダーを共通化

# テスト追加
test(session): セッション期限切れ時のsilent refreshのユニットテストを追加

# 依存関係更新
chore: sqlx を 0.8.0 から 0.8.1 に更新

# ビルド・CI 設定変更
chore(ci): modules.yaml の maturity チェックステップを追加

# 本文付きコミット（なぜ変えたかを説明）
fix(bff-proxy): upstream base_url のデフォルト値を Kong のアドレスに変更

本番環境では bff-proxy が Kong を経由してバックエンドにルーティングする必要があるが、
デフォルト値が auth-server を直接指していたため、本番デプロイ時に手動設定漏れが発生しやすかった。
config.docker.yaml のデフォルト値を Kong アドレスに変更することで設定ミスを防止する。

# =============================================
# 悪い例 --- 何が変わったかわからない
# =============================================

修正          # ← 何を修正したかわからない
update        # ← 英語でも内容不明
WIP           # ← 作業中のまま。コミットしない
あとで直す    # ← 意図が伝わらない
feat: いろいろ追加  # ← 変更が複数混在している場合は分割する
```

---

## PRプロセス

### PRの出し方

```bash
# GitHub CLI での作成例
gh pr create --title "feat(task): タスクステータス更新APIを追加" --body "$(cat <<'EOF'
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

### PRテンプレート（書くべき内容）

| セクション | 内容 |
| --- | --- |
| 概要 | この PR で何を実現するか（1-2 文） |
| 変更内容 | 具体的に何を変えたか（箇条書き） |
| テスト方法 | どうやって動作を確認したか |
| 影響範囲 | 他のサービスやコンポーネントへの影響 |
| 関連 Issue | 関連する Issue 番号（例: `Closes #123`） |

### レビュー基準

コードレビューでは以下の観点を確認します:

| # | 観点 | 確認内容 |
| - | --- | --- |
| 1 | 機能要件 | 仕様通りに動作するか |
| 2 | Tier 依存ルール | 依存方向が正しいか（下位→上位のみ） |
| 3 | エラーハンドリング | 適切なエラー型を使用しているか、エラーメッセージは十分か |
| 4 | セキュリティ | 入力バリデーション、認証・認可チェックが適切か |
| 5 | テスト | ユニットテスト・統合テストが追加されているか |
| 6 | パフォーマンス | N+1 クエリ、不要なメモリアロケーションがないか |

### マージ条件

- PR はなるべく小さくする（目安: 変更ファイル数が10以下）
- タイトルは Conventional Commits 形式で書く
- レビュアーを必ず指定する
- CI（自動テスト）が全て通っていることを確認してからレビューを依頼する
- **通常の変更**: 最低 1 名の Approve が必要
- **system Tier の変更**: 2 名の Approve が必要

### レビュー指摘への対応

レビュー指摘は新しいコミットとして追加します（既存コミットへの amend は避ける）。

```bash
# 修正をコミット
git add regions/service/task/server/rust/src/domain/model/task.rs
git commit -m "fix(task): レビュー指摘対応 - ステータス遷移の検証を強化"

# プッシュ（PR が自動的に更新される）
git push
```

---

## コーディング規約

言語別の詳細なコーディング規約は [`docs/architecture/conventions/コーディング規約.md`](docs/architecture/conventions/コーディング規約.md) を参照してください。

### 言語別ツールチェーン早見表

| 言語 | フォーマッター | リンター | テストランナー |
| --- | --- | --- | --- |
| **Rust** | `rustfmt` | `clippy` | `cargo test` |
| **Go** | `gofmt` | `golangci-lint` | `go test` |
| **TypeScript** | `prettier` | `eslint` | `vitest` |
| **Dart** | `dart format` | `dart analyze` | `flutter_test` |

### 命名規則

| 言語 | 変数・関数 | 型・構造体 | 定数 | ファイル名 |
| --- | --- | --- | --- | --- |
| Go | camelCase | PascalCase | SCREAMING_SNAKE_CASE | snake_case.go |
| Rust | snake_case | PascalCase | SCREAMING_SNAKE_CASE | snake_case.rs |
| TypeScript | camelCase | PascalCase | SCREAMING_SNAKE_CASE | kebab-case.ts |
| Dart | camelCase | PascalCase | camelCase | snake_case.dart |

### 共通方針

- 各言語の公式スタイルガイドに従う
- フォーマットは保存時に自動実行する（Dev Container の設定で統一）
- Linter の警告はすべて解消してからコミットする
- `// nolint`・`#[allow]`・`// eslint-disable` 等の抑制コメントには理由を明記する

---

## 開発環境のセットアップ（IDE 設定）

### VS Code（推奨）

`.vscode/settings.json` に推奨設定が含まれています。以下の拡張機能をインストールしてください:

```json
// .vscode/extensions.json（推奨拡張機能）
{
  "recommendations": [
    "rust-lang.rust-analyzer",    // Rust 言語サポート
    "golang.go",                   // Go 言語サポート
    "ms-vscode.vscode-typescript-next", // TypeScript サポート
    "Dart-Code.dart-code",         // Dart/Flutter サポート
    "tamasfe.even-better-toml",    // TOML ファイルサポート
    "ms-azuretools.vscode-docker", // Docker サポート
    "redhat.vscode-yaml",          // YAML サポート
    "yzhang.markdown-all-in-one"   // Markdown サポート
  ]
}
```

### IntelliJ IDEA / GoLand / RustRover

1. **Rust**: RustRover または IntelliJ IDEA + Rust プラグインを使用
2. **Go**: GoLand または IntelliJ IDEA + Go プラグインを使用
3. **TypeScript**: WebStorm または IntelliJ IDEA を使用
4. **Dart/Flutter**: IntelliJ IDEA + Flutter プラグインを使用

EditorConfig (`.editorconfig`) は IDE が自動で読み込むため、インデントやラインエンドの設定は自動適用されます。

### 共通設定

`.editorconfig` により以下が自動設定されます:
- インデント: スペース4（Rust/Go/TypeScript/Dart 共通）
- 文字コード: UTF-8
- 行末: LF（Unix）
- 末尾改行: あり

---

## セットアップ手順

- **Day 1 クイックスタート**: [`docs/onboarding/quickstart.md`](docs/onboarding/quickstart.md)
- Windows 環境のセットアップ: [`docs/infrastructure/devenv/windows-quickstart.md`](docs/infrastructure/devenv/windows-quickstart.md)
- オンボーディングガイド全般: [`docs/onboarding/README.md`](docs/onboarding/README.md)
- Git ワークフロー詳細（tier2）: [`docs/onboarding/tier2/07-Gitワークフロー.md`](docs/onboarding/tier2/07-Gitワークフロー.md)
- Git ワークフロー詳細（tier3/service）: [`docs/onboarding/tier3/07-Gitワークフロー.md`](docs/onboarding/tier3/07-Gitワークフロー.md)

### k1s0 CLI インストール

```bash
# リポジトリルートで実行
cargo install --path CLI/crates/k1s0-cli

# 環境診断
just doctor
```
