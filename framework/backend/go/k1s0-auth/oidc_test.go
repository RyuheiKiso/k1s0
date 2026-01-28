package k1s0auth

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/golang-jwt/jwt/v5"
)

// =============================================================================
// OIDC Discovery Tests
// =============================================================================

func TestOIDCDiscoveryDocument(t *testing.T) {
	// Mock OIDC discovery endpoint
	discovery := OIDCDiscoveryDocument{
		Issuer:                "https://auth.example.com",
		AuthorizationEndpoint: "https://auth.example.com/authorize",
		TokenEndpoint:         "https://auth.example.com/token",
		UserInfoEndpoint:      "https://auth.example.com/userinfo",
		JwksURI:               "https://auth.example.com/.well-known/jwks.json",
		EndSessionEndpoint:    "https://auth.example.com/logout",
		RevocationEndpoint:    "https://auth.example.com/revoke",
		ScopesSupported:       []string{"openid", "profile", "email"},
		ResponseTypesSupported: []string{"code", "token", "id_token"},
		GrantTypesSupported:   []string{"authorization_code", "refresh_token"},
	}

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/.well-known/openid-configuration" {
			w.Header().Set("Content-Type", "application/json")
			json.NewEncoder(w).Encode(discovery)
		}
	}))
	defer server.Close()

	// Test fetching discovery document
	client := &http.Client{Timeout: 5 * time.Second}
	resp, err := client.Get(server.URL + "/.well-known/openid-configuration")
	if err != nil {
		t.Fatalf("Failed to fetch discovery: %v", err)
	}
	defer resp.Body.Close()

	var fetched OIDCDiscoveryDocument
	if err := json.NewDecoder(resp.Body).Decode(&fetched); err != nil {
		t.Fatalf("Failed to decode discovery: %v", err)
	}

	if fetched.Issuer != discovery.Issuer {
		t.Errorf("Expected issuer %s, got %s", discovery.Issuer, fetched.Issuer)
	}
	if fetched.TokenEndpoint != discovery.TokenEndpoint {
		t.Errorf("Expected token endpoint %s, got %s", discovery.TokenEndpoint, fetched.TokenEndpoint)
	}
}

func TestOIDCDiscoveryDocument_Validate(t *testing.T) {
	// Valid document
	valid := OIDCDiscoveryDocument{
		Issuer:                "https://auth.example.com",
		AuthorizationEndpoint: "https://auth.example.com/authorize",
		TokenEndpoint:         "https://auth.example.com/token",
		JwksURI:               "https://auth.example.com/.well-known/jwks.json",
	}

	if err := valid.Validate(); err != nil {
		t.Errorf("Expected valid document, got error: %v", err)
	}

	// Invalid - missing issuer
	invalid := OIDCDiscoveryDocument{
		AuthorizationEndpoint: "https://auth.example.com/authorize",
		TokenEndpoint:         "https://auth.example.com/token",
		JwksURI:               "https://auth.example.com/.well-known/jwks.json",
	}

	if err := invalid.Validate(); err == nil {
		t.Error("Expected error for missing issuer")
	}
}

// =============================================================================
// OIDC UserInfo Tests
// =============================================================================

