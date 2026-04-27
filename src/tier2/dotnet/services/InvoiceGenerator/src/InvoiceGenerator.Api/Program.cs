// InvoiceGenerator Api エントリポイント。

using K1s0.Tier2.InvoiceGenerator.Api.Controllers;
using K1s0.Tier2.InvoiceGenerator.Application.UseCases;
using K1s0.Tier2.InvoiceGenerator.Domain.Interfaces;
using K1s0.Tier2.InvoiceGenerator.Infrastructure.Persistence;

var builder = WebApplication.CreateBuilder(args);

// Repository（リリース時点 in-memory）。
builder.Services.AddSingleton<IInvoiceRepository, InMemoryInvoiceRepository>();
// UseCase。
builder.Services.AddScoped<GenerateInvoiceUseCase>();

var app = builder.Build();

app.MapGet("/healthz", () => Results.Ok(new { status = "ok" }));
app.MapGet("/readyz", () => Results.Ok(new { status = "ready" }));
app.MapInvoiceEndpoints();

app.Run();
