// k1s0 Companion CLR Profiler Hook インターフェース定義
// ICorProfilerCallback2 の Managed ラッパーとして CLR profiler hook インターフェースを定義する
// 実際の IL rewrite は CLR unmanaged profiler DLL が行い、本プロジェクトは managed side の定義のみを提供する
// CLR Profiling API: https://docs.microsoft.com/en-us/dotnet/framework/unmanaged-api/profiling/

// System 名前空間: IntPtr / Exception 等の基本型に使用する
using System;
// System.Diagnostics: ActivitySource / Activity の操作に使用する
using System.Diagnostics;
// System.Runtime.InteropServices: COM Interop / GUID 定義に使用する
using System.Runtime.InteropServices;

namespace K1s0.Companion.NetFx.OTelExt
{
    /// <summary>
    /// IK1s0ProfilerCallback インターフェース
    /// CLR ICorProfilerCallback2 の managed side hook を定義する
    /// 実際の unmanaged CLR profiler は別途の native DLL として実装する
    /// 本インターフェースは profiler イベントを managed コードから受け取るための定義のみを提供する
    /// </summary>
    public interface IK1s0ProfilerCallback
    {
        /// <summary>
        /// CLR JIT コンパイル開始イベント
        /// メソッドが JIT コンパイルされる直前に呼び出される
        /// </summary>
        /// <param name="functionId">JIT コンパイル対象のメソッド ID（CLR unmanaged API の FunctionID 相当）</param>
        void OnJitCompilationStarted(ulong functionId);

        /// <summary>
        /// CLR JIT コンパイル完了イベント
        /// メソッドの JIT コンパイルが完了した直後に呼び出される
        /// </summary>
        /// <param name="functionId">JIT コンパイルが完了したメソッド ID</param>
        /// <param name="compilationSucceeded">JIT コンパイルが成功した場合 true</param>
        void OnJitCompilationFinished(ulong functionId, bool compilationSucceeded);

        /// <summary>
        /// CLR スレッド生成イベント
        /// 新しいマネージドスレッドが生成された時に呼び出される
        /// </summary>
        /// <param name="threadId">生成されたスレッドの ID（CLR ThreadID 相当）</param>
        void OnThreadCreated(ulong threadId);

        /// <summary>
        /// CLR スレッド破棄イベント
        /// マネージドスレッドが破棄される直前に呼び出される
        /// </summary>
        /// <param name="threadId">破棄されるスレッドの ID（CLR ThreadID 相当）</param>
        void OnThreadDestroyed(ulong threadId);

        /// <summary>
        /// HTTP リクエスト開始イベント（HttpWebRequest / WebClient / HttpClient 対応）
        /// HTTP スタックがリクエストを送信する直前に呼び出される（IL rewrite による）
        /// </summary>
        /// <param name="requestUrl">リクエスト先 URL 文字列</param>
        /// <param name="httpMethod">HTTP メソッド名（GET / POST 等）</param>
        void OnHttpRequestStart(string requestUrl, string httpMethod);

        /// <summary>
        /// HTTP レスポンス完了イベント
        /// HTTP スタックがレスポンスを受信した直後に呼び出される（IL rewrite による）
        /// </summary>
        /// <param name="requestUrl">リクエスト先 URL 文字列</param>
        /// <param name="statusCode">HTTP ステータスコード（200 / 404 等）</param>
        void OnHttpResponseEnd(string requestUrl, int statusCode);
    }

    /// <summary>
    /// K1s0ProfilerHooks: IK1s0ProfilerCallback のデフォルト実装
    /// OTel ActivitySource を使って CLR profiler イベントを OpenTelemetry スパンとして記録する
    /// </summary>
    public class K1s0ProfilerHooks : IK1s0ProfilerCallback
    {
        // CLR profiler hook の ActivitySource インスタンス
        // K1s0ActivitySources.ProfilerSource で定義した名前を使用する
        private static readonly ActivitySource _activitySource =
            new ActivitySource(K1s0ActivitySources.ProfilerSource);

        // JIT コンパイル中スパンを保持する Dictionary（functionId → Activity）
        // スレッドセーフのため ConcurrentDictionary を使用する
        private readonly System.Collections.Concurrent.ConcurrentDictionary<ulong, Activity?> _jitActivities =
            new System.Collections.Concurrent.ConcurrentDictionary<ulong, Activity?>();

        /// <summary>
        /// JIT コンパイル開始イベントのハンドラー実装
        /// OTel スパンを開始して _jitActivities に記録する
        /// </summary>
        /// <param name="functionId">JIT 対象のメソッド ID</param>
        public void OnJitCompilationStarted(ulong functionId)
        {
            // JIT コンパイル開始スパンを開始する
            var activity = _activitySource.StartActivity("clr.jit.compilation");
            // activity が null でない場合（OTel が有効な場合）のみ属性を設定する
            if (activity != null)
            {
                // function_id 属性を設定する（CLR FunctionID を 16 進数で記録する）
                activity.SetTag("clr.function_id", functionId.ToString("X16"));
            }
            // functionId をキーとして Activity を記録する
            _jitActivities[functionId] = activity;
        }

