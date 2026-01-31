package consensus

import (
	"fmt"
	"sync/atomic"
)

// FencingValidator validates that fence tokens are monotonically increasing.
// It is used to detect stale lock holders that may still attempt to perform
// operations after their lock has expired and been re-acquired by another node.
//
// Usage:
//
//	validator := consensus.NewFencingValidator()
//	if !validator.Validate(guard.FenceToken) {
//	    // Stale operation detected, reject it.
//	}
type FencingValidator struct {
	highestToken atomic.Uint64
}

// NewFencingValidator creates a new FencingValidator.
func NewFencingValidator() *FencingValidator {
	return &FencingValidator{}
}

// Validate checks that the given token is strictly greater than any
// previously validated token. Returns true if the token is valid
// (monotonically increasing), false otherwise.
func (v *FencingValidator) Validate(token uint64) bool {
	for {
		current := v.highestToken.Load()
		if token <= current {
			metricsFenceTokenViolations.Inc()
			return false
		}
		if v.highestToken.CompareAndSwap(current, token) {
			return true
		}
		// CAS failed, another goroutine updated; retry.
	}
}

// ValidateOrError is like Validate but returns an error on failure.
func (v *FencingValidator) ValidateOrError(token uint64) error {
	if !v.Validate(token) {
		return fmt.Errorf("consensus: token %d is not greater than current: %w",
			token, ErrFenceTokenViolation)
	}
	return nil
}

// Current returns the highest validated token seen so far.
func (v *FencingValidator) Current() uint64 {
	return v.highestToken.Load()
}

// Reset resets the validator to its initial state (token 0).
func (v *FencingValidator) Reset() {
	v.highestToken.Store(0)
}
