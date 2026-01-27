# k1s0 サブエージェント構成

k1s0 プロジェクト専用の Claude Code サブエージェント群です。

## 概要

```
orchestrator (代表エージェント)
├── rust-dev       Rust CLI/Framework 開発
├── go-dev         Go バックエンド開発
├── frontend-dev   React/Flutter 開発
├── template-mgr   テンプレート管理
├── lint-quality   Lint/品質管理
├── docs-writer    ドキュメント作成
├── api-designer   gRPC/OpenAPI 設計
├── ci-cd          CI/CD 管理
└── researcher     調査・分析
```

## エージェント一覧

| エージェント | 担当領域 | 主な作業 |
|-------------|---------|---------|
| **orchestrator** | 全体調整 | タスク分析、エージェント選定、結果統合 |
| **rust-dev** | Rust 開発 | CLI, Framework crate, マイクロサービス |
| **go-dev** | Go 開発 | Go バックエンド、テンプレート |
| **frontend-dev** | フロントエンド | React, Flutter, UI コンポーネント |
| **template-mgr** | テンプレート | テンプレート作成、manifest 管理 |
| **lint-quality** | 品質管理 | Lint ルール実装、コード品質 |
| **docs-writer** | ドキュメント | 設計書、規約、ADR |
| **api-designer** | API 設計 | gRPC, OpenAPI, 契約管理 |
| **ci-cd** | CI/CD | GitHub Actions, パイプライン |
| **researcher** | 調査・分析 | コードベース調査、原因分析、影響範囲調査 |

## 使用方法

### 代表エージェント経由（推奨）

```
Claude に対して:
「orchestrator エージェントを使って、新しい Framework crate を追加してほしい」
```

orchestrator が要求を分析し、適切なサブエージェント（rust-dev, docs-writer など）を起動します。

### 直接指定

特定の作業が明確な場合は直接指定も可能:

```
「rust-dev エージェントを使って、k1s0-cache の Redis 接続を改善して」
「docs-writer エージェントを使って、新しい ADR を作成して」
```

## タスク例

### 単一エージェント

| タスク | エージェント |
|-------|-------------|
| CLI に新コマンド追加 | rust-dev |
| React フック作成 | frontend-dev |
| Lint ルール K033 追加 | lint-quality |
| ADR 作成 | docs-writer |
| gRPC サービス定義 | api-designer |
| GitHub Actions 追加 | ci-cd |
| バグの原因調査 | researcher |
| 機能の実装箇所特定 | researcher |

### 複数エージェント協調

**「新しい Framework crate を追加」**
1. api-designer → API 設計
2. rust-dev → 実装
3. docs-writer → ドキュメント

**「新しいテンプレートタイプを追加」**
1. template-mgr → テンプレート設計
2. rust-dev / go-dev → コード
3. lint-quality → Lint ルール
4. docs-writer → ドキュメント

**「パフォーマンス問題を解決」**
1. researcher → 問題箇所特定、原因分析
2. rust-dev / 該当エージェント → 修正実装

## ディレクトリ構造

```
.claude/
├── agents/
│   ├── README.md           # このファイル
│   ├── orchestrator.md     # 代表エージェント
│   ├── rust-dev.md         # Rust 開発
│   ├── go-dev.md           # Go 開発
│   ├── frontend-dev.md     # フロントエンド
│   ├── template-mgr.md     # テンプレート
│   ├── lint-quality.md     # Lint/品質
│   ├── docs-writer.md      # ドキュメント
│   ├── api-designer.md     # API 設計
│   ├── ci-cd.md            # CI/CD
│   └── researcher.md       # 調査・分析
└── settings.local.json     # 権限設定
```

## カスタマイズ

各エージェントの `.md` ファイルを編集することで、動作をカスタマイズできます:

- 担当領域の追加・変更
- 開発規約の更新
- 作業手順の改善
