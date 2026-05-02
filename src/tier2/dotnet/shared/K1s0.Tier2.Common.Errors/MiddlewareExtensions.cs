// 本ファイルは tier2 .NET 共通の DomainException middleware を pipeline に組み込む拡張。

using Microsoft.AspNetCore.Builder;

namespace K1s0.Tier2.Common.Errors;

/// <summary>
/// IApplicationBuilder の拡張。各 Api 層が
/// `app.UseK1s0DomainException()` 1 行で DomainException → JSON 変換を有効化できる。
/// </summary>
public static class MiddlewareExtensions
{
    /// <summary>
    /// HTTP pipeline の最前段で DomainExceptionMiddleware を実行する。
    /// pipeline の他 middleware より早く try/catch する必要があるため、
    /// 通常は <c>app.UseAuthentication()</c> より前で呼ぶ。
    /// </summary>
    public static IApplicationBuilder UseK1s0DomainException(this IApplicationBuilder app) =>
        app.UseMiddleware<DomainExceptionMiddleware>();
}
