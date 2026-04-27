// 認可 middleware。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/04_bff配置.md
//
// 役割:
//   リリース時点 では最小実装として `Authorization: Bearer <token>` ヘッダの存在チェック + ロール抽出のみ。
//   リリース時点 で Keycloak OIDC JWT 検証（jwks 取得 + 署名 + 期限 + audience）に拡張する。

// Package auth は BFF の認可 middleware を提供する。
package auth

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// JSON エンコード。
	"encoding/json"
	// HTTP server。
	"net/http"
	// 文字列処理。
	"strings"

	// BFF 共通エラー型。
	bffErrors "github.com/k1s0/k1s0/src/tier3/bff/internal/shared/errors"
)

// contextKey は context 経由でユーザ識別を渡す際のキー。
type contextKey string

// SubjectKey は認証済 subject を context から取り出すキー。
const SubjectKey contextKey = "k1s0.subject"

// RolesKey は認証済 roles を context から取り出すキー。
const RolesKey contextKey = "k1s0.roles"

// Required はトークン必須の middleware を返す。requireRole が空文字なら role チェック skip。
func Required(requireRole string) func(http.Handler) http.Handler {
	return func(next http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			// Authorization ヘッダから Bearer token を取得する。
			authHeader := r.Header.Get("Authorization")
			if !strings.HasPrefix(authHeader, "Bearer ") {
				writeUnauthorized(w, "missing bearer token")
				return
			}
			token := strings.TrimPrefix(authHeader, "Bearer ")
			if strings.TrimSpace(token) == "" {
				writeUnauthorized(w, "empty token")
				return
			}
			// リリース時点 の最小実装: token 内容を検証せず、subject / roles を仮に展開する。
			// リリース時点 で go-jose 等で JWT 検証に置換する。
			subject := "user-" + token[:min(8, len(token))]
			roles := []string{"user"}
			// 開発用 fixed admin token（"admin-token"）を受けたら admin role を付与する。
			if token == "admin-token" {
				subject = "admin-user"
				roles = []string{"admin", "user"}
			}
			// require role チェック。
			if requireRole != "" {
				ok := false
				for _, r := range roles {
					if r == requireRole {
						ok = true
						break
					}
				}
				if !ok {
					writeForbidden(w, "missing role: "+requireRole)
					return
				}
			}
			// context にセットして後段に流す。
			ctx := context.WithValue(r.Context(), SubjectKey, subject)
			ctx = context.WithValue(ctx, RolesKey, roles)
			next.ServeHTTP(w, r.WithContext(ctx))
		})
	}
}

// SubjectFromContext は middleware が context にセットした subject を取り出す。
func SubjectFromContext(ctx context.Context) string {
	v, ok := ctx.Value(SubjectKey).(string)
	if !ok {
		return ""
	}
	return v
}

// RolesFromContext は middleware が context にセットした roles を取り出す。
func RolesFromContext(ctx context.Context) []string {
	v, ok := ctx.Value(RolesKey).([]string)
	if !ok {
		return nil
	}
	return v
}

func writeUnauthorized(w http.ResponseWriter, msg string) {
	writeJSONError(w, bffErrors.New(bffErrors.CategoryUnauthorized, "E-T3-BFF-AUTH-001", msg))
}

func writeForbidden(w http.ResponseWriter, msg string) {
	writeJSONError(w, bffErrors.New(bffErrors.CategoryForbidden, "E-T3-BFF-AUTH-002", msg))
}

func writeJSONError(w http.ResponseWriter, de *bffErrors.DomainError) {
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.WriteHeader(de.Category.HTTPStatus())
	_ = json.NewEncoder(w).Encode(map[string]any{
		"error": map[string]any{
			"code":     de.Code,
			"message":  de.Message,
			"category": string(de.Category),
		},
	})
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}
