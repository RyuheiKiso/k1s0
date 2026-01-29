# サービス構成規約

本ドキュメントは、k1s0 における個別サービス（feature）および framework 共通サービスの構成規約を定義する。

## 1. サービスの粒度

- 機能単位 = 1 マイクロサービス
- 1 ディレクトリ = 1 サービス
- サービス間の共通化は `feature/` 配下で行わない（共通にしたいものは `framework/` へ移す）

## 2. 命名規則

| 対象 | 規則 | 例 |
|------|------|-----|
| `{feature_name}` | kebab-case | `user-management`, `order-processing` |
| `{service_name}` | `{feature_name}` と同一 | `user-management` |
| framework 共通サービス | kebab-case + `-service` | `auth-service`, `config-service` |
| `{env}` | 固定 4 値 | `default`, `dev`, `stg`, `prod` |

## 3. サービスの配置先

### 3.1 個別機能サービス（feature）

```
feature/
├── backend/
│   ├── rust/{feature_name}/
│   ├── go/{feature_name}/
│   ├── csharp/{feature_name}/
│   └── python/{feature_name}/
└── frontend/
    ├── react/{feature_name}/
    └── flutter/{feature_name}/
```

### 3.2 framework 共通サービス

```
framework/backend/{lang}/services/{service_name}/
```

例：
- `framework/backend/rust/services/auth-service/`
- `framework/backend/rust/services/config-service/`
- `framework/backend/rust/services/endpoint-service/`

## 4. 必須ファイル/ディレクトリ

### 4.1 共通必須（全サービス）

| ファイル/ディレクトリ | 説明 |
|-----------------------|------|
| `README.md` | サービスの概要・責務・API・依存等（見出し規約あり） |

### 4.2 バックエンド（Rust）

| ファイル/ディレクトリ | 説明 |
|-----------------------|------|
| `Cargo.toml` | Rust パッケージ定義 |
| `config/` | 設定ファイル置き場 |
| `config/{default,dev,stg,prod}.yaml` | 環境別設定 |
| `deploy/` | Kubernetes マニフェスト |
| `deploy/base/` | 共通マニフェスト |
| `deploy/overlays/{dev,stg,prod}/` | 環境別オーバーレイ |
| `src/` | ソースコード |
| `src/application/` | ユースケース・アプリケーションサービス |
| `src/domain/` | エンティティ・値オブジェクト・リポジトリ traits |
| `src/infrastructure/` | リポジトリ実装・外部 I/O |
| `src/presentation/` | HTTP/gRPC ハンドラ・ルーティング |

### 4.3 バックエンド（C#）

| ファイル/ディレクトリ | 説明 |
|-----------------------|------|
| `{FeatureName}.sln` | ソリューションファイル |
| `Directory.Build.props` | 共通ビルド設定（net8.0, Nullable, TreatWarningsAsErrors） |
| `Directory.Packages.props` | Central Package Management |
| `config/` | 設定ファイル置き場 |
| `config/{default,dev,stg,prod}.yaml` | 環境別設定 |
| `deploy/` | Kubernetes マニフェスト |
| `src/{FeatureName}.Domain/` | エンティティ・値オブジェクト・リポジトリインターフェース |
| `src/{FeatureName}.Application/` | ユースケース・アプリケーションサービス・DTO |
| `src/{FeatureName}.Infrastructure/` | リポジトリ実装・外部 I/O・EF Core |
| `src/{FeatureName}.Presentation/` | ASP.NET Core ホスト・Controllers・gRPC サービス |
| `tests/{FeatureName}.Domain.Tests/` | ドメイン層ユニットテスト |
| `tests/{FeatureName}.Application.Tests/` | アプリケーション層ユニットテスト |
| `tests/{FeatureName}.Integration.Tests/` | 統合テスト |

### 4.4 バックエンド（Python）

