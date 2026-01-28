package k1s0auth

import (
	"crypto/ecdsa"
	"crypto/rsa"
	"crypto/x509"
	"encoding/pem"
	"errors"
	"fmt"
	"strings"
	"sync"
	"time"

	"github.com/golang-jwt/jwt/v5"
)

// Common authentication errors.
var (
	ErrNotAuthenticated = errors.New("not authenticated")
	ErrTokenExpired     = errors.New("token expired")
	ErrTokenInvalid     = errors.New("token invalid")
	ErrTokenMalformed   = errors.New("token malformed")
	ErrInvalidIssuer    = errors.New("invalid issuer")
	ErrInvalidAudience  = errors.New("invalid audience")
	ErrInvalidAlgorithm = errors.New("invalid algorithm")
	ErrInvalidSignature = errors.New("invalid signature")
)

// Validator defines the interface for token validation.
type Validator interface {
	// Validate validates a token and returns the claims.
	Validate(token string) (*Claims, error)

	// ValidateWithAudience validates a token with a specific audience.
	ValidateWithAudience(token string, audience string) (*Claims, error)
}

// JWTValidator validates JWT tokens.
type JWTValidator struct {
	config         *JWTConfig
	secret         []byte
	publicKey      interface{}
	allowedMethods map[string]bool
	cache          map[string]*cachedToken
	cacheMu        sync.RWMutex
}

// cachedToken holds a cached token validation result.
type cachedToken struct {
	claims    *Claims
	expiresAt time.Time
}

// NewJWTValidator creates a new JWT validator.
//
// Example:
//
//	config := &k1s0auth.JWTConfig{
//	    Secret: "your-secret-key",
//	    Issuer: "https://auth.example.com",
//	    Audience: []string{"myapp"},
//	}
//
//	validator := k1s0auth.NewJWTValidator(config)
//	claims, err := validator.Validate(tokenString)
func NewJWTValidator(config *JWTConfig) (*JWTValidator, error) {
	if err := config.Validate(); err != nil {
		return nil, err
	}

	v := &JWTValidator{
		config:         config,
		allowedMethods: make(map[string]bool),
		cache:          make(map[string]*cachedToken),
	}

	// Set allowed algorithms
	for _, alg := range config.Algorithms {
		v.allowedMethods[alg] = true
	}

	// Load secret or public key
	secret, err := config.GetSecret()
	if err != nil {
		return nil, fmt.Errorf("failed to get secret: %w", err)
	}
	if secret != "" {
		v.secret = []byte(secret)
	}

	keyData, err := config.GetPublicKey()
	if err != nil {
		return nil, fmt.Errorf("failed to get public key: %w", err)
	}
	if keyData != nil {
		key, err := parsePublicKey(keyData)
		if err != nil {
			return nil, fmt.Errorf("failed to parse public key: %w", err)
		}
		v.publicKey = key
	}

	return v, nil
}

// parsePublicKey parses a PEM-encoded public key.
func parsePublicKey(data []byte) (interface{}, error) {
	block, _ := pem.Decode(data)
	if block == nil {
		return nil, errors.New("failed to decode PEM block")
	}

	switch block.Type {
	case "PUBLIC KEY":
		return x509.ParsePKIXPublicKey(block.Bytes)
	case "RSA PUBLIC KEY":
		return x509.ParsePKCS1PublicKey(block.Bytes)
	case "EC PUBLIC KEY":
		return x509.ParsePKIXPublicKey(block.Bytes)
	default:
		return nil, fmt.Errorf("unsupported key type: %s", block.Type)
	}
}

// Validate validates a JWT token and returns the claims.
func (v *JWTValidator) Validate(tokenString string) (*Claims, error) {
	return v.ValidateWithAudience(tokenString, "")
}

