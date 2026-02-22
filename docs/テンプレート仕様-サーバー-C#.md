# テンプレート仕様 — サーバー — C# (ASP.NET Core)

本ドキュメントは、[テンプレート仕様-サーバー](テンプレート仕様-サーバー.md) から分割された C# (ASP.NET Core) テンプレートの詳細仕様である。

---

## プロジェクト構造テンプレート

C# サーバーはクリーンアーキテクチャに基づく4レイヤー構成を採用する。

```
{{ service_name }}/
├── src/
│   ├── {{ service_name_pascal }}.Api/           # エントリポイント・ミドルウェア
│   │   ├── Program.cs
│   │   ├── Controllers/                         # REST コントローラー
│   │   ├── GrpcServices/                        # gRPC サービス実装
│   │   ├── Middleware/                           # 認証・エラーハンドリング
│   │   └── {{ service_name_pascal }}.Api.csproj
│   ├── {{ service_name_pascal }}.Domain/         # ドメインモデル・リポジトリインターフェース
│   │   ├── Entities/
│   │   ├── Repositories/
│   │   └── {{ service_name_pascal }}.Domain.csproj
│   ├── {{ service_name_pascal }}.UseCase/        # ビジネスロジック
│   │   ├── Services/
│   │   └── {{ service_name_pascal }}.UseCase.csproj
│   └── {{ service_name_pascal }}.Infra/          # DB 接続・メッセージング・設定
│       ├── Persistence/
│       ├── Messaging/
│       ├── Config/
│       └── {{ service_name_pascal }}.Infra.csproj
├── tests/
│   ├── {{ service_name_pascal }}.UnitTests/
│   │   └── {{ service_name_pascal }}.UnitTests.csproj
│   └── {{ service_name_pascal }}.IntegrationTests/
│       └── {{ service_name_pascal }}.IntegrationTests.csproj
├── config/
│   └── config.yaml
├── Dockerfile
├── {{ service_name_pascal }}.sln
└── README.md
```

---

## テンプレート変数

C# サーバーテンプレートで使用する変数を以下に示す。変数の定義と導出ルールの詳細は [テンプレートエンジン仕様](テンプレートエンジン仕様.md) を参照。

| 変数名                | 型       | 用途                                    |
| --------------------- | -------- | --------------------------------------- |
| `service_name`        | String   | ソリューション名・Dockerfile 名         |
| `service_name_snake`  | String   | DB テーブル名・設定キー                 |
| `service_name_pascal` | String   | プロジェクト名・クラス名・名前空間      |
| `service_name_camel`  | String   | JSON プロパティ名                       |
| `tier`                | String   | Docker プロジェクト名の導出             |
| `api_styles`          | [String] | REST / gRPC によるコード分岐            |
| `has_database`        | bool     | DB 関連コードの生成制御                 |
| `database_type`       | String   | DB ドライバの選択                       |
| `has_kafka`           | bool     | Kafka 関連コードの生成制御              |
| `has_redis`           | bool     | Redis 関連コードの生成制御              |

---

## Program.cs / DI 設定

`src/{{ service_name_pascal }}.Api/Program.cs.tera` — エントリポイント。DI コンテナ設定・ミドルウェア・サーバー起動。

