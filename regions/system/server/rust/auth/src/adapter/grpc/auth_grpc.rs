// gRPC AuthService 実装。
// tonic proto コンパイル後に ValidateToken / GetUser / ListUsers /
// GetUserRoles / CheckPermission を実装する。
//
// proto ファイル: api/proto/k1s0/system/auth/v1/auth.proto
//
// 使用例:
// ```
// pub mod proto {
//     tonic::include_proto!("k1s0.system.auth.v1");
// }
// ```
