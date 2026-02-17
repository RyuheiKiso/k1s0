package grpc

// proto 生成コードが未生成のため、gRPC サービス記述子を手動定義する。
// buf generate 後にこのファイルは生成コードの RegisterXxxServiceServer に置き換える。
//
// 手動型 (types.go) は proto.Message を実装していないため、
// 標準的な protobuf コーデックでは decode/encode できない。
// ここでは encoding/json ベースのカスタムコーデックを使用して
// gRPC フレームワーク上で手動型を直接やり取りする。

import (
	"context"
	"encoding/json"

	"google.golang.org/grpc"
	"google.golang.org/grpc/encoding"
)

func init() {
	// JSON コーデックを "json" 名で登録。
	// クライアントが content-type: application/grpc+json を使う場合に有効。
	encoding.RegisterCodec(JSONCodec{})
}

// JSONCodec は JSON ベースの gRPC コーデック。
// proto 生成コードが無い状態でも gRPC サービスを動作させるために使用する。
type JSONCodec struct{}

func (JSONCodec) Marshal(v interface{}) ([]byte, error) {
	return json.Marshal(v)
}

func (JSONCodec) Unmarshal(data []byte, v interface{}) error {
	return json.Unmarshal(data, v)
}

func (JSONCodec) Name() string { return "json" }

// RegisterAuthServiceServer は AuthGRPCService を gRPC サーバーに登録する。
// proto 生成コードが揃った時点で生成コードの Register 関数に置き換えること。
func RegisterAuthServiceServer(s *grpc.Server, svc *AuthGRPCService) {
	s.RegisterService(&_AuthService_serviceDesc, svc)
}

// RegisterAuditServiceServer は AuditGRPCService を gRPC サーバーに登録する。
func RegisterAuditServiceServer(s *grpc.Server, svc *AuditGRPCService) {
	s.RegisterService(&_AuditService_serviceDesc, svc)
}

// _AuthService_serviceDesc は AuthService の gRPC サービス記述子。
var _AuthService_serviceDesc = grpc.ServiceDesc{
	ServiceName: "k1s0.system.auth.v1.AuthService",
	HandlerType: (*AuthServiceServer)(nil),
	Methods: []grpc.MethodDesc{
		{
			MethodName: "ValidateToken",
			Handler:    _AuthService_ValidateToken_Handler,
		},
		{
			MethodName: "GetUser",
			Handler:    _AuthService_GetUser_Handler,
		},
		{
			MethodName: "ListUsers",
			Handler:    _AuthService_ListUsers_Handler,
		},
		{
			MethodName: "GetUserRoles",
			Handler:    _AuthService_GetUserRoles_Handler,
		},
		{
			MethodName: "CheckPermission",
			Handler:    _AuthService_CheckPermission_Handler,
		},
	},
	Streams:  []grpc.StreamDesc{},
	Metadata: "v1/auth.proto",
}

// _AuditService_serviceDesc は AuditService の gRPC サービス記述子。
var _AuditService_serviceDesc = grpc.ServiceDesc{
	ServiceName: "k1s0.system.auth.v1.AuditService",
	HandlerType: (*AuditServiceServer)(nil),
	Methods: []grpc.MethodDesc{
		{
			MethodName: "RecordAuditLog",
			Handler:    _AuditService_RecordAuditLog_Handler,
		},
		{
			MethodName: "SearchAuditLogs",
			Handler:    _AuditService_SearchAuditLogs_Handler,
		},
	},
	Streams:  []grpc.StreamDesc{},
	Metadata: "v1/auth.proto",
}

// AuthServiceServer は gRPC AuthService のサーバーインターフェース。
type AuthServiceServer interface {
	ValidateToken(ctx context.Context, req *ValidateTokenRequest) (*ValidateTokenResponse, error)
	GetUser(ctx context.Context, req *GetUserRequest) (*GetUserResponse, error)
	ListUsers(ctx context.Context, req *ListUsersRequest) (*ListUsersResponse, error)
	GetUserRoles(ctx context.Context, req *GetUserRolesRequest) (*GetUserRolesResponse, error)
	CheckPermission(ctx context.Context, req *CheckPermissionRequest) (*CheckPermissionResponse, error)
}

// AuditServiceServer は gRPC AuditService のサーバーインターフェース。
type AuditServiceServer interface {
	RecordAuditLog(ctx context.Context, req *RecordAuditLogRequest) (*RecordAuditLogResponse, error)
	SearchAuditLogs(ctx context.Context, req *SearchAuditLogsRequest) (*SearchAuditLogsResponse, error)
}

// --- AuthService Handlers ---

func _AuthService_ValidateToken_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	req := new(ValidateTokenRequest)
	if err := dec(req); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(AuthServiceServer).ValidateToken(ctx, req)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/k1s0.system.auth.v1.AuthService/ValidateToken",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(AuthServiceServer).ValidateToken(ctx, req.(*ValidateTokenRequest))
	}
	return interceptor(ctx, req, info, handler)
}

func _AuthService_GetUser_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	req := new(GetUserRequest)
	if err := dec(req); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(AuthServiceServer).GetUser(ctx, req)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/k1s0.system.auth.v1.AuthService/GetUser",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(AuthServiceServer).GetUser(ctx, req.(*GetUserRequest))
	}
	return interceptor(ctx, req, info, handler)
}

func _AuthService_ListUsers_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	req := new(ListUsersRequest)
	if err := dec(req); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(AuthServiceServer).ListUsers(ctx, req)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/k1s0.system.auth.v1.AuthService/ListUsers",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(AuthServiceServer).ListUsers(ctx, req.(*ListUsersRequest))
	}
	return interceptor(ctx, req, info, handler)
}

func _AuthService_GetUserRoles_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	req := new(GetUserRolesRequest)
	if err := dec(req); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(AuthServiceServer).GetUserRoles(ctx, req)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/k1s0.system.auth.v1.AuthService/GetUserRoles",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(AuthServiceServer).GetUserRoles(ctx, req.(*GetUserRolesRequest))
	}
	return interceptor(ctx, req, info, handler)
}

func _AuthService_CheckPermission_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	req := new(CheckPermissionRequest)
	if err := dec(req); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(AuthServiceServer).CheckPermission(ctx, req)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/k1s0.system.auth.v1.AuthService/CheckPermission",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(AuthServiceServer).CheckPermission(ctx, req.(*CheckPermissionRequest))
	}
	return interceptor(ctx, req, info, handler)
}

// --- AuditService Handlers ---

func _AuditService_RecordAuditLog_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	req := new(RecordAuditLogRequest)
	if err := dec(req); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(AuditServiceServer).RecordAuditLog(ctx, req)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/k1s0.system.auth.v1.AuditService/RecordAuditLog",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(AuditServiceServer).RecordAuditLog(ctx, req.(*RecordAuditLogRequest))
	}
	return interceptor(ctx, req, info, handler)
}

func _AuditService_SearchAuditLogs_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	req := new(SearchAuditLogsRequest)
	if err := dec(req); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(AuditServiceServer).SearchAuditLogs(ctx, req)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/k1s0.system.auth.v1.AuditService/SearchAuditLogs",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(AuditServiceServer).SearchAuditLogs(ctx, req.(*SearchAuditLogsRequest))
	}
	return interceptor(ctx, req, info, handler)
}
