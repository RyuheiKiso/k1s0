#!/usr/bin/env python3
"""
drift_report_formatter.py — k1s0 スキーマドリフトレポートフォーマッター

schema_drift_detector.py が出力する DriftResult を、複数の出力形式に変換する。

対応する出力形式:
  - テキスト形式 (text): 人間が読みやすいプレーンテキスト
  - JSON 形式 (json): CI/CD パイプラインが消費する構造化 JSON
  - Markdown 形式 (markdown): PR コメントやドキュメントとして貼り付ける形式
  - JUnit XML 形式 (junit): GitHub Actions / Jenkins の test result として取り込む形式
  - SARIF 形式 (sarif): GitHub Code Scanning / IDE に取り込む静的解析標準フォーマット

利用例:
  result = drift_between(old_schema, new_schema)
  formatter = DriftReportFormatter(result, source_label="k1s0_transport v1->v2")
  print(formatter.as_text())
  print(formatter.as_json())
  formatter.write_junit("drift_report.xml")
"""

from __future__ import annotations

# dataclasses: フォーマッター設定値オブジェクトに使用する
import dataclasses
# json: JSON 形式の出力に使用する
import json
# pathlib: ファイル出力パスに使用する
from pathlib import Path
# textwrap: Markdown の長いテキストを折り返すために使用する
import textwrap
# typing: 型ヒントに使用する
from typing import Any, Dict, List, Optional

# 同一パッケージの schema_drift_detector から必要なクラスをインポートする
from .schema_drift_detector import (
    DriftEntry,
    DriftResult,
    DriftSeverity,
)

# SARIF の schema URL を定義する（GitHub Code Scanning 対応版）
_SARIF_SCHEMA_URL: str = "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json"
# SARIF のバージョン文字列を定義する
_SARIF_VERSION: str = "2.1.0"
# ツール名を定義する（SARIF の tool.driver.name に使用する）
_TOOL_NAME: str = "k1s0-schema-drift-detector"
# ツールバージョンを定義する
_TOOL_VERSION: str = "1.0.0"

# DriftSeverity を SARIF level にマッピングする辞書を定義する
_SARIF_LEVEL_MAP: Dict[DriftSeverity, str] = {
    DriftSeverity.BREAKING: "error",
    DriftSeverity.ADDITIVE: "warning",
    DriftSeverity.SAFE: "note",
}

# DriftSeverity を JUnit status にマッピングする辞書を定義する
_JUNIT_SEVERITY_MAP: Dict[DriftSeverity, str] = {
    DriftSeverity.BREAKING: "failure",
    DriftSeverity.ADDITIVE: "warning",
    DriftSeverity.SAFE: "info",
}

# Markdown テーブルのヘッダを定義する
_MARKDOWN_TABLE_HEADER: str = (
    "| 深刻度 | 場所 | 変化の種類 | 旧値 | 新値 | 説明 |\n"
    "|--------|------|------------|------|------|------|\n"
)

# テキスト出力の区切り線を定義する
_TEXT_SEPARATOR: str = "─" * 80


@dataclasses.dataclass
class FormatterConfig:
    """DriftReportFormatter の設定を保持するデータクラス。"""

    # include_safe_entries: SAFE 分類のエントリを出力に含めるかどうか
    include_safe_entries: bool = True
    # max_description_length: 説明文の最大文字数（超えた場合は省略する）
    max_description_length: int = 200
    # indent_spaces: JSON/SARIF 出力のインデントスペース数
    indent_spaces: int = 2
    # junit_suite_name: JUnit スイート名（デフォルトは "schema-drift"）
    junit_suite_name: str = "schema-drift"


