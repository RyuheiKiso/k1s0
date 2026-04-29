// 本ファイルは tier1 Go の **t1-secret Pod** の起動エントリポイント。
//
// 設計: docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/01_tier1全体配置.md
//       docs/02_構想設計/02_tier1設計/
// 関連 ID: IMP-DIR-002 / IMP-BUILD-002 / FR-T1-SECRETS / ADR-TIER1-001 / ADR-TIER1-003 / ADR-SEC-002
//
// 担当 API（docs 正典）:
//   - SecretsService（k1s0.tier1.secrets.v1）
//
// 注: Binding API は docs 正典で t1-state Pod 担当（5 API Router）。本 Pod は Secrets のみ。
//
// scope:
//   - :50001 で listen（flag で上書き、docs 正典 EXPOSE 50001）
//   - 標準 gRPC health protocol 応答 + reflection
//   - SIGINT / SIGTERM で graceful shutdown
//   - OpenBao adapter を環境変数で結線（OPENBAO_ADDR / OPENBAO_TOKEN / OPENBAO_KV_MOUNT）
//   - 環境変数未設定時は in-memory KVv2 backend で fallback（dev / CI 用途、process 内永続）
//
// production / dev / CI の挙動分岐:
//   - OPENBAO_ADDR が設定されている → 実 OpenBao server に gRPC 接続して KVv2 を使う
//   - OPENBAO_ADDR が未設定           → InMemoryKV backend で起動（process 内 永続のみ）

// パッケージ宣言。`go build ./cmd/secret` で t1-secret Pod 用バイナリを生成する。
package main

// 標準ライブラリと共通 runtime を import する。
import (
	// adapter 初期化に context を渡す。
	"context"
	// listen address を flag で受け取り、ConfigMap での上書きに備える。
	"flag"
	// 起動 / shutdown / エラーログを stderr に出す。
	"log"
	// HTTP/JSON 互換 gateway 用。
	"net/http"
	// 環境変数読出。
	"os"
	// HealthService.Readiness の probe ごと timeout 制御。
	"time"

	// OpenBao adapter（本 Pod 専用）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/openbao"
	// 共通ランタイム（gRPC bootstrap + health + graceful shutdown）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	// HealthService.Readiness 用 DependencyProbe 型。
	"github.com/k1s0/k1s0/src/tier1/go/internal/health"
	// t1-secret Pod の handler（SecretsService 単独）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/secret"
)

// :50001 は docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md（EXPOSE 50001）正典準拠。
const defaultListen = ":50001"

// HTTP/JSON 互換 gateway の既定 listen address。Secret は機密性が高いため、デフォルトでは
// Pod-internal 経路のみ提供する想定で 127.0.0.1 にバインドする運用を推奨（Service の
// containerPort も exposing しない）。flag / env で外向き port に上書き可能。
const defaultHTTPListen = "127.0.0.1:50081"

