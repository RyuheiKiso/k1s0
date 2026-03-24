package middleware

import (
	"net/http"
	"sync"
	"time"

	"github.com/gin-gonic/gin"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/config"
)

// visitorBucket はIPアドレスごとのトークンバケット状態を保持する。
type visitorBucket struct {
	mu       sync.Mutex
	tokens   float64
	lastSeen time.Time
}

// RateLimitMiddleware はIPアドレスベースのトークンバケットアルゴリズムによるレート制限を実装する。
// H-2対応: BFF Proxy レベルで IP ベースのレート制限を追加し、
// DDoS攻撃や大量リクエストから保護する。
// 外部依存なし（標準ライブラリ sync/time のみ使用）。
func RateLimitMiddleware(cfg config.RateLimitConfig) gin.HandlerFunc {
	// レート制限が無効の場合はパススルー
	if !cfg.Enabled {
		return func(c *gin.Context) { c.Next() }
	}

	// rps のデフォルト値（未設定時: 100 req/sec）
	rps := cfg.RPS
	if rps <= 0 {
		rps = 100.0
	}

	// burst のデフォルト値（未設定時: rps の2倍）
	burst := float64(cfg.Burst)
	if burst <= 0 {
		burst = rps * 2
	}

	// IPアドレスごとのバケットを保持するマップ（sync.Mapはロックフリー読み取り最適化）
	var visitors sync.Map

	// 古いバケットを定期クリーンアップするゴルーチン（メモリリーク防止）
	go func() {
		ticker := time.NewTicker(10 * time.Minute)
		defer ticker.Stop()
		for range ticker.C {
			expiry := time.Now().Add(-10 * time.Minute)
			visitors.Range(func(key, value any) bool {
				b := value.(*visitorBucket)
				b.mu.Lock()
				stale := b.lastSeen.Before(expiry)
				b.mu.Unlock()
				if stale {
					visitors.Delete(key)
				}
				return true
			})
		}
	}()

	return func(c *gin.Context) {
		ip := c.ClientIP()
		now := time.Now()

		// IPごとのバケットを取得または新規作成する
		actual, _ := visitors.LoadOrStore(ip, &visitorBucket{
			tokens:   burst,
			lastSeen: now,
		})
		b := actual.(*visitorBucket)

		b.mu.Lock()
		// 前回のリクエストからの経過時間に応じてトークンを補充する
		elapsed := now.Sub(b.lastSeen).Seconds()
		b.tokens += elapsed * rps
		if b.tokens > burst {
			b.tokens = burst
		}
		b.lastSeen = now

		// トークンが不足している場合は 429 Too Many Requests を返す
		if b.tokens < 1.0 {
			b.mu.Unlock()
			c.Header("Retry-After", "1")
			c.AbortWithStatus(http.StatusTooManyRequests)
			return
		}
		b.tokens--
		b.mu.Unlock()

		c.Next()
	}
}
