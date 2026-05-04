// 本ファイルは tier1 facade の動的レベル可変ロガー実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/07_Log_API.md (FR-T1-LOG-004)
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//
// 役割:
//   FR-T1-LOG-004「稼働中アプリのログレベルを動的変更する。反映は次回ログ呼び出しから」
//   を満たすため、atomic.Int32 ベースで level を runtime 切替可能な軽量 logger を
//   提供する。Pod 再起動を伴わない level 変更が tier1 facade と tier2 アプリの両方で
//   可能になる。
//
// 役割分担:
//   - tier1 facade 内部の info / warn / error ログ出力 (本 logger)
//   - tier2 アプリの runtime ログ (tier2 SDK が同 atomic.Int32 を共有して使う)
//   - 動的変更経路: K1S0_LOG_LEVEL 環境変数 + SIGHUP で reload (Step 1, 本リリース時点)
//                   Backstage プラグイン / CLI からの SetLogLevel RPC (Step 2, 採用初期)
//
// 出力形式:
//   stdout JSON Lines (1 行 1 entry、Loki / OpenObserve 等のログ集約と互換)
//   {"timestamp":"<RFC3339Nano>","level":"<UPPER>","message":"<text>","attrs":{...}}
//
// Backstage / CLI 結線 (Step 2 採用初期):
//   - log_service.proto に LogAdminService.SetLogLevel(service_name, level) RPC を追加
//   - handler は本 logger.SetLevel を呼ぶ
//   - Audit auto-emit (NFR-E-MON-003): privilegedRPCs 集合に SetLogLevel を追加
//   本リリース時点では env + SIGHUP の経路のみ提供する。

package common

import (
	// stdout JSON 出力用 encoding。
	"encoding/json"
	// SetLevel での未知 level の error 返却。
	"fmt"
	// stdout への書込先 (テスト時に差替可能)。
	"io"
	// 環境変数読取。
	"os"
	// SIGHUP 受信での reload。
	"os/signal"
	// 大文字小文字非依存比較。
	"strings"
	// 並行アクセス保護 + level 切替の lock-free 化。
	"sync/atomic"
	// 出力 entry の RFC3339Nano timestamp。
	"time"

	// SIGHUP 定数 (signal.Notify で使う)。
	"syscall"
)

// LogLevel は順序を持つ log level 列挙。値が大きいほど deeper severity。
type LogLevel int32

// LogLevel 定数。値の順序は GTE 比較で gating する。
const (
	// LevelDebug は最も詳細な開発時 trace。
	LevelDebug LogLevel = 1
	// LevelInfo は一般的な情報。
	LevelInfo LogLevel = 2
	// LevelWarn は注意要の事象。
	LevelWarn LogLevel = 3
	// LevelError はエラー。
	LevelError LogLevel = 4
	// LevelFatal は致命的エラー (出力後 caller が exit を判断)。
	LevelFatal LogLevel = 5
)

// String は level の上位表記を返す (JSON 出力 + reflection 用)。
func (l LogLevel) String() string {
	// switch で各 level の表示文字列を返す。
	switch l {
	case LevelDebug:
		return "DEBUG"
	case LevelInfo:
		return "INFO"
	case LevelWarn:
		return "WARN"
	case LevelError:
		return "ERROR"
	case LevelFatal:
		return "FATAL"
	}
	// 未知値は数値表現で返す (defensive)。
	return fmt.Sprintf("LEVEL(%d)", int32(l))
}

// parseLogLevel は "DEBUG"/"INFO"/"WARN"/"ERROR"/"FATAL" (case-insensitive) を LogLevel に変換する。
// 未知文字列は error を返す (caller は default を選ぶ)。
func parseLogLevel(s string) (LogLevel, error) {
	// 大文字に正規化する。
	switch strings.ToUpper(strings.TrimSpace(s)) {
	case "DEBUG":
		return LevelDebug, nil
	case "INFO":
		return LevelInfo, nil
	case "WARN", "WARNING":
		return LevelWarn, nil
	case "ERROR":
		return LevelError, nil
	case "FATAL":
		return LevelFatal, nil
	}
	// 不正値は error を返却する。
	return LevelInfo, fmt.Errorf("common/logger: unknown log level %q (want DEBUG/INFO/WARN/ERROR/FATAL)", s)
}

// DynamicLogger は atomic.Int32 で level を保持する動的可変 logger。
// 全メソッドは goroutine-safe (level 比較は atomic.Load、出力は io.Writer の同期に依存)。
type DynamicLogger struct {
	// 現在 level (atomic.Int32 で lock-free に runtime 切替可能)。
	level atomic.Int32
	// 出力先 (default os.Stdout、テスト時は bytes.Buffer 等を渡す)。
	out io.Writer
}

// NewDynamicLogger は指定 level / 出力先で logger を生成する。
// out が nil の場合は os.Stdout を使う。
func NewDynamicLogger(initial LogLevel, out io.Writer) *DynamicLogger {
	// nil safe: stdout fallback。
	if out == nil {
		out = os.Stdout
	}
	// logger を生成する。
	l := &DynamicLogger{out: out}
	// atomic に初期 level を書き込む。
	l.level.Store(int32(initial))
	// 構築済 logger を返却する。
	return l
}

