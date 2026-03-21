package main

import (
	"context"
	"fmt"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"sync"
	"syscall"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/go-playground/validator/v10"
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

	// 設定値のバリデーション（validateタグによる必須・範囲チェック）
	validate := validator.New()
	if err := validate.Struct(cfg); err != nil {
		return fmt.Errorf("invalid configuration: %w", err)
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

	// Redis接続を確認する。
	// redis.Cmdable インターフェース経由で Ping を呼び出すことで、
	// スタンドアロン・Sentinel どちらのモードでも安全に動作する。
	// ALLOW_REDIS_SKIP は development 環境のみ有効。production/staging では無視してエラーで終了する。
	if err := redisClient.Ping(ctx).Err(); err != nil {
		env := cfg.App.Environment
		allowSkip := os.Getenv("ALLOW_REDIS_SKIP") == "true" && env == "development"
		if allowSkip {
			logger.Warn("Redis接続に失敗しました。ALLOW_REDIS_SKIP=trueのためスキップします（development環境のみ）", slog.String("error", err.Error()))
		} else {
			logger.Error("Redis接続に失敗しました", slog.String("error", err.Error()), slog.String("environment", env))
			return fmt.Errorf("Redis接続に失敗しました: %w", err)
		}
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

	// OIDC discovery URL が TLS を使用していない場合に警告を出力する。
	// 本番環境では IdP との通信に https を使用すべき。
	if !strings.HasPrefix(cfg.Auth.DiscoveryURL, "https://") {
		logger.Warn("OIDC discovery_url が TLS (https) を使用していません。本番環境では https を使用してください",
			slog.String("discovery_url", cfg.Auth.DiscoveryURL),
			slog.String("environment", cfg.App.Environment),
		)
	}

	// OIDC discoveryを実行する。失敗した場合はバックグラウンドで再試行する。
	// WaitGroupでゴルーチンのライフサイクルを追跡し、シャットダウン時に安全に完了を待機する
	var oidcWg sync.WaitGroup
	if _, err := oauthClient.Discover(ctx); err != nil {
		logger.Warn("OIDC discovery failed at startup, will retry in background", slog.String("error", err.Error()))
		// discoveryが未完了の場合、バックグラウンドで定期的に再試行するゴルーチンを起動する
		oidcWg.Add(1)
		go func() {
			defer oidcWg.Done()
			retryOIDCDiscovery(ctx, oauthClient, logger)
		}()
	}

	// Determine secure cookies based on environment.
	secureCookie := cfg.App.Environment != "dev"

	// ハンドラを初期化する。HealthHandlerにはRedisとOIDCクライアントを渡す。
	healthHandler := handler.NewHealthHandler(redisClient, oauthClient)
	authHandler := handler.NewAuthHandler(
		oauthClient, sessionStore, sessionTTL,
		cfg.Auth.PostLogout, secureCookie, cfg.Cookie.Domain, logger,
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
	tp, err := initTracerProvider(ctx, cfg.Observability.Trace, cfg.App)
	if err != nil {
		logger.Warn("Failed to initialize OTel tracer provider", slog.String("error", err.Error()))
	} else if tp != nil {
		defer func() {
			// トレーサープロバイダーのシャットダウンエラーをログ出力する（M-4）
			if err := tp.Shutdown(context.Background()); err != nil {
				logger.Warn("トレーサープロバイダーのシャットダウンに失敗", slog.String("error", err.Error()))
			}
		}()
	}

	// Set up Gin router.
	if isProductionEnvironment(cfg.App.Environment) {
		gin.SetMode(gin.ReleaseMode)
	}
	router := gin.New()
	router.Use(gin.Recovery())
	// セキュリティレスポンスヘッダーを全リクエストに付与する
	router.Use(middleware.SecurityHeadersMiddleware())
	router.Use(middleware.PrometheusMiddleware())
	router.Use(otelgin.Middleware("bff-proxy"))
	router.Use(middleware.OTelTraceIDMiddleware())
	router.Use(middleware.CorrelationMiddleware())

	// Health / Metrics endpoints (no auth required).
	router.GET("/healthz", healthHandler.Healthz)
	router.GET("/readyz", healthHandler.Readyz)
	if cfg.Observability.Metrics.Enabled {
		metricsPath := cfg.Observability.Metrics.Path
		if metricsPath == "" {
			metricsPath = "/metrics"
		}
		router.GET(metricsPath, gin.WrapH(promhttp.Handler()))
	}

	// Auth endpoints (no session required).
	router.GET("/auth/login", authHandler.Login)
	router.GET("/auth/callback", authHandler.Callback)
	router.GET("/auth/session", authHandler.Session)
	router.GET("/auth/exchange", authHandler.Exchange)
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

	// OIDCリトライゴルーチンの完了を待機する（コンテキストキャンセル済みのため速やかに終了する）
	oidcWg.Wait()

	logger.Info("BFF Proxy stopped")
	return nil
}

func initTracerProvider(
	ctx context.Context,
	traceCfg config.TraceConfig,
	appCfg config.AppConfig,
) (*sdktrace.TracerProvider, error) {
	if !traceCfg.Enabled {
		return nil, nil
	}

	endpoint := traceCfg.Endpoint
	if endpoint == "" {
		endpoint = "localhost:4317"
	}

	opts := []otlptracegrpc.Option{
		otlptracegrpc.WithEndpoint(endpoint),
	}
	if !strings.HasPrefix(endpoint, "https://") {
		opts = append(opts, otlptracegrpc.WithInsecure())
	}

	exporter, err := otlptracegrpc.New(ctx, opts...)
	if err != nil {
		return nil, fmt.Errorf("failed to create OTLP exporter: %w", err)
	}

	res, err := resource.New(ctx,
		resource.WithAttributes(
			semconv.ServiceNameKey.String(appCfg.Name),
		),
	)
	if err != nil {
		return nil, fmt.Errorf("failed to create resource: %w", err)
	}

	sampleRate := traceCfg.SampleRate
	if sampleRate < 0 {
		sampleRate = 0
	} else if sampleRate > 1 {
		sampleRate = 1
	}

	tp := sdktrace.NewTracerProvider(
		sdktrace.WithBatcher(exporter),
		sdktrace.WithResource(res),
		sdktrace.WithSampler(sdktrace.ParentBased(sdktrace.TraceIDRatioBased(sampleRate))),
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

	opts := &slog.HandlerOptions{Level: level}
	if strings.EqualFold(logCfg.Format, "text") {
		return slog.New(slog.NewTextHandler(os.Stdout, opts))
	}
	return slog.New(slog.NewJSONHandler(os.Stdout, opts))
}

func isProductionEnvironment(env string) bool {
	switch strings.ToLower(strings.TrimSpace(env)) {
	case "prod", "production":
		return true
	default:
		return false
	}
}

// retryOIDCDiscovery はOIDC discoveryが完了するまでバックグラウンドで再試行する。
// 指数バックオフ（5s→10s→20s→最大60s）でリトライし、コンテキストがキャンセルされたら終了する。
func retryOIDCDiscovery(ctx context.Context, client *oauth.Client, logger *slog.Logger) {
	// 初回リトライ間隔
	interval := 5 * time.Second
	// リトライ間隔の上限
	maxInterval := 60 * time.Second

	for {
		select {
		case <-ctx.Done():
			logger.Info("OIDC discovery retry stopped due to context cancellation")
			return
		case <-time.After(interval):
			if _, err := client.Discover(ctx); err != nil {
				logger.Warn("OIDC discovery retry failed",
					slog.String("error", err.Error()),
					slog.Duration("next_retry_in", interval*2),
				)
				// 指数バックオフで次のリトライ間隔を計算する
				interval *= 2
				if interval > maxInterval {
					interval = maxInterval
				}
				continue
			}
			logger.Info("OIDC discovery succeeded after retry")
			return
		}
	}
}
