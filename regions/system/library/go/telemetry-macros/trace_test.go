// trace_test.go: telemetrymacros パッケージのユニットテスト。
// OpenTelemetry の noop トレーサーを使用して副作用なしにテストを実行する。
package telemetrymacros_test

import (
	"context"
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	telemetrymacros "github.com/k1s0-platform/system-library-go-telemetry-macros"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/trace/noop"
)

// setupNoop は全テストで共通して使用するnoop TracerProviderをセットアップする。
func setupNoop() {
	otel.SetTracerProvider(noop.NewTracerProvider())
}

// TestTrace_Success は Trace が fn を実行してnilを返すことを確認する。
func TestTrace_Success(t *testing.T) {
	setupNoop()
	called := false
	err := telemetrymacros.Trace(context.Background(), "test.span", func(ctx context.Context) error {
		called = true
		return nil
	})
	require.NoError(t, err)
	assert.True(t, called, "fn が呼び出されていること")
}

// TestTrace_Error は Trace が fn のエラーをそのまま返すことを確認する。
func TestTrace_Error(t *testing.T) {
	setupNoop()
	want := errors.New("something failed")
	err := telemetrymacros.Trace(context.Background(), "test.span", func(ctx context.Context) error {
		return want
	})
	assert.ErrorIs(t, err, want)
}

// TestTraceValue_Success は TraceValue が値とnilエラーを返すことを確認する。
func TestTraceValue_Success(t *testing.T) {
	setupNoop()
	got, err := telemetrymacros.TraceValue(context.Background(), "test.value", func(ctx context.Context) (int, error) {
		return 42, nil
	})
	require.NoError(t, err)
	assert.Equal(t, 42, got)
}

// TestTraceValue_Error は TraceValue がエラー時にゼロ値とエラーを返すことを確認する。
func TestTraceValue_Error(t *testing.T) {
	setupNoop()
	want := errors.New("value error")
	got, err := telemetrymacros.TraceValue(context.Background(), "test.value", func(ctx context.Context) (string, error) {
		return "", want
	})
	assert.ErrorIs(t, err, want)
	// エラー時はゼロ値が返ること。
	assert.Equal(t, "", got)
}

// TestInstrumentDB_Success は InstrumentDB が fn を実行してnilを返すことを確認する。
func TestInstrumentDB_Success(t *testing.T) {
	setupNoop()
	called := false
	err := telemetrymacros.InstrumentDB(context.Background(), "query", "users", func(ctx context.Context) error {
		called = true
		return nil
	})
	require.NoError(t, err)
	assert.True(t, called, "fn が呼び出されていること")
}

// TestInstrumentDB_Error は InstrumentDB が fn のエラーをそのまま返すことを確認する。
func TestInstrumentDB_Error(t *testing.T) {
	setupNoop()
	want := errors.New("db error")
	err := telemetrymacros.InstrumentDB(context.Background(), "exec", "orders", func(ctx context.Context) error {
		return want
	})
	assert.ErrorIs(t, err, want)
}

// TestKafkaTracingMiddleware_Success はミドルウェアが handler を呼び出すことを確認する。
func TestKafkaTracingMiddleware_Success(t *testing.T) {
	setupNoop()
	called := false
	middleware := telemetrymacros.KafkaTracingMiddleware("user.created", func(ctx context.Context, payload []byte, headers map[string]string) error {
		called = true
		return nil
	})
	err := middleware(context.Background(), []byte(`{"id":"1"}`), map[string]string{})
	require.NoError(t, err)
	assert.True(t, called, "handler が呼び出されていること")
}

// TestKafkaTracingMiddleware_Error は handler のエラーがミドルウェアから返されることを確認する。
func TestKafkaTracingMiddleware_Error(t *testing.T) {
	setupNoop()
	want := errors.New("handler error")
	middleware := telemetrymacros.KafkaTracingMiddleware("order.placed", func(ctx context.Context, payload []byte, headers map[string]string) error {
		return want
	})
	err := middleware(context.Background(), []byte(`{}`), map[string]string{"traceparent": "00-abc-def-01"})
	assert.ErrorIs(t, err, want)
}
