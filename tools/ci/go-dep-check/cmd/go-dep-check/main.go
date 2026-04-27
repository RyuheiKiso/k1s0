// 本ファイルは go-dep-check CLI のエントリポイント。
// 設計正典: docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md
package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/k1s0/k1s0/tools/ci/go-dep-check/internal/checker"
)

// プロセスエントリポイント。
func main() {
	// 走査開始ディレクトリ（既定: git rev-parse --show-toplevel）。
	root := flag.String("root", "", "リポジトリ root（既定: git rev-parse --show-toplevel）")
	// 違反を JSON で出力するか。
	jsonOutput := flag.Bool("json", false, "違反を JSON 形式で出力（CI PR コメント用）")
	flag.Parse()

	// root が未指定なら git で解決する。
	if *root == "" {
		out, err := exec.Command("git", "rev-parse", "--show-toplevel").Output()
		if err != nil {
			fmt.Fprintf(os.Stderr, "git rev-parse failed: %v\n", err)
			os.Exit(2)
		}
		*root = strings.TrimSpace(string(out))
	}
	abs, err := filepath.Abs(*root)
	if err != nil {
		fmt.Fprintf(os.Stderr, "abs path failed: %v\n", err)
		os.Exit(2)
	}

	// チェックを実行する。
	violations, err := checker.Check(abs)
	if err != nil {
		fmt.Fprintf(os.Stderr, "check failed: %v\n", err)
		os.Exit(2)
	}

	// 結果を表示する。
	if *jsonOutput {
		// CI から再消費可能な形式（PR コメント生成用）。
		_ = json.NewEncoder(os.Stdout).Encode(violations)
	} else {
		// 人間可読な 1 行形式。
		for _, v := range violations {
			fmt.Printf("%s:%d: tier=%s が %s を import（許容外、prefix=%s）\n",
				v.File, v.Line, v.SourceTier, v.ImportPath, v.ForbiddenPrefix)
		}
	}

	// 違反があれば exit 1（CI の build fail trigger）。
	if len(violations) > 0 {
		fmt.Fprintf(os.Stderr, "\n%d 件の依存方向違反を検出。詳細は docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md を参照。\n", len(violations))
		os.Exit(1)
	}
}
