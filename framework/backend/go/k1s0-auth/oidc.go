package k1s0auth

import (
	"context"
	"fmt"

	"github.com/coreos/go-oidc/v3/oidc"
)

// OIDCValidator validates OIDC tokens.
type OIDCValidator struct {
	config   *OIDCConfig
	provider *oidc.Provider
	verifier *oidc.IDTokenVerifier
}

// NewOIDCValidator creates a new OIDC validator.
//
// Example:
//
//	config := &k1s0auth.OIDCConfig{
//	    Enabled:  true,
//	    Issuer:   "https://accounts.google.com",
//	    ClientID: "your-client-id",
//	}
//
//	validator, err := k1s0auth.NewOIDCValidator(ctx, config)
//	if err != nil {
//	    log.Fatal(err)
//	}
//
//	claims, err := validator.Validate(ctx, idToken)
func NewOIDCValidator(ctx context.Context, config *OIDCConfig) (*OIDCValidator, error) {
	if err := config.Validate(); err != nil {
		return nil, err
	}

	provider, err := oidc.NewProvider(ctx, config.Issuer)
	if err != nil {
		return nil, fmt.Errorf("failed to create OIDC provider: %w", err)
	}

	verifierConfig := &oidc.Config{
		ClientID:          config.ClientID,
		SkipIssuerCheck:   config.SkipIssuerCheck,
		SkipClientIDCheck: false,
	}

	verifier := provider.Verifier(verifierConfig)

	return &OIDCValidator{
		config:   config,
		provider: provider,
		verifier: verifier,
	}, nil
}

// Validate validates an OIDC ID token.
func (v *OIDCValidator) Validate(ctx context.Context, rawIDToken string) (*Claims, error) {
	idToken, err := v.verifier.Verify(ctx, rawIDToken)
	if err != nil {
		return nil, fmt.Errorf("failed to verify ID token: %w", err)
	}

	var claims Claims
	if err := idToken.Claims(&claims); err != nil {
		return nil, fmt.Errorf("failed to extract claims: %w", err)
	}

	return &claims, nil
}

// Provider returns the OIDC provider.
func (v *OIDCValidator) Provider() *oidc.Provider {
	return v.provider
}

// Endpoint returns the OIDC endpoint.
func (v *OIDCValidator) Endpoint() OIDCEndpoint {
	endpoint := v.provider.Endpoint()
	return OIDCEndpoint{
		AuthURL:  endpoint.AuthURL,
		TokenURL: endpoint.TokenURL,
	}
}

// OIDCEndpoint holds OIDC endpoint URLs.
type OIDCEndpoint struct {
	AuthURL  string
	TokenURL string
}

// UserInfo fetches user info from the OIDC provider.
func (v *OIDCValidator) UserInfo(ctx context.Context, accessToken string) (*OIDCUserInfo, error) {
	// Note: This requires implementing token source
	// For now, return an error indicating not implemented
	return nil, fmt.Errorf("UserInfo is not yet implemented")
}

// OIDCUserInfo holds user info from the OIDC provider.
type OIDCUserInfo struct {
	Subject       string `json:"sub"`
	Profile       string `json:"profile,omitempty"`
	Email         string `json:"email,omitempty"`
	EmailVerified bool   `json:"email_verified,omitempty"`
	Picture       string `json:"picture,omitempty"`
	Name          string `json:"name,omitempty"`
	GivenName     string `json:"given_name,omitempty"`
	FamilyName    string `json:"family_name,omitempty"`
	Locale        string `json:"locale,omitempty"`
}
