// proxy_usecase.go はリバースプロキシ処理におけるセッション検証・トークンリフレッシュの
// ビジネスロジックを提供する。handler が直接 OAuth クライアントとセッションストアを操作していた
// 処理を抽象化し、handler の責務を HTTP ヘッダー操作と上流転送のみに限定する。
package usecase

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"log/slog"
	"time"

	"golang.org/x/sync/singleflight"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/oauth"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/port"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/util"
)

// PrepareProxyInput は PrepareProxy ユースケースの入力パラメータ。
// handler が SessionMiddleware から取得した情報を保持する。
type PrepareProxyInput struct {
	// SessionData は SessionMiddleware が gin context に格納したセッションデータ。
	SessionData *session.SessionData
	// SessionID は gin context から取得したセッション ID。singleflight のキーに使用する。
	SessionID string
	// NeedsRefresh は SessionMiddleware が設定したサイレントリフレッシュ要否フラグ。
	// 「アクセストークン期限切れ かつ refresh token あり」の場合のみ true になる。
	NeedsRefresh bool
}

// PrepareProxyOutput は PrepareProxy ユースケースの出力。
// handler はこの出力を元に Authorization ヘッダーを設定して上流に転送する。
type PrepareProxyOutput struct {
	// AccessToken は上流 API へのリクエストに使用するアクセストークン。
	AccessToken string
	// CSRFToken はトークンリフレッシュ後に再生成された CSRF トークン（H-10 監査対応）。
	// リフレッシュが発生しなかった場合は空文字列。
	// 非空の場合、handler は X-CSRF-Token レスポンスヘッダーに設定してクライアントに通知する。
	CSRFToken string
	// TokenRefreshed はトークンリフレッシュが発生したかどうか（H-10 監査対応）。
	// true の場合、クライアントは新しい CSRFToken を受け取り更新する必要がある。
	TokenRefreshed bool
}

// ProxyUseCase はリバースプロキシ処理のビジネスロジックを提供する。
// セッション取得・有効性判定・サイレントリフレッシュ・セッション更新を担当する。
type ProxyUseCase struct {
	// oauthClient は OAuth2/OIDC プロバイダーとの通信を担うポートインターフェース。
	// nil の場合はリフレッシュ処理をスキップする（テスト用途）。
	oauthClient port.OAuthClient
	// sessionStore はセッションデータの永続化を担うポートインターフェース。
	sessionStore port.SessionStore
	// sessionTTL はリフレッシュ後のセッション更新に使用する TTL。
	sessionTTL time.Duration
	// logger はエラーやワーニングの記録に使用する。
	logger *slog.Logger
	// refreshGroup は G-03 対応: トークンリフレッシュの重複実行を防止する singleflight グループ。
	// 同一セッション ID に対して複数の並行リクエストが同時にリフレッシュを試みる場合、
	// 最初の 1 件のみ実際に RefreshToken を呼び出し、他は結果を共有する。
	refreshGroup singleflight.Group
}

// NewProxyUseCase は ProxyUseCase のコンストラクタ。
// 依存するポートインターフェースを注入する。
// oauthClient は nil 可能（リフレッシュ処理不要なテスト環境向け）。
func NewProxyUseCase(oauthClient port.OAuthClient, sessionStore port.SessionStore, sessionTTL time.Duration, logger *slog.Logger) *ProxyUseCase {
	return &ProxyUseCase{
		oauthClient:  oauthClient,
		sessionStore: sessionStore,
		sessionTTL:   sessionTTL,
		logger:       logger,
	}
}

