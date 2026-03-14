package codegen

import (
	"fmt"
	"os"
	"regexp"
	"strconv"
	"strings"
)

// ProtoService は .proto ファイルから解析されたサービス定義を保持する。
type ProtoService struct {
	Package     string
	ServiceName string
	Methods     []ProtoMethod
	Messages    []ProtoMessage
}

// ProtoMethod は gRPC メソッド定義を保持する。
type ProtoMethod struct {
	Name       string
	InputType  string
	OutputType string
}

// ProtoMessage は proto メッセージ定義を保持する。
type ProtoMessage struct {
	Name   string
	Fields []ProtoField
}

// ProtoField は proto フィールド定義を保持する。
type ProtoField struct {
	Name      string
	FieldType string
	Number    int
}

// ParseProto は .proto ファイルを解析して ProtoService を返す。
// package 宣言と service ブロックを抽出する。
func ParseProto(path string) (*ProtoService, error) {
	content, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("codegen: read proto file %s: %w", path, err)
	}
	return ParseProtoContent(string(content))
}

// ParseProtoContent は .proto 文字列を解析して ProtoService を返す。
// テスト時はファイル読み込みなしで直接呼び出せる。
func ParseProtoContent(content string) (*ProtoService, error) {
	pkgRe := regexp.MustCompile(`package\s+([\w.]+)\s*;`)
	svcRe := regexp.MustCompile(`service\s+(\w+)\s*\{([^}]*)\}`)
	rpcRe := regexp.MustCompile(`rpc\s+(\w+)\s*\(\s*(\w+)\s*\)\s*returns\s*\(\s*(\w+)\s*\)`)
	msgRe := regexp.MustCompile(`message\s+(\w+)\s*\{([^}]*)\}`)
	fieldRe := regexp.MustCompile(`(\w+)\s+(\w+)\s*=\s*(\d+)\s*;`)

	// package 宣言を取得する
	pkgMatch := pkgRe.FindStringSubmatch(content)
	if pkgMatch == nil {
		return nil, fmt.Errorf("codegen: missing package declaration in proto")
	}

	// service ブロックを取得する
	svcMatch := svcRe.FindStringSubmatch(content)
	if svcMatch == nil {
		return nil, fmt.Errorf("codegen: missing service declaration in proto")
	}
	serviceName := svcMatch[1]
	serviceBody := svcMatch[2]

	// rpc メソッドを抽出する
	var methods []ProtoMethod
	for _, m := range rpcRe.FindAllStringSubmatch(serviceBody, -1) {
		methods = append(methods, ProtoMethod{
			Name:       m[1],
			InputType:  m[2],
			OutputType: m[3],
		})
	}

	// message 定義を抽出する
	var messages []ProtoMessage
	for _, m := range msgRe.FindAllStringSubmatch(content, -1) {
		msgName := m[1]
		msgBody := m[2]

		var fields []ProtoField
		for _, f := range fieldRe.FindAllStringSubmatch(msgBody, -1) {
			num, _ := strconv.Atoi(f[3])
			fields = append(fields, ProtoField{
				FieldType: f[1],
				Name:      f[2],
				Number:    num,
			})
		}
		messages = append(messages, ProtoMessage{Name: msgName, Fields: fields})
	}

	return &ProtoService{
		Package:     pkgMatch[1],
		ServiceName: serviceName,
		Methods:     methods,
		Messages:    messages,
	}, nil
}

// ProtoPackageToGoPackage は proto パッケージ名 (例: "k1s0.system.auth.v1") を
// Go パッケージ名 (例: "authv1") に変換する。
func ProtoPackageToGoPackage(pkg string) string {
	parts := strings.Split(pkg, ".")
	if len(parts) == 0 {
		return pkg
	}
	return strings.Join(parts[len(parts)-2:], "")
}
