package config

import (
	"context"
	"fmt"
	"io"

	configv1 "github.com/k1s0-platform/api/gen/go/k1s0/system/config/v1"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

// ConfigChangeEvent represents a single config change notification.
type ConfigChangeEvent struct {
	Namespace  string
	Key        string
	OldValue   []byte
	NewValue   []byte
	ChangeType string
}

// WatchConfigClient wraps the gRPC ConfigServiceClient for streaming config changes.
type WatchConfigClient struct {
	conn   *grpc.ClientConn
	client configv1.ConfigServiceClient
}

// NewWatchConfigClient creates a new WatchConfigClient connected to target.
// The caller is responsible for calling Close when done.
func NewWatchConfigClient(ctx context.Context, target string, opts ...grpc.DialOption) (*WatchConfigClient, error) {
	if len(opts) == 0 {
		opts = append(opts, grpc.WithTransportCredentials(insecure.NewCredentials()))
	}
	conn, err := grpc.NewClient(target, opts...)
	if err != nil {
		return nil, fmt.Errorf("watch config: dial %s: %w", target, err)
	}
	return &WatchConfigClient{
		conn:   conn,
		client: configv1.NewConfigServiceClient(conn),
	}, nil
}

// Watch opens a server-streaming RPC and returns a channel of ConfigChangeEvents.
// The channel is closed when the stream ends or ctx is cancelled.
func (w *WatchConfigClient) Watch(ctx context.Context, namespaces []string) (<-chan ConfigChangeEvent, error) {
	stream, err := w.client.WatchConfig(ctx, &configv1.WatchConfigRequest{
		Namespaces: namespaces,
	})
	if err != nil {
		return nil, fmt.Errorf("watch config: open stream: %w", err)
	}

	ch := make(chan ConfigChangeEvent, 64)
	go func() {
		defer close(ch)
		for {
			resp, err := stream.Recv()
			if err != nil {
				if err == io.EOF {
					return
				}
				select {
				case <-ctx.Done():
				default:
				}
				return
			}
			ch <- ConfigChangeEvent{
				Namespace:  resp.GetNamespace(),
				Key:        resp.GetKey(),
				OldValue:   resp.GetOldValue(),
				NewValue:   resp.GetNewValue(),
				ChangeType: resp.GetChangeType(),
			}
		}
	}()
	return ch, nil
}

// Close closes the underlying gRPC connection.
func (w *WatchConfigClient) Close() error {
	return w.conn.Close()
}
