package middleware

import (
	"fmt"
	"net/http"
	"strconv"
	"strings"

	"github.com/gin-gonic/gin"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/config"
)

// requiresCredentials はリクエストパスが credentials_paths に一致するかを判定する。
// LOW-07 監査対応: credentialsPaths が空（未設定）の場合は false を返す（最小権限原則）。
// 明示的に credentials_paths を設定したパスのみ Access-Control-Allow-Credentials: true を付与する。
// 注意: この変更により、credentials_paths を設定していない環境では全パスで credentials が false になる。
// 既存クライアントに影響がある場合は credentials_paths に明示的なパスを設定すること。
func requiresCredentials(path string, credentialsPaths []string) bool {
	if len(credentialsPaths) == 0 {
		// LOW-07: 未設定時は false を返し、意図せず全パスで credentials を許可しない
		return false
	}
	for _, prefix := range credentialsPaths {
		if strings.HasPrefix(path, prefix) {
			return true
		}
	}
	return false
}

// CORSMiddleware はCross-Origin Resource Sharing（CORS）を制御するミドルウェア。
// H-1対応: BFF Proxy に明示的な Origin ホワイトリストを設定し、
// Web UI からの異オリジンリクエストを安全に受け付ける。
// 許可外のオリジンからのリクエストはCORSヘッダーを付与せず、ブラウザに拒否させる。
func CORSMiddleware(cfg config.CORSConfig) (gin.HandlerFunc, error) {
	// CORSが無効の場合はパススルー
	if !cfg.Enabled {
		return func(c *gin.Context) { c.Next() }, nil
	}

	// ワイルドカード "*" が指定されている場合は起動を阻止する（SH-2 監査対応）
	// "*" を許可すると全オリジンからのリクエストが通過し、CSRF 等の攻撃に対して脆弱になる。
	// log.Fatalf の代わりに error を返すことで、defer 登録済みのクリーンアップ関数が確実に実行される。
	for _, o := range cfg.AllowOrigins {
		if o == "*" {
			return nil, fmt.Errorf("CORS設定エラー: AllowOrigins にワイルドカード '*' は使用できません。明示的なオリジンを指定してください")
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

	// credentials 判定に使用するパスプレフィックスリスト
	credentialsPaths := cfg.CredentialsPaths

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
			// H-13 監査対応: エンドポイント毎に Credentials の必要性を評価する。
			// /healthz や /metrics 等の公開エンドポイントには credentials を付与しない。
			// credentials_paths に一致するパスのみ true を返すことで最小権限を実現する。
			if requiresCredentials(c.Request.URL.Path, credentialsPaths) {
				c.Header("Access-Control-Allow-Credentials", "true")
			}
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
	}, nil
}
