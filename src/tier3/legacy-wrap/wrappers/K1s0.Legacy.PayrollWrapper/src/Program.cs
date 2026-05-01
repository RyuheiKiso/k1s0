// K1s0.Legacy.PayrollWrapper のエントリポイント（ASP.NET Core minimal hosting）。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/05_レガシーラップ配置.md
//
// scope:
//   Sidecar パターンから本 Wrapper パターンへの移行例。Linux container として
//   k1s0 基盤上で動作するため Windows Node 依存が解消される。
//
// 環境変数:
//   K1S0_BFF_URL    - 接続先 BFF URL (例: http://portal-bff.tier3-bff.svc.cluster.local:8080)
//   K1S0_TENANT_ID  - 全 RPC に付与する X-Tenant-Id

using K1s0.Legacy.PayrollWrapper.Services;

// アプリ builder を組み立てる。
var builder = WebApplication.CreateBuilder(args);

// BFF URL とテナント ID を環境変数から取得する（appsettings で上書き可）。
var bffUrl = builder.Configuration["K1S0_BFF_URL"]
    ?? Environment.GetEnvironmentVariable("K1S0_BFF_URL")
    ?? "http://portal-bff.tier3-bff.svc.cluster.local:8080";
var tenantId = builder.Configuration["K1S0_TENANT_ID"]
    ?? Environment.GetEnvironmentVariable("K1S0_TENANT_ID")
    ?? "tenant-dev";

// HttpClient を DI 登録する（Singleton 共有、timeout 10s）。
builder.Services.AddSingleton(_ => new HttpClient { Timeout = TimeSpan.FromSeconds(10) });

// IK1s0SdkAdapter / PayrollService を DI 登録する。
builder.Services.AddSingleton<IK1s0SdkAdapter>(sp =>
    new K1s0SdkAdapter(sp.GetRequiredService<HttpClient>(), bffUrl, tenantId));
builder.Services.AddSingleton<PayrollService>();

// Controller を登録する。
builder.Services.AddControllers();

// アプリを構築する。
var app = builder.Build();

// k8s probe（コントローラと別に最低限を minimal API で）。
app.MapGet("/healthz", () => Results.Ok(new { status = "ok" }));
app.MapGet("/readyz", () => Results.Ok(new { status = "ready" }));

// Controller ルートを登録する。
app.MapControllers();

// 起動。
app.Run();

// テスト容易性のため明示的に partial class を出す（WebApplicationFactory<Program> から参照可能にする）。
public partial class Program { }
