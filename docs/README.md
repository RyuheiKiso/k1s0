# k1s0 ドキュメントポータル

> 新規参加者は [オンボーディングガイド](./onboarding/README.md) から始めてください。

k1s0 プロジェクト全ドキュメントのナビゲーションマップ。設計書・仕様書・ガイドへの入口となるポータル。

## ドキュメントマップ

### アーキテクチャ

[architecture/README.md](./architecture/README.md) — システム全体設計・API・認証・規約・テスト・メッセージング・可観測性・デプロイ方針

### サーバー設計書

[servers/README.md](./servers/README.md) — system / business / service 各 Tier のサーバー設計書（32サーバー）

### ライブラリ設計書

[libraries/README.md](./libraries/README.md) — 全共通ライブラリの設計書（50+ライブラリ）

### インフラ

[infrastructure/README.md](./infrastructure/README.md) — CI/CD・開発環境・Docker・Kubernetes・セキュリティ・Terraform

### CLI・GUI

[cli/README.md](./cli/README.md) — k1s0 CLI の設計・コード生成・マイグレーション・Tauri GUI

### テンプレート仕様

[templates/README.md](./templates/README.md) — CLI コード生成テンプレート仕様（サーバー・クライアント・インフラ等）

### オンボーディング

[onboarding/README.md](./onboarding/README.md) — Tier 別の入門ガイド（tier1 / tier2 / tier3）

---

## よく使うリンク

| 用途 | リンク |
|------|--------|
| プロジェクトコンセプト | [architecture/overview/コンセプト.md](./architecture/overview/コンセプト.md) |
| Tier 構成 | [architecture/overview/tier-architecture.md](./architecture/overview/tier-architecture.md) |
| コーディング規約 | [architecture/conventions/コーディング規約.md](./architecture/conventions/コーディング規約.md) |
| 開発環境セットアップ | [infrastructure/devenv/windows-quickstart.md](./infrastructure/devenv/windows-quickstart.md) |
| ディレクトリ構成図 | [architecture/overview/ディレクトリ構成図.md](./architecture/overview/ディレクトリ構成図.md) |
| 認証認可設計 | [architecture/auth/認証認可設計.md](./architecture/auth/認証認可設計.md) |
| API 設計 | [architecture/api/API設計.md](./architecture/api/API設計.md) |
| ライブラリ概要 | [libraries/_common/概要.md](./libraries/_common/概要.md) |
