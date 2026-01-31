package k1s0grpcserver

import (
	"context"
	"sync"
	"testing"
	"time"
)

func TestFlowControlledStream_Acquire(t *testing.T) {
	config := &StreamBackpressureConfig{
		SendBufferSize:      2,
		SlowProducerTimeout: time.Second,
	}
	stream := NewFlowControlledStream(config)
	ctx := context.Background()

	if err := stream.Acquire(ctx); err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if err := stream.Acquire(ctx); err != nil {
		t.Errorf("expected no error, got %v", err)
	}

	stream.Release()
	stream.Release()
}

func TestFlowControlledStream_BackpressureWithTimeout(t *testing.T) {
	config := &StreamBackpressureConfig{
		SendBufferSize:      1,
		SlowProducerTimeout: 50 * time.Millisecond,
	}
	stream := NewFlowControlledStream(config)
	ctx := context.Background()

	// Fill the buffer
	if err := stream.Acquire(ctx); err != nil {
		t.Fatalf("expected no error, got %v", err)
	}

	// Should timeout
	start := time.Now()
	err := stream.Acquire(ctx)
	elapsed := time.Since(start)

	if err != ErrStreamBackpressure {
		t.Errorf("expected ErrStreamBackpressure, got %v", err)
	}
	if elapsed < 40*time.Millisecond {
		t.Errorf("expected to wait ~50ms, waited %v", elapsed)
	}

	stream.Release()
}

func TestFlowControlledStream_BackpressureImmediate(t *testing.T) {
	config := &StreamBackpressureConfig{
		SendBufferSize:      1,
		SlowProducerTimeout: 0,
	}
	stream := NewFlowControlledStream(config)
	ctx := context.Background()

	_ = stream.Acquire(ctx)

	err := stream.Acquire(ctx)
	if err != ErrStreamBackpressure {
		t.Errorf("expected ErrStreamBackpressure, got %v", err)
	}

	stream.Release()
}

func TestFlowControlledStream_ContextCancelled(t *testing.T) {
	config := &StreamBackpressureConfig{
		SendBufferSize:      1,
		SlowProducerTimeout: time.Second,
	}
	stream := NewFlowControlledStream(config)
	ctx, cancel := context.WithCancel(context.Background())

	_ = stream.Acquire(ctx)

	cancel()
	err := stream.Acquire(ctx)
	if err == nil {
		t.Error("expected error for cancelled context")
	}

	stream.Release()
}

func TestFlowControlledStream_ReleaseUnblocks(t *testing.T) {
	config := &StreamBackpressureConfig{
		SendBufferSize:      1,
		SlowProducerTimeout: time.Second,
	}
	stream := NewFlowControlledStream(config)
	ctx := context.Background()

	_ = stream.Acquire(ctx)

	done := make(chan error, 1)
	go func() {
		done <- stream.Acquire(ctx)
	}()

	time.Sleep(10 * time.Millisecond)
	stream.Release()

	select {
	case err := <-done:
		if err != nil {
			t.Errorf("expected no error after release, got %v", err)
		}
		stream.Release() // release second acquire
	case <-time.After(time.Second):
		t.Error("expected acquire to succeed after release")
	}
}

func TestFlowControlledStream_Concurrent(t *testing.T) {
	config := &StreamBackpressureConfig{
		SendBufferSize:      5,
		SlowProducerTimeout: 100 * time.Millisecond,
	}
	stream := NewFlowControlledStream(config)
	ctx := context.Background()

	var wg sync.WaitGroup
	for i := 0; i < 20; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			if err := stream.Acquire(ctx); err == nil {
				time.Sleep(10 * time.Millisecond)
				stream.Release()
			}
		}()
	}
	wg.Wait()
}

func TestFlowControlledStream_Stats(t *testing.T) {
	config := &StreamBackpressureConfig{
		SendBufferSize:      2,
		SlowProducerTimeout: 0,
	}
	stream := NewFlowControlledStream(config)
	ctx := context.Background()

	_ = stream.Acquire(ctx)

	stats := stream.Stats()
	if stats.BufferUsage != 0.5 {
		t.Errorf("expected 0.5 usage, got %f", stats.BufferUsage)
	}

	_ = stream.Acquire(ctx)

	stats = stream.Stats()
	if stats.BufferUsage != 1.0 {
		t.Errorf("expected 1.0 usage, got %f", stats.BufferUsage)
	}

	// Trigger backpressure
	_ = stream.Acquire(ctx)

	stats = stream.Stats()
	if stats.BackpressureCount != 1 {
		t.Errorf("expected 1 backpressure, got %d", stats.BackpressureCount)
	}

	stream.Release()
	stream.Release()
}

func TestStreamBackpressureConfig_Validate(t *testing.T) {
	config := &StreamBackpressureConfig{SendBufferSize: -1, SlowProducerTimeout: -1}
	validated := config.Validate()

	if validated.SendBufferSize != 16 {
		t.Errorf("expected 16, got %d", validated.SendBufferSize)
	}
	if validated.SlowProducerTimeout != 5*time.Second {
		t.Errorf("expected 5s, got %v", validated.SlowProducerTimeout)
	}
}

func TestDefaultStreamBackpressureConfig(t *testing.T) {
	config := DefaultStreamBackpressureConfig()

	if config.SendBufferSize != 16 {
		t.Errorf("expected 16, got %d", config.SendBufferSize)
	}
	if config.SlowProducerTimeout != 5*time.Second {
		t.Errorf("expected 5s, got %v", config.SlowProducerTimeout)
	}
}
