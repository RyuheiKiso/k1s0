// metrics パッケージは bff-proxy 全体で共有する Prometheus メトリクスを一元管理する。
// 複数パッケージ（middleware/usecase）からセッション削除失敗を記録するため、
// 共通パッケージに定義してパッケージ間での重複登録エラーを防ぐ。
package metrics

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

var (
	// L-003 監査対応: セッション削除失敗をモニタリングするためのカウンタ。
	// セッション削除失敗はサイレントに処理されるため、この指標で異常を検知可能にする。
	// reason ラベルには削除が発生したコンテキストを設定する:
	//   "session_expired"       - 絶対有効期限切れセッションの削除失敗（SessionMiddleware）
	//   "callback"              - コールバック時の既存セッション削除失敗（AuthUseCase.Callback）
	//   "logout"                - ログアウト時のセッション削除失敗（AuthUseCase.Logout）
	//   "token_refresh_fail"    - トークンリフレッシュ失敗後のセッション削除失敗（ProxyUseCase）
	SessionDeleteErrorsTotal = promauto.NewCounterVec(prometheus.CounterOpts{
		Name: "bff_session_delete_errors_total",
		Help: "セッション削除に失敗した回数",
	}, []string{"reason"})
)