// PrepareProxy はプロキシリクエスト前のセッション検証とトークンリフレッシュを行う。
// セッションデータが有効であれば上流 API に使用するアクセストークンを返す。
// NeedsRefresh が true の場合はサイレントリフレッシュを試み、失敗時は無効なセッションを削除する。
func (uc *ProxyUseCase) PrepareProxy(ctx context.Context, input PrepareProxyInput) (*PrepareProxyOutput, error) {
	sess := input.SessionData

	// SessionMiddleware が session_needs_refresh フラグを立てた場合のみ silent refresh を試みる。
	// フラグは「期限切れ かつ refresh token あり」の場合のみ middleware が設定する。
	if input.NeedsRefresh && uc.oauthClient != nil {
		// G-03 対応: singleflight でセッション単位のリフレッシュ重複を排除する。
		// 同一 sessionID に対して並行リクエストが殺到した場合、1 件のみ実際にリフレッシュし
		// 残りは同じ結果を共有する。これにより RefreshToken のレート制限エラーを防ぐ。
		type refreshResult struct {
			tokenResp *oauth.TokenResponse
		}
		val, err, _ := uc.refreshGroup.Do(input.SessionID, func() (any, error) {
			resp, e := uc.oauthClient.RefreshToken(ctx, sess.RefreshToken)
			if e != nil {
				return nil, e
			}
			return &refreshResult{tokenResp: resp}, nil
		})
		if err != nil {
			if uc.logger != nil {
				uc.logger.Warn("トークンリフレッシュに失敗しました。セッションを削除します",
					slog.String("error", err.Error()),
				)
			}
			// リフレッシュ失敗時に無効なセッションを削除し、再利用を防止する（H-003）
			// セッション ID は漏洩防止のためマスクして出力する（HIGH-7 対応）
			if delErr := uc.sessionStore.Delete(ctx, input.SessionID); delErr != nil {
				if uc.logger != nil {
					uc.logger.Error("期限切れセッションの削除に失敗しました",
						slog.String("error", delErr.Error()),
						slog.String("session_id", util.MaskSessionID(input.SessionID)),
					)
				}
			}
			return nil, &ProxyUseCaseError{Code: "BFF_PROXY_TOKEN_EXPIRED", Err: err}
		}

		tokenResp := val.(*refreshResult).tokenResp

		// M-5 対応: Redis を先に更新し、成功後にのみメモリ上のセッションを更新する。
		// Redis 更新前にメモリを変更すると、Redis 失敗時に状態が乖離してセッション不整合が生じる。

		// 新しいセッションデータを一時オブジェクトに構築する（メモリはまだ変更しない）
		updatedAccessToken := tokenResp.AccessToken
		updatedRefreshToken := sess.RefreshToken
		updatedIDToken := sess.IDToken
		updatedExpiresAt := time.Now().Add(time.Duration(tokenResp.ExpiresIn) * time.Second).Unix()

		// リフレッシュレスポンスに新しい refresh token / ID token が含まれる場合は上書きする
		if tokenResp.RefreshToken != "" {
			updatedRefreshToken = tokenResp.RefreshToken
		}
		if tokenResp.IDToken != "" {
			updatedIDToken = tokenResp.IDToken
		}

		// H-10 監査対応: トークンリフレッシュ後に CSRF トークンを再生成する。
		// 長期間同一の CSRF トークンが使われ続けるリスクを軽減する。
		newCSRFToken, csrfErr := generateProxyRandomHex(32)
		if csrfErr != nil {
			// CSRF 再生成に失敗しても既存トークンを継続使用し、処理は続行する
			if uc.logger != nil {
				uc.logger.Warn("リフレッシュ後の CSRF トークン再生成に失敗しました（既存トークンを継続使用）",
					slog.String("error", csrfErr.Error()),
				)
			}
			newCSRFToken = sess.CSRFToken
		}

		// 一時オブジェクトで Redis を先に更新する（shallow copy: SessionData にポインタフィールドなし）
		tempSess := *sess
		tempSess.AccessToken = updatedAccessToken
		tempSess.RefreshToken = updatedRefreshToken
		tempSess.IDToken = updatedIDToken
		tempSess.ExpiresAt = updatedExpiresAt
		// H-10 監査対応: 更新したセッションに新しい CSRF トークンを格納する
		tempSess.CSRFToken = newCSRFToken

		// Redis 更新失敗時はエラーを返し、メモリ上のセッションは変更しない
		// セッション ID は漏洩防止のためマスクして出力する（HIGH-7 対応）
		if err := uc.sessionStore.Update(ctx, input.SessionID, &tempSess, uc.sessionTTL); err != nil {
			if uc.logger != nil {
				uc.logger.Error("リフレッシュ後のセッション更新に失敗しました",
					slog.String("session_id", util.MaskSessionID(input.SessionID)),
					slog.String("error", err.Error()),
				)
			}
			return nil, &ProxyUseCaseError{Code: "BFF_PROXY_SESSION_UPDATE_FAILED", Err: err}
		}

		// Redis 更新成功後にメモリ上のセッションを更新する（M-10 監査対応）。
		// tempSess ローカルコピーで Redis を先に更新し、成功後のみ元のポインタ (*sess) を変更する。
		// これにより Redis 失敗時の状態乖離を防ぐ（副作用による不具合リスクを最小化する）。
		sess.AccessToken = updatedAccessToken
		sess.RefreshToken = updatedRefreshToken
		sess.IDToken = updatedIDToken
		sess.ExpiresAt = updatedExpiresAt
		// H-10 監査対応: メモリ上のセッションにも新しい CSRF トークンを反映する
		sess.CSRFToken = newCSRFToken

		return &PrepareProxyOutput{
			AccessToken:    sess.AccessToken,
			CSRFToken:      newCSRFToken,
			TokenRefreshed: true,
		}, nil
	}

	return &PrepareProxyOutput{
		AccessToken: sess.AccessToken,
	}, nil
}

// generateProxyRandomHex はバイト長 n のランダムな hex エンコード文字列を生成する（H-10 監査対応）。
// CSRF トークン再生成に使用する。proxy_usecase パッケージ内でのみ使用する。
func generateProxyRandomHex(n int) (string, error) {
	b := make([]byte, n)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return hex.EncodeToString(b), nil
}

// ProxyUseCaseError は ProxyUseCase が返すエラー型。
// エラーコードと元のエラーをラップする。
type ProxyUseCaseError struct {
	// Code はエラーコード（例: "BFF_PROXY_TOKEN_EXPIRED"）。
	Code string
	// Err は元のエラー（省略可能）。
	Err error
}

// Error は error インターフェースの実装。
func (e *ProxyUseCaseError) Error() string {
	if e.Err != nil {
		return e.Code + ": " + e.Err.Error()
	}
	return e.Code
}

// Unwrap は元のエラーを返す（errors.Is/As で辿れるようにする）。
func (e *ProxyUseCaseError) Unwrap() error {
	return e.Err
}
