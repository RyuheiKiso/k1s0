// Package k1s0auth provides JWT/OIDC authentication for the k1s0 framework.
//
// This package implements:
//   - JWT token validation
//   - OIDC provider integration
//   - HTTP middleware for authentication
//   - gRPC interceptors for authentication
//   - Claims extraction and principal management
//
// # Configuration
//
// Example config.yaml:
//
//	auth:
//	  jwt:
//	    secret_file: /var/run/secrets/k1s0/jwt_secret
//	    issuer: https://auth.example.com
//	    audience:
//	      - myapp
//	    algorithms:
//	      - HS256
//	      - RS256
//	    public_key_file: /var/run/secrets/k1s0/jwt_public.pem
//	  oidc:
//	    issuer: https://accounts.google.com
//	    client_id: your-client-id
//	    client_secret_file: /var/run/secrets/k1s0/oidc_secret
//
// # Usage
//
//	// JWT validation
//	validator := k1s0auth.NewJWTValidator(authConfig)
//	claims, err := validator.Validate(tokenString)
//
//	// HTTP middleware
//	authMiddleware := k1s0auth.NewHTTPMiddleware(validator)
//	http.Handle("/api/", authMiddleware(apiHandler))
//
//	// gRPC interceptor
//	authInterceptor := k1s0auth.NewGRPCInterceptor(validator)
//	server := grpc.NewServer(grpc.UnaryInterceptor(authInterceptor))
package k1s0auth

import (
	"errors"
	"os"
	"strings"
	"time"
)

// AuthConfig holds authentication configuration.
type AuthConfig struct {
	// JWT is the JWT configuration.
	JWT JWTConfig

	// OIDC is the OIDC configuration.
	OIDC OIDCConfig

	// Cache enables token caching.
	Cache CacheConfig
}

// JWTConfig holds JWT configuration.
type JWTConfig struct {
	// Secret is the secret key for HMAC algorithms (direct value, not recommended).
	Secret string

	// SecretFile is the path to a file containing the secret (recommended).
	SecretFile string

	// PublicKeyFile is the path to the public key file for RSA/ECDSA algorithms.
	PublicKeyFile string

	// Issuer is the expected token issuer.
	Issuer string

	// Audience is the expected token audience.
	Audience []string

	// Algorithms is the list of allowed signing algorithms.
	// Default is ["HS256", "RS256"].
	Algorithms []string

	// SkipExpiryCheck skips token expiry validation (not recommended).
	SkipExpiryCheck bool

	// ClockSkew is the allowed clock skew for token validation.
	// Default is 1 minute.
	ClockSkew time.Duration
}

// OIDCConfig holds OIDC configuration.
type OIDCConfig struct {
	// Enabled enables OIDC authentication.
	Enabled bool

	// Issuer is the OIDC issuer URL (e.g., https://accounts.google.com).
	Issuer string

	// ClientID is the OIDC client ID.
	ClientID string

	// ClientSecret is the OIDC client secret (direct value, not recommended).
	ClientSecret string

	// ClientSecretFile is the path to a file containing the client secret (recommended).
	ClientSecretFile string

	// RedirectURL is the OAuth callback URL.
	RedirectURL string

	// Scopes is the list of OIDC scopes to request.
	// Default is ["openid", "profile", "email"].
	Scopes []string

	// SkipIssuerCheck skips issuer verification (not recommended).
	SkipIssuerCheck bool
}

// CacheConfig holds token cache configuration.
type CacheConfig struct {
	// Enabled enables token caching.
	Enabled bool

	// TTL is the cache TTL.
	// Default is 5 minutes.
	TTL time.Duration

	// MaxSize is the maximum number of cached tokens.
	// Default is 1000.
	MaxSize int
}

// DefaultAuthConfig returns an AuthConfig with default values.
func DefaultAuthConfig() *AuthConfig {
	return &AuthConfig{
		JWT: JWTConfig{
			Algorithms: []string{"HS256", "RS256"},
			ClockSkew:  time.Minute,
		},
		OIDC: OIDCConfig{
			Scopes: []string{"openid", "profile", "email"},
		},
		Cache: CacheConfig{
			TTL:     5 * time.Minute,
			MaxSize: 1000,
		},
	}
}

// Validate validates the authentication configuration.
func (c *AuthConfig) Validate() error {
	if err := c.JWT.Validate(); err != nil {
		return err
	}
	if c.OIDC.Enabled {
		if err := c.OIDC.Validate(); err != nil {
			return err
		}
	}
	c.Cache.Validate()
	return nil
}

// Validate validates the JWT configuration.
func (c *JWTConfig) Validate() error {
	if c.Secret == "" && c.SecretFile == "" && c.PublicKeyFile == "" {
		return errors.New("JWT secret, secret_file, or public_key_file is required")
	}
	if len(c.Algorithms) == 0 {
		c.Algorithms = []string{"HS256", "RS256"}
	}
	if c.ClockSkew <= 0 {
		c.ClockSkew = time.Minute
	}
	return nil
}

