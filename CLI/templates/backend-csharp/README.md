# Backend C# Template

C# (ASP.NET Core) バックエンドサービスのテンプレート。

## ディレクトリ構成

```
backend-csharp/
├── feature/      # 機能雛形（k1s0 new-feature 用）
└── domain/       # ドメインライブラリ雛形（k1s0 new-domain 用）
```

## feature/ の生成物

`k1s0 new-feature --type backend-csharp --name {name}` で生成される構成：

```
feature/backend/csharp/{name}/
├── {Name}.sln
├── config/
│   ├── default.yaml
│   ├── dev.yaml
│   ├── stg.yaml
│   └── prod.yaml
├── deploy/
│   ├── base/
│   └── overlays/
├── src/
│   ├── {Name}.Domain/              # ビジネスロジック層
│   │   ├── Entities/
│   │   ├── ValueObjects/
│   │   ├── Repositories/
│   │   └── Services/
│   ├── {Name}.Application/         # アプリケーション層
│   │   ├── UseCases/
│   │   ├── Services/
│   │   └── Dtos/
│   ├── {Name}.Infrastructure/      # インフラストラクチャ層
│   │   ├── Repositories/
│   │   ├── External/
│   │   └── Persistence/
│   └── {Name}.Presentation/        # プレゼンテーション層
│       ├── Grpc/
│       ├── Controllers/
│       └── Middleware/
├── Dockerfile
├── .dockerignore
└── docker-compose.yml
```

## 技術スタック

- フレームワーク: ASP.NET Core 8.0
- gRPC: Grpc.AspNetCore
- ORM: Entity Framework Core
- 設定: K1s0.Config (YAML)
- ログ/トレース: OpenTelemetry