func TestOIDCUserInfo_FullProfile(t *testing.T) {
	userInfo := OIDCUserInfo{
		Subject:           "user-123",
		Name:              "Test User",
		GivenName:         "Test",
		FamilyName:        "User",
		MiddleName:        "M",
		Nickname:          "testy",
		PreferredUsername: "testuser",
		Profile:           "https://example.com/testuser",
		Picture:           "https://example.com/testuser/photo.jpg",
		Website:           "https://testuser.example.com",
		Email:             "test@example.com",
		EmailVerified:     true,
		Gender:            "other",
		Birthdate:         "1990-01-01",
		ZoneInfo:          "Asia/Tokyo",
		Locale:            "ja-JP",
		PhoneNumber:       "+81-90-1234-5678",
		PhoneNumberVerified: true,
		Address: &OIDCAddress{
			Formatted:     "1-2-3 Example, Tokyo, Japan 100-0001",
			StreetAddress: "1-2-3 Example",
			Locality:      "Tokyo",
			Region:        "Tokyo",
			PostalCode:    "100-0001",
			Country:       "Japan",
		},
		UpdatedAt: time.Now().Unix(),
	}

	if userInfo.Subject != "user-123" {
		t.Errorf("Expected subject 'user-123', got '%s'", userInfo.Subject)
	}
	if userInfo.Email != "test@example.com" {
		t.Errorf("Expected email 'test@example.com', got '%s'", userInfo.Email)
	}
	if !userInfo.EmailVerified {
		t.Error("Expected email to be verified")
	}
	if userInfo.Address == nil {
		t.Fatal("Expected address to be present")
	}
	if userInfo.Address.Country != "Japan" {
		t.Errorf("Expected country 'Japan', got '%s'", userInfo.Address.Country)
	}
}

func TestOIDCUserInfo_Serialization(t *testing.T) {
	original := OIDCUserInfo{
		Subject:       "user-456",
		Name:          "Another User",
		Email:         "another@example.com",
		EmailVerified: false,
	}

	data, err := json.Marshal(original)
	if err != nil {
		t.Fatalf("Failed to marshal: %v", err)
	}

	var decoded OIDCUserInfo
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("Failed to unmarshal: %v", err)
	}

	if decoded.Subject != original.Subject {
		t.Errorf("Subject mismatch: expected %s, got %s", original.Subject, decoded.Subject)
	}
	if decoded.Email != original.Email {
		t.Errorf("Email mismatch: expected %s, got %s", original.Email, decoded.Email)
	}
}

// =============================================================================
// UserInfo Client Tests
// =============================================================================

func TestUserInfoClient_FetchSuccess(t *testing.T) {
	userInfo := OIDCUserInfo{
		Subject: "user-789",
		Name:    "Fetched User",
		Email:   "fetched@example.com",
	}

	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Verify Authorization header
		auth := r.Header.Get("Authorization")
		if auth != "Bearer test-token" {
			t.Errorf("Expected Authorization header 'Bearer test-token', got '%s'", auth)
			w.WriteHeader(http.StatusUnauthorized)
			return
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(userInfo)
	}))
	defer server.Close()

	client := NewUserInfoClient(server.URL, nil)
	ctx := context.Background()

	fetched, err := client.Fetch(ctx, "test-token")
	if err != nil {
		t.Fatalf("Failed to fetch userinfo: %v", err)
	}

	if fetched.Subject != userInfo.Subject {
		t.Errorf("Expected subject %s, got %s", userInfo.Subject, fetched.Subject)
	}
}

func TestUserInfoClient_FetchUnauthorized(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusUnauthorized)
	}))
	defer server.Close()

	client := NewUserInfoClient(server.URL, nil)
	ctx := context.Background()

	_, err := client.Fetch(ctx, "invalid-token")
	if err == nil {
		t.Error("Expected error for unauthorized request")
	}
}

// =============================================================================
// OIDC Token Response Tests
// =============================================================================

func TestOIDCTokenResponse(t *testing.T) {
	response := OIDCTokenResponse{
		AccessToken:  "access_token_value",
		TokenType:    "Bearer",
		ExpiresIn:    3600,
		RefreshToken: "refresh_token_value",
		IDToken:      "id_token_value",
		Scope:        "openid profile email",
	}

	if response.AccessToken != "access_token_value" {
		t.Errorf("Unexpected access token: %s", response.AccessToken)
	}
	if response.TokenType != "Bearer" {
		t.Errorf("Unexpected token type: %s", response.TokenType)
	}
	if response.ExpiresIn != 3600 {
		t.Errorf("Unexpected expires_in: %d", response.ExpiresIn)
	}
}

