# Backend Framework（C#）

k1s0 Backend Framework（C#）は、ASP.NET Core 8.0 ベースのマイクロサービス開発のための共通 NuGet パッケージ群を提供します。Rust 版・Go 版と同等の機能を C# / .NET で実装しています。

## パッケージ一覧

```
framework/backend/csharp/src/
├── K1s0.Error/            # エラー表現の統一（RFC 7807）
├── K1s0.Config/           # YAML 設定読み込み
├── K1s0.Validation/       # 入力バリデーション（FluentValidation）
├── K1s0.Observability/    # OpenTelemetry 統合
├── K1s0.Grpc.Server/      # gRPC サーバー共通基盤
├── K1s0.Grpc.Client/      # gRPC クライアント共通
├── K1s0.Health/           # ヘルスチェック（liveness/readiness）
└── K1s0.Db/               # DB 接続（EF Core + Npgsql）
```

## Tier 構成

### Tier 1（依存なし）

| パッケージ | 説明 | 主要依存 |
|-----------|------|---------|
| K1s0.Error | 統一エラーハンドリング。K1s0Exception 基底クラス、ErrorCode レコード、RFC 7807 ProblemDetails 生成 | - |
| K1s0.Config | YAML 設定管理。`AddK1s0YamlConfig()` 拡張で `--env`/`--secrets-dir` 対応 | YamlDotNet |
| K1s0.Validation | 入力検証。`K1s0Validator<T>` 基底クラス、`AddK1s0Validation()` で自動登録 | FluentValidation |

### Tier 2（Tier 1 のみ依存可）

| パッケージ | 説明 | 依存先 |
|-----------|------|--------|
| K1s0.Observability | OpenTelemetry 統合。トレーシング（ActivitySource）、メトリクス（Meter）、ロギング | K1s0.Error |
| K1s0.Grpc.Server | gRPC サーバー基盤。ErrorHandlingInterceptor、TracingInterceptor | K1s0.Error, K1s0.Observability |
| K1s0.Grpc.Client | gRPC クライアント。GrpcClientFactory でチャネル生成 | K1s0.Error |
| K1s0.Health | ヘルスチェック。`/healthz/live`、`/healthz/ready` エンドポイント | K1s0.Error, K1s0.Config |
| K1s0.Db | DB 接続。EF Core + Npgsql、UnitOfWork、RepositoryBase<TEntity, TId> | K1s0.Error, K1s0.Config |

## ビルド設定

### Central Package Management

`Directory.Packages.props` で全パッケージのバージョンを一元管理:

```xml
<Project>
  <PropertyGroup>
    <ManagePackageVersionsCentrally>true</ManagePackageVersionsCentrally>
  </PropertyGroup>
  <ItemGroup>
    <PackageVersion Include="YamlDotNet" Version="16.3.0" />
    <PackageVersion Include="FluentValidation" Version="11.11.0" />
    <PackageVersion Include="OpenTelemetry.Extensions.Hosting" Version="1.10.0" />
    <!-- ... -->
  </ItemGroup>
</Project>
```

### 共通ビルドプロパティ

`Directory.Build.props`:

```xml
<Project>
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
    <Nullable>enable</Nullable>
    <ImplicitUsings>enable</ImplicitUsings>
    <TreatWarningsAsErrors>true</TreatWarningsAsErrors>
  </PropertyGroup>
</Project>
```

## テスト

- テストフレームワーク: xUnit
- モック: Moq
- アサーション: FluentAssertions
- 全 74 テスト

## Rust 版との対応表

| Rust crate | C# パッケージ | 備考 |
|-----------|--------------|------|
| k1s0-error | K1s0.Error | RFC 7807 対応 |
| k1s0-config | K1s0.Config | YamlDotNet 使用 |
| k1s0-validation | K1s0.Validation | FluentValidation 使用 |
| k1s0-observability | K1s0.Observability | OpenTelemetry .NET SDK 使用 |
| k1s0-grpc-server | K1s0.Grpc.Server | Grpc.AspNetCore 使用 |
| k1s0-grpc-client | K1s0.Grpc.Client | Grpc.Net.Client 使用 |
| k1s0-health | K1s0.Health | ASP.NET Core HealthChecks 使用 |
| k1s0-db | K1s0.Db | EF Core + Npgsql 使用 |
| k1s0-resilience | （未実装） | Polly v8 で今後対応予定 |
| k1s0-cache | （未実装） | StackExchange.Redis で今後対応予定 |
| k1s0-domain-event | （未実装） | 今後対応予定 |
| k1s0-auth | （未実装） | 今後対応予定 |
