// session_port.go は BFF-Proxy の usecase 層がセッション操作に使用するポートインターフェースを定義する。
// session.Store インターフェースをそのまま参照することで、usecase 層が session パッケージに
// 直接依存しつつも、ポートとしての意図を明示する。
// モックはこのエイリアスを実装することで、usecase のテストを可能にする。
package port

import (
	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/session"
)

// SessionStore は BFF-Proxy の usecase 層が利用するセッションストアのポートインターフェース。
// session.Store と同一の契約を持ち、usecase 層からはこのエイリアスで参照する。
// 既存の session.RedisStore や session.EncryptedStore は session.Store を実装しているため、
// そのまま SessionStore としても使用できる。
type SessionStore = session.Store
