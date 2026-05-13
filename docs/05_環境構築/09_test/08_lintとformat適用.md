---
id: env.test.test_lint_format
axis: test
phase: env_setup
kind: enforcement
status: draft
depends_on:
  - env.test.test_test_environment
covered_by:
  defense_in_depth_layers: [B]
  proof_classes: []
---

# lint と format 適用

## 一文方針

- test 軸エンジニアは Playwright ESLint / pytest lint（ruff）/ mutation threshold check スクリプトの 3 lint を手元で実行できることを本ページの検収条件とする。

## Playwright ESLint

```bash
pnpm add -D eslint eslint-plugin-playwright @typescript-eslint/parser @typescript-eslint/eslint-plugin

# .eslintrc.json（Playwright 用）
cat > .eslintrc-playwright.json << 'EOF'
{
  "parser": "@typescript-eslint/parser",
  "extends": [
    "eslint:recommended",
    "plugin:playwright/recommended"
  ],
  "plugins": ["playwright", "@typescript-eslint"],
  "parserOptions": { "ecmaVersion": 2022, "sourceType": "module" }
}
EOF

# lint 実行
pnpm eslint --config .eslintrc-playwright.json tests/ --ext .ts
```

## pytest lint（ruff）

ruff は flake8 / pylint の高速代替。pytest ファイルにも適用する。

```bash
pip install ruff

# ruff 設定（pyproject.toml に追記）
cat >> pyproject.toml << 'EOF'
[tool.ruff]
line-length = 120
select = ["E", "F", "I", "N", "W"]
ignore = ["E501"]
[tool.ruff.per-file-ignores]
"tests/**" = ["S101"]  # assert は test ファイルで許可
EOF

# lint 実行
ruff check tests/
ruff check src/

# format チェック
ruff format --check tests/
```

## mutation threshold check スクリプト

mutation score が閾値を下回った場合に CI を fail させるスクリプト。

```bash
# mutation threshold check（Python）
cat > tools/check_mutation_threshold.py << 'SCRIPT'
#!/usr/bin/env python3
"""mutation score threshold check."""
import sys
import json
import argparse

THRESHOLDS = {
    "python": 0.75,   # 75%
    "typescript": 0.75,
    "rust": 0.80,
    "java": 0.80,
}

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--lang", required=True, choices=THRESHOLDS.keys())
    parser.add_argument("--score", type=float, required=True)
    args = parser.parse_args()
    threshold = THRESHOLDS[args.lang]
    if args.score < threshold:
        print(f"FAIL: {args.lang} mutation score {args.score:.2%} < threshold {threshold:.2%}")
        sys.exit(1)
    print(f"OK: {args.lang} mutation score {args.score:.2%} >= threshold {threshold:.2%}")

if __name__ == "__main__":
    main()
SCRIPT
chmod +x tools/check_mutation_threshold.py
```

使用例:

```bash
python3 tools/check_mutation_threshold.py --lang rust --score 0.82
```

## 検収コマンド

```bash
ruff --version
pnpm eslint --version
python3 tools/check_mutation_threshold.py --lang rust --score 0.82
```

## 関連参照

- [07_テスト検証環境](07_テスト検証環境.md)
- [09_docs_lint実行手順](09_docs_lint実行手順.md)
- [11_軸固有環境設定](11_軸固有環境設定.md)
