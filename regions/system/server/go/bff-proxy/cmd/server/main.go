package main

import (
	"context"
	"encoding/hex"
	"fmt"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"sync"
	"sync/atomic"
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
	// コネクションプール設定を適用する（M-011）。
	// PoolSize=0 の場合は redis/go-redis のデフォルト値（CPU数 * 10）が使われる。
	redisCfg := cfg.Session.Redis
	redisDialTimeout := config.ParseDuration(redisCfg.DialTimeout, 5*time.Second)
	redisReadTimeout := config.ParseDuration(redisCfg.ReadTimeout, 3*time.Second)
	redisWriteTimeout := config.ParseDuration(redisCfg.WriteTimeout, 3*time.Second)
	var redisClient redis.Cmdable
	if redisCfg.MasterName != "" {
		redisClient = redis.NewFailoverClient(&redis.FailoverOptions{
			MasterName:    redisCfg.MasterName,
			SentinelAddrs: []string{redisCfg.Addr},
			Password:      redisCfg.Password,
			DB:            redisCfg.DB,
			PoolSize:      redisCfg.PoolSize,
			MinIdleConns:  redisCfg.MinIdleConns,
			MaxRetries:    redisCfg.MaxRetries,
			DialTimeout:   redisDialTimeout,
			ReadTimeout:   redisReadTimeout,
			WriteTimeout:  redisWriteTimeout,
		})
	} else {
		redisClient = redis.NewClient(&redis.Options{
			Addr:         redisCfg.Addr,
			Password:     redisCfg.Password,
			DB:           redisCfg.DB,
			PoolSize:     redisCfg.PoolSize,
			MinIdleConns: redisCfg.MinIdleConns,
			MaxRetries:   redisCfg.MaxRetries,
			DialTimeout:  redisDialTimeout,
			ReadTimeout:  redisReadTimeout,
			WriteTimeout: redisWriteTimeout,
		})
	}

	// Redis接続を確認する。
	// redis.Cmdable インターフェース経由で Ping を呼び出すことで、
	// スタンドアロン・Sentinel どちらのモードでも安全に動作する。
	// ALLOW_REDIS_SKIP は dev/development/local 環境のみ有効。production/staging では無視してエラーで終了する。
	if err := redisClient.Ping(ctx).Err(); err != nil {
		env := cfg.App.Environment
		allowSkip := os.Getenv("ALLOW_REDIS_SKIP") == "true" && config.IsDevEnvironment(env)
		if allowSkip {
			logger.Warn("Redis接続に失敗しました。ALLOW_REDIS_SKIP=trueのためスキップします（dev/development/local環境のみ）", slog.String("error", err.Error()))
		} else {
			logger.Error("Redis接続に失敗しました", slog.String("error", err.Error()), slog.String("environment", env))
			return fmt.Errorf("Redis接続に失敗しました: %w", err)
		}
	}

	// Initialize session store.
	// SESSION_ENCRYPTION_KEY が設定されている場合は AES-GCM 暗号化セッションストアを使用する（S-04 対応）。
	// 鍵は hex エンコードされた 32 バイト（AES-256）を期待する。
	// 未設定の場合は暗号化なしの RedisStore を使用し、本番環境向け警告を出力する。
	prefix := cfg.Session.Prefix
	if prefix == "" {
		prefix = "bff:session:"
	}
	var sessionStore session.Store
	if encKeyHex := os.Getenv("SESSION_ENCRYPTION_KEY"); encKeyHex != "" {
		encKey, err := hex.DecodeString(encKeyHex)
		if err != nil || len(encKey) != 32 {
			return fmt.Errorf("SESSION_ENCRYPTION_KEY は hex エンコードされた 32 バイト（64 hex 文字）である必要があります")
		}
		encStore, err := session.NewEncryptedStore(redisClient, prefix, encKey)
		if err != nil {
			return fmt.Errorf("暗号化セッションストアの初期化に失敗: %w", err)
		}
		sessionStore = encStore
		logger.Info("AES-GCM 暗号化セッションストアを使用します")
	} else {
		sessionStore = session.NewRedisStore(redisClient, prefix)
		logger.Warn("SESSION_ENCRYPTION_KEY が設定されていません。セッションデータは Redis に平文で保存されます。本番環境では必ず設定してください。")
	}
	sessionTTL := config.ParseDuration(cfg.Session.TTL, 30*time.Minute)

	// OIDC クライアントを初期化する。
	// ctx（アプリケーションレベルのコンテキスト）を渡すことで、シャットダウン時に
	// JWKS バックグラウンドフェッチがキャンセルされるようになる。
	oauthClient := oauth.NewClient(
		ctx,
		cfg.Auth.DiscoveryURL,
		cfg.Auth.ClientID,
		cfg.Auth.ClientSecret,
		cfg.Auth.RedirectURI,
		cfg.Auth.Scopes,
	)

	// OIDC DiscoveryURL が HTTPS でない場合は環境に応じて処理を分岐する。
	// 本番環境では IdP との通信に TLS が必須のため、非 HTTPS 設定を即時終了で拒否する（M-11 対応）。
	if !strings.HasPrefix(cfg.Auth.DiscoveryURL, "https://") {
		// 本番環境では HTTPS が必須のため即座に終了する
		if isProductionEnvironment(cfg.App.Environment) {
			logger.Error("OIDC discovery_url が TLS (https) を使用していません。本番環境では https が必須です",
				slog.String("discovery_url", cfg.Auth.DiscoveryURL),
				slog.String("environment", cfg.App.Environment),
			)
			os.Exit(1)
		}
		// 開発・ステージング環境では警告のみ
		logger.Warn("OIDC discovery_url が TLS (https) を使用していません。本番環境では https を使用してください",
			slog.String("discovery_url", cfg.Auth.DiscoveryURL),
			slog.String("environment", cfg.App.Environment),
		)
	}

	// OIDC discoveryを実行する。失敗した場合はバックグラウンドで再試行する。
	// WaitGroupでゴルーチンのライフサイクルを追跡し、シャットダウン時に安全に完了を待機する。
	// H-07 対応: oidcReady フラグでバックグラウンドリトライの成否を外部から参照可能にする。
	// HealthHandler がこのフラグを参照し、/readyz で Kubernetes にトラフィック制御の判断を委ねる。
	var oidcWg sync.WaitGroup
	// H-07 対応: OIDC discovery の最終的な成否を追跡する atomic フラグ。
	// 初期値は false。成功時のみ true に更新される。全リトライ失敗時は false のまま維持される。
	var oidcReady atomic.Bool
	if _, err := oauthClient.Discover(ctx); err != nil {
		logger.Warn("OIDC discovery failed at startup, will retry in background", slog.String("error", err.Error()))
		// discoveryが未完了の場合、バックグラウンドで定期的に再試行するゴルーチンを起動する
		oidcWg.Add(1)
		go func() {
			defer oidcWg.Done()
			// H-07 対応: oidcReady ポインタを渡し、リトライ成功時に true を格納させる
			retryOIDCDiscovery(ctx, oauthClient, logger, &oidcReady)
		}()
	} else {
		// 起動時の初回 discovery が成功した場合はフラグを即座に true にする
		oidcReady.Store(true)
	}

	// 開発環境（dev/development/local）では secure フラグを外す。
	// IsDevEnvironment で統一的に判定することで dev/development の不一致を防ぐ。
	secureCookie := !config.IsDevEnvironment(cfg.App.Environment)

	// ハンドラを初期化する。HealthHandlerにはRedisとOIDCクライアント、OIDCreadyフラグを渡す。
	// H-07 対応: oidcReady を HealthHandler に渡すことで、バックグラウンドリトライの
	// 全失敗時に /readyz が 503 を返し、Kubernetes がトラフィックを遮断できるようにする。
	healthHandler := handler.NewHealthHandler(redisClient, oauthClient, &oidcReady)
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
		logger.Warn("OTel トレーサープロバイダーの初期化に失敗しました", slog.String("error", err.Error()))
	} else if tp != nil {
		defer func() {
			// タイムアウト付きコンテキストでシャットダウンし、OTel Collector 無応答時の無限ブロックを防ぐ（H-004）
			shutdownTraceCtx, cancelTrace := context.WithTimeout(context.Background(), 5*time.Second)
			defer cancelTrace()
			if err := tp.Shutdown(shutdownTraceCtx); err != nil {
				logger.Warn("トレーサープロバイダーのシャットダウンに失敗", slog.String("error", err.Error()))
			}
		}()
	}

	// Set up Gin router.
	if isProductionEnvironment(cfg.App.Environment) {
		gin.SetMode(gin.ReleaseMode)
	}
	router := gin.New()
	// G-06 対応: リバースプロキシ信頼設定を明示的に行う。
	// nil を指定することでプロキシを信頼せず、X-Forwarded-For 等のヘッダーを
	// 直接の接続元 IP で上書きする。ロードバランサー配下では適切な CIDR に変更すること。
	// 例: router.SetTrustedProxies([]string{"10.0.0.0/8"})
	if err := router.SetTrustedProxies(nil); err != nil {
		logger.Warn("SetTrustedProxies 設定に失敗しました", slog.String("error", err.Error()))
	}
	router.Use(gin.Recovery())
	// G-02 対応: リクエストボディサイズを 64MB に制限する。
	// 無制限のリクエストボディによる DoS 攻撃や OOM を防止する。
	router.Use(func(c *gin.Context) {
		c.Request.Body = http.MaxBytesReader(c.Writer, c.Request.Body, 64*1024*1024)
		c.Next()
	})
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
	// G-01 対応: タイムアウトをプロキシ用途に合わせて調整する。
	// ReadTimeout=60s: 大きなリクエストボディやスロークライアントに対応する。
	// WriteTimeout=120s: 上流サービスの応答時間（upstreamTimeout 30s + バッファ）を考慮する。
	addr := fmt.Sprintf("%s:%d", cfg.Server.Host, cfg.Server.Port)
	srv := &http.Server{
		Addr:         addr,
		Handler:      router,
		ReadTimeout:  config.ParseDuration(cfg.Server.ReadTimeout, 60*time.Second),
		WriteTimeout: config.ParseDuration(cfg.Server.WriteTimeout, 120*time.Second),
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
		logger.Info("シャットダウンシグナルを受信しました")
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
// G-05 対応: 最大リトライ回数を 20 回に制限し、無限リトライによる長時間待機を防止する。
// H-07 対応: oidcReady パラメータを追加し、discovery 成功時に true を格納する。
//
//	全リトライ失敗時はフラグを false のまま維持し、/readyz を通じて
//	Kubernetes がトラフィックを遮断できるようにする。
func retryOIDCDiscovery(ctx context.Context, client *oauth.Client, logger *slog.Logger, oidcReady *atomic.Bool) {
	// 初回リトライ間隔
	interval := 5 * time.Second
	// リトライ間隔の上限
	maxInterval := 60 * time.Second
	// G-05 対応: リトライ上限回数（20 回で諦めてバックグラウンドゴルーチンを終了する）
	const maxRetries = 20

	for attempt := 1; attempt <= maxRetries; attempt++ {
		select {
		case <-ctx.Done():
			logger.Info("OIDC discovery retry stopped due to context cancellation")
			return
		case <-time.After(interval):
			if _, err := client.Discover(ctx); err != nil {
				logger.Warn("OIDC discovery retry failed",
					slog.String("error", err.Error()),
					slog.Int("attempt", attempt),
					slog.Int("max_retries", maxRetries),
					slog.Duration("next_retry_in", interval*2),
				)
				// 指数バックオフで次のリトライ間隔を計算する
				interval *= 2
				if interval > maxInterval {
					interval = maxInterval
				}
				continue
			}
			// H-07 対応: OIDC discovery 全失敗後、readiness を false にして
			// Kubernetes がトラフィックを遮断できるようにする。
			// リトライ成功時のみ oidcReady を true に更新する。
			oidcReady.Store(true)
			logger.Info("OIDC discovery succeeded after retry", slog.Int("attempt", attempt))
			return
		}
	}
	// G-05 対応: 最大リトライ回数に達した場合はエラーログを出してゴルーチンを終了する。
	// H-07 対応: oidcReady は false のまま維持され、/readyz が 503 を返し続ける。
	// これにより Kubernetes の readinessProbe がポッドをサービスから切り離し、
	// OIDC 未対応状態のインスタンスへのトラフィックを遮断する。
	logger.Error("OIDC discovery failed after maximum retries, giving up. Readiness will remain false.",
		slog.Int("max_retries", maxRetries),
	)
}