// GetSecret returns the secret, reading from file if necessary.
func (c *JWTConfig) GetSecret() (string, error) {
	if c.Secret != "" {
		return c.Secret, nil
	}
	if c.SecretFile != "" {
		data, err := os.ReadFile(c.SecretFile)
		if err != nil {
			return "", err
		}
		return strings.TrimSpace(string(data)), nil
	}
	return "", nil
}

// GetPublicKey returns the public key from file.
func (c *JWTConfig) GetPublicKey() ([]byte, error) {
	if c.PublicKeyFile == "" {
		return nil, nil
	}
	return os.ReadFile(c.PublicKeyFile)
}

// Validate validates the OIDC configuration.
func (c *OIDCConfig) Validate() error {
	if c.Issuer == "" {
		return errors.New("OIDC issuer is required")
	}
	if c.ClientID == "" {
		return errors.New("OIDC client_id is required")
	}
	if len(c.Scopes) == 0 {
		c.Scopes = []string{"openid", "profile", "email"}
	}
	return nil
}

// GetClientSecret returns the client secret, reading from file if necessary.
func (c *OIDCConfig) GetClientSecret() (string, error) {
	if c.ClientSecret != "" {
		return c.ClientSecret, nil
	}
	if c.ClientSecretFile != "" {
		data, err := os.ReadFile(c.ClientSecretFile)
		if err != nil {
			return "", err
		}
		return strings.TrimSpace(string(data)), nil
	}
	return "", nil
}

// Validate validates the cache configuration.
func (c *CacheConfig) Validate() *CacheConfig {
	if c.TTL <= 0 {
		c.TTL = 5 * time.Minute
	}
	if c.MaxSize <= 0 {
		c.MaxSize = 1000
	}
	return c
}

// AuthConfigBuilder builds an AuthConfig.
type AuthConfigBuilder struct {
	config *AuthConfig
}

// NewAuthConfigBuilder creates a new AuthConfigBuilder.
func NewAuthConfigBuilder() *AuthConfigBuilder {
	return &AuthConfigBuilder{
		config: DefaultAuthConfig(),
	}
}

// JWTSecret sets the JWT secret.
func (b *AuthConfigBuilder) JWTSecret(secret string) *AuthConfigBuilder {
	b.config.JWT.Secret = secret
	return b
}

// JWTSecretFile sets the JWT secret file.
func (b *AuthConfigBuilder) JWTSecretFile(path string) *AuthConfigBuilder {
	b.config.JWT.SecretFile = path
	return b
}

// JWTPublicKeyFile sets the JWT public key file.
func (b *AuthConfigBuilder) JWTPublicKeyFile(path string) *AuthConfigBuilder {
	b.config.JWT.PublicKeyFile = path
	return b
}

// JWTIssuer sets the JWT issuer.
func (b *AuthConfigBuilder) JWTIssuer(issuer string) *AuthConfigBuilder {
	b.config.JWT.Issuer = issuer
	return b
}

// JWTAudience sets the JWT audience.
func (b *AuthConfigBuilder) JWTAudience(audience ...string) *AuthConfigBuilder {
	b.config.JWT.Audience = audience
	return b
}

// JWTAlgorithms sets the JWT algorithms.
func (b *AuthConfigBuilder) JWTAlgorithms(algorithms ...string) *AuthConfigBuilder {
	b.config.JWT.Algorithms = algorithms
	return b
}

// OIDCEnabled enables OIDC.
func (b *AuthConfigBuilder) OIDCEnabled(enabled bool) *AuthConfigBuilder {
	b.config.OIDC.Enabled = enabled
	return b
}

// OIDCIssuer sets the OIDC issuer.
func (b *AuthConfigBuilder) OIDCIssuer(issuer string) *AuthConfigBuilder {
	b.config.OIDC.Issuer = issuer
	return b
}

// OIDCClientID sets the OIDC client ID.
func (b *AuthConfigBuilder) OIDCClientID(clientID string) *AuthConfigBuilder {
	b.config.OIDC.ClientID = clientID
	return b
}

// OIDCClientSecretFile sets the OIDC client secret file.
func (b *AuthConfigBuilder) OIDCClientSecretFile(path string) *AuthConfigBuilder {
	b.config.OIDC.ClientSecretFile = path
	return b
}

// CacheEnabled enables token caching.
func (b *AuthConfigBuilder) CacheEnabled(enabled bool) *AuthConfigBuilder {
	b.config.Cache.Enabled = enabled
	return b
}

// CacheTTL sets the cache TTL.
func (b *AuthConfigBuilder) CacheTTL(ttl time.Duration) *AuthConfigBuilder {
	b.config.Cache.TTL = ttl
	return b
}

// Build creates the AuthConfig.
func (b *AuthConfigBuilder) Build() (*AuthConfig, error) {
	if err := b.config.Validate(); err != nil {
		return nil, err
	}
	return b.config, nil
}