```csharp
using {{ service_name_pascal }}.Api.Middleware;
using {{ service_name_pascal }}.Domain.Repositories;
using {{ service_name_pascal }}.Infra.Persistence;
using {{ service_name_pascal }}.UseCase.Services;

var builder = WebApplication.CreateBuilder(args);

// --- Configuration ---
builder.Configuration.AddYamlFile("config/config.yaml", optional: false);

// --- DI ---
{% if has_database %}
builder.Services.AddScoped<I{{ service_name_pascal }}Repository, {{ service_name_pascal }}Repository>();
{% endif %}
builder.Services.AddScoped<{{ service_name_pascal }}Service>();

{% if api_styles is containing("rest") %}
// --- REST ---
builder.Services.AddControllers();
builder.Services.AddEndpointsApiExplorer();
builder.Services.AddSwaggerGen();
{% endif %}

{% if api_styles is containing("grpc") %}
// --- gRPC ---
builder.Services.AddGrpc();
{% endif %}

{% if has_database and database_type == "postgresql" %}
// --- Database ---
builder.Services.AddNpgsql<{{ service_name_pascal }}DbContext>(
    builder.Configuration.GetConnectionString("DefaultConnection"));
{% elif has_database and database_type == "mysql" %}
builder.Services.AddDbContext<{{ service_name_pascal }}DbContext>(options =>
    options.UseMySql(
        builder.Configuration.GetConnectionString("DefaultConnection"),
        ServerVersion.AutoDetect(builder.Configuration.GetConnectionString("DefaultConnection"))));
{% endif %}

{% if has_redis %}
// --- Redis ---
builder.Services.AddStackExchangeRedisCache(options =>
{
    options.Configuration = builder.Configuration["Redis:Addr"];
});
{% endif %}

// --- Health Checks ---
builder.Services.AddHealthChecks()
{% if has_database %}
    .AddDbContextCheck<{{ service_name_pascal }}DbContext>()
{% endif %}
{% if has_redis %}
    .AddRedis(builder.Configuration["Redis:Addr"]!)
{% endif %}
    ;

// --- OpenTelemetry ---
builder.Services.AddOpenTelemetry()
    .WithTracing(tracing => tracing
        .AddAspNetCoreInstrumentation()
{% if has_database %}
        .AddEntityFrameworkCoreInstrumentation()
{% endif %}
        .AddOtlpExporter())
    .WithMetrics(metrics => metrics
        .AddAspNetCoreInstrumentation()
        .AddOtlpExporter());

var app = builder.Build();

// --- Middleware ---
app.UseK1s0JwtAuth();

{% if api_styles is containing("rest") %}
app.UseSwagger();
app.UseSwaggerUI();
app.MapControllers();
{% endif %}

{% if api_styles is containing("grpc") %}
app.MapGrpcService<{{ service_name_pascal }}GrpcService>();
{% endif %}

app.MapHealthChecks("/healthz");
app.MapHealthChecks("/readyz");

app.Run();
```

---

## ミドルウェア設定パターン

### 認証ミドルウェア（UseK1s0JwtAuth）

`src/{{ service_name_pascal }}.Api/Middleware/JwtAuthMiddleware.cs.tera` — [認証認可設計.md](認証認可設計.md) 準拠の JWT 認証ミドルウェア。

```csharp
namespace {{ service_name_pascal }}.Api.Middleware;

public static class JwtAuthMiddlewareExtensions
{
    public static IApplicationBuilder UseK1s0JwtAuth(this IApplicationBuilder builder)
    {
        return builder.UseMiddleware<JwtAuthMiddleware>();
    }
}

public class JwtAuthMiddleware
{
    private readonly RequestDelegate _next;
    private readonly IConfiguration _configuration;

    public JwtAuthMiddleware(RequestDelegate next, IConfiguration configuration)
    {
        _next = next;
        _configuration = configuration;
    }

    public async Task InvokeAsync(HttpContext context)
    {
        // ヘルスチェックエンドポイントはスキップ
        if (context.Request.Path.StartsWithSegments("/healthz") ||
            context.Request.Path.StartsWithSegments("/readyz"))
        {
            await _next(context);
            return;
        }

        var token = context.Request.Headers.Authorization
            .FirstOrDefault()?.Replace("Bearer ", "");

        if (string.IsNullOrEmpty(token))
        {
            context.Response.StatusCode = 401;
            await context.Response.WriteAsJsonAsync(new { code = "UNAUTHORIZED", message = "Missing token" });
            return;
        }

        // TODO: JWKS エンドポイントからの公開鍵取得と JWT 検証
        // var jwksUrl = _configuration["Auth:Jwks:Url"];

        await _next(context);
    }
}
```

