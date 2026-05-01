// 本ファイルは tier2 .NET 共通の DomainException → JSON 応答変換 middleware。
//
// docs 正典:
//   src/tier2/go/shared/errors/errors.go (Go 側 middleware の writeError と同等の挙動)
//
// 設計動機:
//   各 Api 層 (TaxCalculator / InvoiceGenerator / ApprovalFlow) で
//   try / catch を繰り返さず、`app.UseK1s0DomainException()` 1 行で
//   DomainException を ErrorBody JSON にシリアライズして返せるようにする。
//   非 DomainException は INTERNAL E-T2-INTERNAL として汎用化し、
//   PII 漏洩を防ぐため詳細メッセージは log のみに残す。

using System.Text.Json;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.Logging;

namespace K1s0.Tier2.Common.Errors;

/// <summary>
/// HTTP pipeline で DomainException / 汎用 Exception を捕捉して
/// <see cref="ErrorBody"/> JSON 応答に変換する middleware。
/// </summary>
public sealed partial class DomainExceptionMiddleware
{
    private readonly RequestDelegate _next;
    private readonly ILogger<DomainExceptionMiddleware> _logger;

    /// <summary>middleware を組み立てる。</summary>
    public DomainExceptionMiddleware(RequestDelegate next, ILogger<DomainExceptionMiddleware> logger)
    {
        _next = next;
        _logger = logger;
    }

    /// <summary>
    /// next を実行し、Exception 発生時に ErrorBody JSON へ変換する。
    /// 既に応答が開始済 (Response.HasStarted) の場合は再 throw する
    /// (HTTP プロトコル上の不整合を避けるため)。
    /// </summary>
    public async Task InvokeAsync(HttpContext ctx)
    {
        // CA1062: ASP.NET Core pipeline からは null は来ないが、契約として明示的に弾く。
        ArgumentNullException.ThrowIfNull(ctx);
        try
        {
            await _next(ctx).ConfigureAwait(false);
        }
        catch (DomainException ex)
        {
            if (ctx.Response.HasStarted)
            {
                LogDomainAfterStarted(_logger, ex.Code, ex);
                throw;
            }
            await WriteAsync(ctx, ex.Category.HttpStatus(), new ErrorBody(ex.Code, ex.Message, ex.Category.Wire())).ConfigureAwait(false);
        }
        catch (Exception ex) when (!ctx.Response.HasStarted)
        {
            LogUnhandled(_logger, ex);
            await WriteAsync(ctx, 500, new ErrorBody("E-T2-INTERNAL", "internal error", Category.Internal.Wire())).ConfigureAwait(false);
        }
    }

    private static async Task WriteAsync(HttpContext ctx, int status, ErrorBody body)
    {
        ctx.Response.StatusCode = status;
        ctx.Response.ContentType = "application/json; charset=utf-8";
        await ctx.Response.WriteAsync(JsonSerializer.Serialize(body)).ConfigureAwait(false);
    }

    // CA1848: LoggerMessage source generator で structured logging を効率化する。
    [LoggerMessage(EventId = 1001, Level = LogLevel.Error, Message = "DomainException after response started: {Code}")]
    private static partial void LogDomainAfterStarted(ILogger logger, string code, Exception exception);

    [LoggerMessage(EventId = 1002, Level = LogLevel.Error, Message = "Unhandled exception")]
    private static partial void LogUnhandled(ILogger logger, Exception exception);
}
