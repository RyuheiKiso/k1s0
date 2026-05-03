// tests/e2e/user/helpers/example_runner.go
//
// examples/ で使う example binary の起動 + cleanup helper。
// examples/<type>/ 配下の binary（あるいは container image）を起動し、
// SDK 経由で叩く準備を整える。
//
// 設計正典:
//   docs/05_実装/30_CI_CD設計/35_e2e_test_design/20_user_suite/02_ディレクトリ構造.md
package helpers

import (
	"context"
	"fmt"
)

// ExampleRunner は examples/<type>/ の binary を起動 + cleanup する wrapper
type ExampleRunner struct {
	// ExampleName は examples/ 配下のディレクトリ名（例: tier2-go-service）
	ExampleName string
}

// NewExampleRunner は example name を受け取って runner を生成
func NewExampleRunner(exampleName string) *ExampleRunner {
	return &ExampleRunner{ExampleName: exampleName}
}

// Start は example の binary / container を起動する。
// リリース時点は skeleton。採用初期で各 example の Dockerfile / catalog-info.yaml 経由起動を実装する。
func (r *ExampleRunner) Start(_ context.Context) error {
	return fmt.Errorf("ExampleRunner.Start 未実装 (example=%s、採用初期で example 個別起動を実装)", r.ExampleName)
}

// Stop は example を停止 + 後片付け
func (r *ExampleRunner) Stop(_ context.Context) error {
	return fmt.Errorf("ExampleRunner.Stop 未実装 (example=%s、採用初期で実装)", r.ExampleName)
}
