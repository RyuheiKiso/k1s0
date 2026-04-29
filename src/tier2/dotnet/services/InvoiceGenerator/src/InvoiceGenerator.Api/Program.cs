// InvoiceGenerator Api エントリポイント。

using K1s0.Tier2.Common.Auth;
using K1s0.Tier2.InvoiceGenerator.Api.Controllers;
using K1s0.Tier2.InvoiceGenerator.Application.UseCases;
using K1s0.Tier2.InvoiceGenerator.Domain.Interfaces;
using K1s0.Tier2.InvoiceGenerator.Infrastructure.Persistence;

var builder = WebApplication.CreateBuilder(args);

// Repository（リリース時点 in-memory）。
builder.Services.AddSingleton<IInvoiceRepository, InMemoryInvoiceRepository>();
// UseCase。
builder.Services.AddScoped<GenerateInvoiceUseCase>();
// docs §共通規約「認証認可」: T2_AUTH_MODE 環境変数で切替。
builder.Services.AddK1s0JwtBearer();

var app = builder.Build();

// Authentication / Authorization middleware（順序固定）。
app.UseAuthentication();
app.UseAuthorization();

app.MapGet("/healthz", () => Results.Ok(new { status = "ok" })).AllowAnonymous();
app.MapGet("/readyz", () => Results.Ok(new { status = "ready" })).AllowAnonymous();
app.MapInvoiceEndpoints();

app.Run();
