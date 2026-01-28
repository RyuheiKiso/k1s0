package k1s0config

import (
	"os"
	"path/filepath"
	"testing"
)

func TestIsValidEnv(t *testing.T) {
	validEnvs := []string{"default", "dev", "stg", "prod"}
	for _, env := range validEnvs {
		if !IsValidEnv(env) {
			t.Errorf("expected %s to be valid", env)
		}
	}

	invalidEnvs := []string{"invalid", "production", "development", ""}
	for _, env := range invalidEnvs {
		if IsValidEnv(env) {
			t.Errorf("expected %s to be invalid", env)
		}
	}
}

func TestConfigOptions(t *testing.T) {
	opts := NewConfigOptions("dev").
		WithConfigPath("/custom/path.yaml").
		WithSecretsDir("/custom/secrets")

	if opts.Env != "dev" {
		t.Errorf("expected dev, got %s", opts.Env)
	}
	if opts.EffectiveConfigPath() != "/custom/path.yaml" {
		t.Errorf("expected /custom/path.yaml, got %s", opts.EffectiveConfigPath())
	}
	if opts.SecretsDir != "/custom/secrets" {
		t.Errorf("expected /custom/secrets, got %s", opts.SecretsDir)
	}
}

func TestConfigOptionsDefaultPath(t *testing.T) {
	opts := NewConfigOptions("prod")
	expected := filepath.Join("config", "prod.yaml")
	if opts.EffectiveConfigPath() != expected {
		t.Errorf("expected %s, got %s", expected, opts.EffectiveConfigPath())
	}
}

func TestNewConfigLoaderInvalidEnv(t *testing.T) {
	opts := NewConfigOptions("invalid").
		WithRequireConfigFile(false)
	_, err := NewConfigLoader(opts)

	if err == nil {
		t.Fatal("expected error for invalid env")
	}

	configErr, ok := err.(*ConfigError)
	if !ok {
		t.Fatalf("expected ConfigError, got %T", err)
	}
	if configErr.Code != ErrCodeInvalidEnv {
		t.Errorf("expected %s, got %s", ErrCodeInvalidEnv, configErr.Code)
	}
}

func TestNewConfigLoaderConfigNotFound(t *testing.T) {
	opts := NewConfigOptions("dev").
		WithConfigPath("/nonexistent/config.yaml").
		WithRequireConfigFile(true)
	_, err := NewConfigLoader(opts)

	if err == nil {
		t.Fatal("expected error for missing config")
	}

	if !IsConfigNotFound(err) {
		t.Errorf("expected config not found error, got %v", err)
	}
}

func TestNewConfigLoaderConfigNotRequired(t *testing.T) {
	opts := NewConfigOptions("dev").
		WithConfigPath("/nonexistent/config.yaml").
		WithRequireConfigFile(false)
	loader, err := NewConfigLoader(opts)

	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if loader == nil {
		t.Fatal("expected loader to be created")
	}
}

func TestConfigLoaderLoad(t *testing.T) {
	// Create temp directory and config file
	dir := t.TempDir()
	configPath := filepath.Join(dir, "dev.yaml")
	content := `
server:
  host: localhost
  port: 8080
`
	if err := os.WriteFile(configPath, []byte(content), 0644); err != nil {
		t.Fatalf("failed to write config: %v", err)
	}

	opts := NewConfigOptions("dev").
		WithConfigPath(configPath)
	loader, err := NewConfigLoader(opts)
	if err != nil {
		t.Fatalf("failed to create loader: %v", err)
	}

	type ServerConfig struct {
		Host string `yaml:"host"`
		Port int    `yaml:"port"`
	}
	type Config struct {
		Server ServerConfig `yaml:"server"`
	}

	var config Config
	if err := loader.Load(&config); err != nil {
		t.Fatalf("failed to load config: %v", err)
	}

	if config.Server.Host != "localhost" {
		t.Errorf("expected localhost, got %s", config.Server.Host)
	}
	if config.Server.Port != 8080 {
		t.Errorf("expected 8080, got %d", config.Server.Port)
	}
}

