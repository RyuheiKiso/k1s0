# Backend Python Template

Python (FastAPI) バックエンドサービスのテンプレート。

## ディレクトリ構成

```
backend-python/
├── feature/      # 機能雛形（k1s0 new-feature 用）
└── domain/       # ドメインライブラリ雛形（k1s0 new-domain 用）
```

## feature/ の生成物

`k1s0 new-feature --type backend-python --name {name}` で生成される構成：

```
feature/backend/python/{name}/
├── pyproject.toml
├── config/
│   ├── default.yaml
│   ├── dev.yaml
│   ├── stg.yaml
│   └── prod.yaml
├── deploy/
│   ├── base/
│   └── overlays/
├── src/{feature_name_snake}/
│   ├── domain/              # ビジネスロジック層
│   │   ├── entities/
│   │   ├── value_objects/
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

- フレームワーク: FastAPI
- gRPC: grpcio
- ORM: SQLAlchemy + asyncpg
- バリデーション: Pydantic
- パッケージマネージャ: uv
- 設定: k1s0-config (YAML)
- ログ/トレース: OpenTelemetry
