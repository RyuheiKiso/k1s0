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

// run はアプリケーションの起動・実行・シャットダウンを管理する。
// L-4 監査対応: initRedis, initSessionStore, initOIDC, initRouter, startServer に分割し
// 関数の行数と循環的複雑度を削減した。
func run() error {
	ctx, stop := signal.NotifyContext(context.Background(), syscall.SIGINT, syscall.SIGTERM)
	defer stop()

	// 設定ファイルの読み込みとバリデーション
	configPath := os.Getenv("CONFIG_PATH")
	if configPath == "" {
		configPath = "config/config.yaml"
	}
	cfg, err := config.Load(configPath, os.Getenv("ENV_CONFIG_PATH"))
	if err != nil {
		return fmt.Errorf("failed to load config: %w", err)
	}
	validate := validator.New()
	if err := validate.Struct(cfg); err != nil {
		return fmt.Errorf("invalid configuration: %w", err)
	}
	logger := newLogger(cfg.Observability.Log)

	// Redis クライアントを初期化する
	redisClient, err := initRedis(ctx, cfg, logger)
	if err != nil {
		return err
	}

	// セッションストアを初期化する
	sessionStore, sessionTTL, err := initSessionStore(cfg, redisClient, logger)
	if err != nil {
		return err
	}

	// OIDC クライアントを初期化しバックグラウンド discovery を開始する
	oauthClient, oidcWg, oidcReady, err := initOIDC(ctx, cfg, logger)
	if err != nil {
		return err
	}

	// OpenTelemetry トレーサープロバイダーを初期化する
	// defer による shutdown が必要なため run() に残す
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

	// Gin ルーターを設定する
	router, err := initRouter(cfg, redisClient, oauthClient, oidcReady, sessionStore, sessionTTL, logger)
	if err != nil {
		return err
	}

	// HTTP サーバーを起動してシグナル待機・グレースフルシャットダウンを行う
	return startServer(ctx, cfg, router, logger, oidcWg)
}

