# 02. .NET ソリューション配置

本ファイルは `src/tier2/dotnet/` 配下の .NET ソリューション構成を確定する。Central Package Management（CPM）による依存管理、サービスごとの独立 `.sln`、Directory.Build.props による共通プロパティを組み合わせる。

## レイアウト

```
src/tier2/dotnet/
├── Tier2.sln                       # 全サービス統合ソリューション（全 csproj を参照）
├── Directory.Build.props           # 共通プロパティ（LangVersion / Nullable / TargetFramework）
├── Directory.Packages.props        # Central Package Management
├── .editorconfig                   # dotnet-format 設定
├── nuget.config                    # 社内 NuGet source 定義
├── services/
│   ├── ApprovalFlow/
│   │   ├── ApprovalFlow.sln        # 単独ビルド用（CI の path-filter 用）
│   │   ├── src/
│   │   │   ├── ApprovalFlow.Api/
│   │   │   │   ├── ApprovalFlow.Api.csproj
│   │   │   │   ├── Program.cs
│   │   │   │   ├── Controllers/
│   │   │   │   └── Startup/
│   │   │   ├── ApprovalFlow.Application/
│   │   │   │   ├── ApprovalFlow.Application.csproj
│   │   │   │   ├── UseCases/
│   │   │   │   └── Services/
│   │   │   ├── ApprovalFlow.Domain/
│   │   │   │   ├── ApprovalFlow.Domain.csproj
│   │   │   │   ├── Entities/
│   │   │   │   ├── ValueObjects/
│   │   │   │   └── Events/
│   │   │   └── ApprovalFlow.Infrastructure/
│   │   │       ├── ApprovalFlow.Infrastructure.csproj
│   │   │       ├── Persistence/
│   │   │       └── ExternalServices/
│   │   ├── tests/
│   │   │   ├── ApprovalFlow.Domain.Tests/
│   │   │   ├── ApprovalFlow.Application.Tests/
│   │   │   ├── ApprovalFlow.Api.Tests/
│   │   │   └── ApprovalFlow.IntegrationTests/
│   │   └── Dockerfile
│   ├── InvoiceGenerator/
│   └── TaxCalculator/
└── shared/                         # tier2 内の共通 lib
    ├── Tier2.Shared.Dapr/
    │   ├── Tier2.Shared.Dapr.csproj
    │   └── src/
    └── Tier2.Shared.Otel/
        └── Tier2.Shared.Otel.csproj
```

## Directory.Build.props の推奨サンプル

```xml
<Project>
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
    <LangVersion>latest</LangVersion>
    <Nullable>enable</Nullable>
    <ImplicitUsings>enable</ImplicitUsings>
    <TreatWarningsAsErrors>true</TreatWarningsAsErrors>
    <WarningsNotAsErrors>CS1591</WarningsNotAsErrors>
    <EnforceCodeStyleInBuild>true</EnforceCodeStyleInBuild>
    <AnalysisLevel>latest</AnalysisLevel>
    <AnalysisMode>All</AnalysisMode>
    <EnableNETAnalyzers>true</EnableNETAnalyzers>
    <AccelerateBuildsInVisualStudio>true</AccelerateBuildsInVisualStudio>
  </PropertyGroup>
  <PropertyGroup>
    <Authors>k1s0 contributors</Authors>
    <Copyright>Copyright (c) k1s0</Copyright>
    <PackageLicenseExpression>Apache-2.0</PackageLicenseExpression>
  </PropertyGroup>
</Project>
```

## Directory.Packages.props の推奨サンプル

Central Package Management により、全 csproj で使う NuGet パッケージのバージョンを集約する。

