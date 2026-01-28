package k1s0auth

import (
	"context"
	"errors"
	"os"
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v5"
)

// =============================================================================
// Config Tests
// =============================================================================

func TestAuthConfig_Validate_Success(t *testing.T) {
	config := DefaultAuthConfig()
	config.JWT.Secret = "test-secret"

	err := config.Validate()
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
}

func TestAuthConfig_Validate_MissingSecret(t *testing.T) {
	config := DefaultAuthConfig()

	err := config.Validate()
	if err == nil {
		t.Error("expected error for missing secret")
	}
}

func TestJWTConfig_GetSecret_Direct(t *testing.T) {
	config := &JWTConfig{Secret: "direct-secret"}

	secret, err := config.GetSecret()
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if secret != "direct-secret" {
		t.Errorf("expected 'direct-secret', got '%s'", secret)
	}
}

func TestJWTConfig_GetSecret_FromFile(t *testing.T) {
	// Create temp file with secret
	tmpFile, err := os.CreateTemp("", "secret")
	if err != nil {
		t.Fatal(err)
	}
	defer os.Remove(tmpFile.Name())

	if _, err := tmpFile.WriteString("file-secret\n"); err != nil {
		t.Fatal(err)
	}
	tmpFile.Close()

	config := &JWTConfig{SecretFile: tmpFile.Name()}

	secret, err := config.GetSecret()
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if secret != "file-secret" {
		t.Errorf("expected 'file-secret', got '%s'", secret)
	}
}

func TestAuthConfigBuilder(t *testing.T) {
	config, err := NewAuthConfigBuilder().
		JWTSecret("test-secret").
		JWTIssuer("https://auth.example.com").
		JWTAudience("app1", "app2").
		JWTAlgorithms("HS256", "RS256").
		CacheEnabled(true).
		CacheTTL(10 * time.Minute).
		Build()

	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if config.JWT.Secret != "test-secret" {
		t.Errorf("expected secret 'test-secret', got '%s'", config.JWT.Secret)
	}
	if config.JWT.Issuer != "https://auth.example.com" {
		t.Errorf("expected issuer, got '%s'", config.JWT.Issuer)
	}
	if len(config.JWT.Audience) != 2 {
		t.Errorf("expected 2 audiences, got %d", len(config.JWT.Audience))
	}
	if !config.Cache.Enabled {
		t.Error("expected cache to be enabled")
	}
}

// =============================================================================
// Claims Tests
// =============================================================================

func TestClaims_HasRole(t *testing.T) {
	claims := &Claims{
		Roles: []string{"admin", "user"},
	}

	if !claims.HasRole("admin") {
		t.Error("expected to have role 'admin'")
	}
	if !claims.HasRole("user") {
		t.Error("expected to have role 'user'")
	}
	if claims.HasRole("superuser") {
		t.Error("expected not to have role 'superuser'")
	}
}

func TestClaims_HasAnyRole(t *testing.T) {
	claims := &Claims{
		Roles: []string{"user"},
	}

	if !claims.HasAnyRole("admin", "user") {
		t.Error("expected to have any role")
	}
	if claims.HasAnyRole("admin", "superuser") {
		t.Error("expected not to have any role")
	}
}

func TestClaims_HasAllRoles(t *testing.T) {
	claims := &Claims{
		Roles: []string{"admin", "user"},
	}

	if !claims.HasAllRoles("admin", "user") {
		t.Error("expected to have all roles")
	}
	if claims.HasAllRoles("admin", "superuser") {
		t.Error("expected not to have all roles")
	}
}

func TestClaims_HasPermission(t *testing.T) {
	claims := &Claims{
		Permissions: []string{"read:users", "write:users"},
	}

	if !claims.HasPermission("read:users") {
		t.Error("expected to have permission 'read:users'")
	}
	if claims.HasPermission("delete:users") {
		t.Error("expected not to have permission 'delete:users'")
	}
}

