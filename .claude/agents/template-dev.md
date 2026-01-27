# テンプレート開発エージェント

k1s0 テンプレートの作成・編集を支援するエージェント。

## 対象領域

- `CLI/templates/` - 生成テンプレート
  - `backend-rust/` - Rust バックエンドテンプレート
  - `backend-go/` - Go バックエンドテンプレート
  - `frontend-react/` - React フロントエンドテンプレート
  - `frontend-flutter/` - Flutter フロントエンドテンプレート
- `CLI/schemas/` - JSON Schema 定義

## テンプレート構造

```
templates/<template-type>/
├── manifest.json           # テンプレートメタデータ
├── template/               # テンプレートファイル群
│   ├── src/
│   │   └── ...
│   ├── config/
│   │   └── ...
│   └── ...
└── README.md               # テンプレート説明
```

## manifest.json

```json
{
  "name": "backend-rust",
  "version": "0.1.0",
  "description": "Rust バックエンドサービステンプレート",
  "template_version": "1",
  "variables": {
    "service_name": {
      "type": "string",
      "description": "サービス名",
      "required": true
    }
  },
  "managed_paths": [
    "src/infrastructure/",
    "config/base.yaml"
  ],
  "protected_paths": [
    "src/domain/",
    "src/application/"
  ]
}
```

## テンプレート変数

Tera テンプレートエンジンを使用:

```
{{ service_name }}          # 変数展開
{{ service_name | upper }}  # フィルター適用
{% if condition %}...{% endif %}  # 条件分岐
{% for item in list %}...{% endfor %}  # ループ
```

## managed vs protected

### managed (自動更新対象)

- テンプレート更新時に自動的に上書き
- インフラ層、設定ファイルのベース部分
- 例: `src/infrastructure/`, `config/base.yaml`

### protected (カスタマイズ保護)

- テンプレート更新時も保護
- ビジネスロジック、ドメイン層
- 例: `src/domain/`, `src/application/`

## テンプレート作成手順

1. `templates/<type>/` ディレクトリ作成
2. `manifest.json` 作成
3. `template/` 内にテンプレートファイル配置
4. テスト: `k1s0 new-feature --type <type> --name test`
5. 生成結果を確認

## テンプレートのテスト

```bash
cd CLI

# ビルド
cargo build

# テスト生成（一時ディレクトリで）
./target/debug/k1s0 new-feature --type backend-rust --name test-service

# lint 検証
./target/debug/k1s0 lint
```

## fingerprint 管理

テンプレートから生成されたファイルの変更追跡:

- `.k1s0-gen.sha256` - ファイルの fingerprint
- 変更検出に使用
- `k1s0 upgrade` 時の差分計算に使用

詳細は `docs/adr/ADR-0003-template-fingerprint-strategy.md` を参照。

## JSON Schema

`CLI/schemas/` に各設定ファイルのスキーマを定義:

- manifest.json のバリデーション
- エディタの IntelliSense サポート

## 規約

### ディレクトリ構造

Clean Architecture に基づく:

```
src/
├── domain/           # ドメイン層（protected）
├── application/      # アプリケーション層（protected）
├── infrastructure/   # インフラ層（managed）
└── presentation/     # プレゼンテーション層
```

### 設定ファイル

```
config/
├── base.yaml         # 共通設定（managed）
├── development.yaml  # 開発環境
├── staging.yaml      # ステージング環境
└── production.yaml   # 本番環境
```

詳細は `docs/conventions/service-structure.md` を参照。
