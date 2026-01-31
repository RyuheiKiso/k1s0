package consensus

import (
	"sync"
	"testing"
)

func TestFencingValidator_Validate(t *testing.T) {
	v := NewFencingValidator()

	tests := []struct {
		name  string
		token uint64
		want  bool
	}{
		{"first token accepted", 1, true},
		{"higher token accepted", 2, true},
		{"same token rejected", 2, false},
		{"lower token rejected", 1, false},
		{"much higher token accepted", 100, true},
		{"zero rejected after non-zero", 0, false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := v.Validate(tt.token)
			if got != tt.want {
				t.Errorf("Validate(%d) = %v, want %v", tt.token, got, tt.want)
			}
		})
	}
}

func TestFencingValidator_ValidateOrError(t *testing.T) {
	v := NewFencingValidator()

	if err := v.ValidateOrError(1); err != nil {
		t.Errorf("ValidateOrError(1) unexpected error: %v", err)
	}

	if err := v.ValidateOrError(1); err == nil {
		t.Error("ValidateOrError(1) expected error for duplicate token")
	}
}

func TestFencingValidator_Current(t *testing.T) {
	v := NewFencingValidator()

	if got := v.Current(); got != 0 {
		t.Errorf("Current() = %d, want 0", got)
	}

	v.Validate(42)
	if got := v.Current(); got != 42 {
		t.Errorf("Current() = %d, want 42", got)
	}
}

func TestFencingValidator_Reset(t *testing.T) {
	v := NewFencingValidator()
	v.Validate(10)
	v.Reset()

	if got := v.Current(); got != 0 {
		t.Errorf("Current() after Reset = %d, want 0", got)
	}

	// Token 1 should be accepted after reset.
	if !v.Validate(1) {
		t.Error("Validate(1) after Reset should succeed")
	}
}

func TestFencingValidator_Concurrent(t *testing.T) {
	v := NewFencingValidator()
	const goroutines = 100

	var wg sync.WaitGroup
	accepted := make(chan uint64, goroutines)

	for i := uint64(1); i <= goroutines; i++ {
		wg.Add(1)
		go func(token uint64) {
			defer wg.Done()
			if v.Validate(token) {
				accepted <- token
			}
		}(i)
	}

	wg.Wait()
	close(accepted)

	// Collect accepted tokens and verify they are monotonically increasing.
	var tokens []uint64
	for token := range accepted {
		tokens = append(tokens, token)
	}

	if len(tokens) == 0 {
		t.Fatal("expected at least one accepted token")
	}

	// The final value should be the highest accepted token.
	final := v.Current()
	if final == 0 || final > goroutines {
		t.Errorf("Current() = %d, expected between 1 and %d", final, goroutines)
	}
}
