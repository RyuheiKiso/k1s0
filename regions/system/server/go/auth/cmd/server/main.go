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

	"github.com/k1s0-platform/system-library-go-telemetry"

	authgrpc "github.com/k1s0-platform/system-server-go-auth/internal/adapter/grpc"
	"github.com/k1s0-platform/system-server-go-auth/internal/adapter/gateway"
	"github.com/k1s0-platform/system-server-go-auth/internal/adapter/handler"
	"github.com/k1s0-platform/system-server-go-auth/internal/adapter/middleware"
	"github.com/k1s0-platform/system-server-go-auth/internal/infra/auth"
	"github.com/k1s0-platform/system-server-go-auth/internal/infra/config"
	"github.com/k1s0-platform/system-server-go-auth/internal/infra/messaging"
	"github.com/k1s0-platform/system-server-go-auth/internal/infra/persistence"
	"github.com/k1s0-platform/system-server-go-auth/internal/usecase"
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

	// --- Telemetry ---
	telemetryCfg := telemetry.TelemetryConfig{
		ServiceName: cfg.App.Name,
		Version:     cfg.App.Version,
		Tier:        cfg.App.Tier,
		Environment: cfg.App.Environment,
		SampleRate:  1.0,
		LogLevel:    "info",
	}
	tp, err := telemetry.InitTelemetry(context.Background(), telemetryCfg)
	if err != nil {
		slog.Error("failed to init telemetry", "error", err)
		os.Exit(1)
	}
	defer tp.Shutdown(context.Background())
	slog.SetDefault(tp.Logger())

	// --- Metrics ---
	metrics := telemetry.NewMetrics(cfg.App.Name)

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
	jwksVerifier := auth.NewJWKSVerifier(cfg.Auth.OIDC.JWKSURI, 1*time.Hour)
	keycloakClient := gateway.NewKeycloakClient(cfg.Auth.OIDC)
	auditRepo := persistence.NewAuditLogRepository(db)

	// Usecases
	jwtConfig := usecase.JWTConfig{
		Issuer:   cfg.Auth.JWT.Issuer,
		Audience: cfg.Auth.JWT.Audience,
	}
	validateTokenUC := usecase.NewValidateTokenUseCase(jwksVerifier, jwtConfig)
	getUserUC := usecase.NewGetUserUseCase(keycloakClient)
	listUsersUC := usecase.NewListUsersUseCase(keycloakClient)
	checkPermissionUC := usecase.NewCheckPermissionUseCase()
	recordAuditLogUC := usecase.NewRecordAuditLogUseCase(auditRepo, producer)
	searchAuditLogsUC := usecase.NewSearchAuditLogsUseCase(auditRepo)

	// --- REST Router ---
	if cfg.App.Environment == "production" {
		gin.SetMode(gin.ReleaseMode)
	}
	r := gin.New()
	r.Use(gin.Recovery())
	r.Use(middleware.RequestID())

	// ヘルスチェック
	r.GET("/healthz", handler.HealthzHandler())
	r.GET("/readyz", handler.ReadyzHandler(db, keycloakClient))

	// メトリクス
	_ = metrics
	r.GET("/metrics", gin.WrapH(telemetry.MetricsHandler()))

	// Auth ハンドラー
	authHandler := handler.NewAuthHandler(validateTokenUC, getUserUC, listUsersUC, checkPermissionUC)
	authHandler.RegisterRoutes(r)

	// Audit ハンドラー
	auditHandler := handler.NewAuditHandler(recordAuditLogUC, searchAuditLogsUC)
	auditHandler.RegisterRoutes(r)

	// --- gRPC Server ---
	authGRPCSvc := authgrpc.NewAuthGRPCService(validateTokenUC, getUserUC, listUsersUC)
	auditGRPCSvc := authgrpc.NewAuditGRPCService(recordAuditLogUC, searchAuditLogsUC)

	grpcServer := grpc.NewServer()
	authgrpc.RegisterAuthServiceServer(grpcServer, authGRPCSvc)
	authgrpc.RegisterAuditServiceServer(grpcServer, auditGRPCSvc)

	grpcPort := 50051
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
