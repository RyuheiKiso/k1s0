package testhelper

import (
	"crypto/hmac"
	"crypto/sha256"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"math/rand"
	"strings"
	"sync"
	"time"

	"github.com/google/uuid"
)

// TestClaims はテスト用 JWT クレーム。
type TestClaims struct {
	Sub      string   `json:"sub"`
	Roles    []string `json:"roles"`
	TenantID string   `json:"tenant_id,omitempty"`
	Iat      int64    `json:"iat"`
	Exp      int64    `json:"exp"`
}

// JwtTestHelper はテスト用 JWT トークン生成ヘルパー。
type JwtTestHelper struct {
	secret string
}

// NewJwtTestHelper は新しい JwtTestHelper を生成する。
func NewJwtTestHelper(secret string) *JwtTestHelper {
	return &JwtTestHelper{secret: secret}
}

// CreateAdminToken は管理者トークンを生成する。
func (h *JwtTestHelper) CreateAdminToken() string {
	now := time.Now().Unix()
	claims := TestClaims{
		Sub:   "admin",
		Roles: []string{"admin"},
		Iat:   now,
		Exp:   now + 3600,
	}
	return h.CreateToken(claims)
}

// CreateUserToken はユーザートークンを生成する。
func (h *JwtTestHelper) CreateUserToken(userID string, roles []string) string {
	now := time.Now().Unix()
	claims := TestClaims{
		Sub:   userID,
		Roles: roles,
		Iat:   now,
		Exp:   now + 3600,
	}
	return h.CreateToken(claims)
}

// CreateToken はカスタムクレームでトークンを生成する。
func (h *JwtTestHelper) CreateToken(claims TestClaims) string {
	header := base64URLEncode([]byte(`{"alg":"HS256","typ":"JWT"}`))
	payloadBytes, _ := json.Marshal(claims)
	payload := base64URLEncode(payloadBytes)
	signingInput := header + "." + payload
	mac := hmac.New(sha256.New, []byte(h.secret))
	mac.Write([]byte(signingInput))
	signature := base64URLEncode(mac.Sum(nil))
	return signingInput + "." + signature
}

// DecodeClaims はトークンのペイロードをデコードしてクレームを返す。
func (h *JwtTestHelper) DecodeClaims(token string) (*TestClaims, error) {
	parts := strings.Split(token, ".")
	if len(parts) != 3 {
		return nil, fmt.Errorf("invalid token format")
	}
	payloadBytes, err := base64URLDecode(parts[1])
	if err != nil {
		return nil, err
	}
	var claims TestClaims
	if err := json.Unmarshal(payloadBytes, &claims); err != nil {
		return nil, err
	}
	return &claims, nil
}

func base64URLEncode(data []byte) string {
	return strings.TrimRight(base64.URLEncoding.EncodeToString(data), "=")
}

func base64URLDecode(s string) ([]byte, error) {
	if m := len(s) % 4; m != 0 {
		s += strings.Repeat("=", 4-m)
	}
	return base64.URLEncoding.DecodeString(s)
}

// MockRoute はモックルート定義。
type MockRoute struct {
	Method string
	Path   string
	Status int
	Body   string
}

// MockServer はインメモリのモックサーバー。
type MockServer struct {
	routes   []MockRoute
	mu       sync.Mutex
	requests []struct{ Method, Path string }
}

// Handle は登録済みルートからレスポンスを取得する。
func (s *MockServer) Handle(method, path string) (int, string, bool) {
	s.mu.Lock()
	s.requests = append(s.requests, struct{ Method, Path string }{method, path})
	s.mu.Unlock()
	for _, r := range s.routes {
		if r.Method == method && r.Path == path {
			return r.Status, r.Body, true
		}
	}
	return 0, "", false
}

// RequestCount は記録されたリクエスト数を返す。
func (s *MockServer) RequestCount() int {
	s.mu.Lock()
	defer s.mu.Unlock()
	return len(s.requests)
}

// MockServerBuilder はモックサーバービルダー。
type MockServerBuilder struct {
	serverType string
	routes     []MockRoute
}

