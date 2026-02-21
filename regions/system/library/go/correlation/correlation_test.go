package correlation_test

import (
	"testing"

	correlation "github.com/k1s0-platform/system-library-go-correlation"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewCorrelationId(t *testing.T) {
	id := correlation.NewCorrelationId()
	assert.NotEmpty(t, string(id))
	assert.False(t, id.IsEmpty())
}

func TestNewCorrelationId_IsUnique(t *testing.T) {
	id1 := correlation.NewCorrelationId()
	id2 := correlation.NewCorrelationId()
	assert.NotEqual(t, id1, id2)
}

func TestParseCorrelationId(t *testing.T) {
	id := correlation.ParseCorrelationId("test-id-123")
	assert.Equal(t, "test-id-123", id.String())
}

func TestCorrelationId_IsEmpty(t *testing.T) {
	empty := correlation.ParseCorrelationId("")
	assert.True(t, empty.IsEmpty())
	nonEmpty := correlation.ParseCorrelationId("some-id")
	assert.False(t, nonEmpty.IsEmpty())
}

func TestNewTraceId(t *testing.T) {
	id := correlation.NewTraceId()
	assert.Len(t, id.String(), 32)
	assert.False(t, id.IsEmpty())
}

func TestNewTraceId_IsLowercaseHex(t *testing.T) {
	id := correlation.NewTraceId()
	for _, c := range id.String() {
		assert.True(t, (c >= '0' && c <= '9') || (c >= 'a' && c <= 'f'),
			"expected lowercase hex, got: %c", c)
	}
}

func TestNewTraceId_IsUnique(t *testing.T) {
	id1 := correlation.NewTraceId()
	id2 := correlation.NewTraceId()
	assert.NotEqual(t, id1, id2)
}

func TestParseTraceId_Valid(t *testing.T) {
	valid := "4bf92f3577b34da6a3ce929d0e0e4736"
	id, err := correlation.ParseTraceId(valid)
	require.NoError(t, err)
	assert.Equal(t, valid, id.String())
}

func TestParseTraceId_InvalidLength(t *testing.T) {
	_, err := correlation.ParseTraceId("short")
	assert.Error(t, err)
}

func TestParseTraceId_InvalidChars(t *testing.T) {
	_, err := correlation.ParseTraceId("4BF92F3577B34DA6A3CE929D0E0E4736") // uppercase
	assert.Error(t, err)
}

func TestNewCorrelationContext(t *testing.T) {
	ctx := correlation.NewCorrelationContext()
	assert.False(t, ctx.CorrelationId.IsEmpty())
	assert.False(t, ctx.TraceId.IsEmpty())
}

func TestToHeaders(t *testing.T) {
	ctx := correlation.CorrelationContext{
		CorrelationId: correlation.ParseCorrelationId("test-correlation-id"),
		TraceId:       correlation.TraceId("4bf92f3577b34da6a3ce929d0e0e4736"),
	}
	headers := correlation.ToHeaders(ctx)
	assert.Equal(t, "test-correlation-id", headers[correlation.HeaderCorrelationId])
	assert.Equal(t, "4bf92f3577b34da6a3ce929d0e0e4736", headers[correlation.HeaderTraceId])
}

func TestFromHeaders_WithExistingHeaders(t *testing.T) {
	headers := map[string]string{
		correlation.HeaderCorrelationId: "existing-id",
		correlation.HeaderTraceId:       "4bf92f3577b34da6a3ce929d0e0e4736",
	}
	ctx := correlation.FromHeaders(headers)
	assert.Equal(t, "existing-id", ctx.CorrelationId.String())
	assert.Equal(t, "4bf92f3577b34da6a3ce929d0e0e4736", ctx.TraceId.String())
}

func TestFromHeaders_AutoGenerate(t *testing.T) {
	headers := map[string]string{}
	ctx := correlation.FromHeaders(headers)
	assert.False(t, ctx.CorrelationId.IsEmpty())
	assert.False(t, ctx.TraceId.IsEmpty())
}

func TestFromHeaders_InvalidTraceId(t *testing.T) {
	headers := map[string]string{
		correlation.HeaderTraceId: "invalid-trace-id",
	}
	ctx := correlation.FromHeaders(headers)
	// 無効な TraceId の場合は自動生成される
	assert.False(t, ctx.TraceId.IsEmpty())
	assert.Len(t, ctx.TraceId.String(), 32)
}

func TestTraceId_IsEmpty(t *testing.T) {
	empty := correlation.TraceId("")
	assert.True(t, empty.IsEmpty())
	nonEmpty := correlation.NewTraceId()
	assert.False(t, nonEmpty.IsEmpty())
}
