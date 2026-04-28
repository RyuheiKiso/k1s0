// 本ファイルは tier1 Go の 3 Pod（state / secret / workflow）共通の
// `k1s0.tier1.health.v1.HealthService` 実装を提供する。
//
// 設計正典:
//   src/contracts/tier1/k1s0/tier1/health/v1/health_service.proto
//     - Liveness: process が応答可能なら OK。依存 backend は見ない。
//     - Readiness: 依存 backend（Postgres / Kafka / OpenBao / Temporal / Dapr 等）が
//       到達可能かどうかも含めて判定する。
//   docs/02_構想設計/02_tier1設計/grpc-reference/v1/k1s0-tier1-grpc.md
//     - LivenessResponse: { version, uptime_seconds }
//     - ReadinessResponse: { ready, dependencies map<string, DependencyStatus> }
//     - DependencyStatus: { reachable, error_message }
//
// 標準 gRPC health protocol（grpc.health.v1.Health）は別途
// `internal/common/runtime.go` で google.golang.org/grpc/health/grpc_health_v1 を
// 直接登録しており、Kubernetes liveness/readiness probe 経路はそちらを使う。
// 本サービスは tier2 / tier3 からの疎通確認と、依存先（OpenBao / Temporal / Dapr）の
// 到達性可視化のための補助 API。

// Package health は `k1s0.tier1.health.v1.HealthService` の Pod 共通実装を提供する。
package health

// 標準 / SDK 生成スタブ / gRPC を import する。
import (
	// Probe 実行時の cancel 制御。
	"context"
	// Probe 並列実行時の Map / Mutex 整合性確保。
	"sync"
	// Pod 起動時刻からの uptime 計算。
	"time"

	// 生成された HealthService gRPC スタブ。
	healthv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/health/v1"
	// gRPC server 型（Register で受ける）。
	"google.golang.org/grpc"
)

// DependencyProbe は Pod が依存する単一 backend の到達性検査。
// Name は ReadinessResponse.dependencies のキー（例 "openbao" / "temporal" / "dapr"）。
// Check は ctx 期限内に到達性を確認し、到達不能なら non-nil error を返す。
type DependencyProbe struct {
	// 依存先論理名（Readiness response の dependencies map のキー）。
	Name string
	// 到達性検査関数。nil error は reachable=true、non-nil は reachable=false + error_message。
	Check func(context.Context) error
}

// Service は HealthService の 2 RPC（Liveness / Readiness）実装。
// ポインタ受信で gRPC server に登録する。
type Service struct {
	// 未実装 RPC を Unimplemented に倒す埋め込み（forward compatibility）。
	healthv1.UnimplementedHealthServiceServer
	// Pod のビルドバージョン（SemVer）。Liveness response で返却する。
	version string
	// Pod 起動時刻。Liveness の uptime_seconds 算出に使う。
	startTime time.Time
	// 依存先到達性検査リスト。Readiness で全件並列実行する。
	probes []DependencyProbe
	// probes スライスへの並行アクセスガード。Pod ライフタイム中は不変だが
	// 将来の hot-reload 用に予約。
	mu sync.RWMutex
}

// New は 起動時 metadata と依存先 probe リストから Service を構築する。
// 起動時刻は New 呼び出し時に固定する（Liveness の uptime 起点）。
func New(version string, probes []DependencyProbe) *Service {
	// 起動時刻を確定（time.Now() を 1 回だけ呼ぶ）。
	return &Service{
		// バージョン文字列を保持する。
		version: version,
		// 起動時刻を確定する。
		startTime: time.Now(),
		// 依存先 probe をコピー保持する（caller が変更しても影響しないように append nil で別スライス化）。
		probes: append([]DependencyProbe(nil), probes...),
	}
}

// Register は本サービスを gRPC server に登録する。
// runtime.Run の Pod 共通登録ロジックから呼ばれる。
func (s *Service) Register(srv *grpc.Server) {
	// 生成スタブの Register 関数を呼び出して HealthService を gRPC server に紐付ける。
	healthv1.RegisterHealthServiceServer(srv, s)
}

