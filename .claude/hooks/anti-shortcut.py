#!/usr/bin/env python3
"""anti-shortcut: Claude Code PreToolUse hook for Edit / Write / MultiEdit.

短絡修正（局所最適に閉じる癖）を Edit 段で検知して warn する。
block しない（誤検知時の足止めを避け、認知的注意喚起のみに留める）。

検知パターン（warn）:
  1. 禁止コメント語彙の新規追加（TODO / FIXME / とりあえず / 暫定 / 仮置き / あとで / for now / temporary / quick fix / hack / workaround）
  2. 同一ファイルへの直近 5 回連続 Edit
  3. 直近で test ファイル touch なしの bug fix らしき Edit（コメントに fix / bug / 修正 を含む）
  4. Unimplemented / unimplemented!() / NotImplementedException / "not impl" の新規導入
  5. 空 catch / silent error suppress（catch {} / except: pass / _ = err スタイル）

無効化: 環境変数 K1S0_ANTI_SHORTCUT_DISABLE=1（commit しない一時 override 用）
しきい値: 本ファイル冒頭の SAME_FILE_STREAK_AT で調整
"""
from __future__ import annotations

import json
import os
import re
import sys
import tempfile
import time
from pathlib import Path

# Windows コンソールの cp932 等で文字化けしないよう stdout を UTF-8 に固定する
try:
    sys.stdout.reconfigure(encoding="utf-8")
except Exception:
    pass

# 同一ファイル連続 Edit の警告閾値
SAME_FILE_STREAK_AT = 5

# Exemption: 禁止語彙を「説明用に正当に含む」ファイル / ディレクトリ
# これらの編集時は、禁止語彙 / Unimplemented / silent suppress の 3 軸検出をスキップする
# （連続 Edit / bug-fix らしき Edit + test 不在 は引き続き検査）
EXEMPT_PATH_PATTERNS = [
    re.compile(r"\.claude/skills/(anti-shortcut-discipline|audit-protocol|principal-architect-mindset|iteration-and-scope-discipline)/"),
    re.compile(r"\.claude/commands/audit\.md$"),
    re.compile(r"\.claude/hooks/anti-shortcut\.py$"),
    re.compile(r"docs/00_format/audit_criteria\.md$"),
    re.compile(r"docs/00_format/review_checklist\.md$"),
    re.compile(r"docs/AUDIT\.md$"),
    re.compile(r"docs/SHIP_STATUS\.md$"),
    re.compile(r"/memory/feedback_anti_shortcut\.md$"),
    re.compile(r"/memory/MEMORY\.md$"),
    re.compile(r"tools/audit/lib/slack\.sh$"),
]


def is_exempt_path(file_path: str) -> bool:
    """禁止語彙等の説明用に正当に含むファイルかどうかを判定する。"""
    for pat in EXEMPT_PATH_PATTERNS:
        if pat.search(file_path):
            return True
    return False
# bug fix らしき Edit と判定するキーワード（コメント / new_string 内）
BUG_FIX_KEYWORDS = re.compile(r"\b(fix|bug|修正|bugfix|hotfix)\b", re.IGNORECASE)
# bug fix 直近で test ファイル touch を要求する直近操作数（lookback 件数）
TEST_TOUCH_LOOKBACK = 10
# test ファイルパターン
TEST_FILE_PATTERN = re.compile(
    r"(_test\.go|_test\.rs|\.test\.ts|\.test\.tsx|\.spec\.ts|\.spec\.tsx|/tests?/|Tests?\.cs|test_.*\.py)"
)

# 禁止コメント語彙（新規追加検知）
FORBIDDEN_COMMENT_PATTERNS = [
    (re.compile(r"\bTODO\b"), "TODO"),
    (re.compile(r"\bFIXME\b"), "FIXME"),
    (re.compile(r"\bXXX\b"), "XXX"),
    (re.compile(r"とりあえず"), "とりあえず"),
    (re.compile(r"暫定"), "暫定"),
    (re.compile(r"仮置き"), "仮置き"),
    (re.compile(r"あとで|後で(?!ろ)"), "あとで / 後で"),
    (re.compile(r"\bfor now\b", re.IGNORECASE), "for now"),
    (re.compile(r"\btemporary\b", re.IGNORECASE), "temporary"),
    (re.compile(r"\bquick fix\b", re.IGNORECASE), "quick fix"),
    (re.compile(r"//\s*hack\b|#\s*hack\b", re.IGNORECASE), "hack"),
    (re.compile(r"\bworkaround\b", re.IGNORECASE), "workaround"),
]

# Unimplemented パターン（新規導入検知、言語横断）
UNIMPLEMENTED_PATTERNS = [
    (re.compile(r"codes\.Unimplemented"), "Go codes.Unimplemented"),
    (re.compile(r"unimplemented!\s*\("), "Rust unimplemented!()"),
    (re.compile(r"todo!\s*\("), "Rust todo!()"),
    (re.compile(r"NotImplementedException"), ".NET NotImplementedException"),
    (re.compile(r"raise\s+NotImplementedError"), "Python NotImplementedError"),
    (re.compile(r'throw\s+new\s+Error\s*\(\s*["\'][^"\']*not\s+impl', re.IGNORECASE), "TS not impl"),
]

