package middleware

import (
	"context"
	"net/http"
	"sort"
	"sync"
	"time"

	"github.com/gin-gonic/gin"

	"github.com/k1s0-platform/system-server-go-bff-proxy/internal/config"
)

// visitorBucket はIPアドレスごとのトークンバケット状態を保持する。
type visitorBucket struct {
	mu       sync.Mutex
	tokens   float64
	lastSeen time.Time
}

// maxVisitors はメモリ保護のためにエントリ数の上限を定義する。
// M-11 監査対応: visitors マップのエントリ数に上限を設け、大量の一意 IP による OOM を防止する。
const maxVisitors = 10000

// RateLimitMiddleware はIPアドレスベースのトークンバケットアルゴリズムによるレート制限を実装する。
// H-2対応: BFF Proxy レベルで IP ベースのレート制限を追加し、
// DDoS攻撃や大量リクエストから保護する。
// H-3 監査対応: ctx を受け取ることでサーバーシャットダウン時に goroutine を確実に停止する。
// M-11 監査対応: クリーンアップ間隔を 3 分に短縮し、マップ上限（10000件）を設ける。
// 外部依存なし（標準ライブラリ context/sync/time のみ使用）。
func RateLimitMiddleware(ctx context.Context, cfg config.RateLimitConfig) gin.HandlerFunc {
	// レート制限が無効の場合はパススルー
	if !cfg.Enabled {
		return func(c *gin.Context) { c.Next() }
	}

	// rps のデフォルト値（未設定時: 100 req/sec）
	rps := cfg.RPS
	if rps <= 0 {
		rps = 100.0
	}

	// burst のデフォルト値（未設定時: rps の2倍）
	burst := float64(cfg.Burst)
	if burst <= 0 {
		burst = rps * 2
	}

	// IPアドレスごとのバケットを保持するマップ（sync.Mapはロックフリー読み取り最適化）
	var visitors sync.Map

	// 古いバケットを定期クリーンアップするゴルーチン（メモリリーク防止）。
	// H-3 監査対応: ctx.Done() を監視してシャットダウン時に goroutine を停止する。
	// M-11 監査対応: クリーンアップ間隔を 10 分から 3 分に短縮する。
	// M-04 対応: カウントと削除を単一の Range パスに統合し、3 回スキャンを 1 回に削減する。
	go func() {
		// クリーンアップ間隔を 3 分に設定する（旧 10 分から短縮）
		ticker := time.NewTicker(3 * time.Minute)
		defer ticker.Stop()
		for {
			select {
			case <-ticker.C:
				// M-04 対応: 単一の Range パスで期限切れエントリの削除と上限超過用エントリ収集を同時に行う
				expiry := time.Now().Add(-10 * time.Minute)
				// 上限超過時に古い順削除するためのエントリ収集用構造体
				type entry struct {
					key      any
					lastSeen time.Time
				}
				var entries []entry
				visitors.Range(func(key, value any) bool {
					b := value.(*visitorBucket)
					b.mu.Lock()
					ls := b.lastSeen
					b.mu.Unlock()
					// 期限切れエントリは即時削除する
					if ls.Before(expiry) {
						visitors.Delete(key)
						return true
					}
					// 上限超過チェック用に有効エントリを収集する
					entries = append(entries, entry{key, ls})
					return true
				})

				// M-11 監査対応: 有効エントリが上限を超えた場合、lastSeen が古い順に削除する
				count := len(entries)
				if count > maxVisitors {
					// 古い順にソートして超過分のエントリを削除する
					sort.Slice(entries, func(i, j int) bool {
						return entries[i].lastSeen.Before(entries[j].lastSeen)
					})
					for i := 0; i < count-maxVisitors; i++ {
						visitors.Delete(entries[i].key)
					}
				}

			case <-ctx.Done():
				// サーバーシャットダウン時に goroutine を停止する（H-3 対応）
				return
			}
		}
	}()

	return func(c *gin.Context) {
		ip := c.ClientIP()
		now := time.Now()

		// IPごとのバケットを取得または新規作成する
		actual, _ := visitors.LoadOrStore(ip, &visitorBucket{
			tokens:   burst,
			lastSeen: now,
		})
		b := actual.(*visitorBucket)

		b.mu.Lock()
		// 前回のリクエストからの経過時間に応じてトークンを補充する
		elapsed := now.Sub(b.lastSeen).Seconds()
		b.tokens += elapsed * rps
		if b.tokens > burst {
			b.tokens = burst
		}
		b.lastSeen = now

		// トークンが不足している場合は 429 Too Many Requests を返す
		if b.tokens < 1.0 {
			b.mu.Unlock()
			c.Header("Retry-After", "1")
			c.AbortWithStatus(http.StatusTooManyRequests)
			return
		}
		b.tokens--
		b.mu.Unlock()

		c.Next()
	}
}
