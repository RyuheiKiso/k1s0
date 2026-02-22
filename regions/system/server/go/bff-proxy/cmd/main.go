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
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/redis/go-redis/v9"
	"go.opentelemetry.io/contrib/instrumentation/github.com/gin-gonic/gin/otelgin"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
	"go.opentelemetry.io/otel/sdk/resource"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	semconv "go.opentelemetry.io/otel/semconv/v1.24.0"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/config"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/handler"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/middleware"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/oauth"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
)

func main() {
	if err := run(); err != nil {
		fmt.Fprintf(os.Stderr, "fatal: %v\n", err)
		os.Exit(1)
	}
}

func run() error {
	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	// Load configuration.
	configPath := os.Getenv("CONFIG_PATH")
	if configPath == "" {
		configPath = "config/config.yaml"
	}
	envConfigPath := os.Getenv("ENV_CONFIG_PATH")

	cfg, err := config.Load(configPath, envConfigPath)
	if err != nil {
		return fmt.Errorf("failed to load config: %w", err)
	}

	// Initialize logger.
	logger := newLogger(cfg.Observability.Log)

	// Initialize Redis client.
	var redisClient redis.Cmdable
	if cfg.Session.Redis.MasterName != "" {
		redisClient = redis.NewFailoverClient(&redis.FailoverOptions{
			MasterName:    cfg.Session.Redis.MasterName,
			SentinelAddrs: []string{cfg.Session.Redis.Addr},
			Password:      cfg.Session.Redis.Password,
			DB:            cfg.Session.Redis.DB,
		})
	} else {
		redisClient = redis.NewClient(&redis.Options{
			Addr:     cfg.Session.Redis.Addr,
			Password: cfg.Session.Redis.Password,
			DB:       cfg.Session.Redis.DB,
		})
	}

	// Verify Redis connectivity.
	if err := redisClient.(*redis.Client).Ping(ctx).Err(); err != nil {
		logger.Warn("Redis not reachable at startup", slog.String("error", err.Error()))
	}

	// Initialize session store.
	prefix := cfg.Session.Prefix
	if prefix == "" {
		prefix = "bff:session:"
	}
	sessionStore := session.NewRedisStore(redisClient, prefix)
	sessionTTL := config.ParseDuration(cfg.Session.TTL, 30*time.Minute)

	// Initialize OIDC client.
	oauthClient := oauth.NewClient(
		cfg.Auth.DiscoveryURL,
		cfg.Auth.ClientID,
		cfg.Auth.ClientSecret,
		cfg.Auth.RedirectURI,
		cfg.Auth.Scopes,
	)

	// Perform OIDC discovery.
	if _, err := oauthClient.Discover(ctx); err != nil {
		logger.Warn("OIDC discovery failed at startup", slog.String("error", err.Error()))
	}

	// Determine secure cookies based on environment.
	secureCookie := cfg.App.Environment != "dev"

	// Initialize handlers.
	healthHandler := handler.NewHealthHandler(redisClient)
	authHandler := handler.NewAuthHandler(
		oauthClient, sessionStore, sessionTTL,
		cfg.Auth.PostLogout, secureCookie, logger,
	)

	upstreamTimeout := config.ParseDuration(cfg.Upstream.Timeout, 30*time.Second)
	proxyHandler, err := handler.NewProxyHandler(
		cfg.Upstream.BaseURL, sessionStore, oauthClient,
		sessionTTL, upstreamTimeout, logger,
	)
	if err != nil {
		return fmt.Errorf("failed to create proxy handler: %w", err)
	}

	// Initialize OpenTelemetry tracer provider.
	tp, err := initTracerProvider(ctx)
	if err != nil {
		logger.Warn("Failed to initialize OTel tracer provider", slog.String("error", err.Error()))
	} else {
		defer func() {
			_ = tp.Shutdown(context.Background())
		}()
	}

	// Set up Gin router.
	if cfg.App.Environment == "prod" {
		gin.SetMode(gin.ReleaseMode)
	}
	router := gin.New()
	router.Use(gin.Recovery())
	router.Use(middleware.PrometheusMiddleware())
	router.Use(otelgin.Middleware("bff-proxy"))
	router.Use(middleware.OTelTraceIDMiddleware())
	router.Use(middleware.CorrelationMiddleware())

	// Health / Metrics endpoints (no auth required).
	router.GET("/healthz", healthHandler.Healthz)
	router.GET("/readyz", healthHandler.Readyz)
	router.GET("/metrics", gin.WrapH(promhttp.Handler()))

	// Auth endpoints (no session required).
	router.GET("/auth/login", authHandler.Login)
	router.GET("/auth/callback", authHandler.Callback)
	router.POST("/auth/logout", authHandler.Logout)

	// Proxy endpoints (session + CSRF required).
	api := router.Group("/api")
	api.Use(middleware.SessionMiddleware(sessionStore, handler.CookieName, sessionTTL, cfg.Session.Sliding))
	if cfg.CSRF.Enabled {
		csrfHeader := cfg.CSRF.HeaderName
		if csrfHeader == "" {
			csrfHeader = middleware.DefaultCSRFHeader
		}
		api.Use(middleware.CSRFMiddleware(sessionStore, csrfHeader, handler.CookieName))
	}
	api.Any("/*path", proxyHandler.Handle)

	// Start HTTP server.
	addr := fmt.Sprintf("%s:%d", cfg.Server.Host, cfg.Server.Port)
	srv := &http.Server{
		Addr:         addr,
		Handler:      router,
		ReadTimeout:  config.ParseDuration(cfg.Server.ReadTimeout, 10*time.Second),
		WriteTimeout: config.ParseDuration(cfg.Server.WriteTimeout, 30*time.Second),
	}

	// Start server in a goroutine.
	errCh := make(chan error, 1)
	go func() {
		logger.Info("BFF Proxy starting", slog.String("addr", addr))
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			errCh <- err
		}
	}()

	// Wait for shutdown signal.
	select {
	case <-ctx.Done():
		logger.Info("Shutdown signal received")
	case err := <-errCh:
		return fmt.Errorf("server error: %w", err)
	}

	// Graceful shutdown.
	shutdownTimeout := config.ParseDuration(cfg.Server.ShutdownTimeout, 15*time.Second)
	shutdownCtx, cancel := context.WithTimeout(context.Background(), shutdownTimeout)
	defer cancel()

	if err := srv.Shutdown(shutdownCtx); err != nil {
		return fmt.Errorf("server shutdown error: %w", err)
	}

	logger.Info("BFF Proxy stopped")
	return nil
}

