# backend-kotlin テンプレート

← [テンプレート設計書](./)

---

## ディレクトリ構造

```
feature/backend/kotlin/{service_name}/
├── .k1s0/
│   └── manifest.json.tera
├── build.gradle.kts.tera
├── settings.gradle.kts.tera
├── gradle.properties
├── README.md.tera
├── Dockerfile.tera
├── .dockerignore
├── docker-compose.yml.tera
├── config/
│   ├── default.yaml.tera
│   ├── dev.yaml.tera
│   ├── stg.yaml.tera
│   └── prod.yaml.tera
├── deploy/
│   └── base/
│       ├── configmap.yaml.tera
│       ├── deployment.yaml.tera
│       ├── service.yaml.tera
│       └── kustomization.yaml.tera
├── proto/
│   └── service.proto.tera
├── openapi/
│   └── openapi.yaml.tera
├── buf.yaml
├── buf.gen.yaml.tera
├── src/
│   └── main/
│       └── kotlin/
│           └── {{ package_path }}/
│               ├── Application.kt.tera
│               ├── domain/
│               │   ├── entities/
│               │   ├── valueobjects/
│               │   ├── repositories/
│               │   └── services/
│               ├── application/
│               │   ├── usecases/
│               │   ├── services/
│               │   └── dtos/
│               ├── infrastructure/
│               │   ├── repositories/
│               │   ├── external/
│               │   └── persistence/
│               └── presentation/
│                   ├── grpc/
│                   ├── rest/
│                   └── middleware/
└── src/
    └── test/
        └── kotlin/
            └── {{ package_path }}/
                └── ApplicationTest.kt
```

## 特徴

- **Ktor 3.x** ベース
- **Gradle Kotlin DSL** によるビルド管理
- **Exposed** で DB アクセス（HikariCP 接続プール）
- **Koin** で依存性注入
- **grpc-kotlin** で gRPC サポート
- **ktlint** でフォーマット
- **detekt** で静的解析
- **JUnit 5 + kotest** でテスト
- **条件付きレンダリング**: `{% if with_grpc %}` で gRPC、`{% if with_db %}` で Exposed 依存を追加
- **Multi-stage Docker build**: Eclipse Temurin 21 ベースイメージ
