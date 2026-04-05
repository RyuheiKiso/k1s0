package handler

import (
	"net/http"
	"sync/atomic"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/redis/go-redis/v9"
)

// OIDCChecker はOIDC discoveryの状態を確認するためのインターフェース。
// テスト時にモック可能にするため、具体型ではなくインターフェースで受け取る。
type OIDCChecker interface {
	IsDiscovered() bool
}

// HealthHandler はliveness/readinessプローブを提供するハンドラ。
type HealthHandler struct {
	// redisClient はRedis接続の疎通確認に使用する
	redisClient redis.Cmdable
	// oauthClient はOIDC discoveryの状態確認に使用する
	oauthClient OIDCChecker
	// oidcReady はバックグラウンドリトライを含む OIDC discovery の最終的な成否を示すフラグ。
	// H-07 対応: retryOIDCDiscovery が全リトライを消費して失敗した場合は false のまま維持され、
	// /readyz が 503 を返すことで Kubernetes がトラフィックを遮断できるようにする。
	// nil の場合は OIDC を使用しないデプロイ構成（テスト環境等）として扱い、チェックをスキップする。
	oidcReady *atomic.Bool
}

// NewHealthHandler はRedisクライアント、OIDCクライアント、OIDCreadyフラグを受け取り、HealthHandlerを生成する。
// H-07 対応: oidcReady を受け取ることで、バックグラウンドリトライの全失敗時も
// readiness が false のまま維持されることを保証する。
func NewHealthHandler(redisClient redis.Cmdable, oauthClient OIDCChecker, oidcReady *atomic.Bool) *HealthHandler {
	return &HealthHandler{
		redisClient: redisClient,
		oauthClient: oauthClient,
		oidcReady:   oidcReady,
	}
}

// Healthz はlivenessプローブ。プロセスが生存していれば200を返す。
func (h *HealthHandler) Healthz(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{"status": "ok"})
}

// Readyz はreadinessプローブ。OIDC discoveryとRedis接続の状態を確認する。
// いずれかが未準備であれば503を返す。
// OIDC discoveryチェックはboolフィールドの参照のみで軽量なため先に実行する。
func (h *HealthHandler) Readyz(c *gin.Context) {
	// H-07 対応: oidcReady フラグを優先チェックする。
	// このフラグは起動時の初回 discovery 成功、またはバックグラウンドリトライ成功時にのみ true になる。
	// 全リトライ失敗後は false のまま維持され、Kubernetes の readinessProbe が
	// 本ポッドをサービスから切り離すことで OIDC 未対応状態のトラフィックを遮断する。
	// nil チェックは OIDC を使用しないデプロイ構成（テスト環境等）でも安全に動作させるための防御的実装（L-004）
	if h.oidcReady != nil && !h.oidcReady.Load() {
		// ADR-0068 準拠: "unhealthy" + checks + timestamp
		c.JSON(http.StatusServiceUnavailable, gin.H{
			"status": "unhealthy",
			"checks": gin.H{
				"oidc":  "not_ready",
				"redis": "skipped",
			},
			"timestamp": time.Now().UTC().Format(time.RFC3339),
		})
		return
	}

	// oauthClient による二重チェック（oidcReady が nil の場合のフォールバック）
	// nil チェックは OIDC を使用しないデプロイ構成（テスト環境等）でも安全に動作させるための防御的実装（L-004）
	if h.oauthClient != nil && !h.oauthClient.IsDiscovered() {
		c.JSON(http.StatusServiceUnavailable, gin.H{
			"status": "unhealthy",
			"checks": gin.H{
				"oidc":  "not_ready",
				"redis": "skipped",
			},
			"timestamp": time.Now().UTC().Format(time.RFC3339),
		})
		return
	}

	// Redis接続の疎通確認
	if err := h.redisClient.Ping(c.Request.Context()).Err(); err != nil {
		c.JSON(http.StatusServiceUnavailable, gin.H{
			"status": "unhealthy",
			"checks": gin.H{
				"oidc":  "ok",
				"redis": "error",
			},
			"timestamp": time.Now().UTC().Format(time.RFC3339),
		})
		return
	}

	// ADR-0068 準拠: "healthy" + checks + timestamp
	c.JSON(http.StatusOK, gin.H{
		"status": "healthy",
		"checks": gin.H{
			"oidc":  "ok",
			"redis": "ok",
		},
		"timestamp": time.Now().UTC().Format(time.RFC3339),
	})
}