class DriftReportFormatter:
    """
    DriftResult を複数の出力形式に変換するフォーマッタークラス。

    各メソッドは文字列またはバイト列を返す純粋な変換操作を行う。
    ファイル書き込みは write_* メソッドが提供する。
    """

    def __init__(
        self,
        result: DriftResult,
        source_label: str = "",
        config: Optional[FormatterConfig] = None,
    ) -> None:
        """DriftResult とソースラベルを受け取って DriftReportFormatter を初期化する。"""
        # result をインスタンス変数に保存する
        self._result: DriftResult = result
        # source_label をインスタンス変数に保存する（例: "k1s0_transport v1->v2"）
        self._source_label: str = source_label
        # config をインスタンス変数に保存する（デフォルト設定を使用する場合は新規生成する）
        self._config: FormatterConfig = config if config is not None else FormatterConfig()

    def _filtered_entries(self) -> List[DriftEntry]:
        """設定に応じてエントリをフィルタリングして返す。"""
        # include_safe_entries が False の場合は SAFE エントリを除外する
        if not self._config.include_safe_entries:
            return [e for e in self._result.entries if e.severity != DriftSeverity.SAFE]
        # フィルタなしで全エントリを返す
        return list(self._result.entries)

    def _truncate_description(self, desc: str) -> str:
        """説明文を最大文字数で切り詰める。"""
        # 最大文字数以内の場合はそのまま返す
        if len(desc) <= self._config.max_description_length:
            return desc
        # 最大文字数で切り詰めて省略記号を付加する
        return desc[: self._config.max_description_length - 3] + "..."

    def as_text(self) -> str:
        """人間が読みやすいプレーンテキスト形式でレポートを返す。"""
        # テキスト行を格納するリストを初期化する
        lines: List[str] = []
        # ヘッダを追加する
        lines.append(_TEXT_SEPARATOR)
        lines.append(f"K1s0 スキーマドリフトレポート: {self._source_label}")
        lines.append(_TEXT_SEPARATOR)
        # 旧スキーマパスを追加する
        if self._result.old_path:
            lines.append(f"  旧スキーマ: {self._result.old_path}")
        # 新スキーマパスを追加する
        if self._result.new_path:
            lines.append(f"  新スキーマ: {self._result.new_path}")
        # 統計情報を追加する
        entries = self._filtered_entries()
        lines.append(f"  総エントリ数    : {len(entries)}")
        breaking = [e for e in entries if e.severity == DriftSeverity.BREAKING]
        additive = [e for e in entries if e.severity == DriftSeverity.ADDITIVE]
        safe = [e for e in entries if e.severity == DriftSeverity.SAFE]
        lines.append(f"  BREAKING       : {len(breaking)}")
        lines.append(f"  ADDITIVE       : {len(additive)}")
        lines.append(f"  SAFE           : {len(safe)}")
        lines.append(_TEXT_SEPARATOR)
        # エントリを深刻度順（BREAKING > ADDITIVE > SAFE）に出力する
        severity_order = [DriftSeverity.BREAKING, DriftSeverity.ADDITIVE, DriftSeverity.SAFE]
        for severity in severity_order:
            # 当該深刻度のエントリを取得する
            sev_entries = [e for e in entries if e.severity == severity]
            # エントリが存在しない場合はスキップする
            if not sev_entries:
                continue
            # セクションヘッダを追加する
            lines.append(f"\n[{severity.value}] ({len(sev_entries)} 件)")
            # 各エントリを追加する
            for entry in sev_entries:
                lines.append(f"  場所   : {entry.location}")
                lines.append(f"  種類   : {entry.change_type}")
                lines.append(f"  旧値   : {entry.old_value}")
                lines.append(f"  新値   : {entry.new_value}")
                lines.append(f"  説明   : {self._truncate_description(entry.description)}")
                lines.append("")
        # フッターを追加する
        lines.append(_TEXT_SEPARATOR)
        overall = "BREAKING 変更あり" if self._result.has_breaking_changes else "互換性問題なし"
        lines.append(f"総合判定: {overall}")
        lines.append(_TEXT_SEPARATOR)
        # 全行を改行で結合して返す
        return "\n".join(lines)

    def as_json(self) -> str:
        """CI/CD パイプラインが消費する JSON 形式でレポートを返す。"""
        # エントリをフィルタリングする
        entries = self._filtered_entries()
        # JSON 構造を構築する
        payload: Dict[str, Any] = {
            "source_label": self._source_label,
            "old_schema": str(self._result.old_path) if self._result.old_path else None,
            "new_schema": str(self._result.new_path) if self._result.new_path else None,
            "summary": {
                "total": len(entries),
                "breaking": len([e for e in entries if e.severity == DriftSeverity.BREAKING]),
                "additive": len([e for e in entries if e.severity == DriftSeverity.ADDITIVE]),
                "safe": len([e for e in entries if e.severity == DriftSeverity.SAFE]),
                "has_breaking_changes": self._result.has_breaking_changes,
            },
            "entries": [
                {
                    "severity": e.severity.value,
                    "location": e.location,
                    "change_type": e.change_type,
                    "old_value": e.old_value,
                    "new_value": e.new_value,
                    "description": self._truncate_description(e.description),
                }
                for e in entries
            ],
        }
        # JSON 文字列を返す（日本語文字を ASCII エスケープしない）
        return json.dumps(payload, ensure_ascii=False, indent=self._config.indent_spaces)

    def as_markdown(self) -> str:
        """GitHub PR コメント用の Markdown 形式でレポートを返す。"""
        # Markdown 行を格納するリストを初期化する
        lines: List[str] = []
        # ヘッダを追加する
        overall_emoji = "BREAKING" if self._result.has_breaking_changes else "OK"
        lines.append(f"## スキーマドリフトレポート [{overall_emoji}]")
        lines.append("")
        # ソースラベルを追加する
        if self._source_label:
            lines.append(f"**対象**: `{self._source_label}`")
            lines.append("")
        # サマリテーブルを追加する
        entries = self._filtered_entries()
        breaking_count = len([e for e in entries if e.severity == DriftSeverity.BREAKING])
        additive_count = len([e for e in entries if e.severity == DriftSeverity.ADDITIVE])
        safe_count = len([e for e in entries if e.severity == DriftSeverity.SAFE])
        lines.append("### サマリ")
        lines.append("")
        lines.append("| 分類 | 件数 |")
        lines.append("|------|------|")
        lines.append(f"| BREAKING | {breaking_count} |")
        lines.append(f"| ADDITIVE | {additive_count} |")
        lines.append(f"| SAFE | {safe_count} |")
        lines.append(f"| **合計** | **{len(entries)}** |")
        lines.append("")
        # BREAKING エントリが存在する場合は警告セクションを追加する
        if breaking_count > 0:
            lines.append("> **警告**: このスキーマ変更には後方互換性を壊す変化が含まれています。")
            lines.append("> 全クライアントのアップグレード後にのみデプロイしてください。")
            lines.append("")
        # 各深刻度のエントリセクションを追加する
        severity_order = [DriftSeverity.BREAKING, DriftSeverity.ADDITIVE, DriftSeverity.SAFE]
        for severity in severity_order:
            # 当該深刻度のエントリを取得する
            sev_entries = [e for e in entries if e.severity == severity]
            # エントリが存在しない場合はスキップする
            if not sev_entries:
                continue
            # セクションヘッダを追加する
            lines.append(f"### {severity.value} 変化 ({len(sev_entries)} 件)")
            lines.append("")
            # テーブルヘッダを追加する
            lines.append(_MARKDOWN_TABLE_HEADER.rstrip("\n"))
            # 各エントリを追加する
            for entry in sev_entries:
                desc = self._truncate_description(entry.description)
                # Markdown テーブルのセル内でパイプ文字をエスケープする
                desc_escaped = desc.replace("|", "\\|")
                old_val = entry.old_value.replace("|", "\\|")
                new_val = entry.new_value.replace("|", "\\|")
                lines.append(
                    f"| {severity.value} | `{entry.location}` | `{entry.change_type}` "
                    f"| `{old_val}` | `{new_val}` | {desc_escaped} |"
                )
            lines.append("")
        # 全行を改行で結合して返す
        return "\n".join(lines)

    def as_junit_xml(self) -> str:
        """JUnit XML 形式でレポートを返す（GitHub Actions test result として取り込み可能）。"""
        # エントリをフィルタリングする
        entries = self._filtered_entries()
        # BREAKING エントリは failures、その他は warnings として扱う
        failure_count = len([e for e in entries if e.severity == DriftSeverity.BREAKING])
        # スイート名をエスケープする
        suite_name = self._config.junit_suite_name
        # XML を手動で構築する（外部ライブラリなし）
        xml_lines: List[str] = []
        xml_lines.append('<?xml version="1.0" encoding="UTF-8"?>')
        xml_lines.append(
            f'<testsuite name="{suite_name}" tests="{len(entries)}" '
            f'failures="{failure_count}" errors="0">'
        )
        # 各エントリをテストケースとして追加する
        for entry in entries:
            # テストケース名を構築する
            tc_name = f"{entry.change_type}:{entry.location}"
            xml_lines.append(f'  <testcase name="{_xml_escape(tc_name)}" classname="schema-drift">')
            # BREAKING エントリは failure タグを追加する
            if entry.severity == DriftSeverity.BREAKING:
                desc = _xml_escape(self._truncate_description(entry.description))
                xml_lines.append(
                    f'    <failure message="{_xml_escape(entry.change_type)}" type="BreakingChange">'
                )
                xml_lines.append(f"      {desc}")
                xml_lines.append("    </failure>")
            # ADDITIVE エントリはシステム出力として追加する
            elif entry.severity == DriftSeverity.ADDITIVE:
                desc = _xml_escape(self._truncate_description(entry.description))
                xml_lines.append(
                    f"    <system-out>{desc}</system-out>"
                )
            # テストケースを閉じる
            xml_lines.append("  </testcase>")
        # スイートを閉じる
        xml_lines.append("</testsuite>")
        # 全行を改行で結合して返す
        return "\n".join(xml_lines)

    def as_sarif(self) -> str:
        """SARIF 2.1.0 形式でレポートを返す（GitHub Code Scanning に取り込み可能）。"""
        # エントリをフィルタリングする
        entries = self._filtered_entries()
        # ルールセットを構築する（change_type をルール ID として使用する）
        rule_ids: Dict[str, Dict[str, str]] = {}
        for entry in entries:
            # ルール ID が未登録の場合は追加する
            if entry.change_type not in rule_ids:
                rule_ids[entry.change_type] = {
                    "id": entry.change_type,
                    "name": entry.change_type.replace("_", " ").title(),
                    "shortDescription": {"text": entry.change_type},
                    "defaultConfiguration": {
                        "level": _SARIF_LEVEL_MAP.get(entry.severity, "note")
                    },
                }
        # SARIF 構造を構築する
        sarif: Dict[str, Any] = {
            "$schema": _SARIF_SCHEMA_URL,
            "version": _SARIF_VERSION,
            "runs": [
                {
                    "tool": {
                        "driver": {
                            "name": _TOOL_NAME,
                            "version": _TOOL_VERSION,
                            "rules": list(rule_ids.values()),
                        }
                    },
                    "results": [
                        {
                            "ruleId": entry.change_type,
                            "level": _SARIF_LEVEL_MAP.get(entry.severity, "note"),
                            "message": {
                                "text": (
                                    f"[{entry.severity.value}] {entry.location}: "
                                    f"{self._truncate_description(entry.description)}"
                                )
                            },
                            "locations": [
                                {
                                    "logicalLocations": [
                                        {
                                            "name": entry.location,
                                            "kind": "namespace",
                                        }
                                    ]
                                }
                            ],
                            "properties": {
                                "old_value": entry.old_value,
                                "new_value": entry.new_value,
                                "severity": entry.severity.value,
                            },
                        }
                        for entry in entries
                    ],
                }
            ],
        }
        # JSON 文字列を返す
        return json.dumps(sarif, ensure_ascii=False, indent=self._config.indent_spaces)

    def write_text(self, path: Path) -> None:
        """テキスト形式のレポートをファイルに書き込む。"""
        # as_text() の結果をファイルに書き込む
        path.write_text(self.as_text(), encoding="utf-8")

    def write_json(self, path: Path) -> None:
        """JSON 形式のレポートをファイルに書き込む。"""
        # as_json() の結果をファイルに書き込む
        path.write_text(self.as_json(), encoding="utf-8")

    def write_markdown(self, path: Path) -> None:
        """Markdown 形式のレポートをファイルに書き込む。"""
        # as_markdown() の結果をファイルに書き込む
        path.write_text(self.as_markdown(), encoding="utf-8")

    def write_junit(self, path: Path) -> None:
        """JUnit XML 形式のレポートをファイルに書き込む。"""
        # as_junit_xml() の結果をファイルに書き込む
        path.write_text(self.as_junit_xml(), encoding="utf-8")

    def write_sarif(self, path: Path) -> None:
        """SARIF 形式のレポートをファイルに書き込む。"""
        # as_sarif() の結果をファイルに書き込む
        path.write_text(self.as_sarif(), encoding="utf-8")


def _xml_escape(s: str) -> str:
    """XML 特殊文字をエスケープした文字列を返す。"""
    # & を最初に置換する（他の置換で & が壊れないようにするため）
    s = s.replace("&", "&amp;")
    # < を置換する
    s = s.replace("<", "&lt;")
    # > を置換する
    s = s.replace(">", "&gt;")
    # " を置換する
    s = s.replace('"', "&quot;")
    # ' を置換する
    s = s.replace("'", "&apos;")
    # エスケープされた文字列を返す
    return s
