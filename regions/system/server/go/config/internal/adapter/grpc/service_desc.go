package grpc

// proto 生成コードが未生成のため、gRPC サービス記述子を手動定義する。
// buf generate 後にこのファイルは生成コードの RegisterConfigServiceServer に置き換える。

import (
	"context"
	"encoding/json"

	"google.golang.org/grpc"
	"google.golang.org/grpc/encoding"
)

func init() {
	encoding.RegisterCodec(JSONCodec{})
}

// JSONCodec は JSON ベースの gRPC コーデック。
type JSONCodec struct{}

func (JSONCodec) Marshal(v interface{}) ([]byte, error) {
	return json.Marshal(v)
}

func (JSONCodec) Unmarshal(data []byte, v interface{}) error {
	return json.Unmarshal(data, v)
}

func (JSONCodec) Name() string { return "json" }

// ConfigServiceServer は gRPC ConfigService のサーバーインターフェース。
type ConfigServiceServer interface {
	GetConfig(ctx context.Context, req *GetConfigRequest) (*GetConfigResponse, error)
	ListConfigs(ctx context.Context, req *ListConfigsRequest) (*ListConfigsResponse, error)
	GetServiceConfig(ctx context.Context, req *GetServiceConfigRequest) (*GetServiceConfigResponse, error)
	UpdateConfig(ctx context.Context, req *UpdateConfigRequest) (*UpdateConfigResponse, error)
	DeleteConfig(ctx context.Context, req *DeleteConfigRequest) (*DeleteConfigResponse, error)
	WatchConfig(ctx context.Context, req *WatchConfigRequest) (*WatchConfigResponse, error)
}

// RegisterConfigServiceServer は ConfigGRPCService を gRPC サーバーに登録する。
func RegisterConfigServiceServer(s *grpc.Server, svc *ConfigGRPCService) {
	s.RegisterService(&_ConfigService_serviceDesc, svc)
}

var _ConfigService_serviceDesc = grpc.ServiceDesc{
	ServiceName: "k1s0.system.config.v1.ConfigService",
	HandlerType: (*ConfigServiceServer)(nil),
	Methods: []grpc.MethodDesc{
		{
			MethodName: "GetConfig",
			Handler:    _ConfigService_GetConfig_Handler,
		},
		{
			MethodName: "ListConfigs",
			Handler:    _ConfigService_ListConfigs_Handler,
		},
		{
			MethodName: "GetServiceConfig",
			Handler:    _ConfigService_GetServiceConfig_Handler,
		},
		{
			MethodName: "UpdateConfig",
			Handler:    _ConfigService_UpdateConfig_Handler,
		},
		{
			MethodName: "DeleteConfig",
			Handler:    _ConfigService_DeleteConfig_Handler,
		},
	},
	Streams:  []grpc.StreamDesc{},
	Metadata: "v1/config.proto",
}

// --- ConfigService Handlers ---

func _ConfigService_GetConfig_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	req := new(GetConfigRequest)
	if err := dec(req); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(ConfigServiceServer).GetConfig(ctx, req)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/k1s0.system.config.v1.ConfigService/GetConfig",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(ConfigServiceServer).GetConfig(ctx, req.(*GetConfigRequest))
	}
	return interceptor(ctx, req, info, handler)
}

func _ConfigService_ListConfigs_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	req := new(ListConfigsRequest)
	if err := dec(req); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(ConfigServiceServer).ListConfigs(ctx, req)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/k1s0.system.config.v1.ConfigService/ListConfigs",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(ConfigServiceServer).ListConfigs(ctx, req.(*ListConfigsRequest))
	}
	return interceptor(ctx, req, info, handler)
}

func _ConfigService_GetServiceConfig_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	req := new(GetServiceConfigRequest)
	if err := dec(req); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(ConfigServiceServer).GetServiceConfig(ctx, req)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/k1s0.system.config.v1.ConfigService/GetServiceConfig",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(ConfigServiceServer).GetServiceConfig(ctx, req.(*GetServiceConfigRequest))
	}
	return interceptor(ctx, req, info, handler)
}

func _ConfigService_UpdateConfig_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	req := new(UpdateConfigRequest)
	if err := dec(req); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(ConfigServiceServer).UpdateConfig(ctx, req)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/k1s0.system.config.v1.ConfigService/UpdateConfig",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(ConfigServiceServer).UpdateConfig(ctx, req.(*UpdateConfigRequest))
	}
	return interceptor(ctx, req, info, handler)
}

func _ConfigService_DeleteConfig_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	req := new(DeleteConfigRequest)
	if err := dec(req); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(ConfigServiceServer).DeleteConfig(ctx, req)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/k1s0.system.config.v1.ConfigService/DeleteConfig",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(ConfigServiceServer).DeleteConfig(ctx, req.(*DeleteConfigRequest))
	}
	return interceptor(ctx, req, info, handler)
}