func initTracerProvider(ctx context.Context) (*sdktrace.TracerProvider, error) {
	endpoint := os.Getenv("OTEL_EXPORTER_OTLP_ENDPOINT")
	if endpoint == "" {
		endpoint = "localhost:4317"
	}

	exporter, err := otlptracegrpc.New(ctx,
		otlptracegrpc.WithEndpoint(endpoint),
		otlptracegrpc.WithInsecure(),
	)
	if err != nil {
		return nil, fmt.Errorf("failed to create OTLP exporter: %w", err)
	}

	res, err := resource.New(ctx,
		resource.WithAttributes(
			semconv.ServiceNameKey.String("bff-proxy"),
		),
	)
	if err != nil {
		return nil, fmt.Errorf("failed to create resource: %w", err)
	}

	tp := sdktrace.NewTracerProvider(
		sdktrace.WithBatcher(exporter),
		sdktrace.WithResource(res),
	)
	otel.SetTracerProvider(tp)
	return tp, nil
}

func newLogger(logCfg config.LogConfig) *slog.Logger {
	level := slog.LevelInfo
	switch logCfg.Level {
	case "debug":
		level = slog.LevelDebug
	case "info":
		level = slog.LevelInfo
	case "warn":
		level = slog.LevelWarn
	case "error":
		level = slog.LevelError
	}

	handler := slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{Level: level})
	return slog.New(handler)
}
