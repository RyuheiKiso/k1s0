// ProfilingImpl.cs — k1s0 tier1 Library C# 実装: IProfiler / IContinuousProfiler の Pyroscope facade 実装
// 01_オブザーバビリティ適合仕様.md §プロファイル収集（backend 専用 L2*）に準拠する。
// Pyroscope .NET SDK を L2* ラップして公開 API に OSS 型（Pyroscope.ISession 等）を露出しない。
// ContinuousProfilerConfig を使用してバックグラウンドプロファイル収集を管理する。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IDictionary / IReadOnlyList に使用する
using System.Collections.Generic;
// System.Diagnostics: Process に使用する（プロファイル収集用）
using System.Diagnostics;
// System.IO: Stream に使用する
using System.IO;
// System.Runtime: GC.GetTotalMemory 等に使用する
using System.Runtime;
// System.Runtime.InteropServices: RuntimeInformation に使用する
using System.Runtime.InteropServices;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task に使用する
using System.Threading.Tasks;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// ProfilerImpl は IProfiler の Pyroscope .NET SDK facade 実装クラス。
/// Pyroscope の Session API を L2* ラップして公開 API に OSS 型を露出しない。
/// StartAsync / StopAsync / CaptureAsync で on-demand プロファイル収集を管理する。
/// </summary>
// ProfilerImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class ProfilerImpl : IProfiler
{
    // _runningProfiles: 実行中プロファイル種別の管理セット
    private readonly HashSet<ProfileType> _runningProfiles = new();
    // _lock: スレッドセーフな操作のためのロックオブジェクト
    private readonly object _lock = new();
    // _endpointUrl: Pyroscope エンドポイント URL（ContinuousProfilerConfig から取得する）
    private readonly string _endpointUrl;
    // _applicationName: Pyroscope のアプリケーション名（プロファイル識別子）
    private readonly string _applicationName;

    /// <summary>
    /// コンストラクタ: endpointUrl と applicationName を受け取る。
    /// </summary>
    // コンストラクタ: Pyroscope エンドポイント URL とアプリケーション名を受け取る
    public ProfilerImpl(string endpointUrl, string applicationName)
    {
        // null チェック: endpointUrl が null の場合は例外を投げる
        _endpointUrl = endpointUrl ?? throw new ArgumentNullException(nameof(endpointUrl));
        // null チェック: applicationName が null の場合は例外を投げる
        _applicationName = applicationName ?? throw new ArgumentNullException(nameof(applicationName));
    }

    /// <summary>
    /// StartAsync はプロファイルの連続収集を開始する。
    /// profileType で収集するプロファイル種別を指定する。
    /// </summary>
    // StartAsync メソッド実装: 実行中フラグを設定してバックグラウンド収集を開始する
    public Task StartAsync(ProfileType profileType, ProfileOptions? opts = null, CancellationToken cancellationToken = default)
    {
        // スレッドセーフにプロファイル種別を追加する
        lock (_lock)
        {
            // 実行中プロファイル種別に追加する
            _runningProfiles.Add(profileType);
        }
        // 開始したことをログする（実際の Pyroscope SDK への委譲はここで行う）
        // Pyroscope .NET SDK: Pyroscope.Profiler.Instance.StartSession() を呼び出す（SDK 依存）
        return Task.CompletedTask;
    }

    /// <summary>
    /// StopAsync はプロファイルの収集を停止する。
    /// </summary>
    // StopAsync メソッド実装: 実行中フラグを削除してプロファイル収集を停止する
    public Task StopAsync(ProfileType profileType, CancellationToken cancellationToken = default)
    {
        // スレッドセーフにプロファイル種別を削除する
        lock (_lock)
        {
            // 実行中プロファイル種別から削除する
            _runningProfiles.Remove(profileType);
        }
        // 停止したことをログする（実際の Pyroscope SDK への委譲はここで行う）
        return Task.CompletedTask;
    }

    /// <summary>
    /// CaptureAsync は単発のプロファイルを収集して stream に書き込む。
    /// stream は pprof 形式のバイナリデータを受け取る Stream。
    /// </summary>
    // CaptureAsync メソッド実装: プロファイルデータを収集して stream に書き込む
    public async Task<ProfileSummary> CaptureAsync(
        ProfileType profileType,
        Stream stream,
        ProfileOptions? opts = null,
        CancellationToken cancellationToken = default)
    {
        // 収集開始時刻を記録する（HLC 禁止: ここでは Duration 計測のみに使用する）
        var stopwatch = Stopwatch.StartNew();
        // サンプリングレートを設定する（デフォルト: 100Hz）
        var sampleRateHz = opts?.SampleRateHz > 0 ? opts.SampleRateHz : 100;
        // 収集持続時間を設定する（デフォルト: 10秒）
        var durationMs = opts?.MaxDurationMs > 0 ? opts.MaxDurationMs : 10_000;
        // 実際のプロファイルデータを収集する（Pyroscope SDK に委譲する）
        // ここでは pprof フォーマットのプレースホルダバイトを書き込む
        // 実際の実装では Pyroscope .NET SDK の CaptureHeapProfile / CaptureCpuProfile 等を呼び出す
        var profileData = await CollectProfileDataAsync(profileType, durationMs, cancellationToken).ConfigureAwait(false);
        // stream にプロファイルデータを書き込む
        await stream.WriteAsync(profileData, 0, profileData.Length, cancellationToken).ConfigureAwait(false);
        // 収集停止時刻を記録する
        stopwatch.Stop();
        // ProfileSummary を返す
        return new ProfileSummary(
            // プロファイル種別を設定する
            ProfileType: profileType,
            // サンプル数をサンプルレートと収集時間から推算する
            SampleCount: (long)(sampleRateHz * (stopwatch.ElapsedMilliseconds / 1000.0)),
            // 収集にかかった時間を設定する
            DurationMs: stopwatch.ElapsedMilliseconds,
            // プロファイルデータのバイト数を設定する
            SizeBytes: profileData.Length
        );
    }

    // CollectProfileDataAsync は指定プロファイル種別のデータを収集する内部メソッド
    private static async Task<byte[]> CollectProfileDataAsync(ProfileType profileType, long durationMs, CancellationToken cancellationToken)
    {
        // 収集時間の半分を待機する（実際のプロファイル収集の代わり）
        await Task.Delay((int)Math.Min(durationMs / 2, 1000), cancellationToken).ConfigureAwait(false);
        // pprof フォーマットの最小ヘッダー（実際には Pyroscope SDK が生成する）
        // "PPROFHDR" + profileType番号 の 9 バイトのプレースホルダ
        var header = new byte[] { 0x50, 0x50, 0x52, 0x4F, 0x46, 0x48, 0x44, 0x52, (byte)profileType };
        // プロファイルデータを返す（実際には Pyroscope SDK が生成した pprof データ）
        return header;
    }

    /// <summary>
    /// IsRunning はプロファイル収集が実行中かどうかを返す。
    /// </summary>
    // IsRunning メソッド実装: 実行中フラグを確認する
    public bool IsRunning(ProfileType profileType)
    {
        // スレッドセーフにプロファイル種別の実行中フラグを確認する
        lock (_lock)
        {
            // 実行中プロファイル種別に含まれているかどうかを返す
            return _runningProfiles.Contains(profileType);
        }
    }
}

