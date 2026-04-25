#!/usr/bin/env python3
"""docs 配下の md 内リンクが実在するかを確認する簡易チェッカー。

対象: 相対パスの markdown リンク (`[label](path)`)。
- http(s):// / mailto: / # のみのリンク は除外。
- アンカー (#section) を取り除き、ファイル/ディレクトリの存在を確認する。
"""

from __future__ import annotations

import re
import sys
from pathlib import Path
from urllib.parse import unquote

ROOT = Path("docs")
LINK_RE = re.compile(r"\[([^\]]+)\]\(([^)]+)\)")
# テンプレ例リンクのリテラル: `<...>` を含む / 既知の例示語 (path, URL)
PLACEHOLDER_PATTERNS = (
    "<",  # `<topic>` `<相対パス>` `<対応章>` 等
)
PLACEHOLDER_LITERALS = {"path", "URL"}


def strip_code(text: str) -> str:
    """fenced code block と inline code を空白で潰してリンクを誤検出しない。"""
    # ``` ... ``` (multiline)
    text = re.sub(r"```.*?```", lambda m: " " * len(m.group(0)), text, flags=re.DOTALL)
    # ` ... ` (inline)
    text = re.sub(r"`[^`\n]*`", lambda m: " " * len(m.group(0)), text)
    return text


def iter_md(root: Path) -> list[Path]:
    return [p for p in root.rglob("*.md")]


def is_external(target: str) -> bool:
    return (
        target.startswith("http://")
        or target.startswith("https://")
        or target.startswith("mailto:")
        or target.startswith("#")
    )


def normalize(target: str) -> str:
    if "#" in target:
        target = target.split("#", 1)[0]
    if "?" in target:
        target = target.split("?", 1)[0]
    return unquote(target.strip())


def check(md: Path) -> list[tuple[str, str]]:
    """返り値: (link target, reason) のリスト"""
    out: list[tuple[str, str]] = []
    raw = md.read_text(encoding="utf-8", errors="replace")
    text = strip_code(raw)
    for m in LINK_RE.finditer(text):
        target = m.group(2).strip()
        if is_external(target):
            continue
        # テンプレ例リンクは除外
        if target in PLACEHOLDER_LITERALS:
            continue
        if any(p in target for p in PLACEHOLDER_PATTERNS):
            continue
        norm = normalize(target)
        if not norm:
            continue
        # 解決
        try:
            resolved = (md.parent / norm).resolve()
        except Exception as e:
            out.append((target, f"resolve error: {e}"))
            continue
        if not resolved.exists():
            out.append((target, "not found"))
    return out


def main(argv: list[str]) -> int:
    files = iter_md(ROOT)
    total_broken = 0
    by_file: dict[Path, list[tuple[str, str]]] = {}
    for md in files:
        broken = check(md)
        if broken:
            by_file[md] = broken
            total_broken += len(broken)
    for md, items in sorted(by_file.items()):
        print(f"\n[{md.as_posix()}]")
        for tgt, reason in items:
            print(f"  - {tgt}  ({reason})")
    print(f"\nbroken: {total_broken}, files with broken: {len(by_file)}")
    return 0 if total_broken == 0 else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
