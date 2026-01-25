# Feature

個別機能チームのサービス実装領域。

## ディレクトリ構成

```
feature/
├── backend/
│   ├── rust/           # Rust バックエンドサービス
│   │   └── {feature_name}/
│   └── go/             # Go バックエンドサービス
│       └── {feature_name}/
├── frontend/
│   ├── react/          # React フロントエンド
│   │   └── {feature_name}/
│   └── flutter/        # Flutter フロントエンド
│       └── {feature_name}/
└── database/
    ├── schema/         # feature 固有スキーマ方針
    └── table/          # feature 固有テーブル定義
```

## サービスの生成

```bash
# Rust バックエンドサービスを生成
k1s0 new-feature --type backend-rust --name user-management

# 生成先: feature/backend/rust/user-management/
```

## 命名規則

- `{feature_name}`: kebab-case
- `{service_name}`: `{feature_name}` と同一

## 構成規約

各サービスは [サービス構成規約](../docs/conventions/service-structure.md) に従う。

## 禁止事項

- `feature/` 配下に `common/` を作るなど、共通化を各チーム裁量にすること
  - 共通にしたいものは `framework/` へ移す
- `domain` から `infrastructure` を直接 import すること
- 環境変数での設定注入
