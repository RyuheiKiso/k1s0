// 本ファイルは tier1 Go の設定読込ユーティリティ。
//
// 設計: docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/05_モジュール依存関係.md
//       （DS-SW-COMP-108: k1s0-common 共通ライブラリ、設定ロード / tenant_id 抽出等の横断 utility）
//       plan/04_tier1_Goファサード実装/02_共通基盤.md（plan 側は `internal/config/` 表記、docs 正典は `internal/common/` 内）
// 関連 ID: IMP-CONFIG-* / NFR-OPS-*
//
// scope（リリース時点最小骨格）:
//   - Config 型と Load() のシグネチャ確定のみ
//   - YAML パース / envvar 上書き（K1S0_*）/ defaults / OpenBao 連携は次セッションで実装
//
// 未実装（plan 04-02 主作業 6 で追加、次セッション以降）:
//   - YAML パース（gopkg.in/yaml.v3 依存追加）
//   - envvar 上書き（K1S0_LISTEN_ADDR / K1S0_LOG_LEVEL 等）
//   - defaults.go の中央定義
//   - secret 注入（OpenBao or Kubernetes secret）

package common

// 標準ライブラリのみ import（minimal skeleton）。
import (
	// envvar 読込用。
	"os"
)

// Config は tier1 Pod の起動時設定を保持する。
//
// 後続実装（次セッション）で field を追加: ListenAddr / LogLevel / OTel エンドポイント /
// retry 設定 / circuit-breaker 設定 / timeout 階層（500ms / 200ms / 100ms 予算）等。
type Config struct {
	// ListenAddr は gRPC server の listen address（:50001 既定、docs 正典 EXPOSE 50001）。
	ListenAddr string
	// 構造体定義を閉じる。
}

// Load は config.yaml の読込 + envvar 上書きで Config を構築する。
//
// scope（リリース時点）: envvar `K1S0_LISTEN_ADDR` のみ参照、未設定時は :50001 を返す。
// YAML / defaults / OpenBao 連携は次セッションで実装。
func Load() (*Config, error) {
	// envvar `K1S0_LISTEN_ADDR` を最優先で参照する（次セッションで YAML / defaults / envvar の優先順を整理）。
	addr := os.Getenv("K1S0_LISTEN_ADDR")
	// envvar 未設定時は docs 正典 default port :50001 を採用する。
	if addr == "" {
		// default を代入する。
		addr = ":50001"
		// if 分岐を閉じる。
	}
	// 最小 Config を返す（後続実装でフィールド追加）。
	return &Config{ListenAddr: addr}, nil
	// Load 関数を閉じる。
}
