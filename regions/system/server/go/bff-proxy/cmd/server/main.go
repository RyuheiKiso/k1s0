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
		// L-17 監査対応: 本番/ステージング環境では OTel 初期化失敗時に起動を停止する。
		// 開発環境では警告ログで継続し、OTel Collector が未起動でもサービスを利用可能にする。
		if !config.IsDevEnvironment(cfg.App.Environment) {
			return fmt.Errorf("OTel トレーサープロバイダーの初期化に失敗しました（非開発環境では必須）: %w", err)
		}
		logger.Warn("OTel トレーサープロバイダーの初期化に失敗しました（開発環境のため継続）", slog.String("error", err.Error()))
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

	// Gin ルーターを設定する（H-3 対応: ctx を渡して goroutine リークを防止する）
	router, err := initRouter(ctx, cfg, redisClient, oauthClient, oidcReady, sessionStore, sessionTTL, logger)
	if err != nil {
		return err
	}

	// HTTP サーバーを起動してシグナル待機・グレースフルシャットダウンを行う
	return startServer(ctx, cfg, router, logger, oidcWg)
}

// H-011 監査対応: Redis Sentinel フェイルオーバー中のトランジェントエラーに対応するための定数。
// 指数バックオフでリトライする最大回数と初回待機時間を定義する。
const (
	// redisMaxRetries は起動時 Ping の最大リトライ回数（1秒 → 2秒 → 4秒の3回）
	redisMaxRetries = 3
	// redisInitialDelay は最初のリトライ待機時間
	redisInitialDelay = 1 * time.Second
)

