// 本ファイルは tier1 Go の 3 Pod（state / secret / workflow）が共通で使う gRPC ランタイム。
//
// 設計: docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/01_tier1全体配置.md
//       docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md
//       plan/04_tier1_Goファサード実装/01_リポジトリレイアウト.md（plan 側は 3 Pod 構成に是正予定）
// 関連 ID: IMP-DIR-002 / IMP-BUILD-002 / ADR-TIER1-001 / ADR-TIER1-003
//
// 役割:
//   各 Pod の main.go を最小化するため、共通する gRPC server bootstrap を本パッケージに集約する。
//   cmd 配下からは internal/ のみ import 可能（他 Pod 内部参照禁止）の規約に整合。
//
// 提供する機能:
//   - :50001 で listen（flag で上書き可、docs 正典 EXPOSE 50001）
//   - 標準 gRPC health protocol（grpc.health.v1.Health/Check）応答 — Kubernetes probe 経路
//   - k1s0 独自 HealthService（k1s0.tier1.health.v1.HealthService.Liveness / Readiness）応答
//     — tier2 / tier3 から version / uptime / 依存先到達性の照会経路
//   - gRPC reflection（dev / staging で grpcurl 疎通用、production は config で無効化予定）
//   - ObservabilityInterceptor（OTel trace 1 span 発行 + RED メトリクス、共通規約 §可観測性）
//   - SIGINT / SIGTERM で graceful shutdown（25s timeout）
//
// 未実装（plan 04-02 以降で追加）:
//   - retry / circuit-breaker / timeout
//   - TLS / mTLS（SPIRE 連携）
//   - 設定読込（YAML + envvar、internal/config/）

// Package common は tier1 Go の 3 Pod（state / secret / workflow）が共通で使う gRPC server bootstrap を提供する。
//
// docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md の
// `internal/{common, adapter, policy, state, secret, workflow}/` 責務分割正典に準拠（common = 横断 utility）。
package common

// 標準ライブラリと gRPC ライブラリを import する。
import (
	// graceful shutdown 制御に context.WithTimeout を使う。
	"context"
	// 起動 / shutdown / エラーログを stderr に出す（OTel logger は plan 04-02 で導入）。
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
	// 開発・運用補助のための gRPC reflection（grpcurl 等で proto ファイル不要のサービス探索を可能にする）。
	"google.golang.org/grpc/reflection"

	// k1s0 独自 HealthService（k1s0.tier1.health.v1）の Pod 共通実装。
	k1s0health "github.com/k1s0/k1s0/src/tier1/go/internal/health"
)

// graceful shutdown の上限。Kubernetes terminationGracePeriodSeconds（既定 30s）より短く設定する。
const shutdownTimeout = 25 * time.Second

// loadAuditEmitterFromEnv は env TIER1_AUDIT_MODE で AuditEmitter を選ぶ。
//   - "log": stderr JSON 風ログ（dev / debug、t1-audit Pod が無くても event が確認できる）
//   - 未指定 / "off": NoopAuditEmitter（既定、既存 test 互換）
//
// production では gRPC client backed emitter を別途 cmd 側で構築して runtime に注入する想定
// （t1-audit Pod の AuditService.Record gRPC を叩く）。本関数は env 駆動 fallback としてのみ機能。
func loadAuditEmitterFromEnv() AuditEmitter {
	switch os.Getenv("TIER1_AUDIT_MODE") {
	case "log":
		return LogAuditEmitter{}
	default:
		return NoopAuditEmitter{}
	}
}

// Pod は 1 Pod を識別するメタデータと service 登録ロジックを集約する。
type Pod struct {
	// 構造体のフィールドを 1 行ずつ宣言する（Go 慣用、ここでは説明コメントのため複数行）。
	// Pod の論理名（例: "state" / "secret" / "workflow"）。ログ出力で使う。
	Name string
	// 既定 listen address（":50001"）。flag で上書き可能。
	DefaultListen string
	// gRPC server に Pod 固有の service を登録する hook。最小骨格では nil でも可。
	Register func(*grpc.Server)
	// Pod のビルドバージョン（SemVer）。HealthService.Liveness response の version に入る。
	// 空文字なら既定値 DefaultVersion を使う（runtime 内で補完）。
	Version string
	// 依存先到達性 probe（HealthService.Readiness で並列実行する）。
	// 例: state Pod → "dapr"、secret Pod → "openbao"、workflow Pod → "temporal" + "dapr"。
	// 空スライスなら ready=true / 依存なし扱い。
	Probes []k1s0health.DependencyProbe
	// 構造体定義を閉じる。
}

// DefaultVersion は Pod.Version 未指定時に HealthService.Liveness が返す既定値。
// 実バイナリでは ldflags で `-X` 注入する想定（IMP-BUILD 系の Go ビルドフラグ）。
const DefaultVersion = "0.1.0-dev"

