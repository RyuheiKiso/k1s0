// 本ファイルは tier1 Go ファサードの Golden Path 最小例の起動エントリポイント。
//
// 設計: docs/05_実装/00_ディレクトリ設計/70_共通資産/03_examples配置.md（IMP-DIR-COMM-113）
//       docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md（main.go の典型形）
// 関連 ID: ADR-TIER1-001 / ADR-TIER1-002 / ADR-TIER1-003 / ADR-DEV-001
//
// 役割:
//   tier1 Go ファサードの「最小だが本番形」を示す。具体的には
//     - gRPC server の起動（標準 health protocol + reflection）
//     - SIGINT / SIGTERM での graceful shutdown
//   までを 1 ファイルで読み切れる範囲に収める。Pod 用 cmd（src/tier1/go/cmd/{state,secret,workflow}）
//   と異なり、本 example は internal/common への依存を持たず単体で完結する。
//
// scope（リリース時点）:
//   - :50001 で listen（Dapr sidecar 既定の app-port、docs 正典）
//   - 標準 gRPC health protocol 応答
//   - reflection（grpcurl での疎通確認用）
//   - SIGINT / SIGTERM で graceful shutdown
//
// 未実装（採用初期で拡張）:
//   - proto handler 登録（src/sdk/go/generated/k1s0/tier1/*/v1/* の RegisterXxxServer）
//   - Dapr Go SDK adapter（internal/adapter/dapr/）
//   - OTel interceptor（trace / metrics / logger）
//   - integration test（Testcontainers + Dapr Local）
//   - Dockerfile（src/tier1/go/Dockerfile.state を踏襲）

// パッケージ宣言。`go build ./cmd/example-facade` で本サンプルの実行バイナリが生成される。
package main

// 標準ライブラリと gRPC ライブラリを import する。
import (
	// graceful shutdown 制御に context.WithTimeout を使う。
	"context"
	// listen address を flag で受け取り、コンテナの引数や ConfigMap での上書きに備える。
	"flag"
	// 起動 / shutdown / エラーログを stderr に出す（OTel logger は採用初期で導入）。
	"log"
	// TCP listener 確保のため net.Listen を使う。
	"net"
	// シグナル受信用に os.Signal を扱う。
	"os"
	// SIGINT / SIGTERM の通知チャネル登録に signal.Notify を使う。
	"os/signal"
	// SIGTERM 等の syscall シグナル定数を参照する。
	"syscall"
	// graceful shutdown のタイムアウト指定に time.Duration を使う。
	"time"

	// gRPC サーバ実装本体。
	"google.golang.org/grpc"
	// 標準 gRPC health protocol の参照実装（grpc-go 公式）。
	"google.golang.org/grpc/health"
	// HealthCheckResponse_SERVING 等の enum と protobuf 定義。
	healthpb "google.golang.org/grpc/health/grpc_health_v1"
	// 開発・運用補助のための gRPC reflection（grpcurl で proto ファイル不要のサービス探索を可能にする）。
	"google.golang.org/grpc/reflection"
)

// 既定 listen address。Dapr sidecar の app-port 既定値（docs 正典 EXPOSE 50001）に揃える。
const defaultListen = ":50001"

// graceful shutdown の上限。Kubernetes terminationGracePeriodSeconds（既定 30s）より短く設定する。
const shutdownTimeout = 25 * time.Second

// プロセスエントリポイント。flag パースと gRPC server 起動 / shutdown を行う。
func main() {
	// listen address の上書き flag を定義する（既定 :50001）。
	addr := flag.String("listen", defaultListen, "gRPC server listen address")
	// flag 解析を起動直後に確定させる。
	flag.Parse()

	// 指定アドレスで TCP listener を確保する。失敗時は exit(1) で停止する。
	lis, err := net.Listen("tcp", *addr)
	// listen 失敗（ポート競合 / 権限不足 / IP 解決失敗）は即時 fatal。
	if err != nil {
		// fatal log は stderr + exit(1) を 1 行で行う Go の慣用。
		log.Fatalf("example-facade: listen %s: %v", *addr, err)
	}

	// gRPC server インスタンスを生成する（interceptor / TLS は採用初期で追加）。
	srv := grpc.NewServer()

	// 標準 gRPC health protocol を登録する。
	hs := health.NewServer()
	// 空文字 "" は service 全体（無指定）の status を意味する。SERVING で公開する。
	hs.SetServingStatus("", healthpb.HealthCheckResponse_SERVING)
	// gRPC server に health service を実装として登録する。
	healthpb.RegisterHealthServer(srv, hs)

	// gRPC reflection を有効化する（dev / staging で grpcurl 疎通用、production は config で無効化予定）。
	reflection.Register(srv)

	// Serve のエラーをメイン goroutine に届けるバッファ付きチャネル。
	errCh := make(chan error, 1)
	// gRPC Serve は blocking なので別 goroutine で起動する。
	go func() {
		// 起動ログ（OTel logger 導入前の暫定）。
		log.Printf("example-facade: gRPC server listening on %s", *addr)
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
		// 受信シグナルをログに残す。
		log.Printf("example-facade: received signal %s, shutting down", sig)
	// 異常停止経路: Serve が error を返した場合は即時 exit。
	case err := <-errCh:
		// Serve が non-nil error を返したら fatal で終了する。
		log.Fatalf("example-facade: serve: %v", err)
	}

	// readiness を NOT_SERVING に倒し、L4 LB から外れる猶予を確保する。
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
		// 完了ログを出力する。
		log.Printf("example-facade: graceful shutdown complete")
	// 異常経路: タイムアウトしたら強制停止に切替（既存 RPC を破棄）。
	case <-ctx.Done():
		// timeout ログを出力する。
		log.Printf("example-facade: graceful shutdown timeout, forcing stop")
		// Stop は in-flight RPC を即座に切断する。
		srv.Stop()
	}
}
