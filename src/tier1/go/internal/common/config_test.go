// 本ファイルは internal/common/config.go の単体テスト。
//
// 設計: plan/04_tier1_Goファサード実装/02_共通基盤.md（主作業 6: 設定読込）
//       docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/05_モジュール依存関係.md
// 関連 ID: IMP-CONFIG-* / NFR-OPS-*

package common

// 標準ライブラリのみ使用（third-party 依存なし）。
import (
	// envvar 操作とテストフレームワーク用。
	"os"
	// 標準テストフレームワーク。
	"testing"
)

// TestLoad_DefaultListenAddr は envvar 未設定時に :50001（docs 正典）が返ることを確認する。
func TestLoad_DefaultListenAddr(t *testing.T) {
	// 既存の envvar を一時的に退避（テスト後に復元）。
	saved, hadValue := os.LookupEnv("K1S0_LISTEN_ADDR")
	// envvar を確実に未設定にする。
	if err := os.Unsetenv("K1S0_LISTEN_ADDR"); err != nil {
		// envvar 操作失敗はテスト基盤側の問題、即時 fatal。
		t.Fatalf("unsetenv failed: %v", err)
		// if 分岐を閉じる。
	}
	// テスト終了時に envvar の状態を復元する。
	defer func() {
		// 元々値があった場合のみ Setenv で戻す。
		if hadValue {
			// 退避した値を復元。
			_ = os.Setenv("K1S0_LISTEN_ADDR", saved)
			// if 分岐を閉じる。
		}
		// defer 関数を閉じる。
	}()

	// Load() を呼び出す。
	cfg, err := Load()
	// エラーは想定外。
	if err != nil {
		// 失敗はテスト fatal。
		t.Fatalf("Load() returned error: %v", err)
		// if 分岐を閉じる。
	}
	// docs 正典の default port :50001 が設定されているか検証する。
	if cfg.ListenAddr != ":50001" {
		// 期待値と異なる場合は fail。
		t.Errorf("ListenAddr = %q, want %q", cfg.ListenAddr, ":50001")
		// if 分岐を閉じる。
	}
	// テスト関数を閉じる。
}

// TestLoad_EnvvarOverride は K1S0_LISTEN_ADDR が設定されている時にその値が反映されることを確認する。
func TestLoad_EnvvarOverride(t *testing.T) {
	// テスト用の envvar 値（既定とは異なる port を使う）。
	const want = ":50099"
	// envvar を一時設定。
	saved, hadValue := os.LookupEnv("K1S0_LISTEN_ADDR")
	// テスト用 envvar を設定する。
	if err := os.Setenv("K1S0_LISTEN_ADDR", want); err != nil {
		// envvar 操作失敗は fatal。
		t.Fatalf("setenv failed: %v", err)
		// if 分岐を閉じる。
	}
	// テスト後に元の状態に復元する。
	defer func() {
		// 元々値があった場合は復元、なかった場合は unset。
		if hadValue {
			// 退避した値を復元。
			_ = os.Setenv("K1S0_LISTEN_ADDR", saved)
			// if 分岐を閉じる。
		} else {
			// 元々未設定だったので unset に戻す。
			_ = os.Unsetenv("K1S0_LISTEN_ADDR")
			// else 分岐を閉じる。
		}
		// defer 関数を閉じる。
	}()

	// Load() を呼び出す。
	cfg, err := Load()
	// エラーは想定外。
	if err != nil {
		// fatal。
		t.Fatalf("Load() returned error: %v", err)
		// if 分岐を閉じる。
	}
	// envvar 値が反映されているか検証する。
	if cfg.ListenAddr != want {
		// 期待値と異なる場合は fail。
		t.Errorf("ListenAddr = %q, want %q", cfg.ListenAddr, want)
		// if 分岐を閉じる。
	}
	// テスト関数を閉じる。
}
