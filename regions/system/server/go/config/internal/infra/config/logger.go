package config

import (
	"log/slog"
	"os"
)

// NewLogger は構造化ロガーを初期化する。
func NewLogger(environment, appName, version, tier string) *slog.Logger {
	var handler slog.Handler

	opts := &slog.HandlerOptions{
		Level: slog.LevelInfo,
	}

	if environment == "production" || environment == "staging" {
		// JSON フォーマット
		handler = slog.NewJSONHandler(os.Stdout, opts)
	} else {
		// テキストフォーマット（開発用）
		handler = slog.NewTextHandler(os.Stdout, opts)
	}

	logger := slog.New(handler).With(
		slog.String("app", appName),
		slog.String("version", version),
		slog.String("tier", tier),
	)

	return logger
}
