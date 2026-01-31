# 基本コマンド（init, new-feature, new-screen）

← [CLI 設計書](./)

---

## init コマンド

### 目的

リポジトリを初期化し、`.k1s0/` ディレクトリと `config.json` を作成する。

### 引数

```rust
pub struct InitArgs {
    /// 初期化するディレクトリ（デフォルト: カレントディレクトリ）
    #[arg(default_value = ".")]
    pub path: String,

    /// 既存の .k1s0/ を上書きする
    #[arg(short, long)]
    pub force: bool,

    /// テンプレートソース（local または registry URL）
    #[arg(long, default_value = "local")]
    pub template_source: String,
}
```

### 処理フロー

```
1. パスの正規化
2. 既存の .k1s0/ 確認
   └─ 存在する場合
      ├─ --force: 削除して続行
      └─ なし: エラー
3. .k1s0/ ディレクトリ作成
4. config.json 作成
5. 完了メッセージ表示
```

### 生成ファイル

`.k1s0/config.json`:

```json
{
  "schema_version": "1.0.0",
  "k1s0_version": "0.1.0",
  "template_source": "local",
  "initialized_at": "2026-01-27T10:00:00Z",
  "project": {
    "default_language": "rust",
    "default_service_type": "backend"
  }
}
```

---

## new-feature コマンド

### 目的

新規サービスの雛形を Tera テンプレートから生成する。

### 引数

```rust
pub struct NewFeatureArgs {
    /// サービスタイプ（対話モード時は省略可能）
    #[arg(short = 't', long = "type", value_enum)]
    pub service_type: Option<ServiceType>,

    /// サービス名（kebab-case）（対話モード時は省略可能）
    #[arg(short, long)]
    pub name: Option<String>,

    /// 所属する domain 名（省略時は domain に属さない独立した feature として作成）
    #[arg(long)]
    pub domain: Option<String>,

    /// 生成先ディレクトリ
    #[arg(short, long)]
    pub output: Option<String>,

    /// 既存のディレクトリを上書きする
    #[arg(short, long)]
    pub force: bool,

    /// gRPC API を含める
    #[arg(long)]
    pub with_grpc: bool,

    /// REST API を含める
    #[arg(long)]
    pub with_rest: bool,

    /// DB マイグレーションを含める
    #[arg(long)]
    pub with_db: bool,

    /// 対話モードを強制する
    #[arg(short = 'i', long)]
    pub interactive: bool,
}
```

### サービスタイプ

| タイプ | テンプレートパス | 出力先 | 言語 |
|--------|----------------|-------|------|
| `backend-rust` | `CLI/templates/backend-rust/feature` | `feature/backend/rust/{name}` | rust |
| `backend-go` | `CLI/templates/backend-go/feature` | `feature/backend/go/{name}` | go |
| `backend-csharp` | `CLI/templates/backend-csharp/feature` | `feature/backend/csharp/{name}` | csharp |
| `backend-python` | `CLI/templates/backend-python/feature` | `feature/backend/python/{name}` | python |
| `frontend-react` | `CLI/templates/frontend-react/feature` | `feature/frontend/react/{name}` | typescript |
| `frontend-flutter` | `CLI/templates/frontend-flutter/feature` | `feature/frontend/flutter/{name}` | dart |

### 処理フロー

```
1. サービス名のバリデーション（kebab-case）
2. 出力パスの決定
3. 既存衝突検査
   └─ 存在する場合
      ├─ --force: 削除して続行
      └─ なし: エラー
4. テンプレートディレクトリの検索
5. fingerprint の算出
6. Tera コンテキストの作成
7. テンプレートの展開
8. manifest.json の作成
9. 完了メッセージ表示
```

### テンプレート変数

| 変数名 | 説明 | 例 |
|--------|------|-----|
| `feature_name` | 機能名（kebab-case） | `user-management` |
| `service_name` | サービス名 | `user-management` |
| `feature_name_snake` | snake_case 変換 | `user_management` |
| `feature_name_pascal` | PascalCase 変換 | `UserManagement` |
| `language` | 言語 | `rust` |
| `service_type` | タイプ | `backend` |
| `k1s0_version` | k1s0 バージョン | `0.1.0` |
| `with_grpc` | gRPC 有効 | `true` |
| `with_rest` | REST 有効 | `false` |
| `with_db` | DB 有効 | `true` |

### サービス名のバリデーション

