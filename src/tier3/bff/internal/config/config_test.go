// 本ファイルは BFF config ローダの単体テスト。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/40_tier3レイアウト/04_bff配置.md
//
// テスト観点:
//   - 必須項目（K1S0_TENANT_ID / K1S0_TARGET / appName）の欠落で error
//   - 既定値が docs と一致（HTTP_ADDR=":8080"、SERVICE_VERSION="0.0.0-dev" 等）
//   - bool / int env のパースが正しく fallback する

package config

import (
	"strings"
	"testing"
)

// withEnv は test 内で環境変数を一時的にセットし、defer で戻す helper。
// 並列テスト未対応（subtest を t.Parallel() しない前提）。
func withEnv(t *testing.T, kv map[string]string) {
	t.Helper()
	for k, v := range kv {
		t.Setenv(k, v)
	}
}

func TestLoad_RequiresTenantID(t *testing.T) {
	withEnv(t, map[string]string{
		"K1S0_TENANT_ID": "",
		"K1S0_TARGET":    "tier1:50001",
	})
	_, err := Load("portal-bff")
	if err == nil || !strings.Contains(err.Error(), "K1S0_TENANT_ID") {
		t.Fatalf("missing K1S0_TENANT_ID should error, got: %v", err)
	}
}

func TestLoad_RequiresAppName(t *testing.T) {
	withEnv(t, map[string]string{
		"K1S0_TENANT_ID": "T1",
		"K1S0_TARGET":    "tier1:50001",
	})
	_, err := Load("")
	if err == nil || !strings.Contains(err.Error(), "appName") {
		t.Fatalf("empty appName should error, got: %v", err)
	}
}

func TestLoad_DefaultsApplied(t *testing.T) {
	// 必須のみ set、他は env 未設定で default 確認。
	withEnv(t, map[string]string{
		"K1S0_TENANT_ID":               "T1",
		"K1S0_TARGET":                  "tier1.k1s0:50001",
		"SERVICE_VERSION":              "",
		"ENVIRONMENT":                  "",
		"OTEL_EXPORTER_OTLP_ENDPOINT":  "",
		"HTTP_ADDR":                    "",
		"HTTP_READ_TIMEOUT_SEC":        "",
		"HTTP_WRITE_TIMEOUT_SEC":       "",
		"K1S0_USE_TLS":                 "",
	})
	cfg, err := Load("portal-bff")
	if err != nil {
		t.Fatalf("Load: %v", err)
	}
	if cfg.AppName != "portal-bff" {
		t.Errorf("AppName = %q", cfg.AppName)
	}
	if cfg.ServiceVersion != "0.0.0-dev" {
		t.Errorf("ServiceVersion default mismatch: %q", cfg.ServiceVersion)
	}
	if cfg.Environment != "dev" {
		t.Errorf("Environment default mismatch: %q", cfg.Environment)
	}
	if cfg.HTTP.Addr != ":8080" {
		t.Errorf("HTTP.Addr default mismatch: %q", cfg.HTTP.Addr)
	}
	if cfg.HTTP.ReadTimeoutSec != 15 {
		t.Errorf("HTTP.ReadTimeoutSec default mismatch: %d", cfg.HTTP.ReadTimeoutSec)
	}
	if cfg.HTTP.WriteTimeoutSec != 15 {
		t.Errorf("HTTP.WriteTimeoutSec default mismatch: %d", cfg.HTTP.WriteTimeoutSec)
	}
	if cfg.K1s0.UseTLS {
		t.Errorf("K1s0.UseTLS default should be false")
	}
	// Subject default is "tier3/" + appName。
	if cfg.K1s0.Subject != "tier3/portal-bff" {
		t.Errorf("K1s0.Subject default mismatch: %q", cfg.K1s0.Subject)
	}
}

func TestLoad_OverridesFromEnv(t *testing.T) {
	withEnv(t, map[string]string{
		"K1S0_TENANT_ID":               "T-PROD",
		"K1S0_TARGET":                  "tier1.prod:50001",
		"SERVICE_VERSION":              "1.2.3",
		"ENVIRONMENT":                  "prod",
		"OTEL_EXPORTER_OTLP_ENDPOINT":  "otel:4317",
		"HTTP_ADDR":                    ":9000",
		"HTTP_READ_TIMEOUT_SEC":        "30",
		"K1S0_USE_TLS":                 "true",
		"K1S0_SUBJECT":                 "custom-subject",
	})
	cfg, err := Load("admin-bff")
	if err != nil {
		t.Fatalf("Load: %v", err)
	}
	if cfg.ServiceVersion != "1.2.3" {
		t.Errorf("ServiceVersion = %q", cfg.ServiceVersion)
	}
	if cfg.HTTP.Addr != ":9000" {
		t.Errorf("HTTP.Addr = %q", cfg.HTTP.Addr)
	}
	if cfg.HTTP.ReadTimeoutSec != 30 {
		t.Errorf("HTTP.ReadTimeoutSec = %d", cfg.HTTP.ReadTimeoutSec)
	}
	if !cfg.K1s0.UseTLS {
		t.Errorf("K1s0.UseTLS should be true")
	}
	if cfg.K1s0.Subject != "custom-subject" {
		t.Errorf("K1s0.Subject = %q", cfg.K1s0.Subject)
	}
	if cfg.OTLPEndpoint != "otel:4317" {
		t.Errorf("OTLPEndpoint = %q", cfg.OTLPEndpoint)
	}
}

func TestGetenvBoolDefault_AcceptsCommonValues(t *testing.T) {
	cases := []struct {
		v    string
		want bool
	}{
		{"1", true}, {"true", true}, {"TRUE", true}, {"yes", true}, {"on", true},
		{"0", false}, {"false", false}, {"no", false}, {"off", false},
	}
	for _, c := range cases {
		t.Setenv("X_BOOL", c.v)
		got := getenvBoolDefault("X_BOOL", !c.want) // 既定値は want と逆
		if got != c.want {
			t.Errorf("getenvBoolDefault(%q) = %v, want %v", c.v, got, c.want)
		}
	}
	// 未設定 / 不正値は default を使う。
	t.Setenv("X_BOOL", "")
	if got := getenvBoolDefault("X_BOOL", true); !got {
		t.Errorf("empty value should fall back to default")
	}
	t.Setenv("X_BOOL", "garbage")
	if got := getenvBoolDefault("X_BOOL", true); !got {
		t.Errorf("invalid value should fall back to default")
	}
}

func TestGetenvIntDefault_FallsBackOnError(t *testing.T) {
	t.Setenv("X_INT", "abc")
	if got := getenvIntDefault("X_INT", 42); got != 42 {
		t.Errorf("invalid int should fall back to default 42, got %d", got)
	}
	t.Setenv("X_INT", "")
	if got := getenvIntDefault("X_INT", 42); got != 42 {
		t.Errorf("empty should fall back, got %d", got)
	}
	t.Setenv("X_INT", "100")
	if got := getenvIntDefault("X_INT", 42); got != 100 {
		t.Errorf("valid int parse failed, got %d", got)
	}
}
