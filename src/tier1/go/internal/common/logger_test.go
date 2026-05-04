// 本ファイルは common/logger.go の単体テスト。
//
// 試験戦略:
//   bytes.Buffer を出力先に渡して JSON Lines 出力を inspect する。
//   SetLevel を runtime で切り替え、次回出力に即時反映されることを検証する。
//
// 検証する不変式:
//   1. SetLevel("WARN") 後は INFO が出力されず、WARN は出力される (FR-T1-LOG-004 の核心)
//   2. LoadFromEnv で K1S0_LOG_LEVEL から level が読み込まれる
//   3. SetLevel(不正値) は既存 level を維持し error を返す
//   4. JSON Lines 出力に timestamp / level / message が必須項目として含まれる

package common

import (
	// 出力 capture 用 buffer。
	"bytes"
	// JSON parse 用 (出力検証)。
	"encoding/json"
	// 環境変数の test 中設定。
	"os"
	// 出力後の改行除去。
	"strings"
	// テスト fail / 報告。
	"testing"
)

// TestDynamicLogger_LevelGate_InfoSuppressedAtWarn は SetLevel(WARN) で
// Info が出力されず、Warn が出力されることを確認する (FR-T1-LOG-004 の核心)。
func TestDynamicLogger_LevelGate_InfoSuppressedAtWarn(t *testing.T) {
	// 出力 capture 用 buffer を用意する。
	var buf bytes.Buffer
	// 初期 INFO で logger を作る。
	l := NewDynamicLogger(LevelInfo, &buf)
	// 動的に WARN に切り替える。
	if err := l.SetLevel("WARN"); err != nil {
		t.Fatalf("SetLevel(WARN): %v", err)
	}
	// Info を呼ぶ (抑制されるべき)。
	l.Info("info-suppressed", nil)
	// Warn を呼ぶ (出力されるべき)。
	l.Warn("warn-emitted", nil)
	// 出力を 1 行ごとに分解する。
	lines := strings.Split(strings.TrimRight(buf.String(), "\n"), "\n")
	// 1 行のみ (warn のみ) であること。
	if len(lines) != 1 {
		t.Fatalf("emitted lines = %d, want 1 (only WARN should pass gate). raw=%q", len(lines), buf.String())
	}
	// JSON parse して level=WARN / message=warn-emitted を確認する。
	var entry map[string]any
	// 1 行目を parse する。
	if err := json.Unmarshal([]byte(lines[0]), &entry); err != nil {
		t.Fatalf("json.Unmarshal: %v", err)
	}
	// level field を検査する。
	if got := entry["level"]; got != "WARN" {
		t.Fatalf("level = %v, want WARN", got)
	}
	// message field を検査する。
	if got := entry["message"]; got != "warn-emitted" {
		t.Fatalf("message = %v, want warn-emitted", got)
	}
}

// TestDynamicLogger_LoadFromEnv_AppliesLevel は K1S0_LOG_LEVEL=ERROR を
// 設定した状態で LoadFromEnv を呼ぶと、Level() が ERROR を返すことを確認する。
func TestDynamicLogger_LoadFromEnv_AppliesLevel(t *testing.T) {
	// 既存 env を保存する (テスト終了時に復元)。
	prev, hadPrev := os.LookupEnv("K1S0_LOG_LEVEL")
	// テスト終了時に env を復元する。
	defer func() {
		if hadPrev {
			_ = os.Setenv("K1S0_LOG_LEVEL", prev)
		} else {
			_ = os.Unsetenv("K1S0_LOG_LEVEL")
		}
	}()
	// 環境変数を ERROR に設定する。
	if err := os.Setenv("K1S0_LOG_LEVEL", "ERROR"); err != nil {
		t.Fatalf("Setenv: %v", err)
	}
	// 初期 INFO で logger を作る。
	l := NewDynamicLogger(LevelInfo, &bytes.Buffer{})
	// env から読み込ませる。
	l.LoadFromEnv()
	// Level() が ERROR を返すこと。
	if got := l.Level(); got != LevelError {
		t.Fatalf("Level() after LoadFromEnv = %v, want LevelError", got)
	}
}

// TestDynamicLogger_SetLevel_Invalid_ReturnsError_KeepsExisting は
// SetLevel に不正値を渡した時に error を返し、既存 level が維持されることを確認する。
func TestDynamicLogger_SetLevel_Invalid_ReturnsError_KeepsExisting(t *testing.T) {
	// INFO で logger を作る。
	l := NewDynamicLogger(LevelInfo, &bytes.Buffer{})
	// 不正 level を渡す。
	err := l.SetLevel("BOGUS")
	// error が返ること。
	if err == nil {
		t.Fatalf("SetLevel(BOGUS) should return error")
	}
	// 既存 level が維持されること。
	if got := l.Level(); got != LevelInfo {
		t.Fatalf("Level() after invalid SetLevel = %v, want LevelInfo (must be unchanged)", got)
	}
}

// TestDynamicLogger_JSONOutput_HasRequiredFields は JSON Lines 出力に
// timestamp / level / message の 3 必須フィールドが含まれることを確認する。
func TestDynamicLogger_JSONOutput_HasRequiredFields(t *testing.T) {
	// buffer を用意する。
	var buf bytes.Buffer
	// DEBUG レベルですべて通すロガーを作る。
	l := NewDynamicLogger(LevelDebug, &buf)
	// 1 件 emit する。
	l.Info("hello", map[string]any{"k": "v"})
	// 出力を JSON parse する。
	var entry map[string]any
	if err := json.Unmarshal([]byte(strings.TrimRight(buf.String(), "\n")), &entry); err != nil {
		t.Fatalf("json.Unmarshal: %v (raw=%q)", err, buf.String())
	}
	// timestamp / level / message が存在すること。
	for _, f := range []string{"timestamp", "level", "message"} {
		if _, ok := entry[f]; !ok {
			t.Fatalf("entry missing %q field. entry=%+v", f, entry)
		}
	}
	// attrs が non-nil であること。
	attrs, ok := entry["attrs"].(map[string]any)
	if !ok {
		t.Fatalf("attrs not present or not map: %+v", entry["attrs"])
	}
	// attrs.k = "v" が含まれること。
	if got := attrs["k"]; got != "v" {
		t.Fatalf("attrs.k = %v, want v", got)
	}
}
