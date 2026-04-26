#!/usr/bin/env python3
"""tools/_link_check.py を pre-commit から呼ぶラッパ。

pre-commit は変更ファイルのリストを引数で渡してくるが、
tools/_link_check.py は docs 配下全体を全件チェックする設計。
本ラッパは「変更された md が docs/ 配下にある場合のみ全 docs link-check を起動」する
差分駆動の運用とする（差分 md 以外のリンクが壊れる事故も検出するため、結局 full 走査）。

Usage:
    tools/git-hooks/link-check-wrapper.py [<file> ...]
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("files", nargs="*")
    parser.add_argument("--quiet", action="store_true")
    args = parser.parse_args()

    # 変更 md が docs/ 配下にあるかチェック
    docs_changed = any(Path(f).as_posix().startswith("docs/") for f in args.files)
    if not docs_changed:
        if not args.quiet:
            print("[link-check] no docs/ md changes, skipped")
        return 0

    # _link_check.py は ROOT = Path("docs") を CWD 起点で見るため、
    # リポジトリルートに移動してから実行
    repo_root = subprocess.check_output(
        ["git", "rev-parse", "--show-toplevel"], text=True
    ).strip()
    os.chdir(repo_root)

    script = Path(repo_root) / "tools" / "_link_check.py"
    if not script.is_file():
        print(f"[link-check] スクリプト未存在: {script}", file=sys.stderr)
        return 0  # fail-soft（pre-commit を止めない）

    rc = subprocess.call([sys.executable, str(script)])
    return rc


if __name__ == "__main__":
    sys.exit(main())
