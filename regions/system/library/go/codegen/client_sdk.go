package codegen

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"text/template"
)

// ClientSDKConfig はクライアント SDK 生成の設定を保持する。
// 対象の proto ファイルと出力先を指定する。
type ClientSDKConfig struct {
	ProtoPath  string // .proto ファイルのパス
	OutputDir  string // 出力先ディレクトリ
	Language   string // "go", "typescript" など
	PackageURL string // Go モジュールパス（Go の場合のみ使用）
}

// ClientSDKResult はクライアント SDK 生成の結果を保持する。
type ClientSDKResult struct {
	CreatedFiles []string
}

// GenerateClientSDK は ProtoService の定義から各言語のクライアント SDK を生成する。
// proto ファイルを解析してサービスインターフェースと実装を出力する。
func GenerateClientSDK(cfg ClientSDKConfig) (*ClientSDKResult, error) {
	if cfg.ProtoPath == "" {
		return nil, fmt.Errorf("codegen: proto_path is required")
	}
	if cfg.OutputDir == "" {
		return nil, fmt.Errorf("codegen: output_dir is required")
	}

	// .proto ファイルを解析する
	svc, err := ParseProto(cfg.ProtoPath)
	if err != nil {
		return nil, fmt.Errorf("codegen: parse proto: %w", err)
	}

	result := &ClientSDKResult{}

	switch cfg.Language {
	case "go":
		if err := generateGoClientSDK(cfg, svc, result); err != nil {
			return nil, err
		}
	default:
		return nil, fmt.Errorf("codegen: unsupported client SDK language: %s", cfg.Language)
	}

	return result, nil
}

// generateGoClientSDK は Go 用クライアント SDK ファイルを生成する。
// インターフェース定義と gRPC 実装を別ファイルに出力する。
func generateGoClientSDK(cfg ClientSDKConfig, svc *ProtoService, result *ClientSDKResult) error {
	data := struct {
		Service    *ProtoService
		PackageURL string
		PkgName    string
	}{
		Service:    svc,
		PackageURL: cfg.PackageURL,
		PkgName:    strings.ToLower(svc.ServiceName) + "client",
	}

	files := map[string]string{
		"client.go": goClientInterfaceTemplate,
		"grpc.go":   goClientGRPCTemplate,
		"mock.go":   goClientMockTemplate,
	}

	for name, tmplStr := range files {
		outPath := filepath.Join(cfg.OutputDir, name)

		// 既存ファイルはスキップ（冪等性を保つ）
		if _, err := os.Stat(outPath); err == nil {
			continue
		}

		if err := os.MkdirAll(cfg.OutputDir, 0755); err != nil {
			return fmt.Errorf("codegen: mkdir %s: %w", cfg.OutputDir, err)
		}

		tmpl, err := template.New("").Parse(tmplStr)
		if err != nil {
			return fmt.Errorf("codegen: parse template %s: %w", name, err)
		}

		f, err := os.Create(outPath)
		if err != nil {
			return fmt.Errorf("codegen: create %s: %w", outPath, err)
		}
		defer f.Close()

		if err := tmpl.Execute(f, data); err != nil {
			return fmt.Errorf("codegen: execute template %s: %w", name, err)
		}

		result.CreatedFiles = append(result.CreatedFiles, outPath)
	}

	return nil
}

// goClientInterfaceTemplate は Go クライアントインターフェースのテンプレート。
var goClientInterfaceTemplate = `package {{.PkgName}}

import "context"

// {{.Service.ServiceName}}Client は {{.Service.ServiceName}} サービスのクライアントインターフェース。
// テスト時はモック実装に差し替えられる。
type {{.Service.ServiceName}}Client interface {
{{- range .Service.Methods}}
	// {{.Name}} はサービスメソッドを呼び出す。
	{{.Name}}(ctx context.Context, req *{{.InputType}}) (*{{.OutputType}}, error)
{{- end}}
}
`

// goClientGRPCTemplate は Go gRPC クライアント実装のテンプレート。
var goClientGRPCTemplate = `package {{.PkgName}}

import (
	"context"

	"google.golang.org/grpc"
)

// GRPC{{.Service.ServiceName}}Client は gRPC 経由で {{.Service.ServiceName}} を呼び出す実装。
type GRPC{{.Service.ServiceName}}Client struct {
	conn *grpc.ClientConn
}

// New{{.Service.ServiceName}}Client は gRPC 接続から {{.Service.ServiceName}}Client を生成する。
func New{{.Service.ServiceName}}Client(conn *grpc.ClientConn) {{.Service.ServiceName}}Client {
	return &GRPC{{.Service.ServiceName}}Client{conn: conn}
}
{{range .Service.Methods}}
// {{.Name}} は gRPC メソッドを呼び出す。
func (c *GRPC{{$.Service.ServiceName}}Client) {{.Name}}(ctx context.Context, req *{{.InputType}}) (*{{.OutputType}}, error) {
	// TODO: generated stub — implement with protobuf client
	return nil, nil
}
{{end}}`

// goClientMockTemplate は Go モッククライアントのテンプレート。
var goClientMockTemplate = `package {{.PkgName}}

import "context"

// Mock{{.Service.ServiceName}}Client はテスト用のモック実装。
type Mock{{.Service.ServiceName}}Client struct {
{{- range .Service.Methods}}
	{{.Name}}Fn func(ctx context.Context, req *{{.InputType}}) (*{{.OutputType}}, error)
{{- end}}
}
{{range .Service.Methods}}
// {{.Name}} は Mock{{$.Service.ServiceName}}Client.{{.Name}}Fn を呼び出す。
func (m *Mock{{$.Service.ServiceName}}Client) {{.Name}}(ctx context.Context, req *{{.InputType}}) (*{{.OutputType}}, error) {
	if m.{{.Name}}Fn != nil {
		return m.{{.Name}}Fn(ctx, req)
	}
	return nil, nil
}
{{end}}`
