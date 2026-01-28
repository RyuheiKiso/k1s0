package k1s0config

import (
	"os"
	"path/filepath"
	"strings"
)

// SecretResolver resolves secret values from files.
type SecretResolver struct {
	secretsDir string
}

// NewSecretResolver creates a new SecretResolver.
func NewSecretResolver(secretsDir string) *SecretResolver {
	return &SecretResolver{
		secretsDir: secretsDir,
	}
}

// Resolve reads a secret from a file.
//
// The fileValue is the value from a *_file key in the config.
// If fileValue is an absolute path, it reads from that path.
// If fileValue is a relative path, it reads from secretsDir/fileValue.
//
// The key parameter is used for error messages to indicate which
// configuration key referenced the secret.
func (r *SecretResolver) Resolve(fileValue, key string) (string, error) {
	var secretPath string
	if filepath.IsAbs(fileValue) {
		secretPath = fileValue
	} else {
		secretPath = filepath.Join(r.secretsDir, fileValue)
	}

	if _, err := os.Stat(secretPath); os.IsNotExist(err) {
		return "", NewSecretNotFoundError(secretPath, key)
	}

	content, err := os.ReadFile(secretPath)
	if err != nil {
		return "", NewSecretReadError(secretPath, key, err)
	}

	// Trim trailing newlines (common in Kubernetes secrets)
	return strings.TrimRight(string(content), "\n\r"), nil
}

// SecretsDir returns the configured secrets directory.
func (r *SecretResolver) SecretsDir() string {
	return r.secretsDir
}
