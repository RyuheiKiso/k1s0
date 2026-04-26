// 本ファイルは t1-secret Pod が gRPC server に登録する SecretsService の handler。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-006（t1-secret: Active 1 / standby 2、HPA 禁止）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/04_Secrets_API.md
//
// 役割（リリース時点 最小骨格）:
//   SecretsService の 3 RPC（Get / BulkGet / Rotate）を登録する。
//   実 OpenBao 接続は plan 04-06 で実装。本リリース時点 では全 RPC が codes.Unimplemented を返す。
//
// 注: t1-secret は Dapr 経由ではなく OpenBao と直接連携する設計（DS-SW-COMP-006）。
//     OpenBao client adapter（internal/adapter/openbao/）の配置はリリース時点 対象外、
//     plan 04-06 で internal/adapter/openbao/openbao.go を追加する。

// Package secret は t1-secret Pod が登録する SecretsService の handler を提供する。
package secret

// 標準 / 内部パッケージ。
import (
	// context 伝搬。
	"context"
	// SDK 生成 stub の SecretsService 型。
	secretsv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/secrets/v1"
	// gRPC server 型。
	"google.golang.org/grpc"
	// gRPC エラーコード。
	"google.golang.org/grpc/codes"
	// gRPC ステータスエラー。
	"google.golang.org/grpc/status"
)

// secretHandler は SecretsService の handler 実装。
type secretHandler struct {
	// 将来 RPC 用埋め込み。
	secretsv1.UnimplementedSecretsServiceServer
}

// Register は SecretsService を gRPC server に登録する hook を返す。
// common.Pod.Register に渡せるシグネチャ。
func Register() func(*grpc.Server) {
	// closure で handler を捕捉する。
	return func(srv *grpc.Server) {
		// SecretsService を登録する（FR-T1-SECRETS-001〜004）。
		secretsv1.RegisterSecretsServiceServer(srv, &secretHandler{})
	}
}

// Get は単一シークレット取得（plan 04-06 で OpenBao 結線）。
func (h *secretHandler) Get(_ context.Context, _ *secretsv1.GetSecretRequest) (*secretsv1.GetSecretResponse, error) {
	// リリース時点 placeholder。
	return nil, status.Error(codes.Unimplemented, "tier1/secrets: Get not yet wired to OpenBao (plan 04-06)")
}

// BulkGet は一括取得（plan 04-06 で OpenBao 結線）。
func (h *secretHandler) BulkGet(_ context.Context, _ *secretsv1.BulkGetSecretRequest) (*secretsv1.BulkGetSecretResponse, error) {
	// リリース時点 placeholder。
	return nil, status.Error(codes.Unimplemented, "tier1/secrets: BulkGet not yet wired to OpenBao (plan 04-06)")
}

// Rotate はローテーション実行（plan 04-06 で OpenBao 結線、Leader Election 必須）。
func (h *secretHandler) Rotate(_ context.Context, _ *secretsv1.RotateSecretRequest) (*secretsv1.RotateSecretResponse, error) {
	// リリース時点 placeholder。
	return nil, status.Error(codes.Unimplemented, "tier1/secrets: Rotate not yet wired to OpenBao (plan 04-06, requires Leader Election)")
}