        /// <summary>
        /// JIT コンパイル完了イベントのハンドラー実装
        /// 開始済みのスパンに結果を記録して終了する
        /// </summary>
        /// <param name="functionId">JIT 完了のメソッド ID</param>
        /// <param name="compilationSucceeded">コンパイル成功フラグ</param>
        public void OnJitCompilationFinished(ulong functionId, bool compilationSucceeded)
        {
            // 対応する JIT スパンを取得する
            if (_jitActivities.TryRemove(functionId, out var activity) && activity != null)
            {
                // コンパイル結果を属性として設定する
                activity.SetTag("clr.jit.succeeded", compilationSucceeded.ToString().ToLowerInvariant());
                // コンパイル失敗の場合はエラーステータスを設定する
                if (!compilationSucceeded)
                {
                    // JIT コンパイル失敗をエラーとして記録する
                    activity.SetStatus(ActivityStatusCode.Error, "JIT compilation failed");
                }
                // スパンを終了する
                activity.Dispose();
            }
        }

        /// <summary>
        /// スレッド生成イベントのハンドラー実装
        /// thread.created イベントを OTel に記録する
        /// </summary>
        /// <param name="threadId">生成されたスレッド ID</param>
        public void OnThreadCreated(ulong threadId)
        {
            // スレッド生成イベントのスパンを開始して即座に終了する（点イベント）
            using var activity = _activitySource.StartActivity("clr.thread.created");
            // activity が null でない場合のみ属性を設定する
            if (activity != null)
            {
                // thread_id 属性を設定する
                activity.SetTag("clr.thread_id", threadId.ToString("X16"));
            }
        }

        /// <summary>
        /// スレッド破棄イベントのハンドラー実装
        /// thread.destroyed イベントを OTel に記録する
        /// </summary>
        /// <param name="threadId">破棄されるスレッド ID</param>
        public void OnThreadDestroyed(ulong threadId)
        {
            // スレッド破棄イベントのスパンを開始して即座に終了する（点イベント）
            using var activity = _activitySource.StartActivity("clr.thread.destroyed");
            // activity が null でない場合のみ属性を設定する
            if (activity != null)
            {
                // thread_id 属性を設定する
                activity.SetTag("clr.thread_id", threadId.ToString("X16"));
            }
        }

        /// <summary>
        /// HTTP リクエスト開始イベントのハンドラー実装
        /// http.request.start スパンを開始する
        /// </summary>
        /// <param name="requestUrl">リクエスト先 URL</param>
        /// <param name="httpMethod">HTTP メソッド名</param>
        public void OnHttpRequestStart(string requestUrl, string httpMethod)
        {
            // HTTP リクエスト開始スパンを開始する（終了は OnHttpResponseEnd で行う）
            var activity = _activitySource.StartActivity($"{httpMethod} {requestUrl}");
            // activity が null でない場合のみ属性を設定する
            if (activity != null)
            {
                // HTTP メソッド属性を OpenTelemetry semantic conventions に従い設定する
                activity.SetTag("http.method", httpMethod);
                // リクエスト URL 属性を設定する
                activity.SetTag("http.url", requestUrl);
                // profiler hook 経由での計装であることを記録する
                activity.SetTag("k1s0.instrumentation", "clr_profiler_hook");
            }
        }

        /// <summary>
        /// HTTP レスポンス完了イベントのハンドラー実装
        /// 対応するスパンにステータスコードを記録して終了する
        /// </summary>
        /// <param name="requestUrl">リクエスト先 URL（スパン特定に使用する）</param>
        /// <param name="statusCode">HTTP ステータスコード</param>
        public void OnHttpResponseEnd(string requestUrl, int statusCode)
        {
            // 現在の Activity（OnHttpRequestStart で開始したスパン）を取得する
            var activity = Activity.Current;
            // activity が null でない場合のみ属性を設定して終了する
            if (activity != null)
            {
                // HTTP ステータスコード属性を設定する
                activity.SetTag("http.status_code", statusCode.ToString());
                // 4xx / 5xx の場合はエラーステータスを設定する
                if (statusCode >= 400)
                {
                    // HTTP エラーとしてスパンステータスを設定する
                    activity.SetStatus(ActivityStatusCode.Error, $"HTTP {statusCode}");
                }
                // スパンを終了する
                activity.Dispose();
            }
        }
    }

    /// <summary>
    /// CLR Profiler GUID 定義
    /// CLR ICorProfilerCallback2 の COM GUID を定義する（unmanaged profiler の GUID と対応させる）
    /// </summary>
    public static class ProfilerGuids
    {
        // k1s0 CLR Profiler の COM CLSID（unmanaged profiler DLL に設定する GUID）
        // この GUID は k1s0 内部で固定値として使用する
        public static readonly Guid K1s0ProfilerClsid =
            new Guid("A1B2C3D4-E5F6-7890-ABCD-EF1234567890");

        // ICorProfilerCallback2 インターフェース IID（CLR が定義する固定 GUID）
        // CLR Profiling API ドキュメントの値を使用する
        public static readonly Guid ICorProfilerCallback2Iid =
            new Guid("8A8CC829-CCF2-49fe-BBAE-0F022228071A");
    }
}
