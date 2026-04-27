// TaxCalculator Api エントリポイント。

using K1s0.Tier2.TaxCalculator.Api.Controllers;
using K1s0.Tier2.TaxCalculator.Application.UseCases;

var builder = WebApplication.CreateBuilder(args);
// UseCase を DI に登録する。
builder.Services.AddSingleton<CalculateTaxUseCase>();

var app = builder.Build();

app.MapGet("/healthz", () => Results.Ok(new { status = "ok" }));
app.MapGet("/readyz", () => Results.Ok(new { status = "ready" }));
app.MapTaxEndpoints();

app.Run();
