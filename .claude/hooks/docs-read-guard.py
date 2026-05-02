#!/usr/bin/env python3
"""docs-read-guard: Claude Code PreToolUse hook for the Read tool.

docs/ 配下の Read をセッション単位で累積監視し、しきい値を超えたら
警告またはブロックして Agent(subagent_type=Explore) への委譲を促す。

- docs/ 配下以外の Read は素通り（ログも残さない）。
- 同一 session_id 内で WARN_AT 件目から警告（systemMessage + additionalContext）。Read は許可。
- 同一 session_id 内で BLOCK_AT 件目以降はブロック（permissionDecision: deny）。
"""
from __future__ import annotations

import json
import re
import sys
import tempfile
import time
from pathlib import Path

# Windows コンソールの cp932 等で文字化けしないよう stdout を UTF-8 に固定する。
# Claude Code のハーネスは hook 出力を UTF-8 として解釈するため、環境に依存しない出力を保証する。
try:
    sys.stdout.reconfigure(encoding="utf-8")
except Exception:
    pass

# しきい値。docs 作業での許容連続 Read 数を調整する場合はここを変更する。
WARN_AT = 3
BLOCK_AT = 5
# 大規模監査などで一時的にしきい値を引き上げたい場合の override。
# 環境変数 K1S0_DOCS_READ_GUARD_DISABLE=1 でガード自体を無効化する。
# 本ファイルの値は通常の docs 編集作業向けで、監査時は env で切り替えること（commit せず）。


def main() -> int:
    # 環境変数で完全無効化（大規模監査時の一時 override 用、commit しない運用）。
    import os as _os
    if _os.environ.get("K1S0_DOCS_READ_GUARD_DISABLE") == "1":
        return 0

    try:
        data = json.load(sys.stdin)
    except Exception:
        # 入力が壊れていても Read を妨げない
        return 0

    file_path = str((data.get("tool_input") or {}).get("file_path") or "")
    session_id = str(data.get("session_id") or "default")

    # docs/ 配下でなければ素通り（POSIX / Windows パス両対応）
    if not re.search(r"[\\/]docs[\\/]", file_path):
        return 0

    log_dir = Path(tempfile.gettempdir()) / "claude-docs-read"
    log_dir.mkdir(parents=True, exist_ok=True)
    # session_id にパス区切り等が混入してもファイル名として安全にする
    safe_sid = re.sub(r"[^A-Za-z0-9_\-]", "_", session_id) or "default"
    log_file = log_dir / f"{safe_sid}.log"

    with log_file.open("a", encoding="utf-8") as f:
        f.write(f"{int(time.time())}|{file_path}\n")
    with log_file.open("r", encoding="utf-8") as f:
        count = sum(1 for _ in f)

    if count >= BLOCK_AT:
        msg = (
            f"docs/ 配下の Read がこのセッションで {count} 回目に達したためブロックしました。"
            "Agent(subagent_type=docs-explorer) に依頼趣旨を渡して委譲し、"
            "親コンテキストには要約のみ返させてください（docs-explorer は Haiku ベースで "
            "INDEX.md → サブ README → 目的ファイルの動線を自走します）。"
            "しきい値は .claude/hooks/docs-read-guard.py の BLOCK_AT。"
        )
        payload = {
            "systemMessage": msg,
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": msg,
            },
        }
        json.dump(payload, sys.stdout, ensure_ascii=False)
    elif count >= WARN_AT:
        warn = (
            f"docs/ 配下の Read がこのセッションで {count} 件目。"
            "以降は Agent(subagent_type=docs-explorer) への委譲が推奨"
            "（docs/INDEX.md の「情報を探す時の推奨順序」参照）。"
            f"{BLOCK_AT} 件目からはブロックされる。"
        )
        payload = {
            "systemMessage": warn,
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "additionalContext": f"docs/ Read 累計: {count} 件。Explore 委譲を検討せよ。",
            },
        }
        json.dump(payload, sys.stdout, ensure_ascii=False)

    return 0


if __name__ == "__main__":
    sys.exit(main())
