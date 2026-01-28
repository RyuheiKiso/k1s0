package k1s0auth

import (
	"context"
	"time"

	"github.com/golang-jwt/jwt/v5"
)

// Claims represents the JWT claims.
type Claims struct {
	jwt.RegisteredClaims

	// Standard claims
	Name          string   `json:"name,omitempty"`
	GivenName     string   `json:"given_name,omitempty"`
	FamilyName    string   `json:"family_name,omitempty"`
	Email         string   `json:"email,omitempty"`
	EmailVerified bool     `json:"email_verified,omitempty"`
	Picture       string   `json:"picture,omitempty"`
	Locale        string   `json:"locale,omitempty"`
	PhoneNumber   string   `json:"phone_number,omitempty"`
	Groups        []string `json:"groups,omitempty"`
	Roles         []string `json:"roles,omitempty"`
	Permissions   []string `json:"permissions,omitempty"`

	// Custom claims
	TenantID string                 `json:"tenant_id,omitempty"`
	Metadata map[string]interface{} `json:"metadata,omitempty"`
}

// Subject returns the subject claim (user ID).
func (c *Claims) Subject() string {
	return c.RegisteredClaims.Subject
}

// Issuer returns the issuer claim.
func (c *Claims) IssuerName() string {
	return c.RegisteredClaims.Issuer
}

// Audience returns the audience claims.
func (c *Claims) AudienceList() []string {
	return c.RegisteredClaims.Audience
}

// ExpiresAt returns the expiration time.
func (c *Claims) ExpirationTime() *time.Time {
	if c.RegisteredClaims.ExpiresAt == nil {
		return nil
	}
	t := c.RegisteredClaims.ExpiresAt.Time
	return &t
}

// IssuedAt returns the issued at time.
func (c *Claims) IssuedTime() *time.Time {
	if c.RegisteredClaims.IssuedAt == nil {
		return nil
	}
	t := c.RegisteredClaims.IssuedAt.Time
	return &t
}

// HasRole checks if the claims include a specific role.
func (c *Claims) HasRole(role string) bool {
	for _, r := range c.Roles {
		if r == role {
			return true
		}
	}
	return false
}

// HasAnyRole checks if the claims include any of the specified roles.
func (c *Claims) HasAnyRole(roles ...string) bool {
	for _, role := range roles {
		if c.HasRole(role) {
			return true
		}
	}
	return false
}

// HasAllRoles checks if the claims include all of the specified roles.
func (c *Claims) HasAllRoles(roles ...string) bool {
	for _, role := range roles {
		if !c.HasRole(role) {
			return false
		}
	}
	return true
}

// HasPermission checks if the claims include a specific permission.
func (c *Claims) HasPermission(permission string) bool {
	for _, p := range c.Permissions {
		if p == permission {
			return true
		}
	}
	return false
}

// HasAnyPermission checks if the claims include any of the specified permissions.
func (c *Claims) HasAnyPermission(permissions ...string) bool {
	for _, perm := range permissions {
		if c.HasPermission(perm) {
			return true
		}
	}
	return false
}

// HasAllPermissions checks if the claims include all of the specified permissions.
func (c *Claims) HasAllPermissions(permissions ...string) bool {
	for _, perm := range permissions {
		if !c.HasPermission(perm) {
			return false
		}
	}
	return true
}

// InGroup checks if the claims include a specific group.
func (c *Claims) InGroup(group string) bool {
	for _, g := range c.Groups {
		if g == group {
			return true
		}
	}
	return false
}

// Principal represents an authenticated principal (user).
type Principal struct {
	// ID is the user ID (subject claim).
	ID string

	// Email is the user's email.
	Email string

	// Name is the user's display name.
	Name string

	// Roles is the list of roles.
	Roles []string

	// Permissions is the list of permissions.
	Permissions []string

	// Groups is the list of groups.
	Groups []string

	// TenantID is the tenant ID.
	TenantID string

	// Claims is the full claims object.
	Claims *Claims
}

// NewPrincipal creates a Principal from Claims.
func NewPrincipal(claims *Claims) *Principal {
	return &Principal{
		ID:          claims.Subject(),
		Email:       claims.Email,
		Name:        claims.Name,
		Roles:       claims.Roles,
		Permissions: claims.Permissions,
		Groups:      claims.Groups,
		TenantID:    claims.TenantID,
		Claims:      claims,
	}
}

// HasRole checks if the principal has a specific role.
func (p *Principal) HasRole(role string) bool {
	return p.Claims.HasRole(role)
}

// HasPermission checks if the principal has a specific permission.
func (p *Principal) HasPermission(permission string) bool {
	return p.Claims.HasPermission(permission)
}

// InGroup checks if the principal is in a specific group.
func (p *Principal) InGroup(group string) bool {
	return p.Claims.InGroup(group)
}

// principalContextKey is the context key for principal.
type principalContextKey struct{}

// ContextWithPrincipal returns a new context with the principal.
func ContextWithPrincipal(ctx context.Context, principal *Principal) context.Context {
	return context.WithValue(ctx, principalContextKey{}, principal)
}

// PrincipalFromContext returns the principal from the context.
// Returns nil if no principal is found.
func PrincipalFromContext(ctx context.Context) *Principal {
	principal, _ := ctx.Value(principalContextKey{}).(*Principal)
	return principal
}

// RequirePrincipal returns the principal from the context or an error if not found.
func RequirePrincipal(ctx context.Context) (*Principal, error) {
	principal := PrincipalFromContext(ctx)
	if principal == nil {
		return nil, ErrNotAuthenticated
	}
	return principal, nil
}

// IsPrincipalAuthenticated checks if the context has an authenticated principal.
func IsPrincipalAuthenticated(ctx context.Context) bool {
	return PrincipalFromContext(ctx) != nil
}

// tokenContextKey is the context key for the raw token.
type tokenContextKey struct{}

// ContextWithToken returns a new context with the raw token.
func ContextWithToken(ctx context.Context, token string) context.Context {
	return context.WithValue(ctx, tokenContextKey{}, token)
}

// TokenFromContext returns the raw token from the context.
func TokenFromContext(ctx context.Context) string {
	token, _ := ctx.Value(tokenContextKey{}).(string)
	return token
}