### エラーハンドリングミドルウェア

`src/{{ service_name_pascal }}.Api/Middleware/ErrorHandlingMiddleware.cs.tera` — [API設計.md](API設計.md) D-007 準拠のエラーレスポンス。

```csharp
namespace {{ service_name_pascal }}.Api.Middleware;

public class ErrorHandlingMiddleware
{
    private readonly RequestDelegate _next;
    private readonly ILogger<ErrorHandlingMiddleware> _logger;

    public ErrorHandlingMiddleware(RequestDelegate next, ILogger<ErrorHandlingMiddleware> logger)
    {
        _next = next;
        _logger = logger;
    }

    public async Task InvokeAsync(HttpContext context)
    {
        try
        {
            await _next(context);
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Unhandled exception");
            context.Response.StatusCode = 500;
            await context.Response.WriteAsJsonAsync(new
            {
                code = "INTERNAL_ERROR",
                message = "An unexpected error occurred"
            });
        }
    }
}
```

---

## REST コントローラーテンプレート

`src/{{ service_name_pascal }}.Api/Controllers/{{ service_name_pascal }}Controller.cs.tera` — {% if api_styles is containing("rest") %} に該当。

```csharp
{% if api_styles is containing("rest") %}
using Microsoft.AspNetCore.Mvc;
using {{ service_name_pascal }}.Domain.Entities;
using {{ service_name_pascal }}.UseCase.Services;

namespace {{ service_name_pascal }}.Api.Controllers;

[ApiController]
[Route("api/v1/{{ service_name }}")]
public class {{ service_name_pascal }}Controller : ControllerBase
{
    private readonly {{ service_name_pascal }}Service _service;

    public {{ service_name_pascal }}Controller({{ service_name_pascal }}Service service)
    {
        _service = service;
    }

    [HttpGet]
    public async Task<ActionResult<IEnumerable<{{ service_name_pascal }}Entity>>> List()
    {
        var result = await _service.GetAllAsync();
        return Ok(result);
    }

    [HttpGet("{id}")]
    public async Task<ActionResult<{{ service_name_pascal }}Entity>> GetById(string id)
    {
        var result = await _service.GetByIdAsync(id);
        if (result is null) return NotFound(new { code = "NOT_FOUND", message = $"Resource not found: {id}" });
        return Ok(result);
    }

    [HttpPost]
    public async Task<ActionResult<{{ service_name_pascal }}Entity>> Create([FromBody] Create{{ service_name_pascal }}Request request)
    {
        var entity = await _service.CreateAsync(request);
        return CreatedAtAction(nameof(GetById), new { id = entity.Id }, entity);
    }
}

public record Create{{ service_name_pascal }}Request(string Name, string? Description);
{% endif %}
```

---

## gRPC サービステンプレート

`src/{{ service_name_pascal }}.Api/GrpcServices/{{ service_name_pascal }}GrpcService.cs.tera` — {% if api_styles is containing("grpc") %} に該当。