func TestConfigLoaderLoadParseError(t *testing.T) {
	dir := t.TempDir()
	configPath := filepath.Join(dir, "invalid.yaml")
	content := `invalid: yaml: content:`
	if err := os.WriteFile(configPath, []byte(content), 0644); err != nil {
		t.Fatalf("failed to write config: %v", err)
	}

	opts := NewConfigOptions("dev").
		WithConfigPath(configPath)
	loader, err := NewConfigLoader(opts)
	if err != nil {
		t.Fatalf("failed to create loader: %v", err)
	}

	var config map[string]interface{}
	err = loader.Load(&config)

	if err == nil {
		t.Fatal("expected parse error")
	}

	configErr, ok := err.(*ConfigError)
	if !ok {
		t.Fatalf("expected ConfigError, got %T", err)
	}
	if configErr.Code != ErrCodeConfigParseError {
		t.Errorf("expected %s, got %s", ErrCodeConfigParseError, configErr.Code)
	}
}

func TestSecretResolverResolve(t *testing.T) {
	// Create temp directory and secret file
	dir := t.TempDir()
	secretsDir := filepath.Join(dir, "secrets")
	if err := os.MkdirAll(secretsDir, 0755); err != nil {
		t.Fatalf("failed to create secrets dir: %v", err)
	}

	secretPath := filepath.Join(secretsDir, "db_password")
	if err := os.WriteFile(secretPath, []byte("my_secret_password\n"), 0644); err != nil {
		t.Fatalf("failed to write secret: %v", err)
	}

	resolver := NewSecretResolver(secretsDir)
	password, err := resolver.Resolve("db_password", "db.password_file")

	if err != nil {
		t.Fatalf("failed to resolve secret: %v", err)
	}
	if password != "my_secret_password" {
		t.Errorf("expected 'my_secret_password', got '%s'", password)
	}
}

func TestSecretResolverResolveAbsolutePath(t *testing.T) {
	dir := t.TempDir()
	secretPath := filepath.Join(dir, "absolute_secret")
	if err := os.WriteFile(secretPath, []byte("absolute_value"), 0644); err != nil {
		t.Fatalf("failed to write secret: %v", err)
	}

	resolver := NewSecretResolver("/nonexistent")
	value, err := resolver.Resolve(secretPath, "key")

	if err != nil {
		t.Fatalf("failed to resolve secret: %v", err)
	}
	if value != "absolute_value" {
		t.Errorf("expected 'absolute_value', got '%s'", value)
	}
}

func TestSecretResolverResolveNotFound(t *testing.T) {
	dir := t.TempDir()
	resolver := NewSecretResolver(dir)
	_, err := resolver.Resolve("nonexistent", "db.password_file")

	if err == nil {
		t.Fatal("expected error for missing secret")
	}

	if !IsSecretNotFound(err) {
		t.Errorf("expected secret not found error, got %v", err)
	}

	configErr, ok := err.(*ConfigError)
	if !ok {
		t.Fatalf("expected ConfigError, got %T", err)
	}
	if configErr.Hint == "" {
		t.Error("expected hint to be set")
	}
}

func TestConfigLoaderResolveSecretFile(t *testing.T) {
	dir := t.TempDir()

	// Create config file
	configPath := filepath.Join(dir, "dev.yaml")
	content := `
db:
  host: localhost
  password_file: db_password
`
	if err := os.WriteFile(configPath, []byte(content), 0644); err != nil {
		t.Fatalf("failed to write config: %v", err)
	}

	// Create secrets directory and file
	secretsDir := filepath.Join(dir, "secrets")
	if err := os.MkdirAll(secretsDir, 0755); err != nil {
		t.Fatalf("failed to create secrets dir: %v", err)
	}
	if err := os.WriteFile(filepath.Join(secretsDir, "db_password"), []byte("secret123\n"), 0644); err != nil {
		t.Fatalf("failed to write secret: %v", err)
	}

	opts := NewConfigOptions("dev").
		WithConfigPath(configPath).
		WithSecretsDir(secretsDir)
	loader, err := NewConfigLoader(opts)
	if err != nil {
		t.Fatalf("failed to create loader: %v", err)
	}

	type DBConfig struct {
		Host         string `yaml:"host"`
		PasswordFile string `yaml:"password_file"`
	}
	type Config struct {
		DB DBConfig `yaml:"db"`
	}

	var config Config
	if err := loader.Load(&config); err != nil {
		t.Fatalf("failed to load config: %v", err)
	}

	password, err := loader.ResolveSecretFile(config.DB.PasswordFile, "db.password_file")
	if err != nil {
		t.Fatalf("failed to resolve secret: %v", err)
	}
	if password != "secret123" {
		t.Errorf("expected 'secret123', got '%s'", password)
	}
}

