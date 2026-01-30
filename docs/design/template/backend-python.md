# backend-python テンプレート

← [テンプレート設計書](./)

---

## ディレクトリ構造

```
feature/backend/python/{service_name}/
├── .k1s0/
│   └── manifest.json.tera
├── pyproject.toml.tera
├── README.md.tera
├── Dockerfile.tera
├── .dockerignore
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
│   └── {{ feature_name_snake }}/
│       ├── __init__.py
│       ├── main.py.tera
│       ├── domain/
│       │   ├── __init__.py
│       │   ├── entities/
│       │   └── errors/
│       ├── application/
│       │   ├── __init__.py
│       │   ├── services/
│       │   └── usecases/
│       ├── infrastructure/
│       │   ├── __init__.py
│       │   └── repositories/
│       └── presentation/
│           ├── __init__.py
│           ├── grpc/
│           └── rest/
└── tests/
    ├── conftest.py
    └── test_health.py
```

## 特徴

- **FastAPI 0.115+** ベース
- **uv** によるパッケージ管理（`pyproject.toml`）
- **Pydantic v2** でバリデーション・DTO
- **SQLAlchemy 2.0 + asyncpg** で非同期 DB アクセス
- **Ruff** でフォーマット・リント統合
- **mypy** で型チェック
- **pytest + pytest-asyncio + httpx** でテスト
- **条件付きレンダリング**: `{% if with_grpc %}` で gRPC、`{% if with_db %}` で SQLAlchemy 依存を追加
- **Multi-stage Docker build**: Python 3.12 ベースイメージ