```csharp
{% if api_styles is containing("grpc") %}
using Grpc.Core;
using {{ service_name_pascal }}.UseCase.Services;

namespace {{ service_name_pascal }}.Api.GrpcServices;

public class {{ service_name_pascal }}GrpcService : {{ service_name_pascal }}Grpc.{{ service_name_pascal }}GrpcBase
{
    private readonly {{ service_name_pascal }}Service _service;

    public {{ service_name_pascal }}GrpcService({{ service_name_pascal }}Service service)
    {
        _service = service;
    }

    public override async Task<Get{{ service_name_pascal }}Response> Get{{ service_name_pascal }}(
        Get{{ service_name_pascal }}Request request, ServerCallContext context)
    {
        var entity = await _service.GetByIdAsync(request.Id);
        if (entity is null)
            throw new RpcException(new Status(StatusCode.NotFound, $"Resource not found: {request.Id}"));

        return new Get{{ service_name_pascal }}Response
        {
            Id = entity.Id,
            Name = entity.Name,
            Description = entity.Description ?? "",
            Status = entity.Status,
            CreatedAt = entity.CreatedAt,
            UpdatedAt = entity.UpdatedAt,
        };
    }

    public override async Task<List{{ service_name_pascal }}Response> List{{ service_name_pascal }}(
        List{{ service_name_pascal }}Request request, ServerCallContext context)
    {
        var entities = await _service.GetAllAsync();
        var response = new List{{ service_name_pascal }}Response();
        foreach (var e in entities)
        {
            response.Items.Add(new Get{{ service_name_pascal }}Response
            {
                Id = e.Id,
                Name = e.Name,
                Description = e.Description ?? "",
                Status = e.Status,
                CreatedAt = e.CreatedAt,
                UpdatedAt = e.UpdatedAt,
            });
        }
        return response;
    }

    public override Task<Create{{ service_name_pascal }}Response> Create{{ service_name_pascal }}(
        Create{{ service_name_pascal }}Request request, ServerCallContext context)
    {
        // TODO: 実装
        return Task.FromResult(new Create{{ service_name_pascal }}Response { Id = "todo" });
    }
}
{% endif %}
```

---

## ドメイン層

### エンティティ

`src/{{ service_name_pascal }}.Domain/Entities/{{ service_name_pascal }}Entity.cs.tera`

```csharp
namespace {{ service_name_pascal }}.Domain.Entities;

public class {{ service_name_pascal }}Entity
{
    public string Id { get; set; } = string.Empty;
    public string Name { get; set; } = string.Empty;
    public string? Description { get; set; }
    public string Status { get; set; } = "active";
    public string CreatedAt { get; set; } = string.Empty;
    public string UpdatedAt { get; set; } = string.Empty;
}
```

### リポジトリインターフェース

`src/{{ service_name_pascal }}.Domain/Repositories/I{{ service_name_pascal }}Repository.cs.tera`

```csharp
using {{ service_name_pascal }}.Domain.Entities;

namespace {{ service_name_pascal }}.Domain.Repositories;

public interface I{{ service_name_pascal }}Repository
{
    Task<{{ service_name_pascal }}Entity?> FindByIdAsync(string id);
    Task<IEnumerable<{{ service_name_pascal }}Entity>> FindAllAsync();
    Task CreateAsync({{ service_name_pascal }}Entity entity);
    Task UpdateAsync({{ service_name_pascal }}Entity entity);
    Task DeleteAsync(string id);
}
```

---

## ユースケース層

`src/{{ service_name_pascal }}.UseCase/Services/{{ service_name_pascal }}Service.cs.tera`

```csharp
using {{ service_name_pascal }}.Domain.Entities;
{% if has_database %}
using {{ service_name_pascal }}.Domain.Repositories;
{% endif %}

namespace {{ service_name_pascal }}.UseCase.Services;

public class {{ service_name_pascal }}Service
{
{% if has_database %}
    private readonly I{{ service_name_pascal }}Repository _repository;

    public {{ service_name_pascal }}Service(I{{ service_name_pascal }}Repository repository)
    {
        _repository = repository;
    }
{% else %}
    public {{ service_name_pascal }}Service()
    {
    }
{% endif %}

    public async Task<{{ service_name_pascal }}Entity?> GetByIdAsync(string id)
    {
{% if has_database %}
        return await _repository.FindByIdAsync(id);
{% else %}
        // TODO: 実装
        return await Task.FromResult<{{ service_name_pascal }}Entity?>(null);
{% endif %}
    }

    public async Task<IEnumerable<{{ service_name_pascal }}Entity>> GetAllAsync()
    {
{% if has_database %}
        return await _repository.FindAllAsync();
{% else %}
        // TODO: 実装
        return await Task.FromResult(Enumerable.Empty<{{ service_name_pascal }}Entity>());
{% endif %}
    }

    public async Task<{{ service_name_pascal }}Entity> CreateAsync(object request)
    {
        // TODO: request → entity 変換、リポジトリ呼び出し
        return await Task.FromResult(new {{ service_name_pascal }}Entity { Id = Guid.NewGuid().ToString() });
    }
}
```

