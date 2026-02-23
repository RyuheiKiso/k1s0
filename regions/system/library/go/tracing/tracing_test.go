package tracing_test

import (
	"context"
	"testing"

	tracing "github.com/k1s0-platform/system-library-go-tracing"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestTraceparentRoundtrip(t *testing.T) {
	tc := tracing.TraceContext{
		TraceID:  "0af7651916cd43dd8448eb211c80319c",
		ParentID: "b7ad6b7169203331",
		Flags:    0x01,
	}

	header := tc.ToTraceparent()
	assert.Equal(t, "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01", header)

	parsed, err := tracing.FromTraceparent(header)
	require.NoError(t, err)
	assert.Equal(t, tc.TraceID, parsed.TraceID)
	assert.Equal(t, tc.ParentID, parsed.ParentID)
	assert.Equal(t, tc.Flags, parsed.Flags)
}

func TestFromTraceparent_Invalid(t *testing.T) {
	_, err := tracing.FromTraceparent("invalid")
	require.Error(t, err)

	_, err = tracing.FromTraceparent("01-abc-def-00")
	require.Error(t, err)
}

func TestBaggageRoundtrip(t *testing.T) {
	b := tracing.NewBaggage()
	b.Set("userId", "alice")
	b.Set("tenantId", "t-1")

	header := b.ToHeader()
	assert.Contains(t, header, "userId=alice")
	assert.Contains(t, header, "tenantId=t-1")

	parsed := tracing.BaggageFromHeader(header)
	v, ok := parsed.Get("userId")
	assert.True(t, ok)
	assert.Equal(t, "alice", v)

	v, ok = parsed.Get("tenantId")
	assert.True(t, ok)
	assert.Equal(t, "t-1", v)
}

func TestBaggageFromHeader_Empty(t *testing.T) {
	b := tracing.BaggageFromHeader("")
	_, ok := b.Get("any")
	assert.False(t, ok)
}

func TestInjectExtract(t *testing.T) {
	tc := &tracing.TraceContext{
		TraceID:  "0af7651916cd43dd8448eb211c80319c",
		ParentID: "b7ad6b7169203331",
		Flags:    0x01,
	}
	bag := tracing.NewBaggage()
	bag.Set("requestId", "req-123")

	headers := make(map[string]string)
	tracing.InjectContext(context.Background(), headers, tc, bag)

	assert.Contains(t, headers, "traceparent")
	assert.Contains(t, headers, "baggage")

	extractedTC, extractedBag := tracing.ExtractContext(headers)
	require.NotNil(t, extractedTC)
	assert.Equal(t, tc.TraceID, extractedTC.TraceID)
	assert.Equal(t, tc.ParentID, extractedTC.ParentID)
	assert.Equal(t, tc.Flags, extractedTC.Flags)

	v, ok := extractedBag.Get("requestId")
	assert.True(t, ok)
	assert.Equal(t, "req-123", v)
}

func TestExtractContext_EmptyHeaders(t *testing.T) {
	headers := make(map[string]string)
	tc, bag := tracing.ExtractContext(headers)
	assert.Nil(t, tc)
	_, ok := bag.Get("any")
	assert.False(t, ok)
}
