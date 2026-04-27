// 本ファイルは rust-dep-check CLI のエントリポイント。
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

	"github.com/k1s0/k1s0/tools/ci/rust-dep-check/internal/checker"
)

func main() {
	root := flag.String("root", "", "リポジトリ root（既定: git rev-parse --show-toplevel）")
	jsonOutput := flag.Bool("json", false, "違反を JSON 形式で出力（CI PR コメント用）")
	flag.Parse()

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

	violations, err := checker.Check(abs)
	if err != nil {
		fmt.Fprintf(os.Stderr, "check failed: %v\n", err)
		os.Exit(2)
	}

	if *jsonOutput {
		_ = json.NewEncoder(os.Stdout).Encode(violations)
	} else {
		for _, v := range violations {
			fmt.Printf("%s: tier=%s が %s（tier=%s）を path 依存（許容外、依存方向ルール違反）\n",
				v.CargoToml, v.SourceTier, v.ResolvedPath, v.TargetTier)
		}
	}

	if len(violations) > 0 {
		fmt.Fprintf(os.Stderr, "\n%d 件の Rust 依存方向違反を検出。詳細は docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md を参照。\n", len(violations))
		os.Exit(1)
	}
}