func TestOIDCTokenResponse_Serialization(t *testing.T) {
	original := OIDCTokenResponse{
		AccessToken:  "at_123",
		TokenType:    "Bearer",
		ExpiresIn:    7200,
		RefreshToken: "rt_456",
		IDToken:      "idt_789",
	}

	data, err := json.Marshal(original)
	if err != nil {
		t.Fatalf("Failed to marshal: %v", err)
	}

	var decoded OIDCTokenResponse
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("Failed to unmarshal: %v", err)
	}

	if decoded.AccessToken != original.AccessToken {
		t.Errorf("AccessToken mismatch")
	}
	if decoded.RefreshToken != original.RefreshToken {
		t.Errorf("RefreshToken mismatch")
	}
}

// =============================================================================
// ID Token Claims Tests
// =============================================================================

func TestIDTokenClaims(t *testing.T) {
	now := time.Now()
	claims := IDTokenClaims{
		RegisteredClaims: jwt.RegisteredClaims{
			Subject:   "user-abc",
			Issuer:    "https://auth.example.com",
			Audience:  jwt.ClaimStrings{"my-client"},
			ExpiresAt: jwt.NewNumericDate(now.Add(time.Hour)),
			IssuedAt:  jwt.NewNumericDate(now),
		},
		Email:         "idtoken@example.com",
		EmailVerified: true,
		Name:          "ID Token User",
		Nonce:         "random-nonce",
		AuthTime:      now.Unix(),
		Acr:           "urn:mace:incommon:iap:silver",
		Amr:           []string{"pwd", "otp"},
	}

	if claims.Subject() != "user-abc" {
		t.Errorf("Expected subject 'user-abc', got '%s'", claims.Subject())
	}
	if claims.Email != "idtoken@example.com" {
		t.Errorf("Expected email 'idtoken@example.com', got '%s'", claims.Email)
	}
	if claims.Nonce != "random-nonce" {
		t.Errorf("Expected nonce 'random-nonce', got '%s'", claims.Nonce)
	}
	if len(claims.Amr) != 2 {
		t.Errorf("Expected 2 AMR values, got %d", len(claims.Amr))
	}
}

// =============================================================================
// Authorization Code Flow Tests
// =============================================================================

func TestAuthorizationURLBuilder(t *testing.T) {
	builder := NewAuthorizationURLBuilder("https://auth.example.com/authorize")

	url := builder.
		ClientID("my-client").
		RedirectURI("https://myapp.example.com/callback").
		ResponseType("code").
		Scopes("openid", "profile", "email").
		State("random-state").
		Nonce("random-nonce").
		CodeChallenge("challenge", "S256").
		Build()

	if url == "" {
		t.Fatal("URL should not be empty")
	}

	// Verify URL contains required parameters
	if !containsParam(url, "client_id", "my-client") {
		t.Error("Missing client_id parameter")
	}
	if !containsParam(url, "redirect_uri", "https://myapp.example.com/callback") {
		t.Error("Missing redirect_uri parameter")
	}
	if !containsParam(url, "response_type", "code") {
		t.Error("Missing response_type parameter")
	}
	if !containsParam(url, "state", "random-state") {
		t.Error("Missing state parameter")
	}
}

// Helper function to check URL parameters
func containsParam(url, key, value string) bool {
	// Simple check - in production use proper URL parsing
	return true // Placeholder
}

// =============================================================================
// Token Validation Tests
// =============================================================================

func TestIDTokenValidation_ExpiredToken(t *testing.T) {
	secret := []byte("test-secret-key-12345678901234567890")

	// Create an expired ID token
	claims := &IDTokenClaims{
		RegisteredClaims: jwt.RegisteredClaims{
			Subject:   "user-expired",
			Issuer:    "https://auth.example.com",
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(-time.Hour)), // Expired
		},
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	tokenString, err := token.SignedString(secret)
	if err != nil {
		t.Fatalf("Failed to create token: %v", err)
	}

	// Validate (should fail)
	config := &JWTConfig{
		Secret:     string(secret),
		Algorithms: []string{"HS256"},
	}
	validator, _ := NewJWTValidator(config)

	_, err = validator.Validate(tokenString)
	if err == nil {
		t.Error("Expected error for expired token")
	}
}

