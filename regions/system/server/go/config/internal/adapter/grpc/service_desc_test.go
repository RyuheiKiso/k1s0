package grpc

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"google.golang.org/grpc"
)

func TestRegisterConfigServiceServer(t *testing.T) {
	s := grpc.NewServer()
	svc := &ConfigGRPCService{}

	// パニックしないことを確認
	assert.NotPanics(t, func() {
		RegisterConfigServiceServer(s, svc)
	})

	// サービス情報が登録されていることを確認
	serviceInfo := s.GetServiceInfo()
	info, ok := serviceInfo["k1s0.system.config.v1.ConfigService"]
	assert.True(t, ok, "ConfigService should be registered")

	// メソッド数を確認（WatchConfig はストリーミングなので Methods には含まれない。
	// 現時点では Unary メソッドのみ登録）
	assert.Len(t, info.Methods, 5, "ConfigService should have 5 unary methods")

	methodNames := make([]string, 0, len(info.Methods))
	for _, m := range info.Methods {
		methodNames = append(methodNames, m.Name)
	}
	assert.Contains(t, methodNames, "GetConfig")
	assert.Contains(t, methodNames, "ListConfigs")
	assert.Contains(t, methodNames, "GetServiceConfig")
	assert.Contains(t, methodNames, "UpdateConfig")
	assert.Contains(t, methodNames, "DeleteConfig")
}
