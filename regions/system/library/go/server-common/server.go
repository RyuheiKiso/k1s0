package servercommon

import (
	"context"
	"fmt"
	"net"
	"net/http"
	"time"
)

// GRPCServable は gRPC サーバーの起動・停止インターフェース。
// 具体的な gRPC 実装（google.golang.org/grpc）への依存を避けるため、
// インターフェースで抽象化する。
type GRPCServable interface {
	// Serve は net.Listener でリクエストを受け付ける。
	Serve(net.Listener) error
	// GracefulStop は進行中のリクエスト完了を待ってから停止する。
	GracefulStop()
}

// ServerConfig は HTTP と gRPC サーバーの統合設定を保持する。
type ServerConfig struct {
	// HTTPAddr は HTTP サーバーのリッスンアドレス（例: ":8080"）。
	HTTPAddr string
	// GRPCAddr は gRPC サーバーのリッスンアドレス（例: ":50051"）。
	// 空の場合は gRPC サーバーを起動しない。
	GRPCAddr string
}

// Server は HTTP と gRPC を統合して管理するサーバー。
// 両プロトコルを同一プロセスで起動・停止する。
type Server struct {
	cfg        ServerConfig
	httpServer *http.Server
	grpcServer GRPCServable
}

// NewServer は Server を生成する。
// grpcServer が nil の場合は HTTP のみ起動する。
func NewServer(cfg ServerConfig, httpHandler http.Handler, grpcServer GRPCServable) *Server {
	addr := cfg.HTTPAddr
	if addr == "" {
		addr = ":8080"
	}
	return &Server{
		cfg:        cfg,
		grpcServer: grpcServer,
		httpServer: &http.Server{
			Addr:              addr,
			Handler:           httpHandler,
			// Slowloris攻撃を防止するためのヘッダー読み取りタイムアウト（app.go と同じパターン）
			ReadHeaderTimeout: 10 * time.Second,
		},
	}
}

// Run は HTTP と gRPC サーバーを起動し、ctx がキャンセルされるまで待機する。
// どちらかのサーバーでエラーが発生した場合は即座に返る。
func (s *Server) Run(ctx context.Context) error {
	errCh := make(chan error, 2)

	// HTTP サーバーを起動する
	go func() {
		if err := s.httpServer.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			errCh <- fmt.Errorf("http server: %w", err)
		}
	}()

	// gRPC サーバーを起動する（設定されている場合のみ）
	if s.grpcServer != nil && s.cfg.GRPCAddr != "" {
		go func() {
			lis, err := net.Listen("tcp", s.cfg.GRPCAddr)
			if err != nil {
				errCh <- fmt.Errorf("grpc listen %s: %w", s.cfg.GRPCAddr, err)
				return
			}
			if err := s.grpcServer.Serve(lis); err != nil {
				errCh <- fmt.Errorf("grpc server: %w", err)
			}
		}()
	}

	select {
	case <-ctx.Done():
		return s.shutdown()
	case err := <-errCh:
		return err
	}
}

// shutdown はグレースフルシャットダウンを実行する。
// gRPC は GracefulStop()、HTTP は Shutdown() で停止する。
func (s *Server) shutdown() error {
	if s.grpcServer != nil {
		// gRPC は既存リクエストの完了を待ってから停止する
		s.grpcServer.GracefulStop()
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultShutdownTimeout)
	defer cancel()

	if err := s.httpServer.Shutdown(ctx); err != nil {
		return fmt.Errorf("http server shutdown: %w", err)
	}
	return nil
}