---

## インフラ層

### DB 接続（Entity Framework Core）

`src/{{ service_name_pascal }}.Infra/Persistence/{{ service_name_pascal }}DbContext.cs.tera` — {% if has_database %} に該当。

```csharp
{% if has_database %}
using Microsoft.EntityFrameworkCore;
using {{ service_name_pascal }}.Domain.Entities;

namespace {{ service_name_pascal }}.Infra.Persistence;

public class {{ service_name_pascal }}DbContext : DbContext
{
    public {{ service_name_pascal }}DbContext(DbContextOptions<{{ service_name_pascal }}DbContext> options)
        : base(options)
    {
    }

    public DbSet<{{ service_name_pascal }}Entity> {{ service_name_pascal }}s => Set<{{ service_name_pascal }}Entity>();

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        modelBuilder.Entity<{{ service_name_pascal }}Entity>(entity =>
        {
            entity.HasKey(e => e.Id);
            entity.Property(e => e.Name).IsRequired().HasMaxLength(255);
            entity.Property(e => e.Status).IsRequired();
        });
    }
}
{% endif %}
```

### リポジトリ実装

`src/{{ service_name_pascal }}.Infra/Persistence/{{ service_name_pascal }}Repository.cs.tera` — {% if has_database %} に該当。

```csharp
{% if has_database %}
using Microsoft.EntityFrameworkCore;
using {{ service_name_pascal }}.Domain.Entities;
using {{ service_name_pascal }}.Domain.Repositories;

namespace {{ service_name_pascal }}.Infra.Persistence;

public class {{ service_name_pascal }}Repository : I{{ service_name_pascal }}Repository
{
    private readonly {{ service_name_pascal }}DbContext _context;

    public {{ service_name_pascal }}Repository({{ service_name_pascal }}DbContext context)
    {
        _context = context;
    }

    public async Task<{{ service_name_pascal }}Entity?> FindByIdAsync(string id)
    {
        return await _context.{{ service_name_pascal }}s.FindAsync(id);
    }

    public async Task<IEnumerable<{{ service_name_pascal }}Entity>> FindAllAsync()
    {
        return await _context.{{ service_name_pascal }}s
            .OrderByDescending(e => e.CreatedAt)
            .ToListAsync();
    }

    public async Task CreateAsync({{ service_name_pascal }}Entity entity)
    {
        _context.{{ service_name_pascal }}s.Add(entity);
        await _context.SaveChangesAsync();
    }

    public async Task UpdateAsync({{ service_name_pascal }}Entity entity)
    {
        _context.{{ service_name_pascal }}s.Update(entity);
        await _context.SaveChangesAsync();
    }

    public async Task DeleteAsync(string id)
    {
        var entity = await _context.{{ service_name_pascal }}s.FindAsync(id);
        if (entity is not null)
        {
            _context.{{ service_name_pascal }}s.Remove(entity);
            await _context.SaveChangesAsync();
        }
    }
}
{% endif %}
```

---

## Dockerfile テンプレート

`Dockerfile.tera` — [Dockerイメージ戦略.md](Dockerイメージ戦略.md) 準拠のマルチステージビルド。

