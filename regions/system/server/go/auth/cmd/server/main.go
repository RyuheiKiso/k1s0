package main

import (
	"context"
	"fmt"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/gin-gonic/gin"

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

	// Auth ハンドラー
	authHandler := handler.NewAuthHandler(validateTokenUC, getUserUC, listUsersUC)
	authHandler.RegisterRoutes(r)

	// Audit ハンドラー
	auditHandler := handler.NewAuditHandler(recordAuditLogUC, searchAuditLogsUC)
	auditHandler.RegisterRoutes(r)

	// --- Start Server ---
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
	slog.Info("shutting down server...")

	shutdownTimeout := cfg.Server.ShutdownTimeout
	if shutdownTimeout == 0 {
		shutdownTimeout = 30 * time.Second
	}
	ctx, cancel := context.WithTimeout(context.Background(), shutdownTimeout)
	defer cancel()

	if err := srv.Shutdown(ctx); err != nil {
		slog.Error("server forced to shutdown", "error", err)
	}
	slog.Info("server exited")
}
