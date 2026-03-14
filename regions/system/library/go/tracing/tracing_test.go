package tracing_test

import (
	"context"
	"testing"

	tracing "github.com/k1s0-platform/system-library-go-tracing"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TraceContextをtraceparentヘッダーに変換し、再度パースして同じ値が得られることを確認する。
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

// 不正なtraceparent文字列に対してFromTraceparentがエラーを返すことを確認する。
func TestFromTraceparent_Invalid(t *testing.T) {
	_, err := tracing.FromTraceparent("invalid")
	require.Error(t, err)

	_, err = tracing.FromTraceparent("01-abc-def-00")
	require.Error(t, err)
}

// Baggageをヘッダーにシリアライズしてからパースすると元の値が復元できることを確認する。
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

// 空文字列からBaggageFromHeaderを呼び出した場合に空のBaggageが返されることを確認する。
func TestBaggageFromHeader_Empty(t *testing.T) {
	b := tracing.BaggageFromHeader("")
	_, ok := b.Get("any")
	assert.False(t, ok)
}

// InjectContextでヘッダーに注入したコンテキストをExtractContextで正しく復元できることを確認する。
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

// 空のヘッダーからExtractContextを呼び出した場合にnilとemptyBaggageが返されることを確認する。
func TestExtractContext_EmptyHeaders(t *testing.T) {
	headers := make(map[string]string)
	tc, bag := tracing.ExtractContext(headers)
	assert.Nil(t, tc)
	_, ok := bag.Get("any")
	assert.False(t, ok)
}
