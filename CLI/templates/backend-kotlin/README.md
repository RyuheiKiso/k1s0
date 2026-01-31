# Backend Kotlin Template

Kotlin (Ktor) バックエンドサービスのテンプレート。

## ディレクトリ構成

```
backend-kotlin/
├── feature/      # 機能雛形（k1s0 new-feature 用）
└── domain/       # ドメインライブラリ雛形（k1s0 new-domain 用）
```

## feature/ の生成物

`k1s0 new-feature --type backend-kotlin --name {name}` で生成される構成：

```
feature/backend/kotlin/{name}/
├── build.gradle.kts
├── settings.gradle.kts
├── config/
│   ├── default.yaml
│   ├── dev.yaml
│   ├── stg.yaml
│   └── prod.yaml
├── deploy/
│   ├── base/
│   └── overlays/
├── src/main/kotlin/{package}/
│   ├── domain/              # ビジネスロジック層
│   │   ├── entities/
│   │   ├── valueobjects/
│   │   ├── repositories/
│   │   └── services/
│   ├── application/         # アプリケーション層
│   │   ├── usecases/
│   │   ├── services/
│   │   └── dtos/
│   ├── infrastructure/      # インフラストラクチャ層
│   │   ├── repositories/
│   │   ├── external/
│   │   └── persistence/
│   └── presentation/        # プレゼンテーション層
│       ├── grpc/
│       ├── rest/
│       └── middleware/
├── Dockerfile
├── .dockerignore
└── docker-compose.yml
```

## 技術スタック

- フレームワーク: Ktor 3.x
- DI: Koin
- gRPC: grpc-kotlin
- ORM: Exposed + HikariCP
- キャッシュ: Lettuce (Redis)
- ビルド: Gradle (Kotlin DSL)
- 設定: k1s0-config (YAML)
- ログ/トレース: OpenTelemetry