```dockerfile
# === Build Stage ===
FROM mcr.microsoft.com/dotnet/sdk:10 AS build
WORKDIR /src

COPY *.sln .
COPY src/{{ service_name_pascal }}.Api/*.csproj src/{{ service_name_pascal }}.Api/
COPY src/{{ service_name_pascal }}.Domain/*.csproj src/{{ service_name_pascal }}.Domain/
COPY src/{{ service_name_pascal }}.UseCase/*.csproj src/{{ service_name_pascal }}.UseCase/
COPY src/{{ service_name_pascal }}.Infra/*.csproj src/{{ service_name_pascal }}.Infra/
RUN dotnet restore

COPY . .
RUN dotnet publish src/{{ service_name_pascal }}.Api -c Release -o /app/publish

# === Runtime Stage ===
FROM mcr.microsoft.com/dotnet/aspnet:10-alpine

WORKDIR /app
COPY --from=build /app/publish .
COPY config/ /app/config/

EXPOSE 8080
USER nonroot:nonroot
ENTRYPOINT ["dotnet", "{{ service_name_pascal }}.Api.dll"]
```

---

## config/config.yaml

`config/config.yaml.tera` — [config設計.md](config設計.md) 準拠のアプリケーション設定ファイル。

```yaml
app:
  name: "{{ service_name }}"
  version: "0.1.0"
  environment: "development"

server:
  port: "8080"
  read_timeout_sec: 30
  write_timeout_sec: 30

{% if has_database %}
database:
  host: "localhost"
{% if database_type == "postgresql" %}
  port: 5432
{% elif database_type == "mysql" %}
  port: 3306
{% endif %}
  user: "{{ service_name_snake }}"
  password: ""
  name: "{{ service_name_snake }}"
{% if database_type == "postgresql" %}
  ssl_mode: "disable"
{% endif %}
  pool:
    max_open: 25
    max_idle: 5
    max_lifetime_sec: 300
{% endif %}

{% if has_kafka %}
kafka:
  brokers:
    - "localhost:9092"
  schema_registry: "http://localhost:8081"
{% endif %}

{% if has_redis %}
redis:
  addr: "localhost:6379"
  password: ""
  db: 0
{% endif %}

observability:
  trace_endpoint: "localhost:4317"
  metric_endpoint: "localhost:4317"
  log_level: "Information"
```

---

## csproj ファイルテンプレート

### Api プロジェクト

`src/{{ service_name_pascal }}.Api/{{ service_name_pascal }}.Api.csproj.tera`

```xml
<Project Sdk="Microsoft.NET.Sdk.Web">

  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework>
    <Nullable>enable</Nullable>
    <ImplicitUsings>enable</ImplicitUsings>
  </PropertyGroup>

  <ItemGroup>
    <ProjectReference Include="..\{{ service_name_pascal }}.UseCase\{{ service_name_pascal }}.UseCase.csproj" />
    <ProjectReference Include="..\{{ service_name_pascal }}.Infra\{{ service_name_pascal }}.Infra.csproj" />
  </ItemGroup>

  <ItemGroup>
    <PackageReference Include="Swashbuckle.AspNetCore" />
    <PackageReference Include="OpenTelemetry.Extensions.Hosting" />
    <PackageReference Include="OpenTelemetry.Instrumentation.AspNetCore" />
    <PackageReference Include="OpenTelemetry.Exporter.OpenTelemetryProtocol" />
{% if api_styles is containing("grpc") %}
    <PackageReference Include="Grpc.AspNetCore" />
{% endif %}
{% if has_redis %}
    <PackageReference Include="Microsoft.Extensions.Caching.StackExchangeRedis" />
{% endif %}
  </ItemGroup>

</Project>
```

### Domain プロジェクト

`src/{{ service_name_pascal }}.Domain/{{ service_name_pascal }}.Domain.csproj.tera`

```xml
<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework>
    <Nullable>enable</Nullable>
    <ImplicitUsings>enable</ImplicitUsings>
  </PropertyGroup>

</Project>
```

### UseCase プロジェクト

`src/{{ service_name_pascal }}.UseCase/{{ service_name_pascal }}.UseCase.csproj.tera`

```xml
<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework>
    <Nullable>enable</Nullable>
    <ImplicitUsings>enable</ImplicitUsings>
  </PropertyGroup>

  <ItemGroup>
    <ProjectReference Include="..\{{ service_name_pascal }}.Domain\{{ service_name_pascal }}.Domain.csproj" />
  </ItemGroup>

</Project>
```

