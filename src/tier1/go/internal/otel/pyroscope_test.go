// 本ファイルは otel/pyroscope.go の単体テスト。
//
// 試験戦略:
//   env 設定 + LoadPyroscopeConfigFromEnv の組合せで config 解決ロジックを検証する。
//   実 Profiler 結線は採用初期 carry-over なので、stub の no-op 動作のみテストする。

package otel

import (
	// 環境変数の test 中設定。
	"os"
	// テスト fail / 報告。
	"testing"
)

// TestLoadPyroscopeConfigFromEnv_AllSet は全 env 設定時に config が正しく組まれることを確認する。
func TestLoadPyroscopeConfigFromEnv_AllSet(t *testing.T) {
	// 既存 env を保存する。
	savedAddr, hadAddr := os.LookupEnv("PYROSCOPE_SERVER_ADDRESS")
	savedApp, hadApp := os.LookupEnv("PYROSCOPE_APPLICATION_NAME")
	savedTenant, hadTenant := os.LookupEnv("PYROSCOPE_TENANT_ID")
	// テスト終了時に env を復元する。
	defer func() {
		if hadAddr {
			_ = os.Setenv("PYROSCOPE_SERVER_ADDRESS", savedAddr)
		} else {
			_ = os.Unsetenv("PYROSCOPE_SERVER_ADDRESS")
		}
		if hadApp {
			_ = os.Setenv("PYROSCOPE_APPLICATION_NAME", savedApp)
		} else {
			_ = os.Unsetenv("PYROSCOPE_APPLICATION_NAME")
		}
		if hadTenant {
			_ = os.Setenv("PYROSCOPE_TENANT_ID", savedTenant)
		} else {
			_ = os.Unsetenv("PYROSCOPE_TENANT_ID")
		}
	}()
	// 全 env を設定する。
	_ = os.Setenv("PYROSCOPE_SERVER_ADDRESS", "http://pyroscope.observability:4040")
	_ = os.Setenv("PYROSCOPE_APPLICATION_NAME", "k1s0.tier1-state")
	_ = os.Setenv("PYROSCOPE_TENANT_ID", "tenant-A")
	// config を読み込む。
	cfg := LoadPyroscopeConfigFromEnv()
	// ServerAddress が一致すること。
	if cfg.ServerAddress != "http://pyroscope.observability:4040" {
		t.Fatalf("ServerAddress = %q, want http://pyroscope.observability:4040", cfg.ServerAddress)
	}
	// ApplicationName が一致すること。
	if cfg.ApplicationName != "k1s0.tier1-state" {
		t.Fatalf("ApplicationName = %q, want k1s0.tier1-state", cfg.ApplicationName)
	}
	// TenantID が一致すること。
	if cfg.TenantID != "tenant-A" {
		t.Fatalf("TenantID = %q, want tenant-A", cfg.TenantID)
	}
	// IsEnabled() が true。
	if !cfg.IsEnabled() {
		t.Fatalf("IsEnabled() = false, want true")
	}
}

// TestLoadPyroscopeConfigFromEnv_NoServer_Disabled は ServerAddress 未設定時に
// IsEnabled が false を返すことを確認する。
func TestLoadPyroscopeConfigFromEnv_NoServer_Disabled(t *testing.T) {
	// 既存 env を保存する。
	saved, had := os.LookupEnv("PYROSCOPE_SERVER_ADDRESS")
	// テスト終了時に復元する。
	defer func() {
		if had {
			_ = os.Setenv("PYROSCOPE_SERVER_ADDRESS", saved)
		} else {
			_ = os.Unsetenv("PYROSCOPE_SERVER_ADDRESS")
		}
	}()
	// env を unset する。
	_ = os.Unsetenv("PYROSCOPE_SERVER_ADDRESS")
	// config を読む。
	cfg := LoadPyroscopeConfigFromEnv()
	// IsEnabled() が false。
	if cfg.IsEnabled() {
		t.Fatalf("IsEnabled() = true with no PYROSCOPE_SERVER_ADDRESS, want false")
	}
}

// TestStartPyroscope_Disabled_ReturnsNoopStop は disabled config で stop が
// nil でない closure を返し、呼出して error しないことを確認する。
func TestStartPyroscope_Disabled_ReturnsNoopStop(t *testing.T) {
	// disabled config を作る。
	cfg := PyroscopeConfig{ServerAddress: ""}
	// Start を呼ぶ。
	stop, err := StartPyroscope(cfg)
	// err がないこと。
	if err != nil {
		t.Fatalf("StartPyroscope(disabled): %v, want nil", err)
	}
	// stop が non-nil。
	if stop == nil {
		t.Fatalf("stop is nil; expected non-nil no-op closure")
	}
	// stop を呼んでも panic しないこと。
	stop()
}