```rust
fn is_valid_kebab_case(s: &str) -> bool {
    // 1. 空でない
    // 2. 先頭は小文字
    // 3. 末尾はハイフンでない
    // 4. 連続するハイフンがない
    // 5. 許可される文字: 小文字、数字、ハイフン
}
```

有効な例: `user-management`, `order`, `auth-service`, `api2`
無効な例: `UserManagement`, `user_management`, `-user`, `user-`, `user--management`

---

## new-screen コマンド

### 目的

既存の feature ディレクトリに React/Flutter 画面の雛形を生成する。
画面追加の手順を「雛形生成 → 画面実装 → config 追記」に統一する。

### 引数

```rust
pub struct NewScreenArgs {
    /// フロントエンドタイプ
    #[arg(short = 't', long = "type", value_enum, default_value = "react")]
    pub frontend_type: FrontendType,

    /// 画面ID（ドット区切り、例: users.list, users.detail）
    #[arg(short, long)]
    pub screen_id: String,

    /// 画面タイトル
    #[arg(short = 'T', long)]
    pub title: String,

    /// 対象の feature ディレクトリ
    #[arg(short, long)]
    pub feature_dir: String,

    /// メニューに追加する（メニュー設定スニペットを出力）
    #[arg(long)]
    pub with_menu: bool,

    /// URL パス（指定しない場合は screen_id から自動生成）
    #[arg(short, long)]
    pub path: Option<String>,

    /// 必要な権限（カンマ区切り）
    #[arg(long)]
    pub permissions: Option<String>,

    /// 必要な feature flag（カンマ区切り）
    #[arg(long)]
    pub flags: Option<String>,

    /// 既存のファイルを上書きする
    #[arg(short = 'F', long)]
    pub force: bool,
}
```

### フロントエンドタイプ

| タイプ | テンプレートパス | 出力ファイル |
|--------|----------------|-------------|
| `react` | `CLI/templates/frontend-react/screen` | `src/pages/{ComponentName}.tsx` |
| `flutter` | `CLI/templates/frontend-flutter/screen` | `lib/src/presentation/pages/{snake_case}_page.dart` |

### 処理フロー

```
1. screen_id のバリデーション
2. feature_dir の存在確認
3. URL パスの生成（指定がない場合）
4. コンポーネント名・ファイル名の生成
5. テンプレートディレクトリの検索
6. 既存ファイルの確認
   └─ 存在する場合
      ├─ --force: 上書き
      └─ なし: エラー
7. Tera コンテキストの作成
8. テンプレートのレンダリング
9. ファイルの書き込み
10. 設定スニペットの出力
    ├─ React: screens.ts, route.yaml
    └─ Flutter: route.yaml
11. --with-menu: menu.yaml スニペット出力
12. 完了メッセージ表示
```

### テンプレート変数

| 変数名 | 説明 | 例 |
|--------|------|-----|
| `screen_id` | 画面ID | `users.list` |
| `title` | 画面タイトル | `ユーザー一覧` |
| `path` | URL パス | `/users/list` |
| `component_name` | コンポーネント名（PascalCase + Page） | `UsersListPage` |
| `file_name` | ファイル名 | `UsersListPage` (React), `users_list` (Flutter) |
| `permissions` | 必要な権限リスト | `["user.read"]` |
| `flags` | 必要なフラグリスト | `["feature_users"]` |
| `with_menu` | メニュー追加フラグ | `true` |

### 画面ID のバリデーション

```rust
fn is_valid_screen_id(s: &str) -> bool {
    // 1. 空でない
    // 2. 先頭・末尾がドットでない
    // 3. 連続するドットがない
    // 4. 許可される文字: 小文字、数字、ドット、アンダースコア
}
```

有効な例: `home`, `users.list`, `settings.profile.edit`
無効な例: `Users`, `.users`, `users.`, `users..list`

### 生成例

#### React

```bash
k1s0 new-screen -t react -s users.list -T "ユーザー一覧" -f ./my-feature --with-menu
```

生成されるファイル:
- `my-feature/src/pages/UsersListPage.tsx`

出力される設定スニペット:
- `src/config/screens.ts` への追加コード
- `config/default.yaml` の `ui.navigation.routes` への追加
- `config/default.yaml` の `ui.navigation.menu.items` への追加（--with-menu 時）

#### Flutter

```bash
k1s0 new-screen -t flutter -s settings.profile -T "プロフィール設定" -f ./my-feature
```

生成されるファイル:
- `my-feature/lib/src/presentation/pages/settings_profile_page.dart`

出力される設定スニペット:
- `config/default.yaml` の `ui.navigation.routes` への追加
