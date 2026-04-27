// ApprovalFlow Api エントリポイント。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/02_dotnet_solution配置.md
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md

using K1s0.Tier2.ApprovalFlow.Api.Controllers;
using K1s0.Tier2.ApprovalFlow.Application.UseCases;
using K1s0.Tier2.ApprovalFlow.Domain.Interfaces;
using K1s0.Tier2.ApprovalFlow.Infrastructure.Persistence;

// minimal hosting で WebApplication を組み立てる。
var builder = WebApplication.CreateBuilder(args);

// Repository（リリース時点 は in-memory、リリース時点 で k1s0 State / EF Core に切替）。
builder.Services.AddSingleton<IApprovalRepository, InMemoryApprovalRepository>();

// UseCases。
builder.Services.AddScoped<SubmitApprovalUseCase>();
builder.Services.AddScoped<DecideApprovalUseCase>();

// アプリ本体を build する。
var app = builder.Build();

// 健全性プローブ。
app.MapGet("/healthz", () => Results.Ok(new { status = "ok" }));
app.MapGet("/readyz", () => Results.Ok(new { status = "ready" }));

// API エンドポイントを登録する。
app.MapApprovalEndpoints();

// 起動する。
app.Run();
