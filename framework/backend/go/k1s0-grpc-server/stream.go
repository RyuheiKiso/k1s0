package k1s0grpcserver

import (
	"context"
	"errors"
	"sync"
	"sync/atomic"
	"time"
)

// ErrStreamBackpressure is returned when the stream buffer is full and the timeout is exceeded.
var ErrStreamBackpressure = errors.New("stream backpressure: buffer full")

// StreamBackpressureConfig holds configuration for stream flow control.
type StreamBackpressureConfig struct {
	// SendBufferSize is the maximum number of concurrent in-flight sends.
	// Must be at least 1. Default is 16.
	SendBufferSize int `yaml:"send_buffer_size"`

	// SlowProducerTimeout is the maximum time to wait for buffer space.
	// If 0, the call returns immediately when the buffer is full.
	// Default is 5 seconds.
	SlowProducerTimeout time.Duration `yaml:"slow_producer_timeout"`
}

// DefaultStreamBackpressureConfig returns a StreamBackpressureConfig with default values.
func DefaultStreamBackpressureConfig() *StreamBackpressureConfig {
	return &StreamBackpressureConfig{
		SendBufferSize:      16,
		SlowProducerTimeout: 5 * time.Second,
	}
}

// Validate validates the configuration and applies defaults.
func (c *StreamBackpressureConfig) Validate() *StreamBackpressureConfig {
	if c.SendBufferSize < 1 {
		c.SendBufferSize = 16
	}
	if c.SlowProducerTimeout < 0 {
		c.SlowProducerTimeout = 5 * time.Second
	}
	return c
}

// StreamMetrics holds stream backpressure metrics.
type StreamMetrics struct {
	// BufferUsage is the current buffer usage ratio (0.0 to 1.0).
	BufferUsage float64

	// BackpressureCount is the total number of backpressure events.
	BackpressureCount int64

	mu sync.RWMutex
}

// FlowControlledStream provides backpressure-aware gRPC streaming.
//
// It uses a semaphore to limit concurrent in-flight sends, applying
// backpressure when the buffer is full.
//
// Example:
//
//	config := k1s0grpcserver.DefaultStreamBackpressureConfig()
//	stream := k1s0grpcserver.NewFlowControlledStream(config)
//
//	// Before sending a message
//	if err := stream.Acquire(ctx); err != nil {
//	    // Handle backpressure
//	}
//	defer stream.Release()
//	// Send message
type FlowControlledStream struct {
	config            *StreamBackpressureConfig
	sem               chan struct{}
	backpressureCount int64
	acquireCount      int64
}

// NewFlowControlledStream creates a new FlowControlledStream with the given configuration.
func NewFlowControlledStream(config *StreamBackpressureConfig) *FlowControlledStream {
	config = config.Validate()
	return &FlowControlledStream{
		config: config,
		sem:    make(chan struct{}, config.SendBufferSize),
	}
}

// Acquire acquires a slot in the send buffer.
// Blocks until a slot is available, the timeout is exceeded, or the context is cancelled.
func (s *FlowControlledStream) Acquire(ctx context.Context) error {
	if ctx.Err() != nil {
		return ctx.Err()
	}

	if s.config.SlowProducerTimeout > 0 {
		select {
		case s.sem <- struct{}{}:
			atomic.AddInt64(&s.acquireCount, 1)
			return nil
		case <-time.After(s.config.SlowProducerTimeout):
			atomic.AddInt64(&s.backpressureCount, 1)
			return ErrStreamBackpressure
		case <-ctx.Done():
			return ctx.Err()
		}
	}

	// No timeout: try immediately
	select {
	case s.sem <- struct{}{}:
		atomic.AddInt64(&s.acquireCount, 1)
		return nil
	default:
		atomic.AddInt64(&s.backpressureCount, 1)
		return ErrStreamBackpressure
	}
}

// Release releases a slot in the send buffer.
func (s *FlowControlledStream) Release() {
	<-s.sem
}

// Stats returns the current stream metrics.
func (s *FlowControlledStream) Stats() StreamMetrics {
	bufferLen := len(s.sem)
	bufferCap := cap(s.sem)
	var usage float64
	if bufferCap > 0 {
		usage = float64(bufferLen) / float64(bufferCap)
	}
	return StreamMetrics{
		BufferUsage:       usage,
		BackpressureCount: atomic.LoadInt64(&s.backpressureCount),
	}
}
