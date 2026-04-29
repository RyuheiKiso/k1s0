// TaxCalculator Api エントリポイント。

using K1s0.Tier2.Common.Auth;
using K1s0.Tier2.TaxCalculator.Api.Controllers;
using K1s0.Tier2.TaxCalculator.Application.UseCases;

var builder = WebApplication.CreateBuilder(args);
// UseCase を DI に登録する。
builder.Services.AddSingleton<CalculateTaxUseCase>();
// docs §共通規約「認証認可」: T2_AUTH_MODE 環境変数で切替。
builder.Services.AddK1s0JwtBearer();

var app = builder.Build();

// Authentication / Authorization middleware（順序固定）。
app.UseAuthentication();
app.UseAuthorization();

app.MapGet("/healthz", () => Results.Ok(new { status = "ok" })).AllowAnonymous();
app.MapGet("/readyz", () => Results.Ok(new { status = "ready" })).AllowAnonymous();
app.MapTaxEndpoints();

app.Run();
