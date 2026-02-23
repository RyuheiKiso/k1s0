package tracing

import (
	"context"
	"fmt"
	"strings"
	"sync"
)

// TraceContext はW3C TraceContext。
type TraceContext struct {
	TraceID  string // 32 hex chars
	ParentID string // 16 hex chars
	Flags    byte
}

// ToTraceparent はtraceparentヘッダー文字列を返す。
func (t TraceContext) ToTraceparent() string {
	return fmt.Sprintf("00-%s-%s-%02x", t.TraceID, t.ParentID, t.Flags)
}

// FromTraceparent はtraceparent文字列をパースする。
func FromTraceparent(s string) (*TraceContext, error) {
	parts := strings.Split(s, "-")
	if len(parts) != 4 {
		return nil, fmt.Errorf("invalid traceparent format: %s", s)
	}
	if parts[0] != "00" {
		return nil, fmt.Errorf("unsupported version: %s", parts[0])
	}
	if len(parts[1]) != 32 {
		return nil, fmt.Errorf("invalid trace-id length: %d", len(parts[1]))
	}
	if len(parts[2]) != 16 {
		return nil, fmt.Errorf("invalid parent-id length: %d", len(parts[2]))
	}
	if len(parts[3]) != 2 {
		return nil, fmt.Errorf("invalid flags length: %d", len(parts[3]))
	}

	var flags byte
	_, err := fmt.Sscanf(parts[3], "%02x", &flags)
	if err != nil {
		return nil, fmt.Errorf("invalid flags: %s", parts[3])
	}

	return &TraceContext{
		TraceID:  parts[1],
		ParentID: parts[2],
		Flags:    flags,
	}, nil
}

// Baggage はW3C Baggage。
type Baggage struct {
	entries map[string]string
	mu      sync.RWMutex
}

// NewBaggage は新しい Baggage を生成する。
func NewBaggage() *Baggage {
	return &Baggage{entries: make(map[string]string)}
}

// Set はエントリを設定する。
func (b *Baggage) Set(key, value string) {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.entries[key] = value
}

// Get はエントリを取得する。
func (b *Baggage) Get(key string) (string, bool) {
	b.mu.RLock()
	defer b.mu.RUnlock()
	v, ok := b.entries[key]
	return v, ok
}

// ToHeader はbaggageヘッダー文字列を返す。
func (b *Baggage) ToHeader() string {
	b.mu.RLock()
	defer b.mu.RUnlock()
	var parts []string
	for k, v := range b.entries {
		parts = append(parts, k+"="+v)
	}
	return strings.Join(parts, ",")
}

// BaggageFromHeader はbaggageヘッダー文字列をパースする。
func BaggageFromHeader(s string) *Baggage {
	b := NewBaggage()
	if s == "" {
		return b
	}
	for _, part := range strings.Split(s, ",") {
		kv := strings.SplitN(strings.TrimSpace(part), "=", 2)
		if len(kv) == 2 {
			b.entries[strings.TrimSpace(kv[0])] = strings.TrimSpace(kv[1])
		}
	}
	return b
}

// InjectContext はheadersにtraceparentとbaggageを書き込む。
func InjectContext(_ context.Context, headers map[string]string, tc *TraceContext, bag *Baggage) {
	if tc != nil {
		headers["traceparent"] = tc.ToTraceparent()
	}
	if bag != nil {
		h := bag.ToHeader()
		if h != "" {
			headers["baggage"] = h
		}
	}
}

// ExtractContext はheadersからTraceContextとBaggageを取り出す。
func ExtractContext(headers map[string]string) (*TraceContext, *Baggage) {
	var tc *TraceContext
	if tp, ok := headers["traceparent"]; ok {
		parsed, err := FromTraceparent(tp)
		if err == nil {
			tc = parsed
		}
	}
	bag := NewBaggage()
	if bh, ok := headers["baggage"]; ok {
		bag = BaggageFromHeader(bh)
	}
	return tc, bag
}
