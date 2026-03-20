package middleware

import "github.com/gin-gonic/gin"

// SecurityHeadersMiddleware はセキュリティ関連のHTTPレスポンスヘッダーを全レスポンスに付与する。
// クリックジャッキング、MIMEスニッフィング、XSS、プロトコルダウングレードなどの
// 一般的なWebセキュリティ脅威に対する防御レイヤーを提供する。
func SecurityHeadersMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		// クリックジャッキング防止: iframeへの埋め込みを全面禁止する
		c.Header("X-Frame-Options", "DENY")

		// MIMEスニッフィング防止: ブラウザによるContent-Typeの推測を禁止する
		c.Header("X-Content-Type-Options", "nosniff")

		// XSS保護: ブラウザ組み込みXSSフィルターを無効化する（CSPで代替するため）
		// 参考: https://owasp.org/www-project-secure-headers/#x-xss-protection
		c.Header("X-XSS-Protection", "0")

		// HSTS: HTTPS通信を強制し、サブドメインにも適用する
		c.Header("Strict-Transport-Security", "max-age=31536000; includeSubDomains")

		// リファラポリシー: 同一オリジンではフルURL、クロスオリジンではオリジンのみ送信する
		c.Header("Referrer-Policy", "strict-origin-when-cross-origin")

		// CSP: BFFプロキシは自身のオリジンからのリソースのみ許可する
		// APIプロキシとしての用途のため、strict な default-src 'self' を適用する
		c.Header("Content-Security-Policy", "default-src 'self'")

		c.Next()
	}
}
