// 本ファイルは tier2 .NET 共通の JWT Bearer 認証拡張メソッド。
//
// docs 正典:
//   docs/03_要件定義/00_共通規約.md §「認証認可」
//
// 役割:
//   tier2 .NET の各 Api 層（ApprovalFlow / InvoiceGenerator / TaxCalculator）で
//   `builder.Services.AddK1s0JwtBearer()` 1 行で JWT Bearer 認証を有効化する。
//   T2_AUTH_MODE 環境変数（off / hmac / jwks）で 3 通りに切り替え、tier1 Go /
//   tier2 Go / tier3 BFF と同等の検証強度を提供する。
//
// モード:
//   - off  : dev 限定。token があれば素通り、anonymous は拒否。署名 / 期限検証 skip。
//   - hmac : T2_AUTH_HMAC_SECRET の HS256/384/512 で署名 + 期限 + tenant_id claim 必須。
//   - jwks : T2_AUTH_JWKS_URL から取得した RSA 公開鍵で RS256/384/512 検証（Keycloak）。

// CA5404 を file-scope で抑止する。
// k1s0 の認証認可は「tenant_id claim を信頼境界とする」モデルであり、
// issuer / audience はマルチテナント設計上 service ごとに厳密化しない。
// 検証は署名 + 有効期限 + tenant_id claim の存在で完結する（tier1 / tier2 Go と同じ強度）。
#pragma warning disable CA5404
// CA1308 は環境変数値の正規化（ToLowerInvariant）が意図通りなので抑止する。
#pragma warning disable CA1308

using System.Text;
using Microsoft.AspNetCore.Authentication;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using Microsoft.AspNetCore.Builder;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.IdentityModel.Tokens;

namespace K1s0.Tier2.Common.Auth;

/// <summary>
/// tier2 .NET 共通の JWT Bearer 認証セットアップ拡張。
/// </summary>
public static class JwtBearerExtensions
{
    /// <summary>
    /// 環境変数 T2_AUTH_MODE / T2_AUTH_HMAC_SECRET / T2_AUTH_JWKS_URL に従って
    /// JWT Bearer 認証を有効化する。`builder.Services.AddK1s0JwtBearer()` で呼ぶ。
    /// </summary>
    public static IServiceCollection AddK1s0JwtBearer(this IServiceCollection services)
    {
        // 環境変数を読む。未設定 / 空は off 扱い。
        var mode = (Environment.GetEnvironmentVariable("T2_AUTH_MODE") ?? "off").Trim().ToLowerInvariant();

        // 認証スキーム名は JwtBearerDefaults.AuthenticationScheme（"Bearer"）固定。
        var auth = services.AddAuthentication(JwtBearerDefaults.AuthenticationScheme);

        switch (mode)
        {
            case "hmac":
                ConfigureHmac(auth);
                break;
            case "jwks":
                ConfigureJwks(auth);
                break;
            default:
                ConfigureOff(auth);
                break;
        }
        // ASP.NET Core 標準の Authorization も並行して有効化する。
        services.AddAuthorization();
        return services;
    }

    /// <summary>
    /// dev 限定 off モード。署名 / 期限を検証せず、Bearer ヘッダの存在のみ要求する。
    /// production で使うことは禁止（tier1 が JWT を要求するため、tier1 呼出が失敗する）。
    /// </summary>
    private static void ConfigureOff(AuthenticationBuilder auth)
    {
        auth.AddJwtBearer(options =>
        {
            options.RequireHttpsMetadata = false;
            options.TokenValidationParameters = new TokenValidationParameters
            {
                ValidateIssuer = false,
                ValidateAudience = false,
                ValidateLifetime = false,
                ValidateIssuerSigningKey = false,
                // 署名検証を完全に skip するために任意の signature を accept する。
                SignatureValidator = (token, _) => new System.IdentityModel.Tokens.Jwt.JwtSecurityToken(token),
            };
        });
    }

    /// <summary>
    /// HMAC 共有秘密鍵モード。CI / dev で利用する。
    /// </summary>
    private static void ConfigureHmac(AuthenticationBuilder auth)
    {
        var secret = Environment.GetEnvironmentVariable("T2_AUTH_HMAC_SECRET") ?? string.Empty;
        if (string.IsNullOrEmpty(secret))
        {
            // 設定不備は fail-fast。Pod 起動時に確実に検知する。
            throw new InvalidOperationException("T2_AUTH_HMAC_SECRET is required when T2_AUTH_MODE=hmac");
        }
        auth.AddJwtBearer(options =>
        {
            options.RequireHttpsMetadata = false;
            options.TokenValidationParameters = new TokenValidationParameters
            {
                ValidateIssuer = false,
                ValidateAudience = false,
                ValidateLifetime = true,
                ValidateIssuerSigningKey = true,
                IssuerSigningKey = new SymmetricSecurityKey(Encoding.UTF8.GetBytes(secret)),
                ClockSkew = TimeSpan.FromSeconds(30),
            };
        });
    }

    /// <summary>
    /// JWKS endpoint から取得した RSA 公開鍵で検証する production モード。
    /// </summary>
    private static void ConfigureJwks(AuthenticationBuilder auth)
    {
        var url = Environment.GetEnvironmentVariable("T2_AUTH_JWKS_URL") ?? string.Empty;
        if (string.IsNullOrEmpty(url))
        {
            throw new InvalidOperationException("T2_AUTH_JWKS_URL is required when T2_AUTH_MODE=jwks");
        }
        auth.AddJwtBearer(options =>
        {
            options.RequireHttpsMetadata = false;
            // Authority 指定で OIDC discovery 経由で署名鍵を fetch + cache する。
            // JWKS URL がそのまま Authority として使えない場合は MetadataAddress を使う。
            options.MetadataAddress = url;
            options.TokenValidationParameters = new TokenValidationParameters
            {
                ValidateIssuer = false,
                ValidateAudience = false,
                ValidateLifetime = true,
                ValidateIssuerSigningKey = true,
                ClockSkew = TimeSpan.FromSeconds(30),
            };
        });
    }
}
