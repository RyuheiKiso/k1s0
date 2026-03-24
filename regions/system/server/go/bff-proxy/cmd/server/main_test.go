package main

import (
	"log/slog"
	"testing"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/config"
)

// isProductionEnvironment の環境判定ロジックをテストする。
// prod/production のみ true、それ以外は false を返すことを検証する。
func TestIsProductionEnvironment(t *testing.T) {
	cases := []struct {
		env      string
		expected bool
	}{
		// 本番環境判定
		{env: "prod", expected: true},
		{env: "production", expected: true},
		// 大文字・スペース混在でも正しく判定する
		{env: "PROD", expected: true},
		{env: "PRODUCTION", expected: true},
		{env: " prod ", expected: true},
		// 本番環境以外は false
		{env: "dev", expected: false},
		{env: "development", expected: false},
		{env: "local", expected: false},
		{env: "staging", expected: false},
		{env: "test", expected: false},
		{env: "", expected: false},
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
		if !logger.Enabled(nil, tc.expectedLevel) {
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
