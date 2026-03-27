package config

import (
	"context"
	"fmt"
	"io"
	"os"

	commonv1 "github.com/k1s0-platform/api/gen/go/k1s0/system/common/v1"
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
	// ChangeType は変更種別を表す enum。CHANGE_TYPE_CREATED / UPDATED / DELETED 等。
	// 旧フィールド（deprecated string）から enum 型に移行済み（L-5 監査対応: SA1019 解消）。
	ChangeType commonv1.ChangeType
}

// WatchConfigClient wraps the gRPC ConfigServiceClient for streaming config changes.
type WatchConfigClient struct {
	conn   *grpc.ClientConn
	client configv1.ConfigServiceClient
}

// NewWatchConfigClient は TLS 必須でターゲットに接続する WatchConfigClient を生成する。
// 追加の DialOption を渡すことで証明書や認証情報を上書きできる。
// 呼び出し元は使用終了後に Close を呼び出すこと。
func NewWatchConfigClient(ctx context.Context, target string, opts ...grpc.DialOption) (*WatchConfigClient, error) {
	conn, err := grpc.NewClient(target, opts...)
	if err != nil {
		return nil, fmt.Errorf("watch config: dial %s: %w", target, err)
	}
	return &WatchConfigClient{
		conn:   conn,
		client: configv1.NewConfigServiceClient(conn),
	}, nil
}

// NewInsecureWatchConfigClient は TLS なし（開発・テスト用）でターゲットに接続する WatchConfigClient を生成する。
// 本番環境では使用しないこと。
// 本番・ステージング環境では insecure 接続を禁止する（S-1: TLS強制化対応）
func NewInsecureWatchConfigClient(ctx context.Context, target string, opts ...grpc.DialOption) (*WatchConfigClient, error) {
	if env := os.Getenv("APP_ENV"); env == "production" || env == "staging" {
		return nil, fmt.Errorf("watch config: insecure gRPC connection is not allowed in %s environment; use NewWatchConfigClient with TLS credentials", env)
	}
	opts = append([]grpc.DialOption{grpc.WithTransportCredentials(insecure.NewCredentials())}, opts...)
	return NewWatchConfigClient(ctx, target, opts...)
}

// Watch opens a server-streaming RPC and returns a channel of ConfigChangeEvents and an error channel.
// The event channel is closed when the stream ends or ctx is cancelled.
// ストリームエラー（EOF / ctx キャンセル以外）はエラーチャネル経由で呼び出し元に伝播する。
func (w *WatchConfigClient) Watch(ctx context.Context, namespaces []string) (<-chan ConfigChangeEvent, <-chan error, error) {
	stream, err := w.client.WatchConfig(ctx, &configv1.WatchConfigRequest{
		Namespaces: namespaces,
	})
	if err != nil {
		return nil, nil, fmt.Errorf("watch config: open stream: %w", err)
	}

	ch := make(chan ConfigChangeEvent, 64)
	// バッファ 1: goroutine がブロックせずエラーを送信できるようにする
	errCh := make(chan error, 1)
	go func() {
		defer close(ch)
		defer close(errCh)
		for {
			resp, err := stream.Recv()
			if err != nil {
				if err == io.EOF {
					return
				}
				// ctx キャンセルによる切断は正常終了として扱う
				if ctx.Err() != nil {
					return
				}
				// 予期しないストリームエラーをエラーチャネルで伝播する
				errCh <- fmt.Errorf("watch config: recv: %w", err)
				return
			}
			event := ConfigChangeEvent{
				Namespace: resp.GetNamespace(),
				Key:       resp.GetKey(),
				OldValue:  resp.GetOldValue(),
				NewValue:  resp.GetNewValue(),
				// GetChangeTypeEnum() を使用して deprecated GetChangeType() を回避する（L-5 監査対応）
				ChangeType: resp.GetChangeTypeEnum(),
			}
			// ctx キャンセル時にチャネル送信でブロックしないよう select でバックプレッシャーを制御する
			select {
			case ch <- event:
			case <-ctx.Done():
				return
			}
		}
	}()
	return ch, errCh, nil
}

// Close closes the underlying gRPC connection.
func (w *WatchConfigClient) Close() error {
	return w.conn.Close()
}
