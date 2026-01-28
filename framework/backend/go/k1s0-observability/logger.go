package k1s0observability

import (
	"context"
	"os"

	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

// Logger provides structured logging with k1s0 required fields.
type Logger struct {
	config    *Config
	zapLogger *zap.Logger
}

// NewLogger creates a new Logger with the given configuration.
func NewLogger(config *Config) (*Logger, error) {
	// Configure zap encoder
	encoderConfig := zapcore.EncoderConfig{
		TimeKey:        "timestamp",
		LevelKey:       "level",
		NameKey:        "logger",
		CallerKey:      "caller",
		FunctionKey:    zapcore.OmitKey,
		MessageKey:     "message",
		StacktraceKey:  "stacktrace",
		LineEnding:     zapcore.DefaultLineEnding,
		EncodeLevel:    zapcore.CapitalLevelEncoder,
		EncodeTime:     zapcore.ISO8601TimeEncoder,
		EncodeDuration: zapcore.StringDurationEncoder,
		EncodeCaller:   zapcore.ShortCallerEncoder,
	}

	// Parse log level
	level := parseLogLevel(config.LogLevel)

	// Create core with JSON encoder
	core := zapcore.NewCore(
		zapcore.NewJSONEncoder(encoderConfig),
		zapcore.AddSync(os.Stdout),
		level,
	)

	// Create logger with default fields
	zapLogger := zap.New(core).With(
		zap.String("service.name", config.ServiceName),
		zap.String("service.env", config.Env),
	)

	if config.Version != "" {
		zapLogger = zapLogger.With(zap.String("service.version", config.Version))
	}
	if config.InstanceID != "" {
		zapLogger = zapLogger.With(zap.String("service.instance_id", config.InstanceID))
	}

	return &Logger{
		config:    config,
		zapLogger: zapLogger,
	}, nil
}

// parseLogLevel converts a string log level to zapcore.Level.
func parseLogLevel(level string) zapcore.Level {
	switch level {
	case "DEBUG":
		return zapcore.DebugLevel
	case "INFO":
		return zapcore.InfoLevel
	case "WARN":
		return zapcore.WarnLevel
	case "ERROR":
		return zapcore.ErrorLevel
	default:
		return zapcore.InfoLevel
	}
}

// contextFields extracts logging fields from a context.
func (l *Logger) contextFields(ctx context.Context) []zap.Field {
	rc := FromContext(ctx)
	if rc == nil {
		return nil
	}

	fields := make([]zap.Field, 0, 5)
	if rc.TraceID != "" {
		fields = append(fields, zap.String("trace.id", rc.TraceID))
	}
	if rc.RequestID != "" {
		fields = append(fields, zap.String("request.id", rc.RequestID))
	}
	if rc.SpanID != "" {
		fields = append(fields, zap.String("span.id", rc.SpanID))
	}
	if rc.UserID != "" {
		fields = append(fields, zap.String("user.id", rc.UserID))
	}
	if rc.TenantID != "" {
		fields = append(fields, zap.String("tenant.id", rc.TenantID))
	}

	return fields
}

// Debug logs a message at DEBUG level.
func (l *Logger) Debug(ctx context.Context, msg string, fields ...zap.Field) {
	fields = append(l.contextFields(ctx), fields...)
	l.zapLogger.Debug(msg, fields...)
}

// Info logs a message at INFO level.
func (l *Logger) Info(ctx context.Context, msg string, fields ...zap.Field) {
	fields = append(l.contextFields(ctx), fields...)
	l.zapLogger.Info(msg, fields...)
}

// Warn logs a message at WARN level.
func (l *Logger) Warn(ctx context.Context, msg string, fields ...zap.Field) {
	fields = append(l.contextFields(ctx), fields...)
	l.zapLogger.Warn(msg, fields...)
}

// Error logs a message at ERROR level.
func (l *Logger) Error(ctx context.Context, msg string, fields ...zap.Field) {
	fields = append(l.contextFields(ctx), fields...)
	l.zapLogger.Error(msg, fields...)
}

// WithFields returns a new Logger with additional fields.
func (l *Logger) WithFields(fields ...zap.Field) *Logger {
	return &Logger{
		config:    l.config,
		zapLogger: l.zapLogger.With(fields...),
	}
}

// WithError returns a new Logger with error information.
func (l *Logger) WithError(err error) *Logger {
	return &Logger{
		config:    l.config,
		zapLogger: l.zapLogger.With(zap.Error(err)),
	}
}

// Sync flushes any buffered log entries.
func (l *Logger) Sync() error {
	return l.zapLogger.Sync()
}

// Zap returns the underlying zap.Logger.
func (l *Logger) Zap() *zap.Logger {
	return l.zapLogger
}

// Config returns the logger configuration.
func (l *Logger) Config() *Config {
	return l.config
}
