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
	// 環境変数読出。
	"os"

	// OpenBao adapter（本 Pod 専用）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/openbao"
	// 共通ランタイム（gRPC bootstrap + health + graceful shutdown）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	// t1-secret Pod の handler（SecretsService 単独）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/secret"
)

// :50001 は docs/05_実装/00_ディレクトリ設計/20_tier1レイアウト/03_go_module配置.md（EXPOSE 50001）正典準拠。
const defaultListen = ":50001"

// プロセスエントリポイント。flag パース、OpenBao 結線、common.Run への委譲を行う。
func main() {
	// listen address の上書き flag を定義（既定 :50001）。
	addr := flag.String("listen", defaultListen, "gRPC server listen address")
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
	deps := secret.Deps{
		// OpenBao Client から SecretsAdapter を生成する。
		SecretsAdapter: openbao.NewSecretsAdapter(openBaoClient),
	}

	// Pod メタデータを構築する（SecretsService 登録）。
	pod := common.Pod{
		// Pod 論理名。ログ出力で "tier1/secret" として表示される。
		Name: "secret",
		// 既定 listen address。
		DefaultListen: defaultListen,
		// service 登録 hook。SecretsService を登録する。
		Register: secret.Register(deps),
	}

	// 共通 runtime に委譲する。エラー時は log.Fatalf で非ゼロ終了する。
	if err := common.Run(pod, *addr); err != nil {
		// fatal log は stderr + exit(1) を 1 行で行う Go の慣用。
		log.Fatalf("t1-secret: %v", err)
	}
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
