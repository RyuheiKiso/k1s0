package k1s0auth

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"time"

	"github.com/coreos/go-oidc/v3/oidc"
	"golang.org/x/oauth2"
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
//
// Example:
//
//	userInfo, err := validator.UserInfo(ctx, accessToken)
//	if err != nil {
//	    log.Printf("Failed to fetch user info: %v", err)
//	    return err
//	}
//	log.Printf("User: %s (%s)", userInfo.Name, userInfo.Email)
func (v *OIDCValidator) UserInfo(ctx context.Context, accessToken string) (*OIDCUserInfo, error) {
	// Create a static token source from the access token
	tokenSource := oauth2.StaticTokenSource(&oauth2.Token{
		AccessToken: accessToken,
		TokenType:   "Bearer",
	})

	// Use go-oidc's UserInfo endpoint
	rawUserInfo, err := v.provider.UserInfo(ctx, tokenSource)
	if err != nil {
		return nil, fmt.Errorf("failed to fetch user info: %w", err)
	}

	// Parse standard claims
	var userInfo OIDCUserInfo
	if err := rawUserInfo.Claims(&userInfo); err != nil {
		return nil, fmt.Errorf("failed to parse user info claims: %w", err)
	}

	// Set Subject from the UserInfo directly (guaranteed by OIDC spec)
	userInfo.Subject = rawUserInfo.Subject

	return &userInfo, nil
}

// UserInfoWithClient fetches user info using a custom HTTP client.
// This is useful when you need to customize timeouts or TLS settings.
//
// Example:
//
//	httpClient := &http.Client{
//	    Timeout: 10 * time.Second,
//	}
//	userInfo, err := validator.UserInfoWithClient(ctx, httpClient, accessToken)
func (v *OIDCValidator) UserInfoWithClient(ctx context.Context, httpClient *http.Client, accessToken string) (*OIDCUserInfo, error) {
	// Get the userinfo endpoint from provider
	userInfoEndpoint := v.provider.UserInfoEndpoint()
	if userInfoEndpoint == "" {
		return nil, fmt.Errorf("userinfo endpoint not available from provider")
	}

	// Create request
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, userInfoEndpoint, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	req.Header.Set("Authorization", "Bearer "+accessToken)
	req.Header.Set("Accept", "application/json")

	// Execute request
	resp, err := httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to execute request: %w", err)
	}
	defer resp.Body.Close()

	// Check response status
	if resp.StatusCode == http.StatusUnauthorized {
		return nil, fmt.Errorf("access token is invalid or expired")
	}
	if resp.StatusCode >= 400 {
		return nil, fmt.Errorf("userinfo request failed with status %d", resp.StatusCode)
	}

	// Parse response
	var userInfo OIDCUserInfo
	if err := json.NewDecoder(resp.Body).Decode(&userInfo); err != nil {
		return nil, fmt.Errorf("failed to decode userinfo response: %w", err)
	}

	return &userInfo, nil
}