### Infra プロジェクト

`src/{{ service_name_pascal }}.Infra/{{ service_name_pascal }}.Infra.csproj.tera`

```xml
<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework>
    <Nullable>enable</Nullable>
    <ImplicitUsings>enable</ImplicitUsings>
  </PropertyGroup>

  <ItemGroup>
    <ProjectReference Include="..\{{ service_name_pascal }}.Domain\{{ service_name_pascal }}.Domain.csproj" />
  </ItemGroup>

  <ItemGroup>
{% if has_database and database_type == "postgresql" %}
    <PackageReference Include="Npgsql.EntityFrameworkCore.PostgreSQL" />
{% elif has_database and database_type == "mysql" %}
    <PackageReference Include="Pomelo.EntityFrameworkCore.MySql" />
{% endif %}
{% if has_database %}
    <PackageReference Include="Microsoft.EntityFrameworkCore" />
{% endif %}
  </ItemGroup>

</Project>
```

---

## テスト構造

### ユニットテスト

`tests/{{ service_name_pascal }}.UnitTests/{{ service_name_pascal }}ServiceTests.cs.tera` — xUnit + NSubstitute によるユニットテスト。

```csharp
using NSubstitute;
using {{ service_name_pascal }}.Domain.Entities;
{% if has_database %}
using {{ service_name_pascal }}.Domain.Repositories;
{% endif %}
using {{ service_name_pascal }}.UseCase.Services;

namespace {{ service_name_pascal }}.UnitTests;

public class {{ service_name_pascal }}ServiceTests
{
    [Fact]
    public async Task GetByIdAsync_NotFound_ReturnsNull()
    {
{% if has_database %}
        var repo = Substitute.For<I{{ service_name_pascal }}Repository>();
        repo.FindByIdAsync("nonexistent").Returns(({{ service_name_pascal }}Entity?)null);
        var service = new {{ service_name_pascal }}Service(repo);
{% else %}
        var service = new {{ service_name_pascal }}Service();
{% endif %}

        var result = await service.GetByIdAsync("nonexistent");

        Assert.Null(result);
    }

    [Fact]
    public async Task GetAllAsync_ReturnsEmpty()
    {
{% if has_database %}
        var repo = Substitute.For<I{{ service_name_pascal }}Repository>();
        repo.FindAllAsync().Returns(Enumerable.Empty<{{ service_name_pascal }}Entity>());
        var service = new {{ service_name_pascal }}Service(repo);
{% else %}
        var service = new {{ service_name_pascal }}Service();
{% endif %}

        var result = await service.GetAllAsync();

        Assert.Empty(result);
    }
}
```

### テストプロジェクト csproj

`tests/{{ service_name_pascal }}.UnitTests/{{ service_name_pascal }}.UnitTests.csproj.tera`

```xml
<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework>
    <Nullable>enable</Nullable>
    <ImplicitUsings>enable</ImplicitUsings>
    <IsPackable>false</IsPackable>
  </PropertyGroup>

  <ItemGroup>
    <ProjectReference Include="..\..\src\{{ service_name_pascal }}.UseCase\{{ service_name_pascal }}.UseCase.csproj" />
    <ProjectReference Include="..\..\src\{{ service_name_pascal }}.Domain\{{ service_name_pascal }}.Domain.csproj" />
  </ItemGroup>

  <ItemGroup>
    <PackageReference Include="Microsoft.NET.Test.Sdk" />
    <PackageReference Include="xunit" />
    <PackageReference Include="xunit.runner.visualstudio" />
    <PackageReference Include="NSubstitute" />
    <PackageReference Include="coverlet.collector" />
  </ItemGroup>

</Project>
```

### 統合テスト

`tests/{{ service_name_pascal }}.IntegrationTests/{{ service_name_pascal }}ApiTests.cs.tera` — WebApplicationFactory を使用した統合テスト。

