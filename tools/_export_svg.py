#!/usr/bin/env python3
"""変更された drawio ファイルから SVG を一括再エクスポート。

git diff --name-only HEAD で取得した *.drawio に対して、
draw.io CLI の --export --format svg を順次実行する。
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

DRAWIO_EXE = r"C:\Program Files\draw.io\draw.io.exe"


def list_changed_drawios() -> list[Path]:
    r = subprocess.run(
        ["git", "-c", "core.quotepath=false", "diff", "--name-only", "HEAD"],
        capture_output=True,
        check=True,
    )
    text = r.stdout.decode("utf-8", errors="replace")
    out: list[Path] = []
    for line in text.splitlines():
        line = line.strip()
        if not line or not line.endswith(".drawio"):
            continue
        p = Path(line)
        if p.exists():
            out.append(p)
    return out


def export_one(drawio: Path) -> tuple[bool, str]:
    svg = drawio.with_suffix(".svg")
    cmd = [
        DRAWIO_EXE,
        "--export",
        "--format",
        "svg",
        "--output",
        str(svg),
        str(drawio),
    ]
    try:
        r = subprocess.run(cmd, capture_output=True, timeout=120)
    except subprocess.TimeoutExpired:
        return False, "timeout"
    if r.returncode != 0:
        err = r.stderr.decode("utf-8", errors="replace").strip()[:200]
        return False, f"rc={r.returncode}: {err}"
    return True, "ok"


def main(argv: list[str]) -> int:
    files = list_changed_drawios()
    print(f"target: {len(files)} drawio files")
    ok = 0
    ng = 0
    for d in files:
        success, msg = export_one(d)
        if success:
            ok += 1
            print(f"  OK : {d}")
        else:
            ng += 1
            print(f"  NG : {d} ({msg})")
    print(f"done: ok={ok}, ng={ng}")
    return 0 if ng == 0 else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
