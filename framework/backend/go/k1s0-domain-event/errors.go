package domainevent

import "errors"

var (
	// ErrPublishFailed indicates that event publishing failed.
	ErrPublishFailed = errors.New("failed to publish event")

	// ErrSubscribeFailed indicates that event subscription failed.
	ErrSubscribeFailed = errors.New("failed to subscribe to event")

	// ErrHandlerFailed indicates that an event handler failed.
	ErrHandlerFailed = errors.New("event handler failed")

	// ErrOutboxDatabase indicates an outbox database operation failure.
	ErrOutboxDatabase = errors.New("outbox database error")

	// ErrSerialization indicates a serialization/deserialization failure.
	ErrSerialization = errors.New("serialization error")
)

// PublishError wraps a publish failure with context.
type PublishError struct {
	Cause error
	Msg   string
}

func (e *PublishError) Error() string {
	if e.Cause != nil {
		return e.Msg + ": " + e.Cause.Error()
	}
	return e.Msg
}

func (e *PublishError) Unwrap() error { return e.Cause }
