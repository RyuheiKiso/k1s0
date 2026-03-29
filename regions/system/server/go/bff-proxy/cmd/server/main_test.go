package main

import (
	"context"
	"log/slog"
	"testing"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/config"
)

// isProductionEnvironment の環境判定ロジックをテストする。
// M-011 監査対応: fail-safe パターン（デフォルト本番）に変更後のテスト。
// dev/development/test/local のみ false、それ以外は true を返すことを検証する。
func TestIsProductionEnvironment(t *testing.T) {
	cases := []struct {
		env      string
		expected bool
	}{
		// 本番環境判定（明示的に本番を指定した場合）
		{env: "prod", expected: true},
		{env: "production", expected: true},
		// 大文字・スペース混在でも正しく判定する
		{env: "PROD", expected: true},
		{env: "PRODUCTION", expected: true},
		{env: " prod ", expected: true},
		// M-011: タイポは安全のため本番扱いとなる（fail-safe）
		{env: "prodction", expected: true},
		// M-011: staging は明示的な非本番リストにないため本番扱いとなる
		{env: "staging", expected: true},
		// M-011: 空文字は安全のため本番扱いとなる（fail-safe）
		{env: "", expected: true},
		// 明示的に開発・テスト環境として指定された場合のみ非本番
		{env: "dev", expected: false},
		{env: "development", expected: false},
		{env: "local", expected: false},
		{env: "test", expected: false},
	}

	for _, tc := range cases {
		got := isProductionEnvironment(tc.env)
		if got != tc.expected {
			t.Errorf("isProductionEnvironment(%q) = %v, want %v", tc.env, got, tc.expected)
		}
	}
}

// newLogger のログレベルマッピングをテストする。
// 各レベル文字列が正しい slog.Level に変換されることを検証する。
func TestNewLoggerLevel(t *testing.T) {
	cases := []struct {
		level         string
		expectedLevel slog.Level
	}{
		{level: "debug", expectedLevel: slog.LevelDebug},
		{level: "info", expectedLevel: slog.LevelInfo},
		{level: "warn", expectedLevel: slog.LevelWarn},
		{level: "error", expectedLevel: slog.LevelError},
		// 不明な値はデフォルトの Info レベルを返す
		{level: "unknown", expectedLevel: slog.LevelInfo},
		{level: "", expectedLevel: slog.LevelInfo},
	}

	for _, tc := range cases {
		cfg := config.LogConfig{
			Level:  tc.level,
			Format: "json",
		}
		logger := newLogger(cfg)
		if logger == nil {
			t.Errorf("newLogger returned nil for level=%q", tc.level)
			continue
		}
		// SA1012: context.Background() を使用して nil コンテキストを回避する（§3.2 監査対応）
		if !logger.Enabled(context.Background(), tc.expectedLevel) {
			t.Errorf("newLogger(level=%q): expected level %v to be enabled", tc.level, tc.expectedLevel)
		}
	}
}

// newLogger の JSON/Text フォーマット切り替えをテストする。
// どちらのフォーマットでも logger が正常に生成されることを検証する。
func TestNewLoggerFormat(t *testing.T) {
	formats := []string{"json", "text", "JSON", "Text", "TEXT"}
	for _, format := range formats {
		cfg := config.LogConfig{
			Level:  "info",
			Format: format,
		}
		logger := newLogger(cfg)
		if logger == nil {
			t.Errorf("newLogger returned nil for format=%q", format)
		}
	}
}
