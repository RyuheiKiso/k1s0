# テンプレートシステム設計書

## 概要

k1s0 テンプレートシステムは、Tera テンプレートエンジンを使用して、サービスの雛形を生成します。複数の言語・フレームワークに対応し、Clean Architecture に基づいたディレクトリ構造を提供します。

## ドキュメント一覧

| ドキュメント | 説明 |
|-------------|------|
| [backend-rust](./backend-rust.md) | Rust バックエンドテンプレート |
| [backend-go](./backend-go.md) | Go バックエンドテンプレート |
| [backend-csharp](./backend-csharp.md) | C# バックエンドテンプレート |
| [backend-python](./backend-python.md) | Python バックエンドテンプレート |
| [backend-kotlin](./backend-kotlin.md) | Kotlin バックエンドテンプレート |
| [frontend-react](./frontend-react.md) | React フロントエンドテンプレート |
| [frontend-flutter](./frontend-flutter.md) | Flutter フロントエンドテンプレート |
| [frontend-android](./frontend-android.md) | Android フロントエンドテンプレート |
| [docker](./docker.md) | Docker テンプレートファイル |
| [guide](./guide.md) | テンプレート追加ガイド |
| [future](./future.md) | 今後の拡張予定 |

---

## テンプレート配置

```
CLI/templates/
├── backend-rust/
│   └── feature/          # Rust バックエンドテンプレート
├── backend-go/
│   └── feature/          # Go バックエンドテンプレート
├── backend-csharp/
│   ├── feature/          # C# バックエンドテンプレート
│   └── domain/           # C# ドメインテンプレート
├── backend-python/
│   ├── feature/          # Python バックエンドテンプレート
│   └── domain/           # Python ドメインテンプレート
├── backend-kotlin/
│   ├── feature/          # Kotlin バックエンドテンプレート
│   └── domain/           # Kotlin ドメインテンプレート
├── frontend-react/
│   └── feature/          # React フロントエンドテンプレート
├── frontend-flutter/
│   └── feature/          # Flutter フロントエンドテンプレート
└── frontend-android/
    └── feature/          # Android フロントエンドテンプレート
```

---

## テンプレート変数

### 基本変数

| 変数名 | 説明 | 例 |
|--------|------|-----|
| `feature_name` | 機能名（kebab-case） | `user-management` |
| `service_name` | サービス名 | `user-management` |
| `language` | 言語 | `rust`, `go`, `csharp`, `python`, `kotlin`, `typescript`, `dart` |
| `service_type` | タイプ | `backend`, `frontend` |
| `k1s0_version` | k1s0 バージョン | `0.1.0` |

### 命名規則変換

| 変数名 | 説明 | 例（入力: `user-management`） |
|--------|------|-----|
| `feature_name_snake` | snake_case | `user_management` |
| `feature_name_pascal` | PascalCase | `UserManagement` |

### オプション変数

| 変数名 | 説明 | デフォルト |
|--------|------|-----------|
| `with_grpc` | gRPC API を含める | `false` |
| `with_rest` | REST API を含める | `false` |
| `with_db` | DB マイグレーションを含める | `false` |

---

## テンプレートファイル規則

### 拡張子

- **`.tera`**: Tera テンプレートとして処理され、拡張子が除去されて出力される
- **その他**: そのままコピーされる

### 例

```
テンプレート:
  Cargo.toml.tera        → 出力: Cargo.toml
  src/main.rs.tera       → 出力: src/main.rs
  .gitignore             → 出力: .gitignore（そのままコピー）
```

---

## managed/protected パス

### managed パス（CLI が自動更新）

| テンプレート | managed パス |
|-------------|-------------|
| backend-rust | `deploy/`, `buf.yaml`, `buf.gen.yaml`, `Dockerfile`, `.dockerignore` |
| backend-go | `deploy/`, `buf.yaml`, `buf.gen.yaml`, `Dockerfile`, `.dockerignore` |
| backend-csharp | `deploy/`, `buf.yaml`, `buf.gen.yaml`, `*.csproj`, `Dockerfile`, `.dockerignore` |
| backend-python | `deploy/`, `buf.yaml`, `buf.gen.yaml`, `pyproject.toml`, `Dockerfile`, `.dockerignore` |
| backend-kotlin | `deploy/`, `buf.yaml`, `buf.gen.yaml`, `build.gradle.kts`, `Dockerfile`, `.dockerignore` |
| frontend-react | `deploy/`, `Dockerfile`, `.dockerignore`, `deploy/nginx.conf` |
| frontend-flutter | `deploy/` |
| frontend-android | `app/build.gradle.kts` |

### protected パス（CLI が変更しない）

| テンプレート | protected パス |
|-------------|---------------|
| backend-rust | `src/domain/`, `src/application/`, `README.md` |
| backend-go | `internal/domain/`, `internal/application/`, `README.md` |
| backend-csharp | `src/*.Domain/`, `src/*.Application/`, `README.md` |
| backend-python | `src/*/domain/`, `src/*/application/`, `README.md` |
| backend-kotlin | `src/main/kotlin/*/domain/`, `src/main/kotlin/*/application/`, `README.md` |
| frontend-react | `src/domain/`, `src/application/`, `src/presentation/`, `README.md` |
| frontend-flutter | `lib/src/domain/`, `lib/src/application/`, `lib/src/presentation/`, `README.md` |
| frontend-android | `app/src/main/kotlin/*/domain/`, `app/src/main/kotlin/*/application/`, `app/src/main/kotlin/*/presentation/`, `README.md` |

### update_policy

| ポリシー | 説明 |
|---------|------|
| `auto` | `k1s0 upgrade` で自動更新 |
| `suggest_only` | 差分を提示するが自動更新しない |
| `protected` | 一切変更しない |

デフォルトの割り当て:

```
deploy/              → auto
buf.yaml             → auto
src/domain/          → protected
src/application/     → protected
README.md            → suggest_only
config/              → suggest_only
```

---

## 条件付きレンダリング

### Tera 構文

```jinja2
{% if with_grpc %}
// gRPC 関連のコード
{% endif %}

{% if with_rest %}
// REST 関連のコード
{% endif %}

{% if with_db %}
// DB 関連のコード
{% endif %}
```

### 例：Cargo.toml

```toml
[dependencies]
{% if with_grpc %}
tonic = "0.12"
prost = "0.13"
{% endif %}

{% if with_rest %}
axum = "0.7"
{% endif %}

{% if with_db %}
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres"] }
{% endif %}
```

---

## fingerprint 計算

### 目的

テンプレートの変更を検出し、`k1s0 upgrade` で差分を適用する。

### 算出方法

1. テンプレートディレクトリを再帰的に走査
2. 除外パターンに一致するファイルをスキップ
3. ファイルを相対パスでソート
4. 各ファイルのパスと内容を SHA-256 でハッシュ化

### 除外パターン

```
.git, .svn, .hg           # バージョン管理
target, node_modules, ...  # ビルド成果物
.DS_Store, Thumbs.db      # OS メタデータ
.idea, .vscode            # IDE
.k1s0                     # k1s0 メタデータ
*.pyc, *.log, *.tmp, ...  # 一時ファイル
.env, .env.local          # 環境設定
```

---

## 変更履歴

- **2026-01**: テンプレートディレクトリ `frontend-android(Kotlin)` を `frontend-android` にリネーム。他テンプレート（`backend-kotlin` 等）と命名規則を統一し、括弧を含むパスによるビルドツールの互換性問題を回避するため。
