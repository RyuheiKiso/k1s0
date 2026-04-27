// auth middleware の単体テスト。

package auth

// 標準 import。
import (
	// HTTP テスト。
	"net/http"
	"net/http/httptest"
	// テスト frameworks。
	"testing"
)

// TestRequired_NoToken_Returns401 はトークンなしリクエストが 401 を返すことを検証する。
func TestRequired_NoToken_Returns401(t *testing.T) {
	// 受け側の handler は呼ばれないはず。
	called := false
	next := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { called = true })
	mw := Required("user")(next)
	// recorder。
	rec := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/foo", nil)
	mw.ServeHTTP(rec, req)
	if rec.Code != http.StatusUnauthorized {
		t.Errorf("expected 401, got %d", rec.Code)
	}
	if called {
		t.Error("next handler should not be called")
	}
}

// TestRequired_AdminToken_Allowed は admin role を要求するハンドラに admin-token でアクセスできることを検証する。
func TestRequired_AdminToken_Allowed(t *testing.T) {
	called := false
	next := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		called = true
		// context から subject が取れること。
		if SubjectFromContext(r.Context()) != "admin-user" {
			t.Errorf("subject = %q", SubjectFromContext(r.Context()))
		}
	})
	mw := Required("admin")(next)
	rec := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/foo", nil)
	req.Header.Set("Authorization", "Bearer admin-token")
	mw.ServeHTTP(rec, req)
	if !called {
		t.Error("next handler should be called")
	}
	if rec.Code != http.StatusOK {
		t.Errorf("expected 200, got %d", rec.Code)
	}
}

// TestRequired_UserToken_AdminEndpoint_Returns403 は user role が admin endpoint を叩いた時に 403 を返すことを検証する。
func TestRequired_UserToken_AdminEndpoint_Returns403(t *testing.T) {
	next := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})
	mw := Required("admin")(next)
	rec := httptest.NewRecorder()
	req := httptest.NewRequest(http.MethodGet, "/foo", nil)
	// admin-token 以外（generic Bearer）は user role のみ。
	req.Header.Set("Authorization", "Bearer some-user-token-value")
	mw.ServeHTTP(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Errorf("expected 403, got %d", rec.Code)
	}
}
