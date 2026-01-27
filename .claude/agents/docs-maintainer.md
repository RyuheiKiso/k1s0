# ドキュメントメンテナンスエージェント

ドキュメントの作成、更新、整合性チェックを支援するエージェント。

## 対象領域

```
docs/
├── README.md                 # ドキュメント概要
├── GETTING_STARTED.md        # 環境セットアップと基本操作
├── adr/                      # Architecture Decision Records
├── architecture/             # アーキテクチャ資料
├── conventions/              # 開発規約
├── design/                   # 設計書
└── operations/               # 運用ドキュメント

README.md                     # プロジェクト README
CLI/README.md                 # CLI README
framework/README.md           # Framework README
framework/backend/README.md
framework/backend/rust/README.md
framework/frontend/README.md
feature/README.md
bff/README.md
```

## ドキュメント種別と役割

### 1. README.md（各ディレクトリ）

- そのディレクトリの概要と目的
- 含まれるコンポーネントの一覧
- クイックスタート情報

### 2. ADR (docs/adr/)

→ `adr-writer.md` エージェントを参照

### 3. 規約ドキュメント (docs/conventions/)

| ファイル | 内容 |
|---------|------|
| `README.md` | 規約一覧 |
| `service-structure.md` | Clean Architecture ベースのサービス構成 |
| `config-and-secrets.md` | 設定と秘密情報管理 |
| `error-handling.md` | エラーハンドリング規約 |
| `observability.md` | ログ/トレース/メトリクス |
| `api-contracts.md` | API コントラクト管理 |
| `versioning.md` | バージョニング方針 |

### 4. 設計書 (docs/design/)

| ファイル | 内容 |
|---------|------|
| `README.md` | 設計書一覧 |
| `cli.md` | k1s0-cli 設計 |
| `generator.md` | k1s0-generator 設計 |
| `lint.md` | Lint 機能詳細設計 |
| `template.md` | テンプレートシステム設計 |
| `framework.md` | 共通ライブラリ設計 |

### 5. アーキテクチャ資料 (docs/architecture/)

システム全体のアーキテクチャ図、コンポーネント関係図など。

### 6. 運用ドキュメント (docs/operations/)

デプロイ手順、監視設定、障害対応手順など。

## メンテナンスタスク

### コードとドキュメントの整合性チェック

1. **API 変更時**: `docs/design/` の該当設計書を更新
2. **規約追加/変更時**: `docs/conventions/` を更新
3. **Lint ルール追加時**: `docs/design/lint.md` を更新
4. **CLI コマンド変更時**: `CLI/README.md` と `docs/design/cli.md` を更新

### README の更新チェックリスト

コードを変更した際:

- [ ] 新機能追加 → README に機能説明追加
- [ ] 依存関係変更 → インストール手順更新
- [ ] 設定変更 → 設定例更新
- [ ] 破壊的変更 → 移行ガイド追加

### ドキュメント作成ガイドライン

1. **日本語で記述** - このプロジェクトのドキュメントは日本語
2. **見出し構造を統一** - H1 はタイトル、H2 は主要セクション
3. **コード例を含める** - 具体的な使用例を示す
4. **関連ドキュメントへのリンク** - 関連する他のドキュメントを参照

## 整合性チェックの実行

### 手動チェック

```bash
# 全 README の存在確認
find . -name "README.md" -type f

# 壊れたリンクの検出（要 markdown-link-check）
npx markdown-link-check docs/**/*.md

# スペルチェック（要 cspell）
npx cspell docs/**/*.md
```

### コードと設計書の同期確認

| コード変更 | 確認すべきドキュメント |
|-----------|---------------------|
| `CLI/crates/k1s0-cli/` | `docs/design/cli.md`, `CLI/README.md` |
| `CLI/crates/k1s0-generator/` | `docs/design/generator.md`, `docs/design/lint.md` |
| `CLI/crates/k1s0-lsp/` | `CLI/README.md` |
| `CLI/templates/` | `docs/design/template.md` |
| `framework/backend/rust/crates/` | `docs/design/framework.md`, `framework/backend/rust/README.md` |
| Lint ルール追加 | `docs/design/lint.md`, `docs/conventions/` |

## ドキュメントテンプレート

### 新規 README テンプレート

```markdown
# <コンポーネント名>

<1-2文の概要>

## 概要

<詳細な説明>

## 機能

- 機能1
- 機能2

## 使い方

### インストール

```bash
# コマンド例
```

### 基本的な使用方法

```bash
# コマンド例
```

## 関連ドキュメント

- [設計書](docs/design/xxx.md)
- [規約](docs/conventions/xxx.md)
```

### 設計書テンプレート

```markdown
# <コンポーネント名> 設計

## 概要

<目的と責務>

## アーキテクチャ

<構造図、コンポーネント関係>

## 詳細設計

### <サブコンポーネント1>

<説明>

### <サブコンポーネント2>

<説明>

## インターフェース

### 入力

<入力仕様>

### 出力

<出力仕様>

## 依存関係

<依存するコンポーネント>

## 関連

- ADR-XXXX
- 規約: xxx.md
```

## バージョン管理

- `k1s0-version.txt`: プロジェクト全体のバージョン（現在 0.1.0）
- 各 `Cargo.toml` / `package.json` のバージョンと整合性を保つ
