# domain/ - ドメイン層

このディレクトリには、特定の業務領域（ドメイン）で共通して使用されるビジネスロジックを配置します。

## 概要

k1s0 は以下の3層構造でコードを管理します：

```
┌─────────────────────────────────────────────────────────┐
│  feature/  (Tier 3)                                     │
│  - 個別の機能サービス                                    │
│  - 特定のユースケースを実装                              │
└─────────────────────────────────────────────────────────┘
                           ↓ 依存可能
┌─────────────────────────────────────────────────────────┐
│  domain/  (Tier 2)  ← このディレクトリ                   │
│  - 業務領域の共通ロジック                                │
│  - 複数の feature で共有されるビジネスルール             │
└─────────────────────────────────────────────────────────┘
                           ↓ 依存可能
┌─────────────────────────────────────────────────────────┐
│  framework/  (Tier 1)                                   │
│  - 技術基盤（認証、設定、DB、キャッシュ等）              │
│  - ビジネスロジックを含まない                            │
└─────────────────────────────────────────────────────────┘
```

## domain層の役割

### 何を配置するか

- **業務領域の共通エンティティ**: 複数の feature で使用されるドメインモデル
- **共通のビジネスルール**: 業務領域固有のバリデーション、計算ロジック
- **ドメインサービス**: 複数のエンティティにまたがる業務ロジック
- **値オブジェクト**: 業務領域で共通して使用される値の型定義

### 具体例

| ドメイン名 | 配置するもの |
|-----------|-------------|
| finance | 請求書エンティティ、金額計算ロジック、税率ルール |
| inventory | 在庫エンティティ、在庫引当ロジック、ロット管理ルール |
| customer | 顧客エンティティ、顧客ランク判定ロジック |
| shipping | 配送エンティティ、配送料金計算、配送可能日判定 |

### 何を配置しないか

- **技術的な共通処理**: framework/ に配置
- **特定機能固有のロジック**: feature/ に配置
- **インフラストラクチャ層のコード**: リポジトリ実装、外部API連携等

## 依存関係ルール

### 許可される依存

| 層 | 依存可能な対象 |
|----|---------------|
| **framework (Tier 1)** | 外部ライブラリのみ |
| **domain (Tier 2)** | framework のみ |
| **feature (Tier 3)** | framework + 自身が属する domain のみ |

### 禁止される依存

- domain から他の domain への依存
- domain から feature への依存
- feature から他の feature への依存
- feature から所属しない domain への依存

```
[禁止される依存の例]

domain/finance/ ──X──> domain/inventory/    # 他のdomainへの依存は禁止
domain/finance/ ──X──> feature/             # featureへの依存は禁止

feature/invoice-service/ ──X──> feature/payment-service/  # 他のfeatureへの依存は禁止
feature/invoice-service/ ──X──> domain/inventory/         # 所属しないdomainへの依存は禁止
```

### 依存関係の正しい例

```
feature/invoice-processing/
    └── depends on ──> domain/finance/        # 所属するdomainに依存 (OK)
    └── depends on ──> framework/k1s0-error/  # frameworkに依存 (OK)

domain/finance/
    └── depends on ──> framework/k1s0-validation/  # frameworkに依存 (OK)
```

## ディレクトリ構造

```
domain/
├── README.md           # このファイル
├── backend/
│   ├── rust/           # Rust製のdomainパッケージ
│   │   ├── finance/    # 例: 財務ドメイン
│   │   └── inventory/  # 例: 在庫ドメイン
│   ├── go/             # Go製のdomainパッケージ
│   │   ├── finance/
│   │   └── inventory/
│   ├── csharp/         # C#製のdomainパッケージ
│   │   ├── finance/
│   │   └── inventory/
│   ├── python/         # Python製のdomainパッケージ
│   │   ├── finance/
│   │   └── inventory/
│   └── kotlin/         # Kotlin製のdomainパッケージ
│       ├── finance/
│       └── inventory/
└── frontend/
    ├── react/          # React用のdomainパッケージ
    │   └── finance/
    ├── flutter/        # Flutter用のdomainパッケージ
    │   └── finance/
    └── android/        # Android用のdomainパッケージ
        └── finance/
```

### 各domainパッケージの内部構造

#### Rust (backend-rust)

```
domain/backend/rust/{domain_name}/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── entities/       # ドメインエンティティ
│   ├── value_objects/  # 値オブジェクト
│   ├── services/       # ドメインサービス
│   └── rules/          # ビジネスルール
└── .k1s0/
    └── manifest.json   # layer: "domain"
```

#### Go (backend-go)

```
domain/backend/go/{domain_name}/
├── go.mod
├── entities/
├── value_objects/
├── services/
├── rules/
└── .k1s0/
    └── manifest.json   # layer: "domain"
```

#### React (frontend-react)

```
domain/frontend/react/{domain_name}/
├── package.json
├── src/
│   ├── index.ts
│   ├── entities/
│   ├── value-objects/
│   └── rules/
└── .k1s0/
    └── manifest.json   # layer: "domain"
```

#### C# (backend-csharp)

