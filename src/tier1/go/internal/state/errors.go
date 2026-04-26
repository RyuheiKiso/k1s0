// 本ファイルは t1-state Pod 内で共通利用するエラー判定ヘルパ。
//
// adapter の ErrNotWired を errors.Is で判定するための薄い utility。

package state

// 標準ライブラリ。
import (
	// errors.Is で sentinel エラー判定。
	"errors"
	// Dapr adapter の sentinel エラー。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
)

// isNotWired は err が adapter.ErrNotWired かどうかを判定する。
// 各 RPC handler の翻訳 helper から呼ばれる。
func isNotWired(err error) bool {
	// errors.Is で sentinel エラー判定。
	return errors.Is(err, dapr.ErrNotWired)
}