// SetLevel は runtime に level を切り替える (FR-T1-LOG-004 の核心)。
// 不正 level は error を返却し、既存 level は維持する。
func (l *DynamicLogger) SetLevel(level string) error {
	// 文字列を LogLevel に変換する。
	parsed, err := parseLogLevel(level)
	// 失敗時は既存 level を維持する。
	if err != nil {
		return err
	}
	// atomic に新 level を書き込む。
	l.level.Store(int32(parsed))
	// 成功は nil。
	return nil
}

// Level は現在の level を返す (atomic.Load)。
func (l *DynamicLogger) Level() LogLevel {
	// atomic に読む。
	return LogLevel(l.level.Load())
}

// LoadFromEnv は K1S0_LOG_LEVEL 環境変数から level を読み込んで適用する。
// env 未設定 / 不正値時は既存 level を維持し、何もエラーを返さない (起動継続を優先)。
func (l *DynamicLogger) LoadFromEnv() {
	// 環境変数を読む。
	v := os.Getenv("K1S0_LOG_LEVEL")
	// 空なら何もしない (既存 level 維持)。
	if v == "" {
		return
	}
	// SetLevel を呼ぶ (失敗は無視)。
	_ = l.SetLevel(v)
}

// StartReloadOnSignal は SIGHUP 受信で K1S0_LOG_LEVEL を再読込する goroutine を起動する。
// 戻り値の cancel func で goroutine 終了を要求できる。
// FR-T1-LOG-004 受け入れ基準「Pod 再起動を伴わずレベル変更可能」を SIGHUP 経路で満たす。
func (l *DynamicLogger) StartReloadOnSignal() (cancel func()) {
	// SIGHUP を受ける channel を作る。
	ch := make(chan os.Signal, 1)
	// SIGHUP のみ受信する。
	signal.Notify(ch, syscall.SIGHUP)
	// goroutine 終了通知用 channel。
	done := make(chan struct{})
	// goroutine を起動する。
	go func() {
		// 終了時に signal subscribe を解除する。
		defer signal.Stop(ch)
		// SIGHUP 受信 / done 受信のいずれかでループを抜ける。
		for {
			// select で multiplex する。
			select {
			case <-ch:
				// SIGHUP → env を re-read。
				l.LoadFromEnv()
			case <-done:
				// cancel された → goroutine 終了。
				return
			}
		}
	}()
	// cancel function を返す。
	return func() { close(done) }
}

// emit は level gating + JSON Lines 出力の共通処理。
// 現在 level より下位 (深い severity でない) は no-op。
func (l *DynamicLogger) emit(level LogLevel, message string, attrs map[string]any) {
	// level gating: 現在 level >= 出力 level の時のみ emit。
	if level < LogLevel(l.level.Load()) {
		// 抑制されたら何もしない (FR-T1-LOG-004「次回ログ呼び出しから反映」を満たす経路)。
		return
	}
	// JSON Lines 形式の entry を組み立てる。
	entry := map[string]any{
		// RFC3339Nano timestamp (Loki/Tempo と互換)。
		"timestamp": time.Now().UTC().Format(time.RFC3339Nano),
		// level の上位表記。
		"level": level.String(),
		// message 本文。
		"message": message,
	}
	// attrs があれば付加する。
	if len(attrs) > 0 {
		entry["attrs"] = attrs
	}
	// JSON エンコード (失敗は無視)。
	b, err := json.Marshal(entry)
	// エンコード失敗は何もしない (defensive)。
	if err != nil {
		return
	}
	// 末尾改行を付けて 1 行で書き出す (Lines 形式)。
	_, _ = fmt.Fprintln(l.out, string(b))
}

// Debug は LevelDebug で emit する (level >= DEBUG の時のみ出力)。
func (l *DynamicLogger) Debug(message string, attrs map[string]any) {
	// LevelDebug で emit に委譲する。
	l.emit(LevelDebug, message, attrs)
}

// Info は LevelInfo で emit する。
func (l *DynamicLogger) Info(message string, attrs map[string]any) {
	// LevelInfo で emit に委譲する。
	l.emit(LevelInfo, message, attrs)
}

// Warn は LevelWarn で emit する。
func (l *DynamicLogger) Warn(message string, attrs map[string]any) {
	// LevelWarn で emit に委譲する。
	l.emit(LevelWarn, message, attrs)
}

// Error は LevelError で emit する。
func (l *DynamicLogger) Error(message string, attrs map[string]any) {
	// LevelError で emit に委譲する。
	l.emit(LevelError, message, attrs)
}

// Fatal は LevelFatal で emit する。caller が exit を判断するため本関数は exit しない。
func (l *DynamicLogger) Fatal(message string, attrs map[string]any) {
	// LevelFatal で emit に委譲する。
	l.emit(LevelFatal, message, attrs)
}