// Liveness は process 生存確認 + version + uptime を返す。依存 backend は見ない。
// proto 規定: process が応答可能なら OK。本実装は常に成功を返す（gRPC handler が
// 起動済 = process が応答可能、の含意）。
func (s *Service) Liveness(_ context.Context, _ *healthv1.LivenessRequest) (*healthv1.LivenessResponse, error) {
	// uptime を秒単位で算出する（int64 へ切り捨て、proto 規定どおり）。
	uptime := int64(time.Since(s.startTime).Seconds())
	// バージョンと uptime を詰めて返却する。
	return &healthv1.LivenessResponse{
		// SemVer 文字列。
		Version: s.version,
		// 起動からの経過秒数。
		UptimeSeconds: uptime,
	}, nil
}

// Readiness は依存先 probe を並列実行し、全件 reachable のとき ready=true を返す。
// proto 規定: 各依存（postgres / kafka / openbao / keycloak / 等）の個別状態を返す。
// probe が空（依存なしの Pod 想定）なら ready=true / 空 map を返す。
func (s *Service) Readiness(ctx context.Context, _ *healthv1.ReadinessRequest) (*healthv1.ReadinessResponse, error) {
	// 並列実行時の整合性ガード（probes を読むのみで書き換えない）。
	s.mu.RLock()
	// 関数末尾で必ず unlock する。
	defer s.mu.RUnlock()

	// probe 結果を集約する map（proto の dependencies フィールドに直接渡す）。
	results := make(map[string]*healthv1.DependencyStatus, len(s.probes))
	// probe 結果を並列収集するため WaitGroup と排他 Mutex を用意する。
	var wg sync.WaitGroup
	// results map への書き込み排他用 Mutex。
	var resultsMu sync.Mutex

	// 各依存先 probe を goroutine で並列実行する（依存数は Pod ごとに少数想定、3 〜 5 件）。
	for _, p := range s.probes {
		// 並列実行中に loop 変数の捕捉を確定するため、ローカル変数に複製する（Go 1.22 以降は不要だが安全側）。
		probe := p
		// WaitGroup カウンタを増やす。
		wg.Add(1)
		// goroutine で検査を実行する。
		go func() {
			// goroutine 終了時に必ず Done する。
			defer wg.Done()
			// probe.Check を呼んで到達性を検査する。
			err := probe.Check(ctx)
			// 結果集約は Mutex 越しに行う。
			resultsMu.Lock()
			// 関数末尾で必ず unlock する（このスコープ内）。
			defer resultsMu.Unlock()
			// err nil なら reachable=true、non-nil なら error_message を埋める。
			if err == nil {
				// 到達可能。
				results[probe.Name] = &healthv1.DependencyStatus{
					// proto 規定: reachable のみ true で error_message は空。
					Reachable: true,
				}
				// 早期 return で reachable=false 経路を skip する。
				return
			}
			// 到達不能。error.Error() を error_message に詰める。
			results[probe.Name] = &healthv1.DependencyStatus{
				// proto 規定: reachable=false の時のみ error_message が意味を持つ。
				Reachable: false,
				// 直近のエラー文字列。
				ErrorMessage: err.Error(),
			}
		}()
	}
	// 全 probe 完了を待機する。
	wg.Wait()

	// 全件 reachable=true なら ready=true。
	ready := true
	// 1 件でも reachable=false があれば ready=false。
	for _, status := range results {
		// reachable=false を見つけたら ready を倒して終了。
		if !status.GetReachable() {
			// 早期 break で短絡評価する。
			ready = false
			// 残りを走査する必要はない。
			break
		}
	}
	// ReadinessResponse を組み立てて返却する。
	return &healthv1.ReadinessResponse{
		// 全体の ready 判定。
		Ready: ready,
		// 各依存の個別状態。
		Dependencies: results,
	}, nil
}
