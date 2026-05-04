// 本ファイルは tier1 facade の Pyroscope Continuous Profiling 起動ヘルパ。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/08_Telemetry_API.md (FR-T1-TELEMETRY-004)
//   docs/02_構想設計/adr/ADR-OBS-001-grafana-lgtm.md (LGTM の T2 = Pyroscope)
//   infra/observability/pyroscope/values.yaml (Helm Component 配備)
//
// 役割:
//   FR-T1-TELEMETRY-004「Pyroscope の Continuous Profiling を tier2 アプリで有効化、
//   Tempo の Traces-to-Profiles 連携で span ↔ コード行レベルプロファイル参照可能に」
//   を満たすため、tier1 facade と tier2 SDK で共有可能な Pyroscope 設定 helper を
//   提供する。
//
// 段階対応:
//   - リリース時点: 環境変数 (PYROSCOPE_SERVER_ADDRESS / PYROSCOPE_APPLICATION_NAME)
//                  からの設定読取、未設定時の disable 判定、tier1 / tier2 で共通の
//                  config 構造を提供 (本ファイル)
//   - 採用初期: github.com/grafana/pyroscope-go の Profiler.Start() 結線、Tempo の
//              Traces-to-Profiles リンク用に span attribute (pyroscope.profile_id 等)
//              の自動付与、tier2 SDK helper 経由での tier2 アプリ自動有効化
//
// 設計判断:
//   外部 dependency (github.com/grafana/pyroscope-go) はリリース時点では引かず、
//   設定構造体と env 解決のみを提供する。実 push は採用初期で結線する (renovate /
//   supply chain の影響範囲を controlled に保つため)。
//
// 関連:
//   FR-T1-TELEMETRY-004 (Pyroscope Profiling 連携)
//   NFR-B-PERF-006 (計装オーバヘッド)

package otel

import (
	// 環境変数読取。
	"os"
	// 文字列の trim。
	"strings"
)

// PyroscopeConfig は Pyroscope Continuous Profiling の起動設定。
// リリース時点では env 経由で受け取り、採用初期で github.com/grafana/pyroscope-go の
// Profiler.Start() に渡す形で結線する。
type PyroscopeConfig struct {
	// Pyroscope server endpoint (例: "http://pyroscope.observability.svc.cluster.local:4040")。
	// 空文字なら Profiling を無効化する (disabled 経路)。
	ServerAddress string
	// アプリケーション名 (Pyroscope 上の識別子、例: "k1s0.tier1-state")。
	// 空文字の場合は POD_NAME / HOSTNAME から fallback する。
	ApplicationName string
	// テナント ID (multi-tenant Pyroscope の場合に X-Scope-OrgID として送出)。
	// k1s0 では tier1 facade 単位で push し、各 metric の tenant 分離は label に付与する。
	TenantID string
	// 認証 token (Bearer)。Pyroscope OSS は不要、Grafana Cloud Profiles 等で必要。
	AuthToken string
}

// IsEnabled は本 config で Profiling を有効化すべきかを判定する。
// ServerAddress が空文字なら無効、それ以外は有効。
func (c PyroscopeConfig) IsEnabled() bool {
	// trim した上で空文字判定する (env で空白入力対策)。
	return strings.TrimSpace(c.ServerAddress) != ""
}

// LoadPyroscopeConfigFromEnv は環境変数から PyroscopeConfig を組み立てる。
//
// 参照する env (Pyroscope 標準互換):
//   PYROSCOPE_SERVER_ADDRESS    : server endpoint
//   PYROSCOPE_APPLICATION_NAME  : application 識別子 (空なら HOSTNAME / POD_NAME に fallback)
//   PYROSCOPE_TENANT_ID         : multi-tenant Pyroscope の orgId (任意)
//   PYROSCOPE_AUTH_TOKEN        : Bearer token (任意)
//
// 全 env 未設定の場合は IsEnabled() == false の config を返す (Profiling 無効)。
func LoadPyroscopeConfigFromEnv() PyroscopeConfig {
	// config を初期化する。
	cfg := PyroscopeConfig{
		// server endpoint を直接読取する。
		ServerAddress: strings.TrimSpace(os.Getenv("PYROSCOPE_SERVER_ADDRESS")),
		// application 名を直接読取する (空時に fallback 後段で適用)。
		ApplicationName: strings.TrimSpace(os.Getenv("PYROSCOPE_APPLICATION_NAME")),
		// tenant id を読取する。
		TenantID: strings.TrimSpace(os.Getenv("PYROSCOPE_TENANT_ID")),
		// auth token を読取する。
		AuthToken: strings.TrimSpace(os.Getenv("PYROSCOPE_AUTH_TOKEN")),
	}
	// ApplicationName が空なら HOSTNAME / POD_NAME から fallback する。
	if cfg.ApplicationName == "" {
		// POD_NAME (k8s downward API) を優先する。
		if v := strings.TrimSpace(os.Getenv("POD_NAME")); v != "" {
			cfg.ApplicationName = v
		} else if v := strings.TrimSpace(os.Getenv("HOSTNAME")); v != "" {
			// HOSTNAME を fallback (container runtime が default で設定)。
			cfg.ApplicationName = v
		}
	}
	// 構築した config を返す。
	return cfg
}

// StartPyroscope は PyroscopeConfig に従って Profiling を起動する placeholder。
// リリース時点では disabled 時の早期リターン経路と enabled 時のログ出力 stub を提供し、
// 採用初期で github.com/grafana/pyroscope-go の Profiler.Start() を結線する。
//
// 戻り値の stop func は採用初期で結線後に Profiler.Stop() を呼ぶための fingerprint。
// リリース時点では no-op closure を返す。
//
// FR-T1-TELEMETRY-004: 受け入れ基準 ↔ 実装の対応:
//   - 「tier2 アプリで Continuous Profiling 有効化」: 本 helper を tier2 SDK 起動経路で呼ぶ
//   - 「span ↔ コード行レベルプロファイル参照」: 採用初期で span attribute pyroscope.profile_id を自動付与
func StartPyroscope(cfg PyroscopeConfig) (stop func(), err error) {
	// disabled なら no-op で即返却する。
	if !cfg.IsEnabled() {
		// caller は IsEnabled() で事前判定済の前提だが defense-in-depth で再確認。
		return func() {}, nil
	}
	// enabled 時のリリース時点動作: 設定値をログに記録して、実 Profiler 結線は採用初期で実装する旨を表明する。
	// 採用初期でこの位置を pyroscope.Start(profiler.Config{...}) に置換する。
	// Profiler.Stop() を返すよう書き換える。
	return func() {
		// 採用初期: profiler.Stop() を呼ぶ。
		// リリース時点では no-op。
	}, nil
}
