# ADR-0002: バージョニングと manifest の型の固定

## ステータス

承認済み（Accepted）

## 日付

2026-01-25

## コンテキスト

k1s0 CLI の `init` / `new-feature` / `upgrade` コマンドを実装するにあたり、以下の情報を永続化・参照する仕組みが必要である：

1. **k1s0 自体のバージョン**: CLI / generator / templates / framework crates を単一バージョンで管理し、整合する組み合わせを常に 1 つにする
2. **生成元テンプレートの情報**: どのテンプレートから生成されたかを記録し、upgrade 時の差分計算に使用する
3. **管理領域の境界**: CLI が自動更新してよいパスと、ビジネスロジックとして保護するパスを明確に分離する

## 決定

### 1. リポジトリ単一バージョン

k1s0 リポジトリのルートに `k1s0-version.txt` を配置し、単一のバージョン文字列（SemVer）を管理する。

```
k1s0-version.txt
```

**内容例:**
```
0.1.0
```

**対象:**
- CLI（`k1s0` バイナリ）
- k1s0-generator crate
- templates（backend-rust, frontend-react 等）
- framework crates（k1s0-config, k1s0-error 等）
- framework services（auth-service, config-service 等）

**理由:**
- 導入・アップグレード時に整合する組み合わせを常に 1 つにする
- 複数バージョンの管理による複雑性を回避する

### 2. manifest.json のスキーマ

生成されたサービスごとに `.k1s0/manifest.json` を配置し、以下の情報を保存する。

**配置先:**
```
feature/backend/rust/{service_name}/.k1s0/manifest.json
```

**スキーマ:**

| キー | 必須 | 説明 |
|------|------|------|
| `schema_version` | Yes | manifest スキーマのバージョン |
| `k1s0_version` | Yes | 生成時の k1s0 バージョン |
| `template` | Yes | 生成元テンプレートの情報 |
| `template.name` | Yes | テンプレート名（例: `backend-rust`） |
| `template.version` | Yes | テンプレートバージョン |
| `template.path` | Yes | テンプレートのパス |
| `template.fingerprint` | Yes | テンプレートの fingerprint（SHA-256） |
| `template.source` | No | ソース（`local` / `registry`） |
| `template.revision` | No | Git コミットハッシュ |
| `service` | Yes | サービスの情報 |
| `service.service_name` | Yes | サービス名（kebab-case） |
| `service.language` | Yes | 言語（`rust` / `go` / `typescript` / `dart`） |
| `service.type` | Yes | タイプ（`backend` / `frontend` / `bff`） |
| `generated_at` | Yes | 生成日時（ISO 8601） |
| `managed_paths` | Yes | CLI が管理するパス |
| `protected_paths` | Yes | CLI が変更しないパス |
| `update_policy` | No | パス別の更新ポリシー |
| `checksums` | No | ファイルのチェックサム（変更検知用） |
| `template_snapshot` | No | 生成時のテンプレートファイル一覧 |
| `dependencies` | No | framework crate への依存情報 |

**JSON Schema:**
- `CLI/schemas/manifest.schema.json` に定義
- CLI はこのスキーマでバリデーションを行う

### 3. managed / protected の分類

| 分類 | 説明 | upgrade 時の挙動 |
|------|------|-----------------|
| `managed_paths` | CLI が管理する領域 | 自動更新される |
| `protected_paths` | ビジネスロジック領域 | 変更しない |
| `update_policy` | 境界領域の個別設定 | `suggest_only` は差分提示のみ |

**デフォルト分類:**

managed_paths（自動更新対象）:
- `deploy/`（Kustomize）
- `config/`（環境別 yaml）
- `openapi/openapi.yaml`（雛形部分）
- `buf.yaml` / `buf.lock`
- `.github/`（CI 設定）

protected_paths（変更しない）:
- `src/domain/`
- `src/application/`

update_policy（境界領域）:
- `src/main.rs`: `suggest_only`
- `src/presentation/`: `suggest_only`
- `src/infrastructure/`: `suggest_only`
- `migrations/`: `suggest_only`

### 4. fingerprint の算出

テンプレートの fingerprint は、テンプレートディレクトリ内の全ファイルの内容から SHA-256 ハッシュを算出する。

**算出対象:**
- テンプレートディレクトリ内の全ファイル
- ファイルパス（相対パス）とファイル内容を結合してハッシュ化

**除外対象:**
- `.git/`
- `target/`（ビルド成果物）
- `node_modules/`

## 帰結

### 正の帰結

- CLI が「テンプレの出自」を正確に把握でき、upgrade 時の差分計算が可能になる
- managed / protected の境界が明確になり、ビジネスロジックの誤上書きを防止できる
- JSON Schema によりバリデーションが可能になり、手動編集による不整合を検知できる

### 負の帰結

- manifest.json のスキーマ変更時に互換性維持が必要になる
- fingerprint 算出のロジック変更で既存 manifest との不整合が生じる可能性がある

### リスクと軽減策

| リスク | 軽減策 |
|--------|--------|
| スキーマ変更による互換性問題 | `schema_version` で管理し、マイグレーションロジックを用意 |
| 手動編集による不整合 | 「手動編集は非推奨」をドキュメントに明記、lint で警告 |
| fingerprint 算出ロジックの変更 | 算出ロジックをバージョン管理し、過去ロジックも保持 |

## 関連ドキュメント

- [ADR-0001](ADR-0001-scope-and-prerequisites.md): 実装スコープと前提
- [CLI/schemas/manifest.schema.json](../../CLI/schemas/manifest.schema.json): JSON Schema 定義
- [CLI/schemas/manifest.example.json](../../CLI/schemas/manifest.example.json): サンプル
- [構想.md](../../work/構想.md): 8. framework のアップグレード支援
