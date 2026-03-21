package handler

import (
	"net/http"

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
}

// NewHealthHandler はRedisクライアントとOIDCクライアントを受け取り、HealthHandlerを生成する。
func NewHealthHandler(redisClient redis.Cmdable, oauthClient OIDCChecker) *HealthHandler {
	return &HealthHandler{
		redisClient: redisClient,
		oauthClient: oauthClient,
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
	// OIDC discoveryの完了状態を確認（軽量チェックを先に実行）
	// nil チェックは OIDC を使用しないデプロイ構成（テスト環境等）でも安全に動作させるための防御的実装（L-004）
	if h.oauthClient != nil && !h.oauthClient.IsDiscovered() {
		c.JSON(http.StatusServiceUnavailable, gin.H{
			"status": "not_ready",
			"reason": "oidc discovery not completed",
		})
		return
	}

	// Redis接続の疎通確認
	if err := h.redisClient.Ping(c.Request.Context()).Err(); err != nil {
		c.JSON(http.StatusServiceUnavailable, gin.H{
			"status": "not_ready",
			"reason": "redis connection failed",
		})
		return
	}

	c.JSON(http.StatusOK, gin.H{"status": "ready"})
}