# 空 catch / silent suppress パターン
SILENT_SUPPRESS_PATTERNS = [
    (re.compile(r"catch\s*\([^)]*\)\s*\{\s*\}"), "空 catch (TS/JS/Java/C#)"),
    (re.compile(r"except[^:]*:\s*pass\b"), "Python except: pass"),
    (re.compile(r"_\s*=\s*err\b"), "Go _ = err"),
    (re.compile(r"\.unwrap_or\s*\(\s*\)"), "Rust unwrap_or() empty"),
    (re.compile(r"if\s+err\s*!=\s*nil\s*\{\s*\}"), "Go silent if err"),
]


def main() -> int:
    # 環境変数で完全無効化
    if os.environ.get("K1S0_ANTI_SHORTCUT_DISABLE") == "1":
        return 0

    try:
        data = json.load(sys.stdin)
    except Exception:
        return 0

    tool_input = data.get("tool_input") or {}
    file_path = str(tool_input.get("file_path") or "")
    session_id = str(data.get("session_id") or "default")

    # ファイルパスがなければ素通り
    if not file_path:
        return 0

    # 編集対象のテキスト（Edit / Write / MultiEdit で位置が違う）
    new_text = ""
    if "new_string" in tool_input:
        new_text = str(tool_input.get("new_string") or "")
    elif "content" in tool_input:
        new_text = str(tool_input.get("content") or "")
    elif "edits" in tool_input:
        edits = tool_input.get("edits") or []
        new_text = "\n".join(str(e.get("new_string", "")) for e in edits if isinstance(e, dict))

    warnings: list[str] = []
    exempt = is_exempt_path(file_path)

    # 1. 禁止コメント語彙（exemption 対象は skip）
    if not exempt:
        for pat, label in FORBIDDEN_COMMENT_PATTERNS:
            if pat.search(new_text):
                warnings.append(
                    f"禁止語彙『{label}』の新規追加。ADR / Issue に昇格せよ（anti-shortcut-discipline 禁止コメント語彙）"
                )
                break

    # 2. Unimplemented 新規導入（exemption 対象は skip）
    if not exempt:
        for pat, label in UNIMPLEMENTED_PATTERNS:
            if pat.search(new_text):
                warnings.append(
                    f"『{label}』の新規導入。採用初期で潰すべき方向と逆向き（principal-architect-mindset / SHIP_STATUS §9）"
                )
                break

    # 3. silent suppress（exemption 対象は skip）
    if not exempt:
        for pat, label in SILENT_SUPPRESS_PATTERNS:
            if pat.search(new_text):
                warnings.append(
                    f"『{label}』検出。silent error suppress は症状治療の典型（anti-shortcut-discipline §症状 vs 根本）"
                )
                break

    # 同一ファイル連続 Edit ログ（test ファイル touch lookback 兼用）
    log_dir = Path(tempfile.gettempdir()) / "claude-anti-shortcut"
    log_dir.mkdir(parents=True, exist_ok=True)
    safe_sid = re.sub(r"[^A-Za-z0-9_\-]", "_", session_id) or "default"
    log_file = log_dir / f"{safe_sid}.log"

    # 直近のログを読む
    recent: list[str] = []
    if log_file.exists():
        try:
            with log_file.open("r", encoding="utf-8") as f:
                recent = [line.strip() for line in f.readlines()]
        except Exception:
            recent = []

    # 4. 同一ファイル連続 Edit
    same_streak = 0
    for line in reversed(recent):
        parts = line.split("|", 2)
        if len(parts) >= 2 and parts[1] == file_path:
            same_streak += 1
        else:
            break
    if same_streak + 1 >= SAME_FILE_STREAK_AT:
        warnings.append(
            f"同一ファイル『{Path(file_path).name}』への {same_streak + 1} 回連続 Edit。"
            "全体構造を見失っていないか（anti-shortcut-discipline §大量編集中の自己点検）"
        )

    # 5. bug fix らしき Edit + 直近 N 件で test touch なし
    if BUG_FIX_KEYWORDS.search(new_text):
        recent_paths = []
        for line in recent[-TEST_TOUCH_LOOKBACK:]:
            parts = line.split("|", 2)
            if len(parts) >= 2:
                recent_paths.append(parts[1])
        has_test_touch = any(TEST_FILE_PATTERN.search(p) for p in recent_paths)
        if not has_test_touch and not TEST_FILE_PATTERN.search(file_path):
            warnings.append(
                f"bug fix らしき Edit だが直近 {TEST_TOUCH_LOOKBACK} 操作で test ファイル touch なし。"
                "regression test を同 commit に含めよ（anti-shortcut-discipline §4 段プロトコル）"
            )

    # ログ追記
    try:
        with log_file.open("a", encoding="utf-8") as f:
            f.write(f"{int(time.time())}|{file_path}\n")
    except Exception:
        pass

    # warn 出力（block はしない）
    if warnings:
        msg = "anti-shortcut-discipline 警告:\n  - " + "\n  - ".join(warnings)
        payload = {
            "systemMessage": msg,
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "additionalContext": msg,
            },
        }
        json.dump(payload, sys.stdout, ensure_ascii=False)

    return 0


if __name__ == "__main__":
    sys.exit(main())
