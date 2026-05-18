// IProfiling.cs — k1s0 tier1 Library C# 実装: Profiling の L2* interface
// 01_オブザーバビリティ適合仕様.md §プロファイル収集（backend 専用 L2*）に準拠する。
// pprof / Pyroscope 等 OSS の API を Library 独自語彙に翻訳する L2* facade を宣言する。
// 公開シグネチャに OSS 型を露出しない（ProfilerSession 等は一切含まない）。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IDictionary に使用する
using System.Collections.Generic;
// System.IO: Stream に使用する
using System.IO;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task / ValueTask に使用する
using System.Threading.Tasks;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// ProfileType はプロファイルの種別を宣言する enum。
/// L2* 族内共通 API として pprof の profile type 名称に準拠した語彙とする。
/// </summary>
// ProfileType 列挙型定義
public enum ProfileType
{
    /// <summary>Cpu: CPU 使用時間のサンプリングプロファイル</summary>
    Cpu,
    /// <summary>Heap: ヒープメモリ使用状況のプロファイル</summary>
    Heap,
    /// <summary>Thread: スレッドのスタックトレースプロファイル</summary>
    Thread,
    /// <summary>Allocs: メモリアロケーションのプロファイル</summary>
    Allocs,
    /// <summary>Block: ブロッキング操作（lock 等）のプロファイル</summary>
    Block,
}

/// <summary>
/// ProfileOptions は Profiler.Start / Profiler.Capture に渡すオプションを宣言する型。
/// L2* 族内共通 API として OSS 概念を Library 独自語彙に翻訳する。
/// </summary>
// ProfileOptions クラス定義
public sealed class ProfileOptions
{
    /// <summary>SampleRateHz: サンプリングレート（Hz）。0 = 実装固有のデフォルト値</summary>
    // SampleRateHz プロパティ
    public int SampleRateHz { get; init; }

    /// <summary>MaxDurationMs: プロファイル収集の最大継続時間（ミリ秒、0 = 無制限）</summary>
    // MaxDurationMs プロパティ
    public long MaxDurationMs { get; init; }

    /// <summary>Labels: プロファイルに付与するラベル（tenant_id / service 等）</summary>
    // Labels プロパティ
    public IReadOnlyDictionary<string, string>? Labels { get; init; }
}

/// <summary>
/// ProfileSummary はプロファイル収集結果のサマリーを宣言する型。
/// OSS の Profile オブジェクトを直接露出せず Library 語彙で結果を表現する。
/// </summary>
// ProfileSummary レコード定義
public sealed record ProfileSummary(
    // ProfileType: 収集したプロファイルの種別
    ProfileType ProfileType,
    // SampleCount: 収集したサンプル数
    long SampleCount,
    // DurationMs: 収集にかかった時間（ミリ秒単位）
    long DurationMs,
    // SizeBytes: プロファイルデータのバイト数
    long SizeBytes
);

/// <summary>
/// IProfiler は L2* プロファイル収集 interface を宣言する。
/// 族内共通 API として pprof / Pyroscope 等を Library 独自語彙に翻訳する。
/// OSS の profiler 型を引数・戻り値に一切含まない。
/// </summary>
// IProfiler インターフェース定義
public interface IProfiler
{
    /// <summary>
    /// StartAsync はプロファイルの連続収集を開始する。
    /// profileType で収集するプロファイル種別を指定する。
    /// opts は収集オプション（null = デフォルト設定を使用する）。
    /// </summary>
    // StartAsync メソッド: プロファイル収集を開始する
    Task StartAsync(ProfileType profileType, ProfileOptions? opts = null, CancellationToken cancellationToken = default);

    /// <summary>
    /// StopAsync はプロファイルの収集を停止する。
    /// </summary>
    // StopAsync メソッド: プロファイル収集を停止する
    Task StopAsync(ProfileType profileType, CancellationToken cancellationToken = default);

    /// <summary>
    /// CaptureAsync は単発のプロファイルを収集して stream に書き込む。
    /// stream は pprof 形式のバイナリデータを受け取る Stream。
    /// </summary>
    // CaptureAsync メソッド: 単発プロファイルを収集する
    Task<ProfileSummary> CaptureAsync(
        ProfileType profileType,
        Stream stream,
        ProfileOptions? opts = null,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// IsRunning はプロファイル収集が実行中かどうかを返す。
    /// </summary>
    // IsRunning メソッド: 実行中かどうかを返す
    bool IsRunning(ProfileType profileType);
}

/// <summary>
/// ContinuousProfilerConfig は常時プロファイリング（Pyroscope 等）の設定を宣言する型。
/// L2* として Pyroscope Agent の設定語彙を Library 独自語彙に翻訳する。
/// </summary>
// ContinuousProfilerConfig クラス定義
public sealed class ContinuousProfilerConfig
{
    /// <summary>EndpointUrl: Pyroscope / OTLP profiles エンドポイント URL</summary>
    // EndpointUrl プロパティ（必須）
    public required string EndpointUrl { get; init; }

    /// <summary>ApplicationName: プロファイルを識別するアプリケーション名</summary>
    // ApplicationName プロパティ（必須）
    public required string ApplicationName { get; init; }

    /// <summary>SampleRateHz: CPU プロファイルのサンプリングレート（Hz 単位）</summary>
    // SampleRateHz プロパティ
    public int SampleRateHz { get; init; } = 100;

    /// <summary>EnabledTypes: 収集するプロファイル種別のリスト</summary>
    // EnabledTypes プロパティ（必須）
    public required IReadOnlyList<ProfileType> EnabledTypes { get; init; }

    /// <summary>Labels: 全プロファイルに付与する共通ラベル（tenant_id / service 等）</summary>
    // Labels プロパティ
    public IReadOnlyDictionary<string, string>? Labels { get; init; }
}

/// <summary>
/// IContinuousProfiler はバックグラウンドで常時プロファイルを収集する L2* interface を宣言する。
/// Pyroscope Agent 等の常時プロファイリング OSS を Library 独自語彙で抽象化する。
/// </summary>
// IContinuousProfiler インターフェース定義
public interface IContinuousProfiler
{
    /// <summary>
    /// StartAsync は設定に従ってバックグラウンドのプロファイル収集を開始する。
    /// cancellationToken のキャンセルでプロファイル収集を停止する。
    /// </summary>
    // StartAsync メソッド: バックグラウンドプロファイル収集を開始する
    Task StartAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// StopAsync はバックグラウンドのプロファイル収集を停止する（グレースフルシャットダウン）。
    /// </summary>
    // StopAsync メソッド: バックグラウンドプロファイル収集を停止する
    Task StopAsync(CancellationToken cancellationToken = default);
}
