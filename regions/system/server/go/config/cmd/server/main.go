package main

import (
	"context"
	"fmt"
	"log/slog"
	"net"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/gin-gonic/gin"
	"google.golang.org/grpc"

	"github.com/k1s0-platform/system-server-go-config/internal/adapter/handler"
	"github.com/k1s0-platform/system-server-go-config/internal/adapter/middleware"
	configrepo "github.com/k1s0-platform/system-server-go-config/internal/adapter/repository"
	"github.com/k1s0-platform/system-server-go-config/internal/infra/config"
	"github.com/k1s0-platform/system-server-go-config/internal/infra/messaging"
	"github.com/k1s0-platform/system-server-go-config/internal/infra/persistence"
	"github.com/k1s0-platform/system-server-go-config/internal/usecase"
)

func main() {
	// --- Config ---
	cfg, err := config.Load("config/config.yaml")
	if err != nil {
		slog.Error("failed to load config", "error", err)
		os.Exit(1)
	}
	if err := cfg.Validate(); err != nil {
		slog.Error("config validation failed", "error", err)
		os.Exit(1)
	}

	// --- Logger ---
	logger := config.NewLogger(
		cfg.App.Environment, cfg.App.Name, cfg.App.Version, cfg.App.Tier,
	)
	slog.SetDefault(logger)

	// --- Database ---
	db, err := persistence.NewDB(cfg.Database)
	if err != nil {
		slog.Error("failed to connect database", "error", err)
		os.Exit(1)
	}
	defer db.Close()

	// --- Kafka ---
	producer := messaging.NewKafkaProducer(cfg.Kafka)
	defer producer.Close()

	// --- DI ---
	repo := configrepo.NewConfigPostgresRepository(db)

	// Usecases
	getConfigUC := usecase.NewGetConfigUseCase(repo)
	listConfigsUC := usecase.NewListConfigsUseCase(repo)
	updateConfigUC := usecase.NewUpdateConfigUseCase(repo, producer)
	deleteConfigUC := usecase.NewDeleteConfigUseCase(repo, producer)
	getServiceConfigUC := usecase.NewGetServiceConfigUseCase(repo)

	// --- REST Router ---
	if cfg.App.Environment == "production" {
		gin.SetMode(gin.ReleaseMode)
	}
	r := gin.New()
	r.Use(gin.Recovery())
	r.Use(middleware.RequestID())

	// ヘルスチェック
	r.GET("/healthz", handler.HealthzHandler())
	r.GET("/readyz", handler.ReadyzHandler(db, producer))

	// Config ハンドラー
	configHandler := handler.NewConfigHandler(
		getConfigUC, listConfigsUC, updateConfigUC, deleteConfigUC, getServiceConfigUC,
	)
	configHandler.RegisterRoutes(r)

	// --- gRPC Server ---
	grpcServer := grpc.NewServer()
	// TODO: pb.RegisterConfigServiceServer(grpcServer, configGRPCSvc) — proto 生成後に有効化

	grpcPort := cfg.GRPC.Port
	if grpcPort == 0 {
		grpcPort = 50053
	}
	go func() {
		lis, err := net.Listen("tcp", fmt.Sprintf(":%d", grpcPort))
		if err != nil {
			slog.Error("failed to listen for gRPC", "error", err)
			os.Exit(1)
		}
		slog.Info("gRPC server starting", "port", grpcPort)
		if err := grpcServer.Serve(lis); err != nil {
			slog.Error("gRPC server failed", "error", err)
			os.Exit(1)
		}
	}()

	// --- REST Server ---
	srv := &http.Server{
		Addr:         fmt.Sprintf(":%d", cfg.Server.Port),
		Handler:      r,
		ReadTimeout:  cfg.Server.ReadTimeout,
		WriteTimeout: cfg.Server.WriteTimeout,
	}

	go func() {
		slog.Info("REST server starting", "port", cfg.Server.Port)
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			slog.Error("REST server failed", "error", err)
			os.Exit(1)
		}
	}()

	// --- Graceful Shutdown ---
	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit
	slog.Info("shutting down servers...")

	// gRPC graceful stop
	grpcServer.GracefulStop()
	slog.Info("gRPC server stopped")

	// REST graceful shutdown
	shutdownTimeout := cfg.Server.ShutdownTimeout
	if shutdownTimeout == 0 {
		shutdownTimeout = 30 * time.Second
	}
	ctx, cancel := context.WithTimeout(context.Background(), shutdownTimeout)
	defer cancel()

	if err := srv.Shutdown(ctx); err != nil {
		slog.Error("REST server forced to shutdown", "error", err)
	}
	slog.Info("servers exited")
}
