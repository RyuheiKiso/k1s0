package outbox

import (
	"context"
	"encoding/json"
	"log"
	"time"

	domainevent "github.com/RyuheiKiso/k1s0/framework/backend/go/k1s0-domain-event"
)

// Relay polls the outbox store and publishes pending entries.
type Relay struct {
	store        Store
	publisher    domainevent.EventPublisher
	pollInterval time.Duration
	batchSize    int
	maxRetries   int
}

// RelayOption configures a Relay.
type RelayOption func(*Relay)

// WithPollInterval sets the polling interval.
func WithPollInterval(d time.Duration) RelayOption {
	return func(r *Relay) { r.pollInterval = d }
}

// WithBatchSize sets the batch size for fetching pending entries.
func WithBatchSize(n int) RelayOption {
	return func(r *Relay) { r.batchSize = n }
}

// WithMaxRetries sets the maximum retry count before marking as failed.
func WithMaxRetries(n int) RelayOption {
	return func(r *Relay) { r.maxRetries = n }
}

// NewRelay creates a new outbox Relay with the given options.
func NewRelay(store Store, publisher domainevent.EventPublisher, opts ...RelayOption) *Relay {
	r := &Relay{
		store:        store,
		publisher:    publisher,
		pollInterval: 5 * time.Second,
		batchSize:    100,
		maxRetries:   3,
	}
	for _, opt := range opts {
		opt(r)
	}
	return r
}

// Run starts the relay loop. It blocks until the context is cancelled.
func (r *Relay) Run(ctx context.Context) {
	ticker := time.NewTicker(r.pollInterval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			if err := r.processBatch(ctx); err != nil {
				log.Printf("outbox relay batch error: %v", err)
			}
		}
	}
}

func (r *Relay) processBatch(ctx context.Context) error {
	entries, err := r.store.FetchPending(ctx, r.batchSize)
	if err != nil {
		return err
	}

	for _, entry := range entries {
		envelope := &domainevent.EventEnvelope{
			EventType: entry.EventType,
			Metadata:  domainevent.NewEventMetadata("outbox-relay"),
			Payload:   json.RawMessage(entry.Payload),
		}

		if pubErr := r.publisher.Publish(ctx, envelope); pubErr != nil {
			errMsg := pubErr.Error()
			if entry.RetryCount >= r.maxRetries {
				log.Printf("outbox entry %s exceeded max retries", entry.ID)
			}
			if markErr := r.store.MarkFailed(ctx, entry.ID, errMsg); markErr != nil {
				return markErr
			}
			continue
		}

		if markErr := r.store.MarkPublished(ctx, entry.ID); markErr != nil {
			return markErr
		}
	}

	return nil
}
