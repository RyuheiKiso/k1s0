#!/usr/bin/env python3
"""flaky-detector — 直近 N 件の workflow run から flaky 候補を抽出する。

設計正典:
  ADR-TEST-007（テスト属性タグ + CI 実行フェーズ分離）
関連 ID:
  IMP-CI-TAG-004（flaky 自動検出: 直近 20 PR で fail 率 >= 5% を quarantine 自動追加）
  IMP-CI-TAG-005（tests/.flaky-quarantine.yaml の PR レビュー必須化）

本リリース時点の射程:
  GitHub Actions API で直近 N 件の nightly.yml workflow run を取得し、conclusion を
  集計した結果を tests/.flaky-report/<date>-summary.md に書き出す。fail 率 >= 5%
  なら summary に warning を含める。

採用初期で本格化する射程:
  - workflow run artifact から go test -json 出力を parse
  - test 別の pass/fail 履歴を集計
  - 5% 超の test を tests/.flaky-quarantine.yaml に自動追加する PR を提出
  - flaky-quarantine.yaml の PR レビューフローを CODEOWNERS で強制

Usage:
  python3 tools/qualify/flaky-detector.py
  python3 tools/qualify/flaky-detector.py --workflow nightly.yml --limit 20

Env:
  GITHUB_TOKEN       Actions runner では自動付与、local 実行では gh auth login で取得
  GITHUB_REPOSITORY  例: k1s0/k1s0（owner/repo 形式）

Exit codes:
  0 = 正常完了 / 1 = API 呼び出し失敗 / 2 = 引数 / 環境エラー
"""

# 標準 library import
from __future__ import annotations

# argparse は CLI 引数解析（ADR-TEST-002 portable 制約と整合した最小依存）
import argparse

# datetime は run の created_at parse + summary の日付出力
import datetime as dt

# json は API response decode
import json

# os は env var 取得
import os

# pathlib は出力 path 構築
import pathlib

# sys は exit code 制御
import sys

# typing は型注釈で意図明示（Python 3.10+ 想定）
from typing import Any

# 外部 library: requests のみ（採用初期で urllib 標準への置き換えを検討、本リリース時点は requests が flaky-report.yml で install 済）
import requests


def parse_args() -> argparse.Namespace:
    """CLI 引数を解析する。"""
    # 引数定義（最小限：workflow 名と取得件数のみ）
    parser = argparse.ArgumentParser(description="flaky-detector: workflow run の集計")
    parser.add_argument(
        "--workflow",
        default="nightly.yml",
        help="対象 workflow file 名（既定: nightly.yml）",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=20,
        help="取得する run 件数（既定: 20）",
    )
    return parser.parse_args()


def get_recent_runs(
    repo: str, workflow: str, limit: int, token: str
) -> list[dict[str, Any]]:
    """GitHub Actions API で workflow の直近 run を取得する。"""
    # GitHub REST API endpoint（owner/repo 形式の repo を分解）
    url = (
        f"https://api.github.com/repos/{repo}/actions/workflows/{workflow}/runs"
        f"?per_page={limit}"
    )
    # Authorization header で API rate limit を 5000 req/h に拡張
    headers = {
        "Accept": "application/vnd.github+json",
        "Authorization": f"Bearer {token}",
        "X-GitHub-Api-Version": "2022-11-28",
    }
    # 短い timeout で stuck を回避
    response = requests.get(url, headers=headers, timeout=30)
    # 200 以外は API 異常 / workflow 不在
    response.raise_for_status()
    # response は workflow_runs[] を含む
    payload: dict[str, Any] = response.json()
    return payload.get("workflow_runs", [])


