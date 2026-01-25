# k1s0

高速な開発サイクルを実現する framework / templates / CLI を含むモノレポ。

## ディレクトリ構成

```
k1s0/
├── CLI/                    # 雛形生成・導入・アップグレード CLI
│   ├── crates/             # CLI を構成する Rust crate 群
│   │   ├── k1s0-cli/       # 実行 CLI (clap)
│   │   └── k1s0-generator/ # テンプレ展開・差分適用ロジック
│   └── templates/          # 生成テンプレ群
│       ├── backend-rust/
│       ├── backend-go/
│       ├── frontend-react/
│       └── frontend-flutter/
├── framework/              # 共通部品・共通サービス
│   ├── backend/
│   │   ├── rust/
│   │   │   ├── crates/     # 共通 crate 群
│   │   │   └── services/   # 共通マイクロサービス
│   │   └── go/
│   ├── frontend/
│   │   ├── react/
│   │   └── flutter/
│   └── database/
│       └── table/          # 共通テーブル定義（DDL 正本）
├── feature/                # 個別機能チームのサービス実装
│   ├── backend/
│   │   ├── rust/
│   │   └── go/
│   ├── frontend/
│   │   ├── react/
│   │   └── flutter/
│   └── database/
├── bff/                    # フロントエンド向け集約 API 層（任意）
├── docs/                   # ドキュメント
│   ├── adr/                # Architecture Decision Records
│   ├── architecture/       # アーキテクチャ設計
│   ├── conventions/        # 規約
│   └── operations/         # 運用
└── work/                   # 検討中の草案
```

## クイックスタート

```bash
# リポジトリ初期化
k1s0 init

# 新規サービスの生成
k1s0 new-feature --type backend-rust --name user-management

# 規約チェック
k1s0 lint
```

## ドキュメント

- [ADR](docs/adr/README.md): アーキテクチャ決定記録
- [規約](docs/conventions/README.md): 開発規約
- [構想](work/構想.md): 全体方針
- [実装プラン](work/プラン.md): 実装計画

## 技術スタック

- **バックエンド**: Rust (axum + tokio), Go
- **フロントエンド**: React, Flutter
- **CLI**: Rust (clap)
- **データベース**: PostgreSQL
- **キャッシュ**: Redis
- **観測性**: OpenTelemetry
- **API**: gRPC (内部), REST/OpenAPI (外部)
