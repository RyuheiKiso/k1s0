// profiling.go — k1s0 tier1 Library Go 実装: Profiling の L2* interface
// 01_オブザーバビリティ適合仕様.md §プロファイル収集（backend 専用 L2*）に準拠する。
// pprof / Pyroscope 等 OSS の API を Library 独自語彙に翻訳する L2* facade を宣言する。
// 公開シグネチャに OSS 型を露出しない（pprof.Profile 等は一切含まない）。

// パッケージ名: profiling（tier1 Library のプロファイリング API を提供する）
package profiling

import (
	// context: context.Context（プロファイル収集制御に使用する）
	"context"
	// io: プロファイルデータ出力先として io.Writer を使用する
	"io"
)

// ProfileType はプロファイルの種別を宣言する型。
// pprof の profile type 名称に準拠した語彙とする（L2* 族内共通 API）。
type ProfileType string

const (
	// ProfileTypeCPU: CPU 使用時間のサンプリングプロファイル
	ProfileTypeCPU ProfileType = "cpu"
	// ProfileTypeHeap: ヒープメモリ使用状況のプロファイル
	ProfileTypeHeap ProfileType = "heap"
	// ProfileTypeGoroutine: goroutine のスタックトレースプロファイル（Go 専用）
	ProfileTypeGoroutine ProfileType = "goroutine"
	// ProfileTypeBlock: ブロッキング操作（mutex / channel 等）のプロファイル
	ProfileTypeBlock ProfileType = "block"
	// ProfileTypeMutex: mutex 競合のプロファイル
	ProfileTypeMutex ProfileType = "mutex"
	// ProfileTypeAllocs: メモリアロケーションのプロファイル
	ProfileTypeAllocs ProfileType = "allocs"
)

// ProfileOptions は Profiler.Start / Profiler.Capture に渡すオプションを宣言する型。
// L2* 族内共通 API として OSS 概念を Library 独自語彙に翻訳する。
type ProfileOptions struct {
	// SampleRate: サンプリングレート（Hz / samples per second）。
	// 0 の場合は実装固有のデフォルト値を使用する。
	SampleRate int
	// MaxDuration: プロファイル収集の最大継続時間（0 = 無制限）。
	// wall-clock TTL 禁止規約に準拠して HLC ベースの duration で管理する。
	MaxDuration int64
	// Labels: プロファイルに付与するラベル（tenant_id / service 等）
	Labels map[string]string
}

// ProfileSummary はプロファイル収集結果のサマリーを宣言する型。
// OSS の pprof.Profile を直接露出せず Library 語彙で結果を表現する。
type ProfileSummary struct {
	// ProfileType: 収集したプロファイルの種別
	ProfileType ProfileType
	// SampleCount: 収集したサンプル数
	SampleCount int64
	// DurationNs: 収集にかかった時間（ナノ秒単位）
	DurationNs int64
	// SizeBytes: プロファイルデータのバイト数
	SizeBytes int64
}

// Profiler は L2* プロファイル収集 interface を宣言する。
// 族内共通 API として pprof / Pyroscope 等を Library 独自語彙に翻訳する。
// OSS の profiler 型（pprof.Profile 等）を引数・戻り値に一切含まない。
type Profiler interface {
	// Start はプロファイルの連続収集を開始する。
	// profileType で収集するプロファイル種別を指定する。
	// opts で収集オプションを指定する（nil = デフォルト設定を使用する）。
	// 返された context を使用してプロファイル収集のライフサイクルを管理する。
	Start(ctx context.Context, profileType ProfileType, opts *ProfileOptions) (context.Context, error)

	// Stop はプロファイルの収集を停止する（Start で開始したものを停止する）。
	// ctx は Start で返された context を渡す。
	Stop(ctx context.Context) error

	// Capture は単発のプロファイルを収集して w に書き込む。
	// profileType で収集するプロファイル種別を指定する。
	// opts で収集オプションを指定する（nil = デフォルト設定を使用する）。
	// w は pprof 形式のバイナリデータを受け取る io.Writer。
	Capture(ctx context.Context, profileType ProfileType, opts *ProfileOptions, w io.Writer) (*ProfileSummary, error)

	// IsRunning はプロファイル収集が実行中かどうかを返す。
	// profileType で確認するプロファイル種別を指定する。
	IsRunning(profileType ProfileType) bool
}

// ContinuousProfilerConfig は常時プロファイリング（Pyroscope 等）の設定を宣言する型。
// L2* として Pyroscope Agent の設定語彙を Library 独自語彙に翻訳する。
type ContinuousProfilerConfig struct {
	// EndpointURL: Pyroscope / OTLP profiles エンドポイント URL
	// OSS の SDK URL を直接使用せず Library 設定として受け取る。
	EndpointURL string
	// ApplicationName: プロファイルを識別するアプリケーション名
	ApplicationName string
	// SampleRateHz: CPU プロファイルのサンプリングレート（Hz 単位）
	SampleRateHz int
	// EnabledTypes: 収集するプロファイル種別のスライス
	EnabledTypes []ProfileType
	// Labels: 全プロファイルに付与する共通ラベル（tenant_id / service 等）
	Labels map[string]string
}

// ContinuousProfiler はバックグラウンドで常時プロファイルを収集する L2* interface を宣言する。
// Pyroscope Agent 等の常時プロファイリング OSS を Library 独自語彙で抽象化する。
type ContinuousProfiler interface {
	// Start は設定に従ってバックグラウンドのプロファイル収集を開始する。
	// ctx のキャンセルでプロファイル収集を停止する。
	Start(ctx context.Context) error

	// Stop はバックグラウンドのプロファイル収集を停止する（グレースフルシャットダウン）。
	Stop(ctx context.Context) error
}