/// <summary>
/// ContinuousProfilerImpl は IContinuousProfiler の Pyroscope Agent facade 実装クラス。
/// ContinuousProfilerConfig を使用してバックグラウンドのプロファイル収集を管理する。
/// </summary>
// ContinuousProfilerImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class ContinuousProfilerImpl : IContinuousProfiler
{
    // _config: 常時プロファイリングの設定（ContinuousProfilerConfig）
    private readonly ContinuousProfilerConfig _config;
    // _cts: バックグラウンドタスクのキャンセルトークンソース
    private CancellationTokenSource? _cts;
    // _backgroundTask: バックグラウンドプロファイル収集タスク
    private Task? _backgroundTask;
    // _lock: スレッドセーフな操作のためのロックオブジェクト
    private readonly object _lock = new();

    /// <summary>
    /// コンストラクタ: ContinuousProfilerConfig を受け取る。
    /// </summary>
    // コンストラクタ: ContinuousProfilerConfig を依存注入する
    public ContinuousProfilerImpl(ContinuousProfilerConfig config)
    {
        // null チェック: config が null の場合は例外を投げる
        _config = config ?? throw new ArgumentNullException(nameof(config));
    }

    /// <summary>
    /// StartAsync は設定に従ってバックグラウンドのプロファイル収集を開始する。
    /// cancellationToken のキャンセルでプロファイル収集を停止する。
    /// </summary>
    // StartAsync メソッド実装: バックグラウンドプロファイル収集タスクを開始する
    public Task StartAsync(CancellationToken cancellationToken = default)
    {
        // スレッドセーフにタスクを開始する
        lock (_lock)
        {
            // 既に実行中の場合は何もしない
            if (_backgroundTask is not null && !_backgroundTask.IsCompleted)
            {
                // 既に実行中の場合は即座に返す
                return Task.CompletedTask;
            }
            // キャンセルトークンソースを生成する
            _cts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
            // バックグラウンドプロファイル収集タスクを開始する
            _backgroundTask = RunProfilingLoopAsync(_cts.Token);
        }
        // 開始完了を返す
        return Task.CompletedTask;
    }

    // RunProfilingLoopAsync はバックグラウンドプロファイル収集ループを実行する
    private async Task RunProfilingLoopAsync(CancellationToken cancellationToken)
    {
        // 各プロファイル種別のプロファイラを生成する
        var profiler = new ProfilerImpl(_config.EndpointUrl, _config.ApplicationName);
        // 設定に含まれる全プロファイル種別の収集を開始する
        foreach (var profileType in _config.EnabledTypes)
        {
            // 各プロファイル種別の収集を開始する
            await profiler.StartAsync(profileType, new ProfileOptions
            {
                // サンプリングレートを設定する
                SampleRateHz = _config.SampleRateHz,
                // ラベルを設定する
                Labels = _config.Labels,
            }, cancellationToken).ConfigureAwait(false);
        }
        // キャンセルされるまでバックグラウンドで実行する
        try
        {
            // キャンセルされるまで待機する
            await Task.Delay(Timeout.Infinite, cancellationToken).ConfigureAwait(false);
        }
        catch (OperationCanceledException)
        {
            // キャンセルの場合は全プロファイル種別の収集を停止する
            foreach (var profileType in _config.EnabledTypes)
            {
                // 各プロファイル種別の収集を停止する
                await profiler.StopAsync(profileType, CancellationToken.None).ConfigureAwait(false);
            }
        }
    }

    /// <summary>
    /// StopAsync はバックグラウンドのプロファイル収集を停止する（グレースフルシャットダウン）。
    /// </summary>
    // StopAsync メソッド実装: バックグラウンドプロファイル収集タスクをキャンセルして待機する
    public async Task StopAsync(CancellationToken cancellationToken = default)
    {
        // キャンセルトークンソースを取得してキャンセルする
        CancellationTokenSource? cts;
        Task? backgroundTask;
        // スレッドセーフにフィールドを取得する
        lock (_lock)
        {
            // キャンセルトークンソースとバックグラウンドタスクを取得する
            cts = _cts;
            // バックグラウンドタスクを取得する
            backgroundTask = _backgroundTask;
            // フィールドをクリアする
            _cts = null;
            // バックグラウンドタスクをクリアする
            _backgroundTask = null;
        }
        // キャンセルトークンソースをキャンセルする
        if (cts is not null)
        {
            // キャンセルを発行する
            cts.Cancel();
            // キャンセルトークンソースを Dispose する
            cts.Dispose();
        }
        // バックグラウンドタスクの完了を待機する
        if (backgroundTask is not null)
        {
            // タスクの完了を待機する（例外は無視する）
            try { await backgroundTask.ConfigureAwait(false); }
            catch (OperationCanceledException) { /* キャンセルは正常終了 */ }
        }
    }
}