// Run は引数 Pod の gRPC server を起動し、SIGINT / SIGTERM を受けて graceful shutdown まで完了させる。
func Run(p Pod, listen string) error {
	// 指定アドレスで TCP listener を確保する。失敗時は呼び出し元（main）が exit(1)。
	lis, err := net.Listen("tcp", listen)
	// listen 失敗（ポート競合 / 権限不足 / IP 解決失敗）は即時 return。
	if err != nil {
		// caller に error を返却する。
		return err
		// if 分岐を閉じる。
	}

	// gRPC server インスタンスを生成する。
	// 共通 interceptor チェイン（呼出順序が重要）:
	//   1. AuthInterceptor: JWT 検証 + L1 テナント上書き（共通規約 §「認証と認可」/§「マルチテナント分離」L1）。
	//      env TIER1_AUTH_MODE で off / hmac / jwks を切替。off は test / 早期 dev で pass-through。
	//   2. ObservabilityInterceptor: OTel 1 span + RED メトリクス（共通規約 §「通信プロトコルと可観測性」）。
	//      span は 3 番目の AuditInterceptor から trace_id / span_id を引くために 2 番目に置く。
	//   3. AuditInterceptor: 特権操作の自動 Audit 発行（共通規約 §「監査と痕跡」）。
	//      env TIER1_AUDIT_MODE が "log" 以外は NoopAuditEmitter で無効化（既存 test 互換）。
	// Provider / 認証鍵が未設定の dev / test では各 interceptor が no-op として動作するため、
	// cmd 側の setup を強制しない（既存テスト互換）。
	srv := grpc.NewServer(
		grpc.ChainUnaryInterceptor(
			AuthInterceptor(LoadAuthConfigFromEnv()),
			ObservabilityInterceptor(),
			AuditInterceptor(loadAuditEmitterFromEnv()),
		),
	)

	// 標準 gRPC health protocol を登録する。
	hs := health.NewServer()
	// 空文字 "" は service 全体（無指定）の status を意味する。SERVING で公開する。
	hs.SetServingStatus("", healthpb.HealthCheckResponse_SERVING)
	// gRPC server に health service を実装として登録する。
	healthpb.RegisterHealthServer(srv, hs)

	// k1s0 独自 HealthService（k1s0.tier1.health.v1）を登録する。
	// version 未指定時は DefaultVersion で補完する（ldflags で上書き想定）。
	version := p.Version
	// 補完ロジック。空文字のときのみ DefaultVersion を採用する。
	if version == "" {
		// DefaultVersion は本ファイル冒頭で宣言済。
		version = DefaultVersion
	}
	// HealthService 実装を生成する（起動時刻を内部で確定）。
	k1s0HealthService := k1s0health.New(version, p.Probes)
	// gRPC server に登録する。
	k1s0HealthService.Register(srv)

	// gRPC reflection を有効化する。
	// dev / staging では grpcurl 等での疎通確認に有用。production では config で無効化する設計（plan 04-02）。
	reflection.Register(srv)

	// Pod 固有 service の登録 hook を呼ぶ（リリース時点は no-op で nil 許容）。
	if p.Register != nil {
		// hook を呼び出して Pod 固有 handler を gRPC server に追加する。
		p.Register(srv)
		// if 分岐を閉じる。
	}

	// Serve のエラーをメイン goroutine に届けるバッファ付きチャネル。
	errCh := make(chan error, 1)
	// gRPC Serve は blocking なので別 goroutine で起動する。
	go func() {
		// 起動ログ（OTel logger 導入前の暫定）。
		log.Printf("tier1/%s: gRPC server listening on %s", p.Name, listen)
		// Serve は shutdown 時 nil を返すか、内部エラー時に non-nil を返す。
		errCh <- srv.Serve(lis)
		// goroutine の関数リテラルを閉じて即時起動する。
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
		log.Printf("tier1/%s: received signal %s, shutting down", p.Name, sig)
	// 異常停止経路: Serve が error を返した場合は即時 return（shutdown 不要）。
	case err := <-errCh:
		// Serve が non-nil error を返したら caller に伝搬。
		return err
		// 1 段目の select を閉じる。
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
		// graceful stop 監視 goroutine を閉じて即時起動する。
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
		log.Printf("tier1/%s: graceful shutdown complete", p.Name)
	// 異常経路: タイムアウトしたら強制停止に切替（既存 RPC を破棄）。
	case <-ctx.Done():
		// timeout ログを出力する。
		log.Printf("tier1/%s: graceful shutdown timeout, forcing stop", p.Name)
		// Stop は in-flight RPC を即座に切断する。
		srv.Stop()
		// 2 段目の select を閉じる。
	}
	// 正常終了。caller（main）は exit(0)。
	return nil
	// Run 関数を閉じる。
}
