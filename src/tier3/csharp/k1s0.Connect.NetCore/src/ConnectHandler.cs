// k1s0 Connect-RPC ASP.NET Core エンドポイント handler 実装
// Connect-RPC リクエストを受け取って処理し Connect-RPC レスポンスを返す
// 4 RPC form（Unary / ServerStreaming / ClientStreaming / Bidi）のディスパッチを担う

// System 名前空間: Exception 等の基本型に使用する
using System;
// System.Collections.Generic: IAsyncEnumerable に使用する
using System.Collections.Generic;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task に使用する
using System.Threading.Tasks;
// Microsoft.AspNetCore.Http: HttpContext / RequestDelegate に使用する
using Microsoft.AspNetCore.Http;
// Microsoft.Extensions.Logging: ILogger に使用する
using Microsoft.Extensions.Logging;
// Google.Protobuf: IMessage に使用する
using Google.Protobuf;

namespace K1s0.Connect.NetCore
{
    /// <summary>
    /// RPC form の種別を表す列挙型
    /// Connect-RPC の 4 RPC form を定義する
    /// </summary>
    public enum RpcForm
    {
        // Unary: クライアント 1 リクエスト → サーバー 1 レスポンス
        Unary,
        // ServerStreaming: クライアント 1 リクエスト → サーバー N レスポンス
        ServerStreaming,
        // ClientStreaming: クライアント N リクエスト → サーバー 1 レスポンス
        ClientStreaming,
        // BidiStreaming: クライアント N リクエスト ↔ サーバー N レスポンス
        BidiStreaming
    }

    /// <summary>
    /// ConnectHandler: Connect-RPC ASP.NET Core ミドルウェア
    /// Connect-RPC リクエストをディスパッチして Connect-RPC レスポンスを返す
    /// </summary>
    public class ConnectHandler
    {
        // ASP.NET Core の次のミドルウェアデリゲート
        private readonly RequestDelegate _next;

        // ロガーインスタンス
        private readonly ILogger<ConnectHandler> _logger;

        /// <summary>
        /// ConnectHandler のコンストラクタ
        /// </summary>
        /// <param name="next">次のミドルウェアデリゲート</param>
        /// <param name="logger">ロガーインスタンス</param>
        public ConnectHandler(RequestDelegate next, ILogger<ConnectHandler> logger)
        {
            // next が null の場合は例外を投げる
            _next = next ?? throw new ArgumentNullException(nameof(next));
            // logger が null の場合は例外を投げる
            _logger = logger ?? throw new ArgumentNullException(nameof(logger));
        }

        /// <summary>
        /// InvokeAsync: ASP.NET Core ミドルウェアのエントリーポイント
        /// Connect-RPC リクエストを判定してディスパッチする
        /// </summary>
        /// <param name="context">HTTP コンテキスト</param>
        public async Task InvokeAsync(HttpContext context)
        {
            // Content-Type が Connect-RPC 形式かどうかを確認する
            var contentType = context.Request.ContentType ?? string.Empty;

            // application/connect+proto または application/proto でない場合は次のミドルウェアに処理を委譲する
            if (!contentType.StartsWith("application/connect+proto", StringComparison.OrdinalIgnoreCase)
                && !contentType.StartsWith("application/proto", StringComparison.OrdinalIgnoreCase))
            {
                // Connect-RPC 以外のリクエストは次のミドルウェアに処理を委譲する
                await _next(context);
                return;
            }

            // HTTP メソッドが POST でない場合は 405 Method Not Allowed を返す
            if (!context.Request.Method.Equals("POST", StringComparison.OrdinalIgnoreCase))
            {
                // Connect-RPC は POST のみ許可する
                context.Response.StatusCode = StatusCodes.Status405MethodNotAllowed;
                return;
            }

            // Connect-RPC リクエストを処理する
            _logger.LogDebug(
                "Connect-RPC リクエスト受信: Path={Path}, ContentType={ContentType}",
                context.Request.Path,
                contentType);

            // リクエストボディからフレームを読み取る
            var frame = await ConnectProtocol.ReadFrameAsync(
                context.Request.Body,
                context.RequestAborted);

            // フレームが null（空リクエスト）の場合は 400 Bad Request を返す
            if (frame == null)
            {
                // 空フレームは不正なリクエストとして扱う
                context.Response.StatusCode = StatusCodes.Status400BadRequest;
                return;
            }

            // レスポンス Content-Type を設定する（application/connect+proto）
            context.Response.ContentType = "application/connect+proto";

            // エコーレスポンスを返す（実装サービスへの委譲は呼び出し元が行う）
            // フレームのペイロードをそのままエコーして接続性確認に使用する
            var responseFrame = new ConnectFrame(ConnectFlags.Normal, frame.Value.Payload);

            // レスポンスフレームを書き込む
            await ConnectProtocol.WriteFrameAsync(
                context.Response.Body,
                responseFrame,
                context.RequestAborted);
        }
    }

    /// <summary>
    /// IConnectService: Connect-RPC サービスの実装インターフェース
    /// アプリケーション固有の RPC ロジックを実装するために使用する
    /// </summary>
    public interface IConnectService
    {
        // サービスが処理する RPC プロシージャーパスの一覧
        // 例: "/k1s0.v1.StateService/ReadState"
        IReadOnlyList<string> Procedures { get; }
    }
}