```
domain/backend/csharp/{domain_name}/
├── {DomainName}.sln
├── src/
│   └── {DomainName}.Domain/
│       ├── {DomainName}.Domain.csproj
│       ├── Entities/
│       ├── ValueObjects/
│       ├── Repositories/
│       ├── Services/
│       └── Rules/
└── .k1s0/
    └── manifest.json   # layer: "domain"
```

#### Python (backend-python)

```
domain/backend/python/{domain_name}/
├── pyproject.toml
├── src/
│   └── {domain_name_snake}/
│       ├── __init__.py
│       ├── entities/
│       ├── value_objects/
│       ├── repositories/
│       ├── services/
│       └── rules/
└── .k1s0/
    └── manifest.json   # layer: "domain"
```

#### Kotlin (backend-kotlin)

```
domain/backend/kotlin/{domain_name}/
├── build.gradle.kts
├── src/
│   └── main/kotlin/{package}/
│       ├── entities/
│       ├── valueobjects/
│       ├── repositories/
│       ├── services/
│       └── rules/
└── .k1s0/
    └── manifest.json   # layer: "domain"
```

#### Flutter (frontend-flutter)

```
domain/frontend/flutter/{domain_name}/
├── pubspec.yaml
├── lib/
│   ├── {domain_name}.dart
│   └── src/
│       ├── entities/
│       ├── value_objects/
│       └── rules/
└── .k1s0/
    └── manifest.json   # layer: "domain"
```

#### Android (frontend-android)

```
domain/frontend/android/{domain_name}/
├── build.gradle.kts
├── src/
│   └── main/kotlin/{package}/
│       ├── entities/
│       ├── valueobjects/
│       ├── repositories/
│       └── rules/
└── .k1s0/
    └── manifest.json   # layer: "domain"
```

## コマンド

### 新しいdomainを作成

```bash
# Rust backend domain
k1s0 new-domain --type backend-rust --name finance

# Go backend domain
k1s0 new-domain --type backend-go --name inventory

# React frontend domain
k1s0 new-domain --type frontend-react --name finance

# C# backend domain
k1s0 new-domain --type backend-csharp --name finance

# Python backend domain
k1s0 new-domain --type backend-python --name finance

# Kotlin backend domain
k1s0 new-domain --type backend-kotlin --name finance

# Flutter frontend domain
k1s0 new-domain --type frontend-flutter --name finance

# Android frontend domain
k1s0 new-domain --type frontend-android --name finance
```

### featureをdomainに所属させて作成

```bash
# financeドメインに所属するfeatureを作成
k1s0 new-feature --type backend-rust --name invoice-processing --domain finance
```

### lint検証

```bash
# 依存関係ルールの検証を含むlint
k1s0 lint

# domain依存関係のみを検証
k1s0 lint --rules K040,K041,K042,K043
```

## manifest.json の設定

### domainパッケージの場合

```json
{
  "schema_version": "1.0.0",
  "k1s0_version": "0.1.1",
  "template": {
    "name": "backend-rust",
    "version": "0.1.1",
    "path": "CLI/templates/backend-rust/domain",
    "fingerprint": "..."
  },
  "service": {
    "service_name": "finance",
    "language": "rust",
    "type": "backend"
  },
  "layer": "domain",
  "domain": null,
  "generated_at": "2026-01-28T10:00:00Z",
  "managed_paths": ["deploy/", "config/"],
  "protected_paths": ["src/"]
}
```

### featureパッケージの場合

```json
{
  "schema_version": "1.0.0",
  "k1s0_version": "0.1.1",
  "template": {
    "name": "backend-rust",
    "version": "0.1.1",
    "path": "CLI/templates/backend-rust/feature",
    "fingerprint": "..."
  },
  "service": {
    "service_name": "invoice-processing",
    "language": "rust",
    "type": "backend"
  },
  "layer": "feature",
  "domain": "finance",
  "generated_at": "2026-01-28T10:00:00Z",
  "managed_paths": ["deploy/", "config/"],
  "protected_paths": ["src/domain/", "src/application/"]
}
```

## ベストプラクティス

### 1. domainの粒度

- 組織の業務部門に対応する粒度が適切
- 細かすぎると管理が煩雑になる
- 大きすぎると再利用性が低下する

### 2. feature間の連携

feature間で直接依存することは禁止されています。連携が必要な場合は以下の方法を使用してください：

- **gRPC/REST API**: サービス間通信
- **イベント駆動**: メッセージキューを介した非同期連携
- **共通domain**: 両方のfeatureが同じdomainに所属

### 3. domainの分割基準

以下の場合にdomainを分割することを検討してください：

- 異なるチームが担当する業務領域
- 独立してデプロイ・スケーリングしたい領域
- ビジネスルールが大きく異なる領域

## 関連ドキュメント

- [アーキテクチャ概要](../docs/architecture/overview.md)
- [Clean Architecture](../docs/architecture/clean-architecture.md)
- [サービス構造](../docs/conventions/service-structure.md)
- [API契約](../docs/conventions/api-contracts.md)