def summarize(runs: list[dict[str, Any]], workflow: str) -> tuple[str, int]:
    """run のリストから summary Markdown と fail 率（パーセント）を生成する。"""
    # 件数 0 は workflow がまだ走っていない（リリース直後の正常状態）
    if not runs:
        return (
            f"# flaky-detector summary — {workflow}\n\n"
            f"対象 workflow `{workflow}` の run は 0 件（まだ実行されていない）。\n",
            0,
        )

    # conclusion 別の件数集計
    counts: dict[str, int] = {}
    for run in runs:
        # conclusion は success / failure / cancelled / skipped / null（実行中）
        conclusion = run.get("conclusion") or "in_progress"
        counts[conclusion] = counts.get(conclusion, 0) + 1

    # fail と判定するもの（success と skipped 以外）
    total = len(runs)
    fail_like = sum(c for k, c in counts.items() if k not in ("success", "skipped"))
    # fail 率（小数 1 位まで）
    fail_pct = round((fail_like / total) * 100, 1) if total > 0 else 0
    # 5% 超で warning
    warning = "**WARNING: fail 率 5% 超**" if fail_pct > 5 else "OK"

    # Markdown 形式の summary を組み立てる
    lines = [
        f"# flaky-detector summary — {workflow}",
        "",
        f"- 対象 workflow: `{workflow}`",
        f"- 集計期間: 直近 {total} 件",
        f"- fail 率: **{fail_pct}%**（{fail_like}/{total}）",
        f"- 判定: {warning}",
        "",
        "## conclusion 別件数",
        "",
        "| conclusion | 件数 |",
        "|---|---|",
    ]
    # conclusion 種別をソートして表に出す
    for conclusion in sorted(counts):
        lines.append(f"| {conclusion} | {counts[conclusion]} |")

    # 個別 run の最新 5 件をリンク付きで列挙（採用初期で test 粒度集計に拡張）
    lines.extend(
        [
            "",
            "## 直近 5 run（時系列降順）",
            "",
            "| created_at | conclusion | URL |",
            "|---|---|---|",
        ]
    )
    for run in runs[:5]:
        # API は ISO 8601 string で返す
        created = run.get("created_at", "")
        conclusion = run.get("conclusion") or "in_progress"
        html_url = run.get("html_url", "")
        lines.append(f"| {created} | {conclusion} | {html_url} |")

    # 採用初期での拡張点を末尾に明示
    lines.extend(
        [
            "",
            "## 採用初期での拡張",
            "",
            "本実装は workflow 全体の fail 率集計に留まる（IMP-CI-TAG-004 最小成立形）。",
            "採用初期で go test -json artifact を parse し test 別 fail 率を",
            "tests/.flaky-quarantine.yaml に自動追加する PR 提出経路を本 script に統合する。",
        ]
    )
    return "\n".join(lines) + "\n", fail_pct


def main() -> int:
    """エントリポイント。集計を実行し summary を出力する。"""
    # 引数解析
    args = parse_args()

    # 環境変数から token / repo を取得
    token = os.environ.get("GITHUB_TOKEN", "")
    repo = os.environ.get("GITHUB_REPOSITORY", "")
    if not token:
        print("[error] GITHUB_TOKEN env var 未設定", file=sys.stderr)
        return 2
    if not repo:
        print("[error] GITHUB_REPOSITORY env var 未設定（例: k1s0/k1s0）", file=sys.stderr)
        return 2

    # API 取得 → summary 生成
    try:
        runs = get_recent_runs(repo, args.workflow, args.limit, token)
    except requests.RequestException as e:
        print(f"[error] GitHub API 呼び出し失敗: {e}", file=sys.stderr)
        return 1
    summary, fail_pct = summarize(runs, args.workflow)

    # 出力 path（YYYY-MM-DD-summary.md、tests/.flaky-report/ 配下）
    repo_root = pathlib.Path(__file__).resolve().parents[2]
    out_dir = repo_root / "tests" / ".flaky-report"
    out_dir.mkdir(parents=True, exist_ok=True)
    today = dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%d")
    out_path = out_dir / f"{today}-{args.workflow.replace('.yml', '')}-summary.md"
    out_path.write_text(summary, encoding="utf-8")

    # stdout にも出して CI ログから即読める
    print(summary)
    print(f"[done] summary: {out_path}")
    print(f"[done] fail_rate: {fail_pct}%")
    return 0


if __name__ == "__main__":
    sys.exit(main())
