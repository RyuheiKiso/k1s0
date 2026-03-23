// proxy_usecase.go はリバースプロキシ処理におけるセッション検証・トークンリフレッシュの
// ビジネスロジックを提供する。handler が直接 OAuth クライアントとセッションストアを操作していた
// 処理を抽象化し、handler の責務を HTTP ヘッダー操作と上流転送のみに限定する。
package usecase

import (
	"context"
	"log/slog"
	"time"

	"golang.org/x/sync/singleflight"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/oauth"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/port"
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
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
			if delErr := uc.sessionStore.Delete(ctx, input.SessionID); delErr != nil {
				if uc.logger != nil {
					uc.logger.Error("期限切れセッションの削除に失敗しました",
						slog.String("error", delErr.Error()),
						slog.String("session_id", input.SessionID),
					)
				}
			}
			return nil, &ProxyUseCaseError{Code: "BFF_PROXY_TOKEN_EXPIRED", Err: err}
		}

		tokenResp := val.(*refreshResult).tokenResp

		// 新しいトークンでセッションデータを更新する
		sess.AccessToken = tokenResp.AccessToken
		if tokenResp.RefreshToken != "" {
			sess.RefreshToken = tokenResp.RefreshToken
		}
		if tokenResp.IDToken != "" {
			sess.IDToken = tokenResp.IDToken
		}
		sess.ExpiresAt = time.Now().Add(time.Duration(tokenResp.ExpiresIn) * time.Second).Unix()

		// リフレッシュ後のセッション更新に失敗した場合はエラーログを記録する
		if err := uc.sessionStore.Update(ctx, input.SessionID, sess, uc.sessionTTL); err != nil {
			if uc.logger != nil {
				uc.logger.Error("リフレッシュ後のセッション更新に失敗しました",
					slog.String("error", err.Error()),
				)
			}
		}
	}

	return &PrepareProxyOutput{
		AccessToken: sess.AccessToken,
	}, nil
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