```csharp
{% if api_styles is containing("rest") %}
using Microsoft.AspNetCore.Mvc.Testing;
using System.Net;

namespace {{ service_name_pascal }}.IntegrationTests;

public class {{ service_name_pascal }}ApiTests : IClassFixture<WebApplicationFactory<Program>>
{
    private readonly HttpClient _client;

    public {{ service_name_pascal }}ApiTests(WebApplicationFactory<Program> factory)
    {
        _client = factory.CreateClient();
    }

    [Fact]
    public async Task HealthCheck_ReturnsOk()
    {
        var response = await _client.GetAsync("/healthz");
        Assert.Equal(HttpStatusCode.OK, response.StatusCode);
    }

    [Fact]
    public async Task List_ReturnsOk()
    {
        var response = await _client.GetAsync("/api/v1/{{ service_name }}");
        Assert.Equal(HttpStatusCode.OK, response.StatusCode);
    }

    [Fact]
    public async Task GetById_NotFound_Returns404()
    {
        var response = await _client.GetAsync("/api/v1/{{ service_name }}/nonexistent");
        Assert.Equal(HttpStatusCode.NotFound, response.StatusCode);
    }
}
{% endif %}
```

---

## テストツール

| ツール               | 用途                           |
| -------------------- | ------------------------------ |
| xUnit                | テストフレームワーク           |
| NSubstitute          | モックライブラリ               |
| WireMock.Net         | HTTP モックサーバー            |
| coverlet             | コードカバレッジ               |
| StyleCop.Analyzers   | コーディング規約の静的解析     |
| dotnet format        | コードフォーマッター           |

---

## 既存ドキュメント参照マップ

| テンプレートファイル        | 参照ドキュメント                               | 該当セクション                                |
| --------------------------- | ---------------------------------------------- | --------------------------------------------- |
| config/config.yaml          | [config設計.md](config設計.md)                 | YAML スキーマ定義                             |
| Controllers (REST)          | [API設計.md](API設計.md)                       | D-007 エラーレスポンス、D-008 バージョニング  |
| GrpcServices (gRPC)         | [API設計.md](API設計.md)                       | D-009 gRPC 定義パターン                       |
| Dockerfile                  | [Dockerイメージ戦略.md](Dockerイメージ戦略.md) | ベースイメージ・マルチステージビルド          |
| JwtAuthMiddleware           | [認証認可設計.md](認証認可設計.md)             | JWT 認証フロー                                |
| OpenTelemetry               | [可観測性設計.md](可観測性設計.md)             | D-110 分散トレーシング                        |
| テスト                      | [コーディング規約.md](コーディング規約.md)     | テストツール（xUnit, NSubstitute）            |

---

## 関連ドキュメント

- [テンプレート仕様-サーバー](テンプレート仕様-サーバー.md) --- 概要・条件付き生成表・Tier別配置パス
- [テンプレート仕様-サーバー-Rust](テンプレート仕様-サーバー-Rust.md) --- Rust/axum+tokio テンプレート
- [テンプレート仕様-サーバー-可観測性](テンプレート仕様-サーバー-可観測性.md) --- 可観測性テンプレート
- [テンプレート仕様-サーバー-認証](テンプレート仕様-サーバー-認証.md) --- 認証認可Middleware テンプレート
- [テンプレートエンジン仕様](テンプレートエンジン仕様.md) --- 変数置換・条件分岐の仕様
- [API設計](API設計.md) --- REST / gRPC 設計
- [config設計](config設計.md) --- config.yaml スキーマと環境別管理
- [可観測性設計](可観測性設計.md) --- 監視・トレーシング設計
- [認証認可設計](認証認可設計.md) --- 認証・認可・シークレット管理
- [Dockerイメージ戦略](Dockerイメージ戦略.md) --- Docker ビルド戦略
- [コーディング規約](コーディング規約.md) --- Linter・Formatter・テストツール
