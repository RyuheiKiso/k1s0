// 本ファイルは Secrets API の HTTP/JSON gateway 用 RPC ハンドラ adapter。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md §「HTTP/JSON 互換」
//
// 役割:
//   common.HTTPGateway.RegisterSecretsRoutes に渡す common.SecretsRPCHandlers を組み立てる。
//   Get / BulkGet / GetDynamic / Rotate の 4 RPC を protojson Unmarshal 経由で in-process
//   SecretsServiceServer に dispatch する。
//
// セキュリティ注意:
//   Secret 値は HTTP response body に protojson で乗るため、必ず TLS（mTLS / Istio Ambient
//   経由など）で経路を保護する運用とする。リリース時点 でも HTTP は単独で使わず、ワークロード
//   identity をもつ proxy 経由でのみアクセス可能にする（docs §「認証と認可」と整合）。

package secret

import (
	"context"

	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	secretsv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/secrets/v1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"
)

// MakeHTTPSecretsHandlers は HTTP/JSON gateway 用 Secrets handler 集合を組み立てる。
func MakeHTTPSecretsHandlers(secretsSvc secretsv1.SecretsServiceServer) common.SecretsRPCHandlers {
	return common.SecretsRPCHandlers{
		Get: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &secretsv1.GetSecretRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if secretsSvc == nil {
				return nil, status.Error(codes.Unavailable, "secrets service not wired")
			}
			return secretsSvc.Get(ctx, req)
		},
		BulkGet: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &secretsv1.BulkGetSecretRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if secretsSvc == nil {
				return nil, status.Error(codes.Unavailable, "secrets service not wired")
			}
			return secretsSvc.BulkGet(ctx, req)
		},
		GetDynamic: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &secretsv1.GetDynamicSecretRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if secretsSvc == nil {
				return nil, status.Error(codes.Unavailable, "secrets service not wired")
			}
			return secretsSvc.GetDynamic(ctx, req)
		},
		Rotate: func(ctx context.Context, body []byte) (proto.Message, error) {
			req := &secretsv1.RotateSecretRequest{}
			if err := common.UnmarshalJSON(body, req); err != nil {
				return nil, err
			}
			if secretsSvc == nil {
				return nil, status.Error(codes.Unavailable, "secrets service not wired")
			}
			return secretsSvc.Rotate(ctx, req)
		},
	}
}

// NewSecretsServiceServer は HTTP gateway / 統合テスト用に secretHandler を直接生成する exported helper。
func NewSecretsServiceServer(deps Deps) secretsv1.SecretsServiceServer {
	return &secretHandler{deps: deps}
}
