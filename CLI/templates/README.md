# k1s0 Templates

`k1s0 new-feature` で生成されるサービス雛形のテンプレート群。

## ディレクトリ構成

```
templates/
├── backend-rust/
│   ├── project/      # リポジトリ初期化テンプレ（共通設定/CI 等）
│   └── feature/      # 機能（= 1 マイクロサービス）雛形
├── backend-go/
│   └── feature/
├── frontend-react/
│   └── feature/
└── frontend-flutter/
    └── feature/
```

## テンプレートの種類

| テンプレート | 生成先 | 説明 |
|-------------|--------|------|
| `backend-rust/feature` | `feature/backend/rust/{name}/` | Rust バックエンドサービス |
| `backend-go/feature` | `feature/backend/go/{name}/` | Go バックエンドサービス |
| `frontend-react/feature` | `feature/frontend/react/{name}/` | React フロントエンド |
| `frontend-flutter/feature` | `feature/frontend/flutter/{name}/` | Flutter フロントエンド |

## テンプレート変数

| 変数 | 説明 | 例 |
|------|------|-----|
| `{{feature_name}}` | 機能名（kebab-case） | `user-management` |
| `{{service_name}}` | サービス名（= feature_name） | `user-management` |
| `{{k1s0_version}}` | k1s0 バージョン | `0.1.0` |

## 使用方法

```bash
# Rust バックエンドサービスを生成
k1s0 new-feature --type backend-rust --name user-management
```