// ValidateWithAudience validates a JWT token with a specific audience.
func (v *JWTValidator) ValidateWithAudience(tokenString string, audience string) (*Claims, error) {
	// Check cache
	v.cacheMu.RLock()
	if cached, ok := v.cache[tokenString]; ok && time.Now().Before(cached.expiresAt) {
		v.cacheMu.RUnlock()
		return cached.claims, nil
	}
	v.cacheMu.RUnlock()

	// Parse token
	token, err := jwt.ParseWithClaims(tokenString, &Claims{}, v.keyFunc)
	if err != nil {
		return nil, v.wrapError(err)
	}

	claims, ok := token.Claims.(*Claims)
	if !ok || !token.Valid {
		return nil, ErrTokenInvalid
	}

	// Validate issuer
	if v.config.Issuer != "" {
		issuer, _ := claims.GetIssuer()
		if issuer != v.config.Issuer {
			return nil, ErrInvalidIssuer
		}
	}

	// Validate audience
	if len(v.config.Audience) > 0 || audience != "" {
		aud, _ := claims.GetAudience()
		if !v.validateAudience(aud, audience) {
			return nil, ErrInvalidAudience
		}
	}

	// Validate expiry
	if !v.config.SkipExpiryCheck {
		exp, _ := claims.GetExpirationTime()
		if exp != nil {
			if time.Now().Add(-v.config.ClockSkew).After(exp.Time) {
				return nil, ErrTokenExpired
			}
		}
	}

	// Cache the result
	if exp, _ := claims.GetExpirationTime(); exp != nil {
		v.cacheMu.Lock()
		v.cache[tokenString] = &cachedToken{
			claims:    claims,
			expiresAt: exp.Time,
		}
		v.cacheMu.Unlock()
	}

	return claims, nil
}

// keyFunc returns the key for validating the token.
func (v *JWTValidator) keyFunc(token *jwt.Token) (interface{}, error) {
	// Check algorithm
	alg := token.Method.Alg()
	if !v.allowedMethods[alg] {
		return nil, ErrInvalidAlgorithm
	}

	// Return appropriate key based on algorithm
	switch alg {
	case "HS256", "HS384", "HS512":
		return v.secret, nil
	case "RS256", "RS384", "RS512":
		if v.publicKey == nil {
			return nil, errors.New("public key required for RSA algorithms")
		}
		if _, ok := v.publicKey.(*rsa.PublicKey); !ok {
			return nil, errors.New("invalid key type for RSA algorithm")
		}
		return v.publicKey, nil
	case "ES256", "ES384", "ES512":
		if v.publicKey == nil {
			return nil, errors.New("public key required for ECDSA algorithms")
		}
		if _, ok := v.publicKey.(*ecdsa.PublicKey); !ok {
			return nil, errors.New("invalid key type for ECDSA algorithm")
		}
		return v.publicKey, nil
	default:
		return nil, ErrInvalidAlgorithm
	}
}

// validateAudience validates the audience claim.
func (v *JWTValidator) validateAudience(tokenAudience []string, requiredAudience string) bool {
	// Check required audience first
	if requiredAudience != "" {
		for _, aud := range tokenAudience {
			if aud == requiredAudience {
				return true
			}
		}
		return false
	}

	// Check configured audiences
	for _, configAud := range v.config.Audience {
		for _, tokenAud := range tokenAudience {
			if configAud == tokenAud {
				return true
			}
		}
	}

	return false
}

// wrapError wraps JWT library errors into our error types.
func (v *JWTValidator) wrapError(err error) error {
	switch {
	case errors.Is(err, jwt.ErrTokenMalformed):
		return ErrTokenMalformed
	case errors.Is(err, jwt.ErrTokenExpired):
		return ErrTokenExpired
	case errors.Is(err, jwt.ErrTokenSignatureInvalid):
		return ErrInvalidSignature
	case errors.Is(err, jwt.ErrTokenNotValidYet):
		return ErrTokenInvalid
	default:
		return fmt.Errorf("%w: %v", ErrTokenInvalid, err)
	}
}

// ClearCache clears the token cache.
func (v *JWTValidator) ClearCache() {
	v.cacheMu.Lock()
	v.cache = make(map[string]*cachedToken)
	v.cacheMu.Unlock()
}

// ExtractBearerToken extracts the bearer token from an Authorization header.
func ExtractBearerToken(authHeader string) (string, error) {
	if authHeader == "" {
		return "", ErrNotAuthenticated
	}

	parts := strings.SplitN(authHeader, " ", 2)
	if len(parts) != 2 || !strings.EqualFold(parts[0], "Bearer") {
		return "", ErrTokenMalformed
	}

	return parts[1], nil
}

// CreateToken creates a new JWT token (for testing/development).
func CreateToken(claims *Claims, secret []byte, method jwt.SigningMethod) (string, error) {
	token := jwt.NewWithClaims(method, claims)
	return token.SignedString(secret)
}

// CreateHS256Token creates a new HS256 JWT token.
func CreateHS256Token(claims *Claims, secret []byte) (string, error) {
	return CreateToken(claims, secret, jwt.SigningMethodHS256)
}
