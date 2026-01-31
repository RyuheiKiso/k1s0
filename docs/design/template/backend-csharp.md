# backend-csharp テンプレート

← [テンプレート設計書](./)

---

## ディレクトリ構造

```
feature/backend/csharp/{service_name}/
├── .k1s0/
│   └── manifest.json.tera
├── {FeatureName}.sln.tera
├── Directory.Build.props.tera
├── Directory.Packages.props.tera
├── .editorconfig
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
│   ├── {FeatureName}.Domain/
│   │   ├── {FeatureName}.Domain.csproj.tera
│   │   ├── Entities/.gitkeep
│   │   ├── ValueObjects/.gitkeep
│   │   ├── Repositories/.gitkeep
│   │   └── Services/.gitkeep
│   ├── {FeatureName}.Application/
│   │   ├── {FeatureName}.Application.csproj.tera
│   │   ├── UseCases/.gitkeep
│   │   ├── Services/.gitkeep
│   │   └── DTOs/.gitkeep
│   ├── {FeatureName}.Infrastructure/
│   │   ├── {FeatureName}.Infrastructure.csproj.tera
│   │   ├── Repositories/.gitkeep
│   │   ├── External/.gitkeep
│   │   └── Persistence/.gitkeep
│   └── {FeatureName}.Presentation/
│       ├── {FeatureName}.Presentation.csproj.tera
│       ├── Program.cs.tera
│       ├── Controllers/.gitkeep
│       ├── Grpc/.gitkeep
│       └── Middleware/.gitkeep
└── tests/
    ├── {FeatureName}.Domain.Tests/
    ├── {FeatureName}.Application.Tests/
    └── {FeatureName}.Integration.Tests/
```

## 特徴

- **ASP.NET Core 8.0** ベース
- **Central Package Management** （`Directory.Packages.props`）でバージョン一元管理
- **4プロジェクト構成**: Domain, Application, Infrastructure, Presentation（Clean Architecture）
- **3テストプロジェクト**: Domain.Tests, Application.Tests, Integration.Tests
- **条件付きレンダリング**: `{% if with_grpc %}` で gRPC、`{% if with_db %}` で EF Core 依存を追加
- **Multi-stage Docker build**: SDK イメージでビルド → ASP.NET ランタイムイメージで実行