// NewNotificationServerMock は Notification サーバーモックを構築する。
func NewNotificationServerMock() *MockServerBuilder {
	return &MockServerBuilder{serverType: "notification"}
}

// NewRatelimitServerMock は Ratelimit サーバーモックを構築する。
func NewRatelimitServerMock() *MockServerBuilder {
	return &MockServerBuilder{serverType: "ratelimit"}
}

// NewTenantServerMock は Tenant サーバーモックを構築する。
func NewTenantServerMock() *MockServerBuilder {
	return &MockServerBuilder{serverType: "tenant"}
}

// WithHealthOK はヘルスチェック用の成功レスポンスを追加する。
func (b *MockServerBuilder) WithHealthOK() *MockServerBuilder {
	b.routes = append(b.routes, MockRoute{
		Method: "GET",
		Path:   "/health",
		Status: 200,
		Body:   `{"status":"ok"}`,
	})
	return b
}

// WithSuccessResponse は成功レスポンスを追加する。
func (b *MockServerBuilder) WithSuccessResponse(path, body string) *MockServerBuilder {
	b.routes = append(b.routes, MockRoute{
		Method: "POST",
		Path:   path,
		Status: 200,
		Body:   body,
	})
	return b
}

// Build はモックサーバーを構築する。
func (b *MockServerBuilder) Build() *MockServer {
	return &MockServer{routes: b.routes}
}

// FixtureBuilder はテスト用フィクスチャビルダー。
type FixtureBuilder struct{}

// UUID はランダム UUID を生成する。
func (FixtureBuilder) UUID() string {
	return uuid.New().String()
}

// Email はランダムなテスト用メールアドレスを生成する。
func (FixtureBuilder) Email() string {
	return fmt.Sprintf("test-%s@example.com", uuid.New().String()[:8])
}

// Name はランダムなテスト用ユーザー名を生成する。
func (FixtureBuilder) Name() string {
	return fmt.Sprintf("user-%s", uuid.New().String()[:8])
}

// Int は指定範囲のランダム整数を生成する。
func (FixtureBuilder) Int(min, max int) int {
	if min >= max {
		return min
	}
	return min + rand.Intn(max-min)
}

// TenantID はテスト用テナント ID を生成する。
func (FixtureBuilder) TenantID() string {
	return fmt.Sprintf("tenant-%s", uuid.New().String()[:8])
}

// AssertionHelper はテスト用アサーションヘルパー。
type AssertionHelper struct{}

// JSONContains は actual が expected の全キーを含んでいるか検証する。
func (AssertionHelper) JSONContains(actual, expected string) error {
	var actualMap, expectedMap map[string]interface{}
	if err := json.Unmarshal([]byte(actual), &actualMap); err != nil {
		return fmt.Errorf("actual is not valid JSON: %w", err)
	}
	if err := json.Unmarshal([]byte(expected), &expectedMap); err != nil {
		return fmt.Errorf("expected is not valid JSON: %w", err)
	}
	if !jsonContains(actualMap, expectedMap) {
		return fmt.Errorf("JSON partial match failed.\nActual: %s\nExpected: %s", actual, expected)
	}
	return nil
}

// EventEmitted はイベント一覧に指定タイプのイベントが含まれるか検証する。
func (AssertionHelper) EventEmitted(events []map[string]interface{}, eventType string) error {
	for _, e := range events {
		if t, ok := e["type"].(string); ok && t == eventType {
			return nil
		}
	}
	return fmt.Errorf("event '%s' not found in events", eventType)
}

func jsonContains(actual, expected map[string]interface{}) bool {
	for k, ev := range expected {
		av, ok := actual[k]
		if !ok {
			return false
		}
		switch evTyped := ev.(type) {
		case map[string]interface{}:
			avMap, ok := av.(map[string]interface{})
			if !ok || !jsonContains(avMap, evTyped) {
				return false
			}
		default:
			aJSON, _ := json.Marshal(av)
			eJSON, _ := json.Marshal(ev)
			if string(aJSON) != string(eJSON) {
				return false
			}
		}
	}
	return true
}
