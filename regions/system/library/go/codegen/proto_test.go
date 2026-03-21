// proto_test.go: ParseProtoContent / ParseProto のユニットテスト。
// proto 文字列解析の正常系・異常系を検証する。
package codegen_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	codegen "github.com/k1s0-platform/system-library-go-codegen"
)

// sampleProtoContent は正常系テスト用の proto 文字列。
const sampleProtoContent = `
syntax = "proto3";

package k1s0.system.auth.v1;

service AuthService {
  rpc Login(LoginRequest) returns (LoginResponse);
  rpc Logout(LogoutRequest) returns (LogoutResponse);
}

message LoginRequest {
  string email = 1;
  string password = 2;
}

message LoginResponse {
  string token = 1;
}

message LogoutRequest {
  string token = 1;
}

message LogoutResponse {
}
`

// TestParseProtoContent_Basic は正常な proto 文字列を解析して期待通りの結果が得られることを確認する。
func TestParseProtoContent_Basic(t *testing.T) {
	svc, err := codegen.ParseProtoContent(sampleProtoContent)
	require.NoError(t, err)
	require.NotNil(t, svc)

	// パッケージ名が正しく解析されること。
	assert.Equal(t, "k1s0.system.auth.v1", svc.Package)
	// サービス名が正しく解析されること。
	assert.Equal(t, "AuthService", svc.ServiceName)
	// rpc メソッドが2件解析されること。
	assert.Len(t, svc.Methods, 2)
	assert.Equal(t, "Login", svc.Methods[0].Name)
	assert.Equal(t, "LoginRequest", svc.Methods[0].InputType)
	assert.Equal(t, "LoginResponse", svc.Methods[0].OutputType)
	assert.Equal(t, "Logout", svc.Methods[1].Name)
	// メッセージ定義が解析されること。
	assert.NotEmpty(t, svc.Messages)
}

// TestParseProtoContent_MissingPackage は package 宣言がない場合にエラーが返ることを確認する。
func TestParseProtoContent_MissingPackage(t *testing.T) {
	content := `
service FooService {
  rpc Bar(BarRequest) returns (BarResponse);
}
`
	_, err := codegen.ParseProtoContent(content)
	require.Error(t, err)
	// エラーメッセージに package に関する情報が含まれること。
	assert.Contains(t, err.Error(), "package")
}

// TestParseProtoContent_MissingService は service 宣言がない場合にエラーが返ることを確認する。
func TestParseProtoContent_MissingService(t *testing.T) {
	content := `
syntax = "proto3";
package mypackage;

message FooMessage {
  string name = 1;
}
`
	_, err := codegen.ParseProtoContent(content)
	require.Error(t, err)
	// エラーメッセージに service に関する情報が含まれること。
	assert.Contains(t, err.Error(), "service")
}

// TestParseProto_FileRead は ParseProto がファイルを読み込んで正常に解析できることを確認する。
func TestParseProto_FileRead(t *testing.T) {
	// 一時 proto ファイルを生成してファイルパス経由での解析をテストする。
	dir := t.TempDir()
	protoPath := filepath.Join(dir, "test.proto")
	err := os.WriteFile(protoPath, []byte(sampleProtoContent), 0644)
	require.NoError(t, err)

	svc, err := codegen.ParseProto(protoPath)
	require.NoError(t, err)
	assert.Equal(t, "AuthService", svc.ServiceName)
}

// TestProtoPackageToGoPackage は proto パッケージ名を Go パッケージ名に変換することを確認する。
// proto パッケージは常に複数セグメント形式（例: k1s0.system.auth.v1）であることを前提とする。
func TestProtoPackageToGoPackage(t *testing.T) {
	// テーブルドリブンで複数パターンを検証する。
	tests := []struct {
		input    string
		expected string
	}{
		{"k1s0.system.auth.v1", "authv1"},
		{"k1s0.service.order.v1", "orderv1"},
		{"k1s0.business.payment.v1", "paymentv1"},
	}
	for _, tc := range tests {
		t.Run(tc.input, func(t *testing.T) {
			got := codegen.ProtoPackageToGoPackage(tc.input)
			assert.Equal(t, tc.expected, got)
		})
	}
}
