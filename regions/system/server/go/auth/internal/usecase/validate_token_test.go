package usecase

import (
	"context"
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"

	"github.com/k1s0-platform/system-server-go-auth/internal/domain/model"
)

// MockTokenVerifier は TokenVerifier のモック実装。
type MockTokenVerifier struct {
	mock.Mock
}

func (m *MockTokenVerifier) VerifyToken(ctx context.Context, tokenString string) (*model.TokenClaims, error) {
	args := m.Called(ctx, tokenString)
	if args.Get(0) == nil {
		return nil, args.Error(1)
	}
	return args.Get(0).(*model.TokenClaims), args.Error(1)
}

func TestValidateTokenUseCase_Execute_Success(t *testing.T) {
	mockVerifier := new(MockTokenVerifier)
	jwtConfig := JWTConfig{
		Issuer:   "https://auth.k1s0.internal.example.com/realms/k1s0",
		Audience: "k1s0-api",
	}
	uc := NewValidateTokenUseCase(mockVerifier, jwtConfig)

	expectedClaims := &model.TokenClaims{
		Sub:              "user-uuid-1234",
		Iss:              "https://auth.k1s0.internal.example.com/realms/k1s0",
		Aud:              "k1s0-api",
		Exp:              1710000900,
		Iat:              1710000000,
		Jti:              "token-uuid-5678",
		PreferredUsername: "taro.yamada",
		Email:            "taro.yamada@example.com",
		RealmAccess: model.RealmAccess{
			Roles: []string{"user", "order_manager"},
		},
	}

	mockVerifier.On("VerifyToken", mock.Anything, "valid-token").Return(expectedClaims, nil)

	claims, err := uc.Execute(context.Background(), "valid-token")

	assert.NoError(t, err)
	assert.NotNil(t, claims)
	assert.Equal(t, "user-uuid-1234", claims.Sub)
	assert.Equal(t, "taro.yamada", claims.PreferredUsername)
	mockVerifier.AssertExpectations(t)
}

func TestValidateTokenUseCase_Execute_EmptyToken(t *testing.T) {
	mockVerifier := new(MockTokenVerifier)
	jwtConfig := JWTConfig{
		Issuer:   "https://auth.k1s0.internal.example.com/realms/k1s0",
		Audience: "k1s0-api",
	}
	uc := NewValidateTokenUseCase(mockVerifier, jwtConfig)

	claims, err := uc.Execute(context.Background(), "")

	assert.ErrorIs(t, err, ErrInvalidToken)
	assert.Nil(t, claims)
}

func TestValidateTokenUseCase_Execute_VerifierError(t *testing.T) {
	mockVerifier := new(MockTokenVerifier)
	jwtConfig := JWTConfig{
		Issuer:   "https://auth.k1s0.internal.example.com/realms/k1s0",
		Audience: "k1s0-api",
	}
	uc := NewValidateTokenUseCase(mockVerifier, jwtConfig)

	mockVerifier.On("VerifyToken", mock.Anything, "bad-token").
		Return(nil, errors.New("signature verification failed"))

	claims, err := uc.Execute(context.Background(), "bad-token")

	assert.Error(t, err)
	assert.Nil(t, claims)
	mockVerifier.AssertExpectations(t)
}

func TestValidateTokenUseCase_Execute_InvalidIssuer(t *testing.T) {
	mockVerifier := new(MockTokenVerifier)
	jwtConfig := JWTConfig{
		Issuer:   "https://auth.k1s0.internal.example.com/realms/k1s0",
		Audience: "k1s0-api",
	}
	uc := NewValidateTokenUseCase(mockVerifier, jwtConfig)

	wrongIssuerClaims := &model.TokenClaims{
		Sub: "user-uuid-1234",
		Iss: "https://wrong-issuer.example.com/realms/other",
		Aud: "k1s0-api",
	}
	mockVerifier.On("VerifyToken", mock.Anything, "wrong-issuer-token").
		Return(wrongIssuerClaims, nil)

	claims, err := uc.Execute(context.Background(), "wrong-issuer-token")

	assert.ErrorIs(t, err, ErrInvalidIssuer)
	assert.Nil(t, claims)
	mockVerifier.AssertExpectations(t)
}

func TestValidateTokenUseCase_Execute_InvalidAudience(t *testing.T) {
	mockVerifier := new(MockTokenVerifier)
	jwtConfig := JWTConfig{
		Issuer:   "https://auth.k1s0.internal.example.com/realms/k1s0",
		Audience: "k1s0-api",
	}
	uc := NewValidateTokenUseCase(mockVerifier, jwtConfig)

	wrongAudClaims := &model.TokenClaims{
		Sub: "user-uuid-1234",
		Iss: "https://auth.k1s0.internal.example.com/realms/k1s0",
		Aud: "wrong-audience",
	}
	mockVerifier.On("VerifyToken", mock.Anything, "wrong-aud-token").
		Return(wrongAudClaims, nil)

	claims, err := uc.Execute(context.Background(), "wrong-aud-token")

	assert.ErrorIs(t, err, ErrInvalidAudience)
	assert.Nil(t, claims)
	mockVerifier.AssertExpectations(t)
}