// OIDCUserInfo holds user info from the OIDC provider.
// This follows the OpenID Connect Core 1.0 specification.
type OIDCUserInfo struct {
	// Subject - Identifier for the End-User at the Issuer (required)
	Subject string `json:"sub"`

	// Name - End-User's full name
	Name string `json:"name,omitempty"`

	// GivenName - End-User's given name(s) or first name(s)
	GivenName string `json:"given_name,omitempty"`

	// FamilyName - End-User's surname(s) or last name(s)
	FamilyName string `json:"family_name,omitempty"`

	// MiddleName - End-User's middle name(s)
	MiddleName string `json:"middle_name,omitempty"`

	// Nickname - Casual name of the End-User
	Nickname string `json:"nickname,omitempty"`

	// PreferredUsername - Shorthand name by which the End-User wishes to be referred
	PreferredUsername string `json:"preferred_username,omitempty"`

	// Profile - URL of the End-User's profile page
	Profile string `json:"profile,omitempty"`

	// Picture - URL of the End-User's profile picture
	Picture string `json:"picture,omitempty"`

	// Website - URL of the End-User's Web page or blog
	Website string `json:"website,omitempty"`

	// Email - End-User's preferred e-mail address
	Email string `json:"email,omitempty"`

	// EmailVerified - True if the End-User's e-mail address has been verified
	EmailVerified bool `json:"email_verified,omitempty"`

	// Gender - End-User's gender
	Gender string `json:"gender,omitempty"`

	// Birthdate - End-User's birthday (YYYY-MM-DD format)
	Birthdate string `json:"birthdate,omitempty"`

	// Zoneinfo - End-User's time zone
	Zoneinfo string `json:"zoneinfo,omitempty"`

	// Locale - End-User's locale (e.g., "en-US" or "ja-JP")
	Locale string `json:"locale,omitempty"`

	// PhoneNumber - End-User's preferred telephone number
	PhoneNumber string `json:"phone_number,omitempty"`

	// PhoneNumberVerified - True if the End-User's phone number has been verified
	PhoneNumberVerified bool `json:"phone_number_verified,omitempty"`

	// Address - End-User's postal address
	Address *OIDCAddress `json:"address,omitempty"`

	// UpdatedAt - Time the End-User's information was last updated (Unix timestamp)
	UpdatedAt int64 `json:"updated_at,omitempty"`
}

// OIDCAddress represents the address claim in OIDC UserInfo.
type OIDCAddress struct {
	// Formatted - Full mailing address
	Formatted string `json:"formatted,omitempty"`

	// StreetAddress - Street address component
	StreetAddress string `json:"street_address,omitempty"`

	// Locality - City or locality component
	Locality string `json:"locality,omitempty"`

	// Region - State, province, prefecture, or region component
	Region string `json:"region,omitempty"`

	// PostalCode - Zip code or postal code component
	PostalCode string `json:"postal_code,omitempty"`

	// Country - Country name component
	Country string `json:"country,omitempty"`
}

// UserInfoClient is a standalone client for fetching OIDC UserInfo.
// Use this when you don't need full OIDC validation, only UserInfo retrieval.
type UserInfoClient struct {
	httpClient       *http.Client
	userInfoEndpoint string
}

// NewUserInfoClient creates a new UserInfo client.
//
// Example:
//
//	client := k1s0auth.NewUserInfoClient("https://auth.example.com/userinfo", nil)
//	userInfo, err := client.GetUserInfo(ctx, accessToken)
func NewUserInfoClient(userInfoEndpoint string, httpClient *http.Client) *UserInfoClient {
	if httpClient == nil {
		httpClient = &http.Client{
			Timeout: 10 * time.Second,
		}
	}

	return &UserInfoClient{
		httpClient:       httpClient,
		userInfoEndpoint: userInfoEndpoint,
	}
}

// GetUserInfo fetches user info from the OIDC provider.
func (c *UserInfoClient) GetUserInfo(ctx context.Context, accessToken string) (*OIDCUserInfo, error) {
	// Create request
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, c.userInfoEndpoint, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	req.Header.Set("Authorization", "Bearer "+accessToken)
	req.Header.Set("Accept", "application/json")

	// Execute request
	resp, err := c.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to execute request: %w", err)
	}
	defer resp.Body.Close()

	// Check response status
	if resp.StatusCode == http.StatusUnauthorized {
		return nil, fmt.Errorf("access token is invalid or expired")
	}
	if resp.StatusCode >= 400 {
		return nil, fmt.Errorf("userinfo request failed with status %d", resp.StatusCode)
	}

	// Parse response
	var userInfo OIDCUserInfo
	if err := json.NewDecoder(resp.Body).Decode(&userInfo); err != nil {
		return nil, fmt.Errorf("failed to decode userinfo response: %w", err)
	}

	return &userInfo, nil
}
