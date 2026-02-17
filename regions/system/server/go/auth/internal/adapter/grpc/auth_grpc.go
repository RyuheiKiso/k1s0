package grpc

// AuthGRPCService は gRPC AuthService の実装。
// 本番実装では protobuf 生成コードと連携する。
// gRPC サービス定義は api/proto/k1s0/system/auth/v1/auth.proto を参照。
//
// 現時点ではスタブ実装。
// proto コンパイル後に pb.UnimplementedAuthServiceServer を埋め込む。
type AuthGRPCService struct {
	// pb.UnimplementedAuthServiceServer
}
