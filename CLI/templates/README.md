# k1s0 Templates

`k1s0 new-feature` および `k1s0 new-screen` で生成されるサービス・画面雛形のテンプレート群。

## ディレクトリ構成

```
templates/
├── backend-rust/
│   ├── project/      # リポジトリ初期化テンプレ（共通設定/CI 等）
│   ├── feature/      # 機能（= 1 マイクロサービス）雛形
│   └── domain/       # ドメインライブラリ雛形
├── backend-go/
│   ├── feature/
│   └── domain/
├── backend-csharp/
│   ├── feature/
│   └── domain/
├── backend-python/
│   ├── feature/
│   └── domain/
├── backend-kotlin/
│   ├── feature/
│   └── domain/
├── frontend-react/
│   ├── feature/
│   ├── domain/
│   └── screen/       # React 画面テンプレート
├── frontend-flutter/
│   ├── feature/
│   ├── domain/
│   └── screen/       # Flutter 画面テンプレート
└── frontend-android/
    ├── feature/
    └── domain/
```

## テンプレートの種類

### Feature テンプレート（`k1s0 new-feature`）

| テンプレート | 生成先 | 説明 |
|-------------|--------|------|
| `backend-rust/feature` | `feature/backend/rust/{name}/` | Rust バックエンドサービス |
| `backend-go/feature` | `feature/backend/go/{name}/` | Go バックエンドサービス |
| `backend-csharp/feature` | `feature/backend/csharp/{name}/` | C# バックエンドサービス |
| `backend-python/feature` | `feature/backend/python/{name}/` | Python バックエンドサービス |
| `backend-kotlin/feature` | `feature/backend/kotlin/{name}/` | Kotlin バックエンドサービス |
| `frontend-react/feature` | `feature/frontend/react/{name}/` | React フロントエンド |
| `frontend-flutter/feature` | `feature/frontend/flutter/{name}/` | Flutter フロントエンド |
| `frontend-android/feature` | `feature/frontend/android/{name}/` | Android フロントエンド |

### Screen テンプレート（`k1s0 new-screen`）

| テンプレート | 生成先 | 説明 |
|-------------|--------|------|
| `frontend-react/screen` | `src/pages/{ScreenId}Page.tsx` | React 画面コンポーネント |
| `frontend-flutter/screen` | `lib/src/presentation/pages/{screen_id}_page.dart` | Flutter 画面ウィジェット |

## テンプレート変数

### 共通変数

| 変数 | 説明 | 例 |
|------|------|-----|
| `{{feature_name}}` | 機能名（kebab-case） | `user-management` |
| `{{feature_name_snake}}` | 機能名（snake_case） | `user_management` |
| `{{feature_name_pascal}}` | 機能名（PascalCase） | `UserManagement` |
| `{{feature_name_kebab}}` | 機能名（kebab-case） | `user-management` |
| `{{feature_name_title}}` | 機能名（Title Case） | `User Management` |
| `{{service_name}}` | サービス名（= feature_name） | `user-management` |
| `{{k1s0_version}}` | k1s0 バージョン | `0.1.0` |
| `{{now}}` | 生成日時（UTC） | `2026-01-27T12:00:00Z` |

### オプション変数

| 変数 | 説明 | デフォルト |
|------|------|-----------|
| `{{with_grpc}}` | gRPC サポート | `false` |
| `{{with_rest}}` | REST API サポート | `false` |
| `{{with_db}}` | データベースサポート | `false` |

## 使用方法

```bash
# Rust バックエンドサービスを生成
k1s0 new-feature -t backend-rust -n user-management

# Go バックエンドサービスを生成
k1s0 new-feature -t backend-go -n payment-service

# React 画面を生成
k1s0 new-screen -t react -s users.list -T "ユーザー一覧"

# Flutter 画面を生成
k1s0 new-screen -t flutter -s settings.profile -T "プロフィール設定"
```
