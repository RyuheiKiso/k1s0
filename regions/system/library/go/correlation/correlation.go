package correlation

import (
	"fmt"

	"github.com/google/uuid"
)

// CorrelationId は分散トレーシングのリクエスト相関 ID。UUID v4 文字列ラッパー。
type CorrelationId string

// NewCorrelationId は新しい CorrelationId を生成する。
func NewCorrelationId() CorrelationId {
	return CorrelationId(uuid.New().String())
}

// ParseCorrelationId は文字列を CorrelationId として受け入れる（バリデーションなし）。
func ParseCorrelationId(s string) CorrelationId {
	return CorrelationId(s)
}

// String は CorrelationId を文字列として返す。
func (c CorrelationId) String() string {
	return string(c)
}

// IsEmpty は CorrelationId が空かどうかを返す。
func (c CorrelationId) IsEmpty() bool {
	return c == ""
}

// TraceId は OpenTelemetry 互換の 32 文字小文字 hex トレース ID。
type TraceId string

// NewTraceId は新しい TraceId を生成する（UUID v4 のハイフン除去）。
func NewTraceId() TraceId {
	id := uuid.New().String()
	// ハイフンを除去して 32 文字 hex に変換
	result := make([]byte, 0, 32)
	for _, c := range id {
		if c != '-' {
			result = append(result, byte(c))
		}
	}
	return TraceId(string(result))
}

// ParseTraceId は文字列を TraceId としてパースする。
// 32 文字小文字 hex でない場合はエラーを返す。
func ParseTraceId(s string) (TraceId, error) {
	if len(s) != 32 {
		return "", fmt.Errorf("invalid trace id length: expected 32, got %d", len(s))
	}
	for _, c := range s {
		if !((c >= '0' && c <= '9') || (c >= 'a' && c <= 'f')) {
			return "", fmt.Errorf("invalid trace id character: %c (expected lowercase hex)", c)
		}
	}
	return TraceId(s), nil
}

// String は TraceId を文字列として返す。
func (t TraceId) String() string {
	return string(t)
}

// IsEmpty は TraceId が空かどうかを返す。
func (t TraceId) IsEmpty() bool {
	return t == ""
}

// CorrelationContext は CorrelationId と TraceId を保持するコンテキスト。
type CorrelationContext struct {
	CorrelationId CorrelationId
	TraceId       TraceId
}

// NewCorrelationContext は新しい CorrelationContext を生成する。
func NewCorrelationContext() CorrelationContext {
	return CorrelationContext{
		CorrelationId: NewCorrelationId(),
		TraceId:       NewTraceId(),
	}
}
