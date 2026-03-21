package handler

import (
	"log/slog"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"golang.org/x/sync/singleflight"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/middleware"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/oauth"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/upstream"
)

// ProxyHandler はクッキーベースのセッションをベアラートークン認証に変換し、
// アップストリーム API へのリバースプロキシ機能を提供する。
type ProxyHandler struct {
	reverseProxy *upstream.ReverseProxy
	sessionStore session.Store
	oauthClient  *oauth.Client
	sessionTTL   time.Duration
	logger       *slog.Logger
	// G-03 対応: トークンリフレッシュの重複実行を防止するための singleflight グループ。
	// 同一セッション ID に対して複数の並行リクエストが同時にリフレッシュを試みる場合、
	// 最初の 1 件のみ実際に RefreshToken を呼び出し、他は結果を共有する。
	refreshGroup singleflight.Group
}

// NewProxyHandler はアップストリーム URL を対象とした新しいリバースプロキシハンドラーを生成する。
func NewProxyHandler(
	upstreamURL string,
	sessionStore session.Store,
	oauthClient *oauth.Client,
	sessionTTL time.Duration,
	timeout time.Duration,
	logger *slog.Logger,
) (*ProxyHandler, error) {
	reverseProxy, err := upstream.NewReverseProxy(upstreamURL, timeout)
	if err != nil {
		return nil, err
	}

	return &ProxyHandler{
		reverseProxy: reverseProxy,
		sessionStore: sessionStore,
		oauthClient:  oauthClient,
		sessionTTL:   sessionTTL,
		logger:       logger,
	}, nil
}

// Handle はセッションからベアラートークンを付加してアップストリーム API へリクエストをプロキシする。
// アクセストークンが期限切れの場合はサイレントリフレッシュを試みる。
func (h *ProxyHandler) Handle(c *gin.Context) {
	sess, ok := middleware.GetSessionData(c)
	if !ok {
		abortErrorWithMessage(c, http.StatusUnauthorized, "BFF_PROXY_NO_SESSION", "セッションが見つかりません")
		return
	}

	// GetSessionID はセッション ID を gin コンテキストから取得する。
	// 取得失敗時（SessionMiddleware がセットしていない場合）は空文字列になり
	// singleflight キーが衝突するため、明示的に 401 を返して処理を中断する。
	sessionID, ok := middleware.GetSessionID(c)
	if !ok || sessionID == "" {
		abortErrorWithMessage(c, http.StatusUnauthorized, "BFF_PROXY_NO_SESSION_ID", "セッション ID が取得できません")
		return
	}

	// SessionMiddleware が session_needs_refresh フラグを立てた場合のみ silent refresh を試みる。
	// フラグは「期限切れ かつ refresh token あり」の場合のみ middleware が設定する。
	needsRefresh, _ := c.Get(middleware.SessionNeedsRefreshKey)
	// 型アサーションの安全化: comma-ok パターンで bool 型を確認する（H-009）
	if refresh, ok := needsRefresh.(bool); ok && refresh {
		// G-03 対応: singleflight でセッション単位のリフレッシュ重複を排除する。
		// 同一 sessionID に対して並行リクエストが殺到した場合、1 件のみ実際にリフレッシュし
		// 残りは同じ結果を共有する。これにより RefreshToken のレート制限エラーを防ぐ。
		type refreshResult struct {
			tokenResp *oauth.TokenResponse
		}
		val, err, _ := h.refreshGroup.Do(sessionID, func() (any, error) {
			resp, e := h.oauthClient.RefreshToken(c.Request.Context(), sess.RefreshToken)
			if e != nil {
				return nil, e
			}
			return &refreshResult{tokenResp: resp}, nil
		})
		if err != nil {
			h.logger.Warn("トークンリフレッシュに失敗しました。セッションを削除します",
				slog.String("error", err.Error()),
			)
			// リフレッシュ失敗時に無効なセッションを削除し、再利用を防止する（H-003）
			if delErr := h.sessionStore.Delete(c.Request.Context(), sessionID); delErr != nil {
				h.logger.Error("期限切れセッションの削除に失敗しました",
					slog.String("error", delErr.Error()),
					slog.String("session_id", sessionID),
				)
			}
			abortErrorWithMessage(c, http.StatusUnauthorized, "BFF_PROXY_TOKEN_EXPIRED", "セッションが期限切れです。再認証してください")
			return
		}

		tokenResp := val.(*refreshResult).tokenResp

		// 新しいトークンでセッションを更新する。
		sess.AccessToken = tokenResp.AccessToken
		if tokenResp.RefreshToken != "" {
			sess.RefreshToken = tokenResp.RefreshToken
		}
		if tokenResp.IDToken != "" {
			sess.IDToken = tokenResp.IDToken
		}
		sess.ExpiresAt = time.Now().Add(time.Duration(tokenResp.ExpiresIn) * time.Second).Unix()

		// リフレッシュ後のセッション更新に失敗した場合はエラーログを記録する。
		if err := h.sessionStore.Update(c.Request.Context(), sessionID, sess, h.sessionTTL); err != nil {
			h.logger.Error("リフレッシュ後のセッション更新に失敗しました",
				slog.String("error", err.Error()),
			)
		}
	}

	// アップストリーム向けに Authorization ヘッダーを付加する。
	c.Request.Header.Set("Authorization", "Bearer "+sess.AccessToken)

	// 相関ヘッダーをアップストリームに伝播する
	if cid, ok := c.Get(middleware.CorrelationIDKey); ok {
		// 型アサーションの安全化: comma-ok パターンで string 型を確認する（M-3）
		if cidStr, ok := cid.(string); ok {
			c.Request.Header.Set(middleware.HeaderCorrelationID, cidStr)
		}
	}
	if tid, ok := c.Get(middleware.TraceIDKey); ok {
		// 型アサーションの安全化: comma-ok パターンで string 型を確認する（M-3）
		if tidStr, ok := tid.(string); ok {
			c.Request.Header.Set(middleware.HeaderTraceID, tidStr)
		}
	}

	// アップストリームへのリクエストからセッション Cookie を除去する。
	c.Request.Header.Del("Cookie")

	h.reverseProxy.ServeHTTP(c.Writer, c.Request)
}
