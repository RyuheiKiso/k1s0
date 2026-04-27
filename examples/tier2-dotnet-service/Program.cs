// 本ファイルは tier2 .NET サービスの Golden Path 完動例。
// ASP.NET Core minimal API で /sample-write エンドポイントを公開し、
// K1s0.Sdk.Grpc 経由で tier1 State API に書き込むサンプル。

using K1s0.Sdk;

// app.UseRouting() 不要の最小 API を組む。
var builder = WebApplication.CreateBuilder(args);

// k1s0 Client を Singleton として登録する（DI 経由で各 endpoint で参照）。
builder.Services.AddSingleton(_ => new K1s0Client(new K1s0Config
{
    // tier1 facade の接続先（dev: http、prod: https）。
    Target = builder.Configuration.GetValue<string>("K1s0:Target") ?? "http://localhost:50001",
    TenantId = builder.Configuration.GetValue<string>("K1s0:TenantId") ?? "tenant-example",
    Subject = builder.Configuration.GetValue<string>("K1s0:Subject") ?? "tier2-example-dotnet",
}));

// app build。
var app = builder.Build();

// /healthz: 単純疎通。
app.MapGet("/healthz", () => Results.Ok("ok"));

// /readyz: tier1 facade との疎通も含めた健全性確認（リリース時点 は単純）。
app.MapGet("/readyz", () => Results.Ok("ready"));

// /sample-write: tier1 State API への書き込みサンプル（K1s0.Sdk.Grpc 利用デモ）。
app.MapPost("/sample-write", async (K1s0Client client, CancellationToken ct) =>
{
    // valkey-default Store の "tier2-dotnet-example/last-call" キーに current time を書く。
    var data = System.Text.Encoding.UTF8.GetBytes(DateTime.UtcNow.ToString("O"));
    var etag = await client.State.SaveAsync("valkey-default", "tier2-dotnet-example/last-call", data, ct: ct);
    return Results.Ok(new { saved = true, etag });
});

// アプリ起動。
app.Run();