// プロセスエントリポイント。flag パース、OpenBao 結線、common.Run への委譲を行う。
func main() {
	// listen address の上書き flag を定義（既定 :50001）。
	addr := flag.String("listen", defaultListen, "gRPC server listen address")
	// HTTP/JSON 互換 gateway の listen address。空文字 / "off" で起動しない。
	httpAddr := flag.String("http-listen", defaultHTTPListen, "HTTP/JSON gateway listen address (empty or \"off\" disables)")
	// flag 解析を起動直後に確定させる。
	flag.Parse()

	// OpenBao Client を環境変数または in-memory backend で構築する。
	openBaoClient, err := newOpenBaoClient(context.Background())
	// 接続失敗（実 OpenBao 経路のみ）は即時 exit(1)。
	if err != nil {
		// 失敗ログを stderr に書く。
		log.Fatalf("t1-secret: openbao client init: %v", err)
	}
	// Pod 終了時に Client を解放する。
	defer func() {
		// Close エラーは ログのみ（exit code に影響させない）。
		if cerr := openBaoClient.Close(); cerr != nil {
			// 失敗を stderr に残す。
			log.Printf("t1-secret: openbao client close: %v", cerr)
		}
	}()

	// SecretsService が依存する adapter を構築する。
	// FR-T1-SECRETS-001 の "30 秒インメモリキャッシュ" を満たすため、
	// SecretsAdapter は CachedSecretsAdapter で wrap する。Rotate 成功時の
	// 同 secret latest invalidate も同 wrapper で担保。
	baseSecrets := openbao.NewSecretsAdapter(openBaoClient)
	// 動的 Secret 発行 adapter（FR-T1-SECRETS-002）。
	// production（OPENBAO_ADDR が設定された経路）では Logical().Read を呼ぶ
	// productionDynamic、dev / CI モードでは in-memory backend を使う。
	dynamicAdapter := newDynamicAdapter(openBaoClient)
	// 30 秒 TTL の cache 付き secrets adapter（FR-T1-SECRETS-001）。
	cachedSecrets := openbao.NewCachedSecretsAdapter(baseSecrets, 0)
	deps := secret.Deps{
		SecretsAdapter: cachedSecrets,
		// 動的 Secret adapter。
		DynamicAdapter: dynamicAdapter,
		// 共通規約 §「冪等性と再試行」: 24h TTL の in-memory idempotency cache を有効化。
		// production の multi-replica deploy では Valkey backed cache に置き換える想定だが、
		// release-initial では in-memory backend で 1 Pod 内 dedup を提供する。
		Idempotency: common.NewInMemoryIdempotencyCache(0),
	}

	// Secret 自動ローテーション（FR-T1-SECRETS-004）を起動する。
	// ROTATION_SCHEDULE 環境変数で per-secret cadence を指定する。
	rotator, err := secret.NewRotatorFromEnv(cachedSecrets, os.Getenv("ROTATION_SCHEDULE"))
	if err != nil {
		// 形式不正は fatal（誤設定を見逃さない）。
		log.Fatalf("t1-secret: invalid ROTATION_SCHEDULE: %v", err)
	}
	// rotator は context cancel で停止する。Pod 起動と同期させる context を渡す。
	rotatorCtx, rotatorCancel := context.WithCancel(context.Background())
	rotator.Start(rotatorCtx)
	defer func() {
		rotatorCancel()
		rotator.Stop()
	}()

	// HTTP/JSON 互換 gateway を別 goroutine で起動する（共通規約 §「HTTP/JSON 互換」）。
	httpServer := startSecretsHTTPGatewayIfEnabled(*httpAddr, deps)
	defer func() {
		if httpServer != nil {
			shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
			defer cancel()
			if err := httpServer.Shutdown(shutdownCtx); err != nil {
				log.Printf("t1-secret: http gateway shutdown: %v", err)
			}
		}
	}()

	// Pod メタデータを構築する（SecretsService 登録）。
	pod := common.Pod{
		// Pod 論理名。ログ出力で "tier1/secret" として表示される。
		Name: "secret",
		// 既定 listen address。
		DefaultListen: defaultListen,
		// service 登録 hook。SecretsService を登録する。
		Register: secret.Register(deps),
		// HealthService 用 Pod バージョン。release ビルドでは ldflags で上書きする想定。
		Version: common.DefaultVersion,
		// HealthService.Readiness で並列実行する依存先 probe。
		// secret Pod は OpenBao（KVv2 + Database Engine）に依存する。
		Probes: []health.DependencyProbe{
			{
				// dependencies map のキーは "openbao" を採用する。
				Name: "openbao",
				// OpenBao Client の到達性を 2 秒以内で確認する。
				Check: func(ctx context.Context) error {
					// 過剰な待機を避けるため probe ごとに timeout を 2 秒に絞る。
					checkCtx, cancel := context.WithTimeout(ctx, 2*time.Second)
					// 関数末尾で必ず cancel する。
					defer cancel()
					// in-memory backend は常に nil、production は KVv2 mount 直下のセンチネル Get で到達性確認。
					return openBaoClient.Ping(checkCtx)
				},
			},
		},
	}

	// 共通 runtime に委譲する。エラー時は log.Fatalf で非ゼロ終了する。
	if err := common.Run(pod, *addr); err != nil {
		// fatal log は stderr + exit(1) を 1 行で行う Go の慣用。
		log.Fatalf("t1-secret: %v", err)
	}
}

