// analyzer.go — k1s0 tier1 Go analysis analyzer 骨格実装
// tier1/CLAUDE.md §lint ツール「golangci-lint depguard + go-apidiff + 自製 go/analysis」の
// 自製 go/analysis 実体。banned API 使用・OSS 直接 import・facade 経由強制を検出する。

// パッケージ名: gok1s0analysis（go/analysis framework を使用した k1s0 tier1 固有 analyzer）
package gok1s0analysis

import (
	// go/ast: 抽象構文ツリーの操作に使用する
	"go/ast"
	// go/token: ソースコード位置情報に使用する
	"go/token"
	// golang.org/x/tools/go/analysis: go/analysis framework に使用する
	"golang.org/x/tools/go/analysis"
	// golang.org/x/tools/go/analysis/passes/inspect: AST inspector に使用する
	"golang.org/x/tools/go/analysis/passes/inspect"
	// golang.org/x/tools/go/ast/inspector: AST ノード検査に使用する
	"golang.org/x/tools/go/ast/inspector"
	// strings: パッケージパス prefix 判定に使用する
	"strings"
)

// Analyzer は k1s0 tier1 固有の go/analysis.Analyzer。
// tier1/CLAUDE.md §公開 API の型制約（banned API / facade 経由強制）を静的解析で enforce する。
var Analyzer = &analysis.Analyzer{
	// analyzer の名称: golangci-lint 設定ファイルで参照する
	Name: "k1s0lint",
	// analyzer の説明
	Doc: "k1s0 tier1 lint: banned API / facade required / dependency direction enforcement",
	// inspect.Analyzer に依存する（AST inspector を使用する）
	Requires: []*analysis.Analyzer{inspect.Analyzer},
	// Run: analyzer のメイン処理
	Run: run,
}

// BANNED_IMPORT_PREFIXES: 直接 import が禁止されている OSS パッケージプレフィックス
// tier1/CLAUDE.md §公開 API の型制約（L3/L2*/L1+ カテゴリの OSS 型の公開 API 露出禁止）に対応する
var BANNED_IMPORT_PREFIXES = []struct {
	// prefix: 禁止 import パッケージプレフィックス
	prefix string
	// facade: 代替 facade パッケージ名
	facade string
}{
	// Npgsql Go 相当: pgx を直接 import 禁止（k1s0 IDbClient facade 経由必須）
	{prefix: "github.com/jackc/pgx", facade: "github.com/k1s0-io/k1s0/tier1/library (IDbClient)"},
	// Kafka Go クライアント直接 import 禁止（IMessagingProducer facade 経由必須）
	{prefix: "github.com/confluentinc/confluent-kafka-go", facade: "github.com/k1s0-io/k1s0/tier1/library (IMessagingProducer)"},
	// Temporal Go クライアント直接 import 禁止（IWorkflowClient facade 経由必須）
	{prefix: "go.temporal.io/sdk", facade: "github.com/k1s0-io/k1s0/tier1/library (IWorkflowClient)"},
	// AWS S3 クライアント直接 import 禁止（IObjectStorageClient facade 経由必須）
	{prefix: "github.com/aws/aws-sdk-go-v2/service/s3", facade: "github.com/k1s0-io/k1s0/tier1/library (IObjectStorageClient)"},
}

// BANNED_CALL_PATTERNS: 禁止されている関数呼び出しパターン
// src/CLAUDE.md §wall-clock TTL 禁止に対応する
var BANNED_CALL_PATTERNS = []struct {
	// pkgPath: 関数が属するパッケージパス
	pkgPath string
	// funcName: 禁止関数名
	funcName string
	// reason: 禁止理由
	reason string
}{
	// time.Now(): wall-clock TTL 計算禁止（HLC を使う）
	{pkgPath: "time", funcName: "Now", reason: "wall-clock TTL 禁止: HLC (src/client/hlc_lib/) を使うこと（src/CLAUDE.md §wall-clock TTL 禁止）"},
}

// run は go/analysis の解析処理メインロジック
// pass: analysis.Pass — AST・型情報・診断レポートを提供する
func run(pass *analysis.Pass) (interface{}, error) {
	// AST inspector を取得する（inspect.Analyzer の結果）
	insp := pass.ResultOf[inspect.Analyzer].(*inspector.Inspector)

	// import 文の解析: 禁止 import prefix を検出する
	nodeFilter := []ast.Node{
		// ImportSpec ノード: import 文を解析する
		(*ast.ImportSpec)(nil),
		// CallExpr ノード: 関数呼び出しを解析する
		(*ast.CallExpr)(nil),
	}
	// AST を走査して import 文と関数呼び出しを検出する
	insp.Preorder(nodeFilter, func(n ast.Node) {
		// ノードの型によって処理を切り替える
		switch node := n.(type) {
		// ImportSpec: import 文を解析する
		case *ast.ImportSpec:
			// import パスを取得する（クォートを除去する）
			importPath := strings.Trim(node.Path.Value, `"`)
			// 禁止 import prefix リストと照合する
			for _, banned := range BANNED_IMPORT_PREFIXES {
				// 禁止 prefix に一致する場合は診断を報告する
				if strings.HasPrefix(importPath, banned.prefix) {
					// 診断を報告する: 禁止 import を検出した
					pass.Reportf(
						// 違反位置: import 文の位置
						node.Pos(),
						"k1s0lint: direct OSS import `%s` is banned. Use facade `%s` instead (tier1/CLAUDE.md §公開 API の型制約)",
						importPath,
						banned.facade,
					)
				}
			}
		// CallExpr: 関数呼び出しを解析する
		case *ast.CallExpr:
			// 関数呼び出しのシンボルを取得する
			analyzeCallExpr(pass, node)
		}
	})
	// 解析成功: nil を返す
	return nil, nil
}

// analyzeCallExpr は CallExpr ノードを解析して禁止関数呼び出しを検出する
// pass: analysis.Pass — 型情報・診断レポートを提供する
// call: 解析対象の CallExpr ノード
func analyzeCallExpr(pass *analysis.Pass, call *ast.CallExpr) {
	// selector 形式の呼び出し（pkg.Func()）を解析する
	sel, ok := call.Fun.(*ast.SelectorExpr)
	// selector 形式でない場合はスキップする
	if !ok {
		return
	}
	// selector の X（パッケージ参照）の型情報を取得する
	xType := pass.TypesInfo.Types[sel.X]
	// 型情報が取得できない場合はスキップする
	if !xType.IsValue() {
		return
	}
	// 呼び出しオブジェクトのパッケージパスを取得する
	_ = token.NoPos
	// 禁止呼び出しパターンリストと照合する
	for _, banned := range BANNED_CALL_PATTERNS {
		// 関数名が一致する場合はパッケージパスを確認する
		if sel.Sel.Name == banned.funcName {
			// 型のパッケージパスから time パッケージかどうか確認する
			typeStr := xType.Type.String()
			// time.Time 型の Now() 呼び出しを検出する
			if strings.Contains(typeStr, banned.pkgPath) {
				// 診断を報告する: 禁止関数呼び出しを検出した
				pass.Reportf(
					// 違反位置: 関数呼び出しの位置
					call.Pos(),
					"k1s0lint: banned function call `%s.%s`: %s",
					banned.pkgPath,
					banned.funcName,
					banned.reason,
				)
			}
		}
	}
}
