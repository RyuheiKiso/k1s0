package middleware

import (
	"log"
	"net/http"
	"strconv"
	"strings"

	"github.com/gin-gonic/gin"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/config"
)

// CORSMiddleware はCross-Origin Resource Sharing（CORS）を制御するミドルウェア。
// H-1対応: BFF Proxy に明示的な Origin ホワイトリストを設定し、
// Web UI からの異オリジンリクエストを安全に受け付ける。
// 許可外のオリジンからのリクエストはCORSヘッダーを付与せず、ブラウザに拒否させる。
func CORSMiddleware(cfg config.CORSConfig) gin.HandlerFunc {
	// CORSが無効の場合はパススルー
	if !cfg.Enabled {
		return func(c *gin.Context) { c.Next() }
	}

	// ワイルドカード "*" が指定されている場合は起動を阻止する（SH-2 監査対応）
	// "*" を許可すると全オリジンからのリクエストが通過し、CSRF 等の攻撃に対して脆弱になる。
	for _, o := range cfg.AllowOrigins {
		if o == "*" {
			log.Fatalf("CORS設定エラー: AllowOrigins にワイルドカード '*' は使用できません。明示的なオリジンを指定してください。")
		}
	}

	// O(1)参照のため許可オリジンをマップに変換する
	allowedSet := make(map[string]struct{}, len(cfg.AllowOrigins))
	for _, o := range cfg.AllowOrigins {
		allowedSet[strings.ToLower(o)] = struct{}{}
	}

	// 許可ヘッダー（未設定時はデフォルト値を使用する）
	allowHeaders := cfg.AllowHeaders
	if len(allowHeaders) == 0 {
		allowHeaders = []string{
			"Origin", "Content-Type", "Accept", "Authorization",
			"X-CSRF-Token", "X-Correlation-ID", "X-Request-ID",
		}
	}
	allowHeadersStr := strings.Join(allowHeaders, ", ")

	// 公開ヘッダー（未設定時はデフォルト値を使用する）
	exposeHeaders := cfg.ExposeHeaders
	if len(exposeHeaders) == 0 {
		exposeHeaders = []string{"X-Correlation-ID", "X-Request-ID"}
	}
	exposeHeadersStr := strings.Join(exposeHeaders, ", ")

	// プリフライトキャッシュ時間（未設定時: 600秒）
	maxAge := "600"
	if cfg.MaxAgeSecs > 0 {
		maxAge = strconv.Itoa(cfg.MaxAgeSecs)
	}

	return func(c *gin.Context) {
		origin := c.GetHeader("Origin")

		// Originヘッダーがない場合はCORSリクエストではないためスキップする
		if origin == "" {
			c.Next()
			return
		}

		// ホワイトリストに含まれるOriginのみCORSヘッダーを付与する
		if _, allowed := allowedSet[strings.ToLower(origin)]; allowed {
			c.Header("Access-Control-Allow-Origin", origin)
			c.Header("Access-Control-Allow-Credentials", "true")
			c.Header("Access-Control-Expose-Headers", exposeHeadersStr)
			c.Header("Vary", "Origin")

			// OPTIONSプリフライトリクエストを処理してリクエストチェーンを終了する
			if c.Request.Method == http.MethodOptions {
				c.Header("Access-Control-Allow-Methods", "GET, POST, PUT, PATCH, DELETE, OPTIONS")
				c.Header("Access-Control-Allow-Headers", allowHeadersStr)
				c.Header("Access-Control-Max-Age", maxAge)
				c.AbortWithStatus(http.StatusNoContent)
				return
			}
		}

		c.Next()
	}
}