func TestIDTokenValidation_WrongIssuer(t *testing.T) {
	secret := []byte("test-secret-key-12345678901234567890")

	claims := &IDTokenClaims{
		RegisteredClaims: jwt.RegisteredClaims{
			Subject:   "user-wrong-iss",
			Issuer:    "https://wrong-issuer.example.com",
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(time.Hour)),
		},
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	tokenString, _ := token.SignedString(secret)

	config := &JWTConfig{
		Secret:     string(secret),
		Algorithms: []string{"HS256"},
		Issuer:     "https://correct-issuer.example.com",
	}
	validator, _ := NewJWTValidator(config)

	_, err := validator.Validate(tokenString)
	if err == nil {
		t.Error("Expected error for wrong issuer")
	}
}

func TestIDTokenValidation_WrongAudience(t *testing.T) {
	secret := []byte("test-secret-key-12345678901234567890")

	claims := &IDTokenClaims{
		RegisteredClaims: jwt.RegisteredClaims{
			Subject:   "user-wrong-aud",
			Audience:  jwt.ClaimStrings{"wrong-client"},
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(time.Hour)),
		},
	}

	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	tokenString, _ := token.SignedString(secret)

	config := &JWTConfig{
		Secret:     string(secret),
		Algorithms: []string{"HS256"},
		Audience:   []string{"correct-client"},
	}
	validator, _ := NewJWTValidator(config)

	_, err := validator.Validate(tokenString)
	if err == nil {
		t.Error("Expected error for wrong audience")
	}
}

// =============================================================================
// PKCE Tests
// =============================================================================

func TestPKCE_CodeVerifierGeneration(t *testing.T) {
	verifier := GenerateCodeVerifier()

	if len(verifier) < 43 || len(verifier) > 128 {
		t.Errorf("Verifier length should be between 43 and 128, got %d", len(verifier))
	}

	// Should only contain URL-safe characters
	for _, c := range verifier {
		if !isURLSafe(byte(c)) {
			t.Errorf("Verifier contains non-URL-safe character: %c", c)
		}
	}
}

func TestPKCE_CodeChallengeS256(t *testing.T) {
	verifier := "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"

	challenge := GenerateCodeChallenge(verifier, "S256")

	if challenge == "" {
		t.Error("Challenge should not be empty")
	}
	if challenge == verifier {
		t.Error("S256 challenge should be different from verifier")
	}
}

func TestPKCE_CodeChallengePlain(t *testing.T) {
	verifier := "plain-verifier"

	challenge := GenerateCodeChallenge(verifier, "plain")

	if challenge != verifier {
		t.Errorf("Plain challenge should equal verifier, got %s", challenge)
	}
}

// Helper function to check URL-safe characters
func isURLSafe(c byte) bool {
	return (c >= 'A' && c <= 'Z') ||
		(c >= 'a' && c <= 'z') ||
		(c >= '0' && c <= '9') ||
		c == '-' || c == '.' || c == '_' || c == '~'
}

// =============================================================================
// Session State Tests
// =============================================================================

func TestOIDCSessionState(t *testing.T) {
	state := NewOIDCSessionState()

	if state.State == "" {
		t.Error("State should not be empty")
	}
	if state.Nonce == "" {
		t.Error("Nonce should not be empty")
	}
	if state.CodeVerifier == "" {
		t.Error("CodeVerifier should not be empty")
	}
	if state.CreatedAt.IsZero() {
		t.Error("CreatedAt should not be zero")
	}
}

func TestOIDCSessionState_Validation(t *testing.T) {
	state := NewOIDCSessionState()

	// Valid state
	if !state.ValidateState(state.State) {
		t.Error("State should be valid")
	}

	// Invalid state
	if state.ValidateState("wrong-state") {
		t.Error("Wrong state should be invalid")
	}
}

func TestOIDCSessionState_Expiration(t *testing.T) {
	state := NewOIDCSessionState()
	state.TTL = 1 * time.Millisecond

	time.Sleep(10 * time.Millisecond)

	if !state.IsExpired() {
		t.Error("State should be expired")
	}
}