```xml
<Project>
  <PropertyGroup>
    <ManagePackageVersionsCentrally>true</ManagePackageVersionsCentrally>
    <CentralPackageTransitivePinningEnabled>true</CentralPackageTransitivePinningEnabled>
  </PropertyGroup>
  <ItemGroup>
    <!-- k1s0 SDK -->
    <PackageVersion Include="K1s0.Sdk" Version="0.1.0" />
    <PackageVersion Include="K1s0.Sdk.Auth" Version="0.1.0" />

    <!-- Dapr -->
    <PackageVersion Include="Dapr.Client" Version="1.14.0" />

    <!-- ASP.NET Core -->
    <PackageVersion Include="Microsoft.AspNetCore.OpenApi" Version="8.0.10" />
    <PackageVersion Include="Swashbuckle.AspNetCore" Version="6.9.0" />

    <!-- OpenTelemetry -->
    <PackageVersion Include="OpenTelemetry.Extensions.Hosting" Version="1.10.0" />
    <PackageVersion Include="OpenTelemetry.Exporter.OpenTelemetryProtocol" Version="1.10.0" />
    <PackageVersion Include="OpenTelemetry.Instrumentation.AspNetCore" Version="1.10.1" />

    <!-- Testing -->
    <PackageVersion Include="xunit" Version="2.9.2" />
    <PackageVersion Include="xunit.runner.visualstudio" Version="2.8.2" />
    <PackageVersion Include="Microsoft.NET.Test.Sdk" Version="17.11.1" />
    <PackageVersion Include="NSubstitute" Version="5.1.0" />
    <PackageVersion Include="Testcontainers" Version="3.10.0" />
  </ItemGroup>
</Project>
```

## 2 つの .sln の使い分け

- **Tier2.sln**: 全サービス + shared を含む統合 .sln。IDE でまとめて開く際や dotnet-format 全体適用時に使う
- **services/<service>/<Service>.sln**: 単独 .sln。CI の path-filter で特定サービスのみビルドする際に使う

Visual Studio / Rider の IDE 起動時は `Tier2.sln` を推奨。CI は path-filter で単独 .sln を選ぶ。

## サービス内部構造

各サービス配下は Clean Architecture（Onion Architecture）の 4 層構成。

- **Api 層**: Controllers / Program.cs。HTTP エンドポイント
- **Application 層**: UseCases / Services。ユースケース実装
- **Domain 層**: Entities / ValueObjects / Events。ドメインモデル
- **Infrastructure 層**: Persistence（EF Core）/ ExternalServices（Dapr / tier1 gRPC クライアント）

依存方向: `Api → Application → Domain`、`Infrastructure → Domain`。Api / Application は Infrastructure を直接参照せず、Application が Domain の interface を使い Infrastructure が実装する。詳細は [04_サービス単位の内部構造.md](04_サービス単位の内部構造.md) 参照。

## NuGet フィード

`nuget.config` で以下を定義。

- `nuget.org`（public）
- 社内プライベート NuGet サーバ（運用蓄積後、Harbor の OCI artifacts / Azure Artifacts / GitHub Packages のいずれか）

社内パッケージ（`K1s0.Sdk` など）は リリース時点で は GitHub Packages / 社内プライベートに移行する。

## Dockerfile

各サービスの Dockerfile は multi-stage build。

```dockerfile
# services/ApprovalFlow/Dockerfile
# build-context 要件: docker build は src/tier2/dotnet/ をルートとして実行する前提で書いている。
# CI は `docker build -f src/tier2/dotnet/services/ApprovalFlow/Dockerfile src/tier2/dotnet/` を使う。
# リポジトリルートから build すると Directory.Build.props / Directory.Packages.props が見つからず restore が失敗する。
FROM mcr.microsoft.com/dotnet/sdk:8.0 AS builder
WORKDIR /workspace
COPY Directory.Build.props Directory.Packages.props nuget.config ./
COPY services/ApprovalFlow/ ./services/ApprovalFlow/
COPY shared/ ./shared/
RUN dotnet restore services/ApprovalFlow/ApprovalFlow.sln
RUN dotnet publish services/ApprovalFlow/src/ApprovalFlow.Api/ApprovalFlow.Api.csproj \
    -c Release -o /publish --no-restore

FROM mcr.microsoft.com/dotnet/aspnet:8.0-bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /publish .
USER app
EXPOSE 8080
ENTRYPOINT ["dotnet", "ApprovalFlow.Api.dll"]
```

## テスト戦略

- unit test: xUnit + NSubstitute で Domain / Application 層
- integration test: Testcontainers で Postgres / Redis / tier1 Pod を起動
- contract test: Pact.Net で tier1 SDK との契約検証

## 対応 IMP-DIR ID

- IMP-DIR-T2-042（dotnet solution 配置）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-TIER1-003（内部言語不可視）
- DS-SW-COMP-019
- FR-\* / DX-CICD-\* / DX-TEST-\*
