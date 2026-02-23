package webhookclient_test

import (
	"testing"

	"github.com/k1s0-platform/system-library-go-webhook-client"
	"github.com/stretchr/testify/assert"
)

func TestGenerateSignature_Deterministic(t *testing.T) {
	secret := "my-secret"
	body := []byte(`{"event_type":"test"}`)
	sig1 := webhookclient.GenerateSignature(secret, body)
	sig2 := webhookclient.GenerateSignature(secret, body)
	assert.Equal(t, sig1, sig2)
	assert.NotEmpty(t, sig1)
}

func TestVerifySignature_Valid(t *testing.T) {
	secret := "my-secret"
	body := []byte(`{"event_type":"test"}`)
	sig := webhookclient.GenerateSignature(secret, body)
	assert.True(t, webhookclient.VerifySignature(secret, body, sig))
}

func TestVerifySignature_WrongSecret(t *testing.T) {
	body := []byte(`{"event_type":"test"}`)
	sig := webhookclient.GenerateSignature("secret1", body)
	assert.False(t, webhookclient.VerifySignature("secret2", body, sig))
}

func TestVerifySignature_TamperedBody(t *testing.T) {
	secret := "my-secret"
	body := []byte(`{"event_type":"test"}`)
	sig := webhookclient.GenerateSignature(secret, body)
	tampered := []byte(`{"event_type":"hacked"}`)
	assert.False(t, webhookclient.VerifySignature(secret, tampered, sig))
}

func TestVerifySignature_EmptyBody(t *testing.T) {
	secret := "my-secret"
	body := []byte{}
	sig := webhookclient.GenerateSignature(secret, body)
	assert.True(t, webhookclient.VerifySignature(secret, body, sig))
}

func TestGenerateSignature_DifferentBodies(t *testing.T) {
	secret := "my-secret"
	sig1 := webhookclient.GenerateSignature(secret, []byte("body1"))
	sig2 := webhookclient.GenerateSignature(secret, []byte("body2"))
	assert.NotEqual(t, sig1, sig2)
}