// startSecretsHTTPGatewayIfEnabled は HTTP/JSON 互換 gateway を別 goroutine で起動する。
// addr が空文字 / "off" なら起動せず nil を返す（純 gRPC 運用）。
//
// Secret API は機密性が高いため:
//   - 既定 bind address は 127.0.0.1 で Pod-internal 経路のみ exposing
//   - 本 gateway 単体では認証を行わず、Service Mesh（Istio Ambient mTLS）または
//     gRPC AuthInterceptor の前段配置で外部認証を担保する
//   - 共通規約 §「認証と認可」と整合
func startSecretsHTTPGatewayIfEnabled(addr string, deps secret.Deps) *http.Server {
	if addr == "" || addr == "off" {
		log.Printf("t1-secret: HTTP/JSON gateway disabled (--http-listen=%q)", addr)
		return nil
	}
	// gRPC server と同じ interceptor chain を HTTP gateway にも適用する。
	g := common.NewHTTPGateway().WithInterceptors(
		common.AuthInterceptor(common.LoadAuthConfigFromEnv()),
		common.RateLimitInterceptor(common.LoadRateLimitConfigFromEnv()),
		common.ObservabilityInterceptor(),
	)
	g.RegisterSecretsRoutes(secret.MakeHTTPSecretsHandlers(secret.NewSecretsServiceServer(deps)))
	srv := &http.Server{
		Addr:              addr,
		Handler:           g.Handler(),
		ReadHeaderTimeout: 5 * time.Second,
	}
	go func() {
		log.Printf("t1-secret: HTTP/JSON gateway listening on %s", addr)
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Printf("t1-secret: http gateway: %v", err)
		}
	}()
	return srv
}

// newDynamicAdapter は OPENBAO_ADDR の有無で production / in-memory backend を切り替えて
// 動的 Secret adapter（FR-T1-SECRETS-002）を返す。
//
// production: OpenBao の `<engine>/creds/<tenant>/<role>` を Logical().Read で叩き、
//             SDK が Database Engine 配下の動的 credential を都度発行する。
// dev / CI:   process 内 in-memory で username / password を crypto/rand から生成する。
func newDynamicAdapter(client *openbao.Client) openbao.DynamicAdapter {
	// OPENBAO_ADDR が設定されていれば production 経路を使う。
	if os.Getenv("OPENBAO_ADDR") != "" {
		// stderr に経路選択を 1 行ログする。
		log.Printf("t1-secret: dynamic secrets backend = OpenBao Database Engine (production)")
		// SDK Logical() narrow を持った adapter を返す。
		return openbao.NewProductionDynamic(client)
	}
	// 未設定時は in-memory backend に fallback する。
	log.Printf("t1-secret: dynamic secrets backend = in-memory (dev/CI mode)")
	return openbao.NewInMemoryDynamic()
}

// newOpenBaoClient は環境変数 OPENBAO_ADDR の有無で実 / in-memory を切替えて Client を生成する。
//
// 環境変数:
//   OPENBAO_ADDR     — OpenBao server URL（例: "https://openbao.k1s0-security.svc.cluster.local:8200"）
//   OPENBAO_TOKEN    — 認証 token（JWT / approle / kubernetes auth で別途取得済の値）
//   OPENBAO_KV_MOUNT — KV mount path（既定: "secret"）
//
// addr が空文字の場合は in-memory backend を起動する。dev / CI で外部依存なしに
// `go run cmd/secret` で gRPC ハンドラが実値（in-memory KV 操作の結果）を返せる。
func newOpenBaoClient(ctx context.Context) (*openbao.Client, error) {
	// addr が未設定なら in-memory backend を起動する。
	addr := os.Getenv("OPENBAO_ADDR")
	// addr 未設定時は in-memory に fallback する。
	if addr == "" {
		// stderr に in-memory モード起動を 1 行ログする。
		log.Printf("t1-secret: OPENBAO_ADDR not set, using in-memory KVv2 backend (dev/CI mode)")
		// in-memory backend を返却する。
		return openbao.NewClientWithInMemoryKV(), nil
	}
	// 実 OpenBao server に接続する。
	return openbao.New(ctx, openbao.Config{
		// server URL。
		Address: addr,
		// 認証トークン。
		Token: os.Getenv("OPENBAO_TOKEN"),
		// KV mount path（既定 "secret"）。
		KVMount: os.Getenv("OPENBAO_KV_MOUNT"),
	})
}
