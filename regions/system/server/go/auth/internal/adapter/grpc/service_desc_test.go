package grpc

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"google.golang.org/grpc"
)

func TestRegisterAuthServiceServer(t *testing.T) {
	s := grpc.NewServer()
	svc := &AuthGRPCService{}

	// パニックしないことを確認
	assert.NotPanics(t, func() {
		RegisterAuthServiceServer(s, svc)
	})

	// サービス情報が登録されていることを確認
	serviceInfo := s.GetServiceInfo()
	info, ok := serviceInfo["k1s0.system.auth.v1.AuthService"]
	assert.True(t, ok, "AuthService should be registered")

	// メソッド数を確認
	assert.Len(t, info.Methods, 5, "AuthService should have 5 methods")

	methodNames := make([]string, 0, len(info.Methods))
	for _, m := range info.Methods {
		methodNames = append(methodNames, m.Name)
	}
	assert.Contains(t, methodNames, "ValidateToken")
	assert.Contains(t, methodNames, "GetUser")
	assert.Contains(t, methodNames, "ListUsers")
	assert.Contains(t, methodNames, "GetUserRoles")
	assert.Contains(t, methodNames, "CheckPermission")
}

func TestRegisterAuditServiceServer(t *testing.T) {
	s := grpc.NewServer()
	svc := &AuditGRPCService{}

	assert.NotPanics(t, func() {
		RegisterAuditServiceServer(s, svc)
	})

	serviceInfo := s.GetServiceInfo()
	info, ok := serviceInfo["k1s0.system.auth.v1.AuditService"]
	assert.True(t, ok, "AuditService should be registered")

	assert.Len(t, info.Methods, 2, "AuditService should have 2 methods")

	methodNames := make([]string, 0, len(info.Methods))
	for _, m := range info.Methods {
		methodNames = append(methodNames, m.Name)
	}
	assert.Contains(t, methodNames, "RecordAuditLog")
	assert.Contains(t, methodNames, "SearchAuditLogs")
}

func TestRegisterBothServices(t *testing.T) {
	s := grpc.NewServer()

	authSvc := &AuthGRPCService{}
	auditSvc := &AuditGRPCService{}

	// 両方のサービスを同じサーバーに登録してもパニックしないこと
	assert.NotPanics(t, func() {
		RegisterAuthServiceServer(s, authSvc)
		RegisterAuditServiceServer(s, auditSvc)
	})

	serviceInfo := s.GetServiceInfo()
	assert.Contains(t, serviceInfo, "k1s0.system.auth.v1.AuthService")
	assert.Contains(t, serviceInfo, "k1s0.system.auth.v1.AuditService")
}