| ファイル/ディレクトリ | 説明 |
|-----------------------|------|
| `pyproject.toml` | Python パッケージ定義（uv 管理） |
| `config/` | 設定ファイル置き場 |
| `config/{default,dev,stg,prod}.yaml` | 環境別設定 |
| `deploy/` | Kubernetes マニフェスト |
| `src/{feature_name_snake}/` | ソースコード（Clean Architecture 構成） |
| `src/{feature_name_snake}/domain/` | エンティティ・値オブジェクト・リポジトリ抽象クラス |
| `src/{feature_name_snake}/application/` | ユースケース・アプリケーションサービス |
| `src/{feature_name_snake}/infrastructure/` | リポジトリ実装・外部 I/O |
| `src/{feature_name_snake}/presentation/` | FastAPI ルーター・gRPC サービス |
| `tests/` | テストコード（pytest） |

### 4.5 バックエンド（Go）

| ファイル/ディレクトリ | 説明 |
|-----------------------|------|
| `go.mod` | Go モジュール定義 |
| `config/` | 設定ファイル置き場 |
| `config/{default,dev,stg,prod}.yaml` | 環境別設定 |
| `deploy/` | Kubernetes マニフェスト |
| `src/` | ソースコード（Clean Architecture 構成） |

### 4.6 フロントエンド（React）

| ファイル/ディレクトリ | 説明 |
|-----------------------|------|
| `package.json` | npm/pnpm パッケージ定義 |
| `config/` | 設定ファイル置き場 |
| `config/{default,dev,stg,prod}.yaml` | 環境別設定 |
| `src/` | ソースコード |
| `src/presentation/` | pages/components |
| `src/application/` | usecases/state |
| `src/domain/` | entities/value_objects |
| `src/infrastructure/` | api client/repository 実装 |

### 4.7 フロントエンド（Flutter）

| ファイル/ディレクトリ | 説明 |
|-----------------------|------|
| `pubspec.yaml` | Dart パッケージ定義 |
| `config/` | 設定ファイル置き場 |
| `config/{default,dev,stg,prod}.yaml` | 環境別設定 |
| `lib/` | ソースコード |
| `lib/src/presentation/` | pages/widgets |
| `lib/src/application/` | usecases/state |
| `lib/src/domain/` | entities/value_objects |
| `lib/src/infrastructure/` | api client/repository 実装 |

## 5. 条件付き必須

| 条件 | 必須ファイル/ディレクトリ |
|------|---------------------------|
| Kubernetes にデプロイ | `deploy/base/`, `deploy/overlays/{dev,stg,prod}/` |
| REST API を提供 | `openapi/openapi.yaml` |
| gRPC API を提供 | `proto/`, `buf.yaml`, `buf.lock` |
| DB を使用 | `migrations/` |

## 6. README.md 見出し規約

サービスの `README.md` は以下の見出しをこの順で持つ：

```markdown
# {service_name}

## 概要
## 責務
## 公開API
## 依存
## 設定
## DB
## 認証・認可（必要な場合）
## 監視
## 起動方法
## リリース
```

公開 API がないサービスは「## 公開API」に「なし」と明記する。

## 7. Clean Architecture 依存方向

```
presentation → application → domain
                    ↑
              infrastructure
```

- `domain` は外部フレームワーク/DB/HTTP に依存しない
- `domain` から `infrastructure` を直接 import しない
- `infrastructure` は `domain` の traits（ports）を実装する

## 8. 禁止事項

- `feature/` 配下に `common/` を作るなど、共通化を各チーム裁量にすること
- `domain` から `infrastructure` を直接 import すること
- 設定の読み替えや上書きに環境変数を利用すること
- ConfigMap（`config/{env}.yaml`）への機密値直書き（`*_file` 参照のみ許可）

## 関連ドキュメント

- [構想.md](../../work/構想.md): 全体方針
- [ADR-0001](../adr/ADR-0001-scope-and-prerequisites.md): 実装スコープと前提