func TestServiceArgsValidate(t *testing.T) {
	// Missing env
	args := NewServiceArgs()
	err := args.Validate()
	if err == nil {
		t.Error("expected error for missing env")
	}

	// Invalid env
	args.Env = "invalid"
	err = args.Validate()
	if err == nil {
		t.Error("expected error for invalid env")
	}

	// Valid env
	args.Env = "dev"
	err = args.Validate()
	if err != nil {
		t.Errorf("unexpected error: %v", err)
	}
}

func TestServiceInit(t *testing.T) {
	dir := t.TempDir()
	configPath := filepath.Join(dir, "dev.yaml")
	content := `
app:
  name: test-service
`
	if err := os.WriteFile(configPath, []byte(content), 0644); err != nil {
		t.Fatalf("failed to write config: %v", err)
	}

	args := &ServiceArgs{
		Env:        "dev",
		ConfigPath: configPath,
	}

	type Config struct {
		App struct {
			Name string `yaml:"name"`
		} `yaml:"app"`
	}

	var config Config
	init, err := Init(args, &config)
	if err != nil {
		t.Fatalf("failed to init: %v", err)
	}

	if config.App.Name != "test-service" {
		t.Errorf("expected 'test-service', got '%s'", config.App.Name)
	}
	if init.Env() != "dev" {
		t.Errorf("expected 'dev', got '%s'", init.Env())
	}
}

func TestLoadFromFile(t *testing.T) {
	dir := t.TempDir()
	configPath := filepath.Join(dir, "config.yaml")
	content := `
server:
  host: 127.0.0.1
  port: 3000
`
	if err := os.WriteFile(configPath, []byte(content), 0644); err != nil {
		t.Fatalf("failed to write config: %v", err)
	}

	type Config struct {
		Server struct {
			Host string `yaml:"host"`
			Port int    `yaml:"port"`
		} `yaml:"server"`
	}

	var config Config
	if err := LoadFromFile(configPath, &config); err != nil {
		t.Fatalf("failed to load: %v", err)
	}

	if config.Server.Host != "127.0.0.1" {
		t.Errorf("expected '127.0.0.1', got '%s'", config.Server.Host)
	}
	if config.Server.Port != 3000 {
		t.Errorf("expected 3000, got %d", config.Server.Port)
	}
}

func TestLoadFromFileNotFound(t *testing.T) {
	var config map[string]interface{}
	err := LoadFromFile("/nonexistent/config.yaml", &config)

	if err == nil {
		t.Fatal("expected error for missing file")
	}
	if !IsConfigNotFound(err) {
		t.Errorf("expected config not found error, got %v", err)
	}
}

func TestConfigLoaderEnvAndPath(t *testing.T) {
	dir := t.TempDir()
	configPath := filepath.Join(dir, "prod.yaml")
	if err := os.WriteFile(configPath, []byte("dummy: value"), 0644); err != nil {
		t.Fatalf("failed to write config: %v", err)
	}

	opts := NewConfigOptions("prod").
		WithConfigPath(configPath)
	loader, err := NewConfigLoader(opts)
	if err != nil {
		t.Fatalf("failed to create loader: %v", err)
	}

	if loader.Env() != "prod" {
		t.Errorf("expected 'prod', got '%s'", loader.Env())
	}
	if loader.ConfigPath() != configPath {
		t.Errorf("expected '%s', got '%s'", configPath, loader.ConfigPath())
	}
}

func TestConfigLoaderLoadRaw(t *testing.T) {
	dir := t.TempDir()
	configPath := filepath.Join(dir, "dev.yaml")
	content := "server:\n  host: localhost\n"
	if err := os.WriteFile(configPath, []byte(content), 0644); err != nil {
		t.Fatalf("failed to write config: %v", err)
	}

	opts := NewConfigOptions("dev").
		WithConfigPath(configPath)
	loader, err := NewConfigLoader(opts)
	if err != nil {
		t.Fatalf("failed to create loader: %v", err)
	}

	raw, err := loader.LoadRaw()
	if err != nil {
		t.Fatalf("failed to load raw: %v", err)
	}
	if string(raw) != content {
		t.Errorf("expected '%s', got '%s'", content, string(raw))
	}

	// Test caching
	raw2, err := loader.LoadRaw()
	if err != nil {
		t.Fatalf("failed to load raw again: %v", err)
	}
	if string(raw2) != content {
		t.Errorf("expected '%s', got '%s'", content, string(raw2))
	}
}