func TestClaims_InGroup(t *testing.T) {
	claims := &Claims{
		Groups: []string{"developers", "admins"},
	}

	if !claims.InGroup("developers") {
		t.Error("expected to be in group 'developers'")
	}
	if claims.InGroup("managers") {
		t.Error("expected not to be in group 'managers'")
	}
}

// =============================================================================
// Principal Tests
// =============================================================================

func TestNewPrincipal(t *testing.T) {
	claims := &Claims{
		RegisteredClaims: jwt.RegisteredClaims{
			Subject: "user123",
		},
		Email:       "user@example.com",
		Name:        "Test User",
		Roles:       []string{"admin"},
		Permissions: []string{"read:users"},
		Groups:      []string{"admins"},
		TenantID:    "tenant1",
	}

	principal := NewPrincipal(claims)

	if principal.ID != "user123" {
		t.Errorf("expected ID 'user123', got '%s'", principal.ID)
	}
	if principal.Email != "user@example.com" {
		t.Errorf("expected email 'user@example.com', got '%s'", principal.Email)
	}
	if principal.Name != "Test User" {
		t.Errorf("expected name 'Test User', got '%s'", principal.Name)
	}
	if principal.TenantID != "tenant1" {
		t.Errorf("expected tenant 'tenant1', got '%s'", principal.TenantID)
	}
}

func TestPrincipalContext(t *testing.T) {
	ctx := context.Background()

	// Initially no principal
	if IsPrincipalAuthenticated(ctx) {
		t.Error("expected no principal in initial context")
	}

	// Add principal
	claims := &Claims{
		RegisteredClaims: jwt.RegisteredClaims{
			Subject: "user123",
		},
	}
	principal := NewPrincipal(claims)
	ctx = ContextWithPrincipal(ctx, principal)

	if !IsPrincipalAuthenticated(ctx) {
		t.Error("expected principal in context")
	}

	retrieved := PrincipalFromContext(ctx)
	if retrieved.ID != "user123" {
		t.Errorf("expected ID 'user123', got '%s'", retrieved.ID)
	}
}

func TestRequirePrincipal(t *testing.T) {
	ctx := context.Background()

	_, err := RequirePrincipal(ctx)
	if !errors.Is(err, ErrNotAuthenticated) {
		t.Error("expected ErrNotAuthenticated")
	}

	claims := &Claims{
		RegisteredClaims: jwt.RegisteredClaims{
			Subject: "user123",
		},
	}
	principal := NewPrincipal(claims)
	ctx = ContextWithPrincipal(ctx, principal)

	p, err := RequirePrincipal(ctx)
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if p.ID != "user123" {
		t.Errorf("expected ID 'user123', got '%s'", p.ID)
	}
}

// =============================================================================
// JWT Validator Tests
// =============================================================================

func TestJWTValidator_Validate(t *testing.T) {
	secret := []byte("test-secret-key-12345678901234567890")

	config := &JWTConfig{
		Secret:     string(secret),
		Algorithms: []string{"HS256"},
		Issuer:     "test-issuer",
		Audience:   []string{"test-audience"},
	}

	validator, err := NewJWTValidator(config)
	if err != nil {
		t.Fatalf("failed to create validator: %v", err)
	}

	// Create a valid token
	claims := &Claims{
		RegisteredClaims: jwt.RegisteredClaims{
			Subject:   "user123",
			Issuer:    "test-issuer",
			Audience:  jwt.ClaimStrings{"test-audience"},
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(time.Hour)),
			IssuedAt:  jwt.NewNumericDate(time.Now()),
		},
		Email: "user@example.com",
		Roles: []string{"admin"},
	}

	token, err := CreateHS256Token(claims, secret)
	if err != nil {
		t.Fatalf("failed to create token: %v", err)
	}

	// Validate the token
	validated, err := validator.Validate(token)
	if err != nil {
		t.Errorf("expected no error, got %v", err)
	}
	if validated.Subject() != "user123" {
		t.Errorf("expected subject 'user123', got '%s'", validated.Subject())
	}
	if validated.Email != "user@example.com" {
		t.Errorf("expected email 'user@example.com', got '%s'", validated.Email)
	}
}