// initRedis は設定から Redis クライアントを生成し、接続確認 Ping を実行する。
// コネクションプール設定を適用する（M-011）。
// H-011 監査対応: Sentinel フェイルオーバー中のトランジェントエラーへの対応として
// 指数バックオフリトライ（最大3回）を適用する。
// ALLOW_REDIS_SKIP=true かつ dev 環境の場合のみ接続失敗を無視する。
func initRedis(ctx context.Context, cfg *config.BFFConfig, logger *slog.Logger) (redis.Cmdable, error) {
	redisCfg := cfg.Session.Redis
	redisDialTimeout := config.ParseDuration(redisCfg.DialTimeout, 5*time.Second)
	redisReadTimeout := config.ParseDuration(redisCfg.ReadTimeout, 3*time.Second)
	redisWriteTimeout := config.ParseDuration(redisCfg.WriteTimeout, 3*time.Second)

	var redisClient redis.Cmdable
	if redisCfg.MasterName != "" {
		// Sentinel モード: MasterName が設定されている場合はフェイルオーバークライアントを使用する
		// H-17 対応: SentinelAddrs が設定されている場合は複数アドレスを使用し、
		// 未設定の場合は後方互換性のために単一の Addr にフォールバックする
		sentinelAddrs := redisCfg.SentinelAddrs
		if len(sentinelAddrs) == 0 {
			// SentinelAddrs が未設定の場合は Addr を Sentinel アドレスとして使用する（後方互換）
			sentinelAddrs = []string{redisCfg.Addr}
		}
		redisClient = redis.NewFailoverClient(&redis.FailoverOptions{
			MasterName:    redisCfg.MasterName,
			SentinelAddrs: sentinelAddrs,
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

	// H-011 監査対応: Redis 接続確認を指数バックオフリトライ付きで実行する。
	// Sentinel フェイルオーバー中はマスター切り替えにより一時的に接続不能となる場合がある。
	// 最大3回（1秒 → 2秒 → 4秒）リトライし、全試行失敗時のみエラーとして扱う。
	var lastErr error
	for i := 0; i < redisMaxRetries; i++ {
		if err := redisClient.Ping(ctx).Err(); err == nil {
			// Ping 成功: リトライループを抜ける
			lastErr = nil
			break
		} else {
			lastErr = err
			if i < redisMaxRetries-1 {
				// 次のリトライまでの待機時間: 1s → 2s → 4s（左シフトで指数計算）
				delay := redisInitialDelay << i
				logger.WarnContext(ctx, "Redis接続に失敗しました。リトライします",
					"attempt", i+1,
					"max_retries", redisMaxRetries,
					"retry_after", delay,
					"error", err,
				)
				time.Sleep(delay)
			}
		}
	}
	if lastErr != nil {
		// 全リトライ失敗: dev環境かつ ALLOW_REDIS_SKIP=true の場合のみ続行する
		env := cfg.App.Environment
		allowSkip := os.Getenv("ALLOW_REDIS_SKIP") == "true" && config.IsDevEnvironment(env)
		if allowSkip {
			logger.Warn("Redis接続に失敗しました。ALLOW_REDIS_SKIP=trueのためスキップします（dev/development/local環境のみ）", slog.String("error", lastErr.Error()))
			// H-002 監査対応: broken な redis クライアントを下流に渡すと panic リスクがある。
			// Redis スキップ時は nil を返し、呼び出し元（initSessionStore）が NoOpStore を使用する。
			return nil, nil
		}
		logger.Error("Redis接続に失敗しました", slog.String("error", lastErr.Error()), slog.String("environment", env))
		return nil, fmt.Errorf("Redis接続に失敗しました: %w", lastErr)
	}
	return redisClient, nil
}

// initSessionStore は Redis クライアントからセッションストアを初期化する。
// SESSION_ENCRYPTION_KEY が設定されている場合は AES-GCM 暗号化セッションストアを使用する（S-04 対応）。
// 非開発環境で SESSION_ENCRYPTION_KEY が未設定の場合は起動を拒否する（M-04 対応）。
// H-5 監査対応: session.FullStore を返すことで ExchangeCodeStore も単一ストアで提供できる。
// H-002 監査対応: redisClient が nil の場合（ALLOW_REDIS_SKIP=true かつ Redis 接続失敗時）は
// NoOpStore を返し、broken な redis クライアントが下流で panic を起こすリスクを排除する。
func initSessionStore(cfg *config.BFFConfig, redisClient redis.Cmdable, logger *slog.Logger) (session.FullStore, time.Duration, error) {
	prefix := cfg.Session.Prefix
	if prefix == "" {
		prefix = "bff:session:"
	}

	// M-14 監査対応: SESSION_ENCRYPTION_KEY の検証を Redis nil チェックの前に実施する。
	// Redis が nil（ALLOW_REDIS_SKIP=true の開発環境）でも不正なキーや本番環境でのキー未設定は
	// 誤設定として早期にエラーを返す。
	encKeyHex := os.Getenv("SESSION_ENCRYPTION_KEY")
	var encKey []byte
	if encKeyHex != "" {
		var decErr error
		encKey, decErr = hex.DecodeString(encKeyHex)
		if decErr != nil || len(encKey) != 32 {
			return nil, 0, fmt.Errorf("SESSION_ENCRYPTION_KEY は hex エンコードされた 32 バイト（64 hex 文字）である必要があります")
		}
	} else if !config.IsDevEnvironment(cfg.App.Environment) {
		return nil, 0, fmt.Errorf("SESSION_ENCRYPTION_KEY は非開発環境では必須です（SESSION_ENCRYPTION_KEY must be set in non-development environments）")
	}

	// H-002 監査対応: Redis 接続スキップ時（redisClient == nil）は NoOpStore を使用する。
	// broken な redis クライアントを下流に渡すと nil dereference による panic が発生するリスクがある。
	// NoOpStore は全操作を no-op で処理し、セッションデータは保持しない（dev 環境専用）。
	if redisClient == nil {
		logger.Warn("Redis クライアントが nil です。NoOpStore を使用します（dev 環境専用）。セッションは保持されません。")
		return session.NewNoOpStore(), config.ParseDuration(cfg.Session.TTL, 30*time.Minute), nil
	}

	var sessionStore session.FullStore
	if encKey != nil {
		// SESSION_ENCRYPTION_KEY が有効: AES-GCM 暗号化セッションストアを使用する（S-04 対応）
		encStore, err := session.NewEncryptedStore(redisClient, prefix, encKey)
		if err != nil {
			return nil, 0, fmt.Errorf("暗号化セッションストアの初期化に失敗: %w", err)
		}
		sessionStore = encStore
		logger.Info("AES-GCM 暗号化セッションストアを使用します")
	} else {
		// dev 環境かつ SESSION_ENCRYPTION_KEY 未設定: 平文 Redis ストアを使用する
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
// G-01/G-02/G-06/H-1/H-2/H-3/M-09/M-11 各対応のミドルウェアを適用する。
// H-3 監査対応: ctx を受け取り RateLimitMiddleware の goroutine をシャットダウン時に停止する。
// H-5 監査対応: sessionStore を session.FullStore 型で受け取ることで、
// ExchangeCodeStore としても使用できる。
func initRouter(
	ctx context.Context,
	cfg *config.BFFConfig,
	redisClient redis.Cmdable,
	oauthClient *oauth.Client,
	oidcReady *atomic.Bool,
	sessionStore session.FullStore,
	sessionTTL time.Duration,
	logger *slog.Logger,
) (*gin.Engine, error) {
	secureCookie := !config.IsDevEnvironment(cfg.App.Environment)
	// M-17 監査対応: セッションの絶対最大有効期間を設定から取得する（デフォルト 24 時間）
	absoluteMaxTTL := config.ParseDuration(cfg.Session.AbsoluteMaxTTL, 24*time.Hour)
	// POLY-003 監査対応: ワンタイム交換コード TTL を設定から取得する（デフォルト 60 秒）
	exchangeCodeTTL := config.ParseDuration(cfg.Session.ExchangeCodeTTL, 60*time.Second)
	// H-07 対応: oidcReady を HealthHandler に渡し、/readyz で discovery 状態を反映する
	healthHandler := handler.NewHealthHandler(redisClient, oauthClient, oidcReady)
	// H-5 監査対応: ExchangeCodeStore を渡すことで SessionData.AccessToken への意味論的誤用を解消する
	// sessionStore は session.ExchangeCodeStore インターフェースも実装している
	authHandler := handler.NewAuthHandler(oauthClient, sessionStore, sessionStore, sessionTTL, absoluteMaxTTL, exchangeCodeTTL, cfg.Auth.PostLogout, secureCookie, cfg.Cookie.Domain, logger)
	upstreamTimeout := config.ParseDuration(cfg.Upstream.Timeout, 30*time.Second)
	// 設定ファイル由来の静的アップストリームホストを許可リストとして抽出する。
	// allowedHosts に含まれるホストは SSRF チェックをバイパスし、Docker/K8s 内部通信を可能にする。
	allowedHosts := cfg.AllowedUpstreamHosts()
	proxyHandler, err := handler.NewProxyHandler(cfg.Upstream.BaseURL, sessionStore, oauthClient, sessionTTL, upstreamTimeout, logger, allowedHosts)
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
	// H-3 監査対応: ctx を渡してシャットダウン時に goroutine を停止する
	router.Use(middleware.RateLimitMiddleware(ctx, cfg.RateLimit))
	router.Use(middleware.SecurityHeadersMiddleware())
	router.Use(middleware.PrometheusMiddleware())
	router.Use(otelgin.Middleware("bff-proxy"))
	router.Use(middleware.OTelTraceIDMiddleware())
	router.Use(middleware.CorrelationMiddleware())

	router.GET("/healthz", healthHandler.Healthz)
	router.GET("/readyz", healthHandler.Readyz)
	// HIGH-GO-001 監査対応: /metrics エンドポイントは内部専用サーバー（9090 ポート）に移動する。
	// 公開ルーターへの登録を廃止し、クラスター外部からのメトリクス取得を防止する。
	// 実際の登録は startServer 内の内部サーバーで行う。
	router.GET("/auth/login", authHandler.Login)
	router.GET("/auth/callback", authHandler.Callback)
	router.GET("/auth/session", authHandler.Session)
	router.GET("/auth/exchange", authHandler.Exchange)

	// H-7 監査対応: 本番環境では CSRF 保護を強制的に有効化する。
	// csrf.enabled を false に設定したまま本番運用されるリスクを防ぐ。
	// log.Fatal ではなく error を返すことで defer によるクリーンアップ（OTel シャットダウン等）が実行される。
	if !config.IsDevEnvironment(cfg.App.Environment) && !cfg.CSRF.Enabled {
		return nil, fmt.Errorf("本番環境では CSRF 保護を無効化できません。csrf.enabled を true に設定してください")
	}

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

	// HIGH-006 対応: /auth/logout は状態変更操作（セッション破棄）のため CSRF 保護が必要。
	// SessionMiddleware でセッション検証、CSRFMiddleware で CSRF トークン照合を行う。
	// login/callback/session/exchange は認証前アクセスが必要なため CSRF 保護対象外。
	authProtected := router.Group("/auth")
	authProtected.Use(middleware.SessionMiddleware(sessionStore, handler.CookieName, sessionTTL, cfg.Session.Sliding))
	if cfg.CSRF.Enabled {
		csrfHeader := cfg.CSRF.HeaderName
		if csrfHeader == "" {
			csrfHeader = middleware.DefaultCSRFHeader
		}
		authProtected.Use(middleware.CSRFMiddleware(sessionStore, csrfHeader, handler.CookieName))
	}
	authProtected.POST("/logout", authHandler.Logout)

	return router, nil
}

// startServer は HTTP サーバーを起動し、シグナルを待機してグレースフルにシャットダウンする。
// G-01 対応: ReadTimeout=60s（大きなリクエストボディ対応）、WriteTimeout=120s（上流応答時間バッファ）。
// H-16 対応: ReadHeaderTimeout=10s を設定し Slowloris 攻撃（ヘッダーを断片的に送信する遅延攻撃）を防止する。
// HIGH-GO-001 監査対応: /metrics を内部専用サーバー（InternalPort）で起動し、公開ルーターから分離する。
func startServer(ctx context.Context, cfg *config.BFFConfig, router *gin.Engine, logger *slog.Logger, oidcWg *sync.WaitGroup) error {
	addr := fmt.Sprintf("%s:%d", cfg.Server.Host, cfg.Server.Port)
	srv := &http.Server{
		Addr:              addr,
		Handler:           router,
		ReadTimeout:       config.ParseDuration(cfg.Server.ReadTimeout, 60*time.Second),
		WriteTimeout:      config.ParseDuration(cfg.Server.WriteTimeout, 120*time.Second),
		// ReadHeaderTimeout はリクエストヘッダー全体を受信するまでの最大時間を制限する（H-16 対応）
		// 未設定の場合 Slowloris 攻撃によりサーバーリソースが枯渇するリスクがある
		ReadHeaderTimeout: 10 * time.Second,
	}

	// HIGH-GO-001 監査対応: Prometheus メトリクスを内部専用ポートで公開する。
	// クラスター内の Prometheus のみがスクレイプできるよう、公開ルーターとは別サーバーを起動する。
	// InternalPort が 0 の場合はデフォルト 9090 を使用する。
	internalPort := cfg.Server.InternalPort
	if internalPort == 0 {
		internalPort = 9090
	}
	var internalSrv *http.Server
	if cfg.Observability.Metrics.Enabled {
		metricsPath := cfg.Observability.Metrics.Path
		if metricsPath == "" {
			metricsPath = "/metrics"
		}
		// DY-003 修正: Prometheus が同一 Pod 内のサイドカーではなく別 Pod からスクレイプするため、
		// 0.0.0.0 にバインドして Pod の IP アドレスでアクセスできるようにする。
		// NetworkPolicy で k1s0-observability namespace からの接続のみを許可し、外部露出を防止する。
		internalMux := http.NewServeMux()
		// メトリクスエンドポイントのみを内部サーバーに登録する
		internalMux.Handle(metricsPath, promhttp.Handler())
		internalAddr := fmt.Sprintf("0.0.0.0:%d", internalPort)
		internalSrv = &http.Server{
			Addr:              internalAddr,
			Handler:           internalMux,
			ReadTimeout:       10 * time.Second,
			WriteTimeout:      10 * time.Second,
			ReadHeaderTimeout: 5 * time.Second,
		}
		go func() {
			logger.Info("内部メトリクスサーバーを起動します", slog.String("addr", internalAddr), slog.String("path", metricsPath))
			if err := internalSrv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
				logger.Error("内部メトリクスサーバーがエラーで終了しました", slog.String("error", err.Error()))
			}
		}()
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

	// HIGH-GO-001 監査対応: 内部メトリクスサーバーも graceful shutdown する。
	if internalSrv != nil {
		internalShutdownCtx, internalCancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer internalCancel()
		if err := internalSrv.Shutdown(internalShutdownCtx); err != nil {
			logger.Warn("内部メトリクスサーバーのシャットダウンに失敗しました", slog.String("error", err.Error()))
		}
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

	// gRPC トレースエクスポーターのオプションを構築する（HIGH-11 対応）
	// 変更前: strings.HasPrefix(endpoint, "https://") による不完全な判定（gRPC はスキームを使わない）
	// 変更後: 明示的な設定フラグ OTLPInsecure で制御し、意図しない insecure 接続を防止する
	opts := []otlptracegrpc.Option{
		otlptracegrpc.WithEndpoint(endpoint),
	}
	if traceCfg.OTLPInsecure {
		opts = append(opts, otlptracegrpc.WithInsecure())
	}

	// H-013 監査対応: otlptracegrpc.New にタイムアウトを設定し、OTel Collector 無応答時の
	// 無限ブロックを防止する。5秒以内に接続できない場合はエラーを返す。
	exporterCtx, exporterCancel := context.WithTimeout(ctx, 5*time.Second)
	defer exporterCancel()
	exporter, err := otlptracegrpc.New(exporterCtx, opts...)
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

// M-011 監査対応: 環境判定を fail-safe（デフォルト本番）パターンに変更する
// タイポ（例: "prodction"）で本番環境が非本番として動作することを防止する
func isProductionEnvironment(env string) bool {
	switch strings.ToLower(strings.TrimSpace(env)) {
	case "dev", "development", "test", "local":
		// 明示的に開発・テスト環境として指定された場合のみ非本番として扱う
		return false
	default:
		// 不明な環境名は安全のため本番として扱う（fail-safe）
		return true
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
