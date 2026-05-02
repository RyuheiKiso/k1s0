// ApprovalFlow Api エントリポイント。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/02_dotnet_solution配置.md
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md

using K1s0.Tier2.ApprovalFlow.Api.Controllers;
using K1s0.Tier2.ApprovalFlow.Application.UseCases;
using K1s0.Tier2.ApprovalFlow.Domain.Interfaces;
using K1s0.Tier2.ApprovalFlow.Infrastructure.Persistence;
using K1s0.Tier2.Common.Auth;
using K1s0.Tier2.Common.Errors;
using K1s0.Tier2.Common.Otel;

// minimal hosting で WebApplication を組み立てる。
var builder = WebApplication.CreateBuilder(args);

// Repository（リリース時点 は in-memory、リリース時点 で k1s0 State / EF Core に切替）。
builder.Services.AddSingleton<IApprovalRepository, InMemoryApprovalRepository>();

// UseCases。
builder.Services.AddScoped<SubmitApprovalUseCase>();
builder.Services.AddScoped<DecideApprovalUseCase>();

// OTel 初期化 (OTEL_EXPORTER_OTLP_ENDPOINT 環境変数で OTLP gRPC を有効化)。
builder.Services.AddK1s0Otel("approval-flow", "0.1.0", Environment.GetEnvironmentVariable("ASPNETCORE_ENVIRONMENT") ?? "dev");

// docs §共通規約「認証認可」: T2_AUTH_MODE 環境変数（off / hmac / jwks）に従って
// JWT Bearer 認証を有効化する。tier1 / tier2 Go / tier3 BFF と同等の検証強度。
builder.Services.AddK1s0JwtBearer();

// アプリ本体を build する。
var app = builder.Build();

// DomainException → ErrorBody JSON 変換 (AuthN/Z より前段で例外捕捉)。
app.UseK1s0DomainException();

// 認証 / 認可 middleware を pipeline に組み込む（順序: Authentication → Authorization）。
app.UseAuthentication();
app.UseAuthorization();

// 健全性プローブ（認証不要）。
app.MapGet("/healthz", () => Results.Ok(new { status = "ok" })).AllowAnonymous();
app.MapGet("/readyz", () => Results.Ok(new { status = "ready" })).AllowAnonymous();

// API エンドポイントを登録（MapApprovalEndpoints 側で .RequireAuthorization() 付与）。
app.MapApprovalEndpoints();

// 起動する。
app.Run();