func TestJWTValidator_ExpiredToken(t *testing.T) {
	secret := []byte("test-secret-key-12345678901234567890")

	config := &JWTConfig{
		Secret:     string(secret),
		Algorithms: []string{"HS256"},
		ClockSkew:  time.Minute,
	}

	validator, err := NewJWTValidator(config)
	if err != nil {
		t.Fatalf("failed to create validator: %v", err)
	}

	// Create an expired token
	claims := &Claims{
		RegisteredClaims: jwt.RegisteredClaims{
			Subject:   "user123",
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(-time.Hour)),
		},
	}

	token, err := CreateHS256Token(claims, secret)
	if err != nil {
		t.Fatalf("failed to create token: %v", err)
	}

	// Validate the token
	_, err = validator.Validate(token)
	if !errors.Is(err, ErrTokenExpired) {
		t.Errorf("expected ErrTokenExpired, got %v", err)
	}
}

func TestJWTValidator_InvalidSignature(t *testing.T) {
	config := &JWTConfig{
		Secret:     "correct-secret",
		Algorithms: []string{"HS256"},
	}

	validator, err := NewJWTValidator(config)
	if err != nil {
		t.Fatalf("failed to create validator: %v", err)
	}

	// Create a token with wrong secret
	claims := &Claims{
		RegisteredClaims: jwt.RegisteredClaims{
			Subject:   "user123",
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(time.Hour)),
		},
	}

	token, err := CreateHS256Token(claims, []byte("wrong-secret"))
	if err != nil {
		t.Fatalf("failed to create token: %v", err)
	}

	// Validate the token
	_, err = validator.Validate(token)
	if !errors.Is(err, ErrInvalidSignature) {
		t.Errorf("expected ErrInvalidSignature, got %v", err)
	}
}

func TestJWTValidator_InvalidIssuer(t *testing.T) {
	secret := []byte("test-secret")

	config := &JWTConfig{
		Secret:     string(secret),
		Algorithms: []string{"HS256"},
		Issuer:     "expected-issuer",
	}

	validator, err := NewJWTValidator(config)
	if err != nil {
		t.Fatalf("failed to create validator: %v", err)
	}

	// Create a token with wrong issuer
	claims := &Claims{
		RegisteredClaims: jwt.RegisteredClaims{
			Subject:   "user123",
			Issuer:    "wrong-issuer",
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(time.Hour)),
		},
	}

	token, err := CreateHS256Token(claims, secret)
	if err != nil {
		t.Fatalf("failed to create token: %v", err)
	}

	// Validate the token
	_, err = validator.Validate(token)
	if !errors.Is(err, ErrInvalidIssuer) {
		t.Errorf("expected ErrInvalidIssuer, got %v", err)
	}
}

// =============================================================================
// ExtractBearerToken Tests
// =============================================================================

func TestExtractBearerToken(t *testing.T) {
	tests := []struct {
		name      string
		header    string
		expected  string
		expectErr error
	}{
		{
			name:     "valid bearer token",
			header:   "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test",
			expected: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test",
		},
		{
			name:     "lowercase bearer",
			header:   "bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test",
			expected: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test",
		},
		{
			name:      "empty header",
			header:    "",
			expectErr: ErrNotAuthenticated,
		},
		{
			name:      "no bearer prefix",
			header:    "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.test",
			expectErr: ErrTokenMalformed,
		},
		{
			name:      "wrong prefix",
			header:    "Basic dXNlcjpwYXNz",
			expectErr: ErrTokenMalformed,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			token, err := ExtractBearerToken(tt.header)
			if tt.expectErr != nil {
				if !errors.Is(err, tt.expectErr) {
					t.Errorf("expected error %v, got %v", tt.expectErr, err)
				}
			} else {
				if err != nil {
					t.Errorf("expected no error, got %v", err)
				}
				if token != tt.expected {
					t.Errorf("expected token '%s', got '%s'", tt.expected, token)
				}
			}
		})
	}
}
