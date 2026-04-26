// 本ファイルは k1s0 tier1 Go ファサード（k1s0d）の起動エントリポイント。
//
// 設計: plan/04_tier1_Goファサード実装/01_リポジトリレイアウト.md
//       docs/02_構想設計/02_tier1設計/
// 関連 ID: IMP-BUILD-002 / IMP-DIR-002 / ADR-TIER1-001 / ADR-TIER1-003
//
// scope（リリース時点最小骨格）:
//   - :50051 で gRPC server を listen
//   - 標準 gRPC health protocol（grpc.health.v1.Health/Check）に応答
//   - SIGINT / SIGTERM で graceful shutdown
//
// 未実装（plan 04-02 〜 04-13 で追加）:
//   - 11 API ハンドラ登録（log/state/pubsub/secret/binding/workflow/serviceinvoke/decision/audit/feature/telemetry）
//   - Dapr Go SDK 経由の backend 接続（internal/dapr/）
//   - OTel trace / metrics / logger（internal/observability/）
//   - retry / circuit-breaker（internal/reliability/）
//   - 設定読込（internal/config/、YAML + envvar）
//   - k1s0 独自 HealthService（liveness / readiness、proto: k1s0.tier1.health.v1）

// パッケージ宣言。`go build ./cmd/k1s0d` で単一バイナリにビルドされる。
package main

// 標準ライブラリと gRPC ライブラリを import する。
import (
	// graceful shutdown 制御に context.WithTimeout を使う。
	"context"
	// listen address を flag で受け取り、ConfigMap での上書きに備える。
	"flag"
	// 起動 / shutdown / エラーログを stderr に出す（OTel logger は plan 04-02 で導入）。
	"log"
	// TCP listener を確保するため net.Listen を使う。
	"net"
	// シグナル受信用に os.Signal を扱う。
	"os"
	// SIGINT / SIGTERM の通知チャネル登録に signal.Notify を使う。
	"os/signal"
	// SIGTERM 等の syscall シグナル定数を参照する。
	"syscall"
	// graceful shutdown のタイムアウトに time.Duration を使う。
	"time"

	// gRPC サーバ実装本体。
	"google.golang.org/grpc"
	// 標準 gRPC health protocol の参照実装（grpc-go 公式）。
	"google.golang.org/grpc/health"
	// HealthCheckResponse_SERVING 等の enum と protobuf 定義。
	healthpb "google.golang.org/grpc/health/grpc_health_v1"
	// 開発・運用補助のための gRPC reflection（grpcurl 等で proto ファイル不要のサービス探索を可能にする）。
	"google.golang.org/grpc/reflection"
)

// 定数群。リリース時点の最小骨格用に hardcode し、将来 ConfigMap 読込で差し替える。
const (
	// :50051 は gRPC の de facto デフォルト。docs/02_構想設計/02_tier1設計/ で確定後 ConfigMap で上書き予定。
	defaultListenAddr = ":50051"

	// graceful shutdown の上限。Kubernetes の terminationGracePeriodSeconds（既定 30s）より短く設定する。
	shutdownTimeout = 25 * time.Second
)

// プロセスエントリポイント。flag パースと run() への委譲のみを行う。
func main() {
	// listen address の上書き flag を定義（既定 :50051、後で ConfigMap → envvar → flag の優先順で読む）。
	addr := flag.String("listen", defaultListenAddr, "gRPC server listen address")
	// flag 解析を起動直後に確定させる。
	flag.Parse()

	// run() に委譲し、エラー時は log.Fatalf で非ゼロ終了する（exit code 1）。
	if err := run(*addr); err != nil {
		// fatal log は stderr + exit(1) を 1 行で行う Go の慣用。
		log.Fatalf("k1s0d: %v", err)
	}
}

// run は gRPC server のライフサイクル全体（listen → serve → shutdown）を司る。
func run(addr string) error {
	// 指定アドレスで TCP listener を確保する。失敗時は呼び出し元（main）が exit(1)。
	lis, err := net.Listen("tcp", addr)
	// listen 失敗（ポート競合 / 権限不足 / IP 解決失敗）は即時 return。
	if err != nil {
		return err
	}

	// gRPC server インスタンスを生成する（interceptor / TLS は plan 04-02 で追加）。
	srv := grpc.NewServer()

	// 標準 gRPC health protocol を登録する。
	// Kubernetes liveness / readiness probe の grpc-health-probe や kubelet の gRPC probe が叩くのは本エンドポイント。
	// k1s0 独自の HealthService（業務的 readiness）は plan 04-13 で別途追加する。
	hs := health.NewServer()
	// 空文字 "" は service 全体（無指定）の status を意味する。SERVING で公開する。
	hs.SetServingStatus("", healthpb.HealthCheckResponse_SERVING)
	// gRPC server に health service を実装として登録する。
	healthpb.RegisterHealthServer(srv, hs)

	// gRPC reflection を有効化する。
	// dev / staging では grpcurl 等での疎通確認に有用。production では config で無効化する設計（plan 04-02）。
	reflection.Register(srv)

	// Serve のエラーをメイン goroutine に届けるバッファ付きチャネル。
	errCh := make(chan error, 1)
	// gRPC Serve は blocking なので別 goroutine で起動する。
	go func() {
		// 起動ログ（OTel logger 導入前の暫定）。
		log.Printf("k1s0d: gRPC server listening on %s", addr)
		// Serve は shutdown 時 nil を返すか、内部エラー時に non-nil を返す。
		errCh <- srv.Serve(lis)
	}()

	// シグナル受信チャネル。SIGINT / SIGTERM の 2 種を購読する。
	sigCh := make(chan os.Signal, 1)
	// k8s の Pod 終了は SIGTERM、ローカル開発は Ctrl-C で SIGINT。
	signal.Notify(sigCh, syscall.SIGINT, syscall.SIGTERM)

	// シグナル or Serve エラーのいずれかを待つ。
	select {
	// 通常停止経路: シグナル受信 → graceful shutdown へ進む。
	case sig := <-sigCh:
		log.Printf("k1s0d: received signal %s, shutting down", sig)
	// 異常停止経路: Serve が error を返した場合は即時 return（shutdown 不要）。
	case err := <-errCh:
		return err
	}

	// readiness を NOT_SERVING に倒し、L4 LB から外れる猶予を確保してから graceful stop する。
	// Shutdown() は登録済の全 service status を NOT_SERVING にする。
	hs.Shutdown()

	// GracefulStop は in-flight RPC の完了を待つため、別 goroutine + timeout で監視する。
	stopped := make(chan struct{})
	// goroutine 内で GracefulStop を呼ぶ（blocking）。
	go func() {
		// 既存接続の RPC が完了するまで新規 accept を停止しつつ待機する。
		srv.GracefulStop()
		// 完了通知。
		close(stopped)
	}()

	// shutdownTimeout の上限を context で表現する。
	ctx, cancel := context.WithTimeout(context.Background(), shutdownTimeout)
	// cancel は通常パスでも忘れず呼ぶ（go vet 対策）。
	defer cancel()

	// graceful shutdown の完了 or タイムアウトを待つ。
	select {
	// 想定経路: in-flight RPC が時間内に完了。
	case <-stopped:
		log.Printf("k1s0d: graceful shutdown complete")
	// 異常経路: タイムアウトしたら強制停止に切替（既存 RPC を破棄）。
	case <-ctx.Done():
		log.Printf("k1s0d: graceful shutdown timeout, forcing stop")
		// Stop は in-flight RPC を即座に切断する。
		srv.Stop()
	}
	// 正常終了。caller（main）は exit(0)。
	return nil
}