// initRedis は設定から Redis クライアントを生成し、接続確認 Ping を実行する。
// コネクションプール設定を適用する（M-011）。
// ALLOW_REDIS_SKIP=true かつ dev 環境の場合のみ接続失敗を無視する。
func initRedis(ctx context.Context, cfg *config.BFFConfig, logger *slog.Logger) (redis.Cmdable, error) {
	redisCfg := cfg.Session.Redis
	redisDialTimeout := config.ParseDuration(redisCfg.DialTimeout, 5*time.Second)
	redisReadTimeout := config.ParseDuration(redisCfg.ReadTimeout, 3*time.Second)
	redisWriteTimeout := config.ParseDuration(redisCfg.WriteTimeout, 3*time.Second)

	var redisClient redis.Cmdable
	if redisCfg.MasterName != "" {
		// Sentinel モード: MasterName が設定されている場合はフェイルオーバークライアントを使用する
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
		// スタンドアロンモード
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

	// Redis 接続確認: redis.Cmdable 経由で Ping を呼び出す
	if err := redisClient.Ping(ctx).Err(); err != nil {
		env := cfg.App.Environment
		allowSkip := os.Getenv("ALLOW_REDIS_SKIP") == "true" && config.IsDevEnvironment(env)
		if allowSkip {
			logger.Warn("Redis接続に失敗しました。ALLOW_REDIS_SKIP=trueのためスキップします（dev/development/local環境のみ）", slog.String("error", err.Error()))
		} else {
			logger.Error("Redis接続に失敗しました", slog.String("error", err.Error()), slog.String("environment", env))
			return nil, fmt.Errorf("Redis接続に失敗しました: %w", err)
		}
	}
	return redisClient, nil
}

// initSessionStore は Redis クライアントからセッションストアを初期化する。
// SESSION_ENCRYPTION_KEY が設定されている場合は AES-GCM 暗号化セッションストアを使用する（S-04 対応）。
// 非開発環境で SESSION_ENCRYPTION_KEY が未設定の場合は起動を拒否する（M-04 対応）。
func initSessionStore(cfg *config.BFFConfig, redisClient redis.Cmdable, logger *slog.Logger) (session.Store, time.Duration, error) {
	prefix := cfg.Session.Prefix
	if prefix == "" {
		prefix = "bff:session:"
	}
	var sessionStore session.Store
	if encKeyHex := os.Getenv("SESSION_ENCRYPTION_KEY"); encKeyHex != "" {
		encKey, err := hex.DecodeString(encKeyHex)
		if err != nil || len(encKey) != 32 {
			return nil, 0, fmt.Errorf("SESSION_ENCRYPTION_KEY は hex エンコードされた 32 バイト（64 hex 文字）である必要があります")
		}
		encStore, err := session.NewEncryptedStore(redisClient, prefix, encKey)
		if err != nil {
			return nil, 0, fmt.Errorf("暗号化セッションストアの初期化に失敗: %w", err)
		}
		sessionStore = encStore
		logger.Info("AES-GCM 暗号化セッションストアを使用します")
	} else {
		if !config.IsDevEnvironment(cfg.App.Environment) {
			return nil, 0, fmt.Errorf("SESSION_ENCRYPTION_KEY は非開発環境では必須です（SESSION_ENCRYPTION_KEY must be set in non-development environments）")
		}
		sessionStore = session.NewRedisStore(redisClient, prefix)
		logger.Warn("SESSION_ENCRYPTION_KEY が設定されていません。セッションデータは Redis に平文で保存されます。本番環境では必ず設定してください。")
	}
	return sessionStore, config.ParseDuration(cfg.Session.TTL, 30*time.Minute), nil
}

// initOIDC は OIDC クライアントを生成し、discovery が失敗した場合はバックグラウンドリトライを開始する。
// H-07 対応: oidcReady を返すことで /readyz が discovery 完了を確認できる。
// M-09 対応: 本番環境では discovery_url が HTTPS でない場合にエラーを返す。
func initOIDC(ctx context.Context, cfg *config.BFFConfig, logger *slog.Logger) (*oauth.Client, *sync.WaitGroup, *atomic.Bool, error) {
	// 本番環境では HTTPS が必須のためエラーを返す
	if !strings.HasPrefix(cfg.Auth.DiscoveryURL, "https://") {
		if isProductionEnvironment(cfg.App.Environment) {
			return nil, nil, nil, fmt.Errorf("OIDC discovery_url が TLS (https) を使用していません: %s", cfg.Auth.DiscoveryURL)
		}
		logger.Warn("OIDC discovery_url が TLS (https) を使用していません。本番環境では https を使用してください",
			slog.String("discovery_url", cfg.Auth.DiscoveryURL),
			slog.String("environment", cfg.App.Environment),
		)
	}

	oauthClient := oauth.NewClient(ctx, cfg.Auth.DiscoveryURL, cfg.Auth.ClientID, cfg.Auth.ClientSecret, cfg.Auth.RedirectURI, cfg.Auth.Scopes)

	var oidcWg sync.WaitGroup
	var oidcReady atomic.Bool
	if _, err := oauthClient.Discover(ctx); err != nil {
		logger.Warn("OIDC discovery failed at startup, will retry in background", slog.String("error", err.Error()))
		oidcWg.Add(1)
		go func() {
			defer oidcWg.Done()
			retryOIDCDiscovery(ctx, oauthClient, logger, &oidcReady)
		}()
	} else {
		oidcReady.Store(true)
	}
	return oauthClient, &oidcWg, &oidcReady, nil
}

// initRouter はハンドラを生成し、Gin ルーターに全ミドルウェアとルートを登録する。
// G-01/G-02/G-06/H-1/H-2/M-09 各対応のミドルウェアを適用する。
func initRouter(
	cfg *config.BFFConfig,
	redisClient redis.Cmdable,
	oauthClient *oauth.Client,
	oidcReady *atomic.Bool,
	sessionStore session.Store,
	sessionTTL time.Duration,
	logger *slog.Logger,
) (*gin.Engine, error) {
	secureCookie := !config.IsDevEnvironment(cfg.App.Environment)
	// H-07 対応: oidcReady を HealthHandler に渡し、/readyz で discovery 状態を反映する
	healthHandler := handler.NewHealthHandler(redisClient, oauthClient, oidcReady)
	authHandler := handler.NewAuthHandler(oauthClient, sessionStore, sessionTTL, cfg.Auth.PostLogout, secureCookie, cfg.Cookie.Domain, logger)
	upstreamTimeout := config.ParseDuration(cfg.Upstream.Timeout, 30*time.Second)
	proxyHandler, err := handler.NewProxyHandler(cfg.Upstream.BaseURL, sessionStore, oauthClient, sessionTTL, upstreamTimeout, logger)
	if err != nil {
		return nil, fmt.Errorf("failed to create proxy handler: %w", err)
	}

	if isProductionEnvironment(cfg.App.Environment) {
		gin.SetMode(gin.ReleaseMode)
	}
	router := gin.New()
	// G-06 対応: nil を指定することでプロキシを信頼せず X-Forwarded-For 等を直接 IP で上書きする
	if err := router.SetTrustedProxies(nil); err != nil {
		logger.Warn("SetTrustedProxies 設定に失敗しました", slog.String("error", err.Error()))
	}
	router.Use(gin.Recovery())
	// G-02 対応: リクエストボディを 64MB に制限して DoS/OOM を防止する
	router.Use(func(c *gin.Context) {
		c.Request.Body = http.MaxBytesReader(c.Writer, c.Request.Body, 64*1024*1024)
		c.Next()
	})
	// H-1 対応: CORS を最初に適用する
	corsHandler, err := middleware.CORSMiddleware(cfg.CORS)
	if err != nil {
		return nil, fmt.Errorf("CORS ミドルウェアの初期化に失敗しました: %w", err)
	}
	router.Use(corsHandler)
	// H-2 対応: IP ベースレート制限を CORS の次に適用する
	router.Use(middleware.RateLimitMiddleware(cfg.RateLimit))
	router.Use(middleware.SecurityHeadersMiddleware())
	router.Use(middleware.PrometheusMiddleware())
	router.Use(otelgin.Middleware("bff-proxy"))
	router.Use(middleware.OTelTraceIDMiddleware())
	router.Use(middleware.CorrelationMiddleware())

	router.GET("/healthz", healthHandler.Healthz)
	router.GET("/readyz", healthHandler.Readyz)
	if cfg.Observability.Metrics.Enabled {
		metricsPath := cfg.Observability.Metrics.Path
		if metricsPath == "" {
			metricsPath = "/metrics"
		}
		router.GET(metricsPath, gin.WrapH(promhttp.Handler()))
	}
	router.GET("/auth/login", authHandler.Login)
	router.GET("/auth/callback", authHandler.Callback)
	router.GET("/auth/session", authHandler.Session)
	router.GET("/auth/exchange", authHandler.Exchange)
	router.POST("/auth/logout", authHandler.Logout)

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
	return router, nil
}

// startServer は HTTP サーバーを起動し、シグナルを待機してグレースフルにシャットダウンする。
// G-01 対応: ReadTimeout=60s（大きなリクエストボディ対応）、WriteTimeout=120s（上流応答時間バッファ）。
func startServer(ctx context.Context, cfg *config.BFFConfig, router *gin.Engine, logger *slog.Logger, oidcWg *sync.WaitGroup) error {
	addr := fmt.Sprintf("%s:%d", cfg.Server.Host, cfg.Server.Port)
	srv := &http.Server{
		Addr:         addr,
		Handler:      router,
		ReadTimeout:  config.ParseDuration(cfg.Server.ReadTimeout, 60*time.Second),
		WriteTimeout: config.ParseDuration(cfg.Server.WriteTimeout, 120*time.Second),
	}

	errCh := make(chan error, 1)
	go func() {
		logger.Info("BFF Proxy starting", slog.String("addr", addr))
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			errCh <- err
		}
	}()

	select {
	case <-ctx.Done():
		logger.Info("シャットダウンシグナルを受信しました")
	case err := <-errCh:
		return fmt.Errorf("server error: %w", err)
	}

	shutdownTimeout := config.ParseDuration(cfg.Server.ShutdownTimeout, 15*time.Second)
	shutdownCtx, cancel := context.WithTimeout(context.Background(), shutdownTimeout)
	defer cancel()
	if err := srv.Shutdown(shutdownCtx); err != nil {
		return fmt.Errorf("server shutdown error: %w", err)
	}

	// OIDC リトライゴルーチンの完了を待機する（コンテキストキャンセル済みのため速やかに終了する）
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

// retryOIDCDiscovery は OIDC discovery が完了するまでバックグラウンドで無限リトライする。
// M-4 対応: Keycloak 復旧後に自動接続できるよう、リトライ上限を撤廃して無限ループに変更する。
// H-07 対応: oidcReady パラメータを追加し、discovery 成功時に true を格納する。
//
// リトライ戦略:
//   - 短期フェーズ（最初 20 回）: 指数バックオフ（5 秒〜60 秒）で素早く復帰を試みる
//   - 長期フェーズ（21 回目以降）: 5 分間隔で継続的にリトライする
//     K8s readinessProbe がポッドをサービスから切り離し続けるため、永続的な 503 を回避できる
//
// コンテキストがキャンセルされた場合（シャットダウン時）のみ終了する。
func retryOIDCDiscovery(ctx context.Context, client *oauth.Client, logger *slog.Logger, oidcReady *atomic.Bool) {
	// 短期フェーズのリトライ上限回数
	const shortPhaseRetries = 20
	// 長期フェーズのリトライ間隔（5分）
	longPhaseInterval := 5 * time.Minute
	// 初回リトライ間隔
	interval := 5 * time.Second
	// リトライ間隔の上限（短期フェーズ）
	maxShortInterval := 60 * time.Second
	// 現在のリトライ試行回数
	attempt := 0

	for {
		// コンテキストキャンセル（シャットダウン）時のみループを終了する
		select {
		case <-ctx.Done():
			logger.Info("OIDC discovery retry stopped due to context cancellation")
			return
		case <-time.After(interval):
		}

		attempt++
		if _, err := client.Discover(ctx); err != nil {
			if attempt <= shortPhaseRetries {
				// 短期フェーズ: 指数バックオフで次のリトライ間隔を計算する
				interval = min(interval*2, maxShortInterval)
				logger.Warn("OIDC discovery retry failed",
					slog.String("error", err.Error()),
					slog.Int("attempt", attempt),
					slog.Duration("next_retry_in", interval),
				)
			} else {
				// 長期フェーズへの移行時に一度だけエラーログを出力する
				if attempt == shortPhaseRetries+1 {
					logger.Error("OIDC discovery 短期リトライ上限到達。5分間隔で継続します",
						slog.Int("attempt", attempt),
						slog.Duration("long_phase_interval", longPhaseInterval),
					)
					interval = longPhaseInterval
				}
				// 長期フェーズ: 5分間隔で継続的にリトライする（K8s readinessProbe がトラフィックを遮断中）
				logger.Info("OIDC discovery 長期リトライ中",
					slog.String("error", err.Error()),
					slog.Int("attempt", attempt),
					slog.Duration("next_retry_in", interval),
				)
			}
			continue
		}

		// H-07 対応: discovery 成功時のみ oidcReady を true に更新する。
		// /readyz が 200 を返すようになり、Kubernetes がトラフィックを再開する。
		oidcReady.Store(true)
		logger.Info("OIDC discovery succeeded after retry", slog.Int("attempt", attempt))
		return
	}
}
