"""src/security/threat_model/risk_reporter.py

STRIDE リスクレポート生成モジュール。
製造業プラットフォームの脅威モデル評価結果を多形式（JSON/YAML/テキスト）で
レポートとして出力し、CI での自動判定・ダッシュボード統合を支援する。
仕様: docs/04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md
"""

# 将来の注釈構文互換のため annotations をインポートする
from __future__ import annotations

# ハッシュ計算に使用する hashlib をインポートする
import hashlib
# JSON 処理に使用する json をインポートする
import json
# ログ出力に使用する logging をインポートする
import logging
# OS 操作に使用する os をインポートする
import os
# システム操作に使用する sys をインポートする
import sys
# テキスト整形に使用する textwrap をインポートする
import textwrap
# 型安全なデータクラスに使用する dataclasses をインポートする
from dataclasses import dataclass, field
# 日時処理に使用する datetime をインポートする
from datetime import datetime, timezone
# パス操作に使用する pathlib をインポートする
from pathlib import Path
# 型ヒント定義に使用する typing をインポートする
from typing import Any, Dict, List, Optional, Tuple

# PyYAML ライブラリのインポートを試みる
try:
    # YAML ファイルの読み込み・書き出しに使用する yaml をインポートする
    import yaml
    # yaml が利用可能であることを示すフラグを設定する
    _HAS_YAML = True
except ImportError:
    # yaml がインストールされていない場合はフラグを False に設定する
    _HAS_YAML = False

# stride_catalog モジュールから必要な型をインポートする
from .stride_catalog import (
    STRIDECatalog,
    ThreatCategory,
    ThreatEntry,
    MitigationEntry,
    ValidationResult,
    CvssCalculator,
)

# モジュール専用ロガーを取得する
logger = logging.getLogger(__name__)

# CI 判定の CVSS 閾値（この値以上の未対処脅威が存在する場合は CI fail）
_CI_FAIL_THRESHOLD_CVSS = 7.0
# 許容できる未対処脅威の最大件数（この件数を超えると CI fail）
_CI_MAX_OPEN_THREATS = 0


# ===========================================================================
# データクラス定義
# ===========================================================================


@dataclass
class RiskSummary:
    """リスクサマリーを表すデータクラス。"""

    # 対象の軸名（例: "tier1", "data", "security"）
    axis_name: str
    # 総資産数
    total_assets: int
    # 総脅威数
    total_threats: int
    # 総緩和策数
    total_mitigations: int
    # 未対処（open）脅威数
    open_threat_count: int
    # 高リスク（CVSS >= 7.0）脅威数
    high_risk_count: int
    # STRIDE カテゴリ別リスクスコア辞書
    risk_by_category: Dict[str, float] = field(default_factory=dict)
    # 全体リスクスコア（全カテゴリの加重平均）
    overall_risk_score: float = 0.0
    # レポート生成時刻
    generated_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

    def passes_ci_gate(self) -> bool:
        """CI ゲートを通過できるかどうかを返す。"""
        # 高リスク未対処脅威が 0 件であれば CI ゲートを通過できる
        return self.high_risk_count == 0 and self.open_threat_count <= _CI_MAX_OPEN_THREATS

    def risk_level(self) -> str:
        """全体リスクスコアに基づいたリスクレベルを返す。"""
        # スコアが 8.0 以上は CRITICAL
        if self.overall_risk_score >= 8.0:
            # 最高リスクレベルを返す
            return "CRITICAL"
        # スコアが 6.0 以上は HIGH
        if self.overall_risk_score >= 6.0:
            # 高リスクレベルを返す
            return "HIGH"
        # スコアが 4.0 以上は MEDIUM
        if self.overall_risk_score >= 4.0:
            # 中リスクレベルを返す
            return "MEDIUM"
        # スコアが 0.1 以上は LOW
        if self.overall_risk_score >= 0.1:
            # 低リスクレベルを返す
            return "LOW"
        # スコアが 0 に近い場合は MINIMAL
        return "MINIMAL"


@dataclass
class ThreatDetail:
    """個別脅威の詳細情報を表すデータクラス。"""

    # 脅威エントリの識別子
    threat_id: str
    # 脅威の STRIDE カテゴリ名
    category: str
    # 対象資産の識別子
    asset_id: str
    # CVSS スコア
    cvss_score: float
    # CVSS 重大度ラベル
    severity: str
    # 脅威の緩和ステータス
    mitigation_status: str
    # 関連する緩和策 ID のリスト
    mitigation_ids: List[str] = field(default_factory=list)
    # 脅威の詳細説明
    description: str = ""

    def to_dict(self) -> Dict[str, Any]:
        """脅威詳細を辞書形式で返す。"""
        # 辞書形式に変換して返す
        return {
            "threat_id": self.threat_id,
            "category": self.category,
            "asset_id": self.asset_id,
            "cvss_score": self.cvss_score,
            "severity": self.severity,
            "mitigation_status": self.mitigation_status,
            "mitigation_ids": self.mitigation_ids,
            "description": self.description,
        }


@dataclass
class RiskReport:
    """完全なリスクレポートを表すデータクラス。"""

    # レポートのサマリー
    summary: RiskSummary
    # 脅威詳細のリスト（CVSS 降順）
    threat_details: List[ThreatDetail] = field(default_factory=list)
    # 緩和策の実装状態リスト
    mitigation_status: List[Dict[str, Any]] = field(default_factory=list)
    # 検証結果
    validation: Optional[ValidationResult] = None
    # レポートのフォーマットバージョン
    report_version: str = "1.0"

    def to_dict(self) -> Dict[str, Any]:
        """レポート全体を辞書形式で返す。"""
        # サマリーを辞書に変換する
        summary_dict = {
            "axis_name": self.summary.axis_name,
            "total_assets": self.summary.total_assets,
            "total_threats": self.summary.total_threats,
            "total_mitigations": self.summary.total_mitigations,
            "open_threat_count": self.summary.open_threat_count,
            "high_risk_count": self.summary.high_risk_count,
            "overall_risk_score": self.summary.overall_risk_score,
            "risk_level": self.summary.risk_level(),
            "passes_ci_gate": self.summary.passes_ci_gate(),
            "generated_at": self.summary.generated_at,
            "risk_by_category": self.summary.risk_by_category,
        }
        # 脅威詳細リストを辞書リストに変換する
        threats_list = [t.to_dict() for t in self.threat_details]
        # 検証結果を辞書に変換する
        validation_dict: Dict[str, Any] = {}
        # 検証結果が存在する場合は変換する
        if self.validation:
            # 検証結果の主要フィールドを辞書に追加する
            validation_dict = {
                "is_valid": self.validation.is_valid,
                "implemented_count": self.validation.implemented_count,
                "missing_count": self.validation.missing_count,
                "errors": self.validation.errors,
                "warnings": self.validation.warnings,
            }
        # レポート全体を辞書形式で返す
        return {
            "report_version": self.report_version,
            "summary": summary_dict,
            "threats": threats_list,
            "mitigations": self.mitigation_status,
            "validation": validation_dict,
        }


# ===========================================================================
# RiskReporter クラス
# ===========================================================================


class RiskReporter:
    """STRIDE カタログからリスクレポートを生成するクラス。

    JSON・YAML・テキスト形式でのエクスポート、CI ゲート判定、
    ダッシュボード用データ生成を提供する。
    """

    def __init__(self, catalog: STRIDECatalog, axis_name: str = "unknown") -> None:
        """RiskReporter を STRIDE カタログで初期化する。

        Args:
            catalog: レポート生成の元となる STRIDECatalog インスタンス
            axis_name: レポートの対象軸名
        """
        # カタログを保存する
        self._catalog = catalog
        # 軸名を保存する
        self._axis_name = axis_name

    def build_report(self) -> RiskReport:
        """カタログ全体を評価してリスクレポートを生成して返す。"""
        # カタログのサマリーを取得する
        catalog_summary = self._catalog.summary_dict()
        # カテゴリ別リスクスコアを計算する
        risk_scores = self._catalog.compute_risk_score()
        # 全体リスクスコアを計算する（全カテゴリの最大値）
        overall_score = max(risk_scores.values()) if risk_scores else 0.0
        # カテゴリ別スコアを文字列キーの辞書に変換する
        risk_by_cat = {cat.value: score for cat, score in risk_scores.items()}
        # 高リスク脅威数を計算する
        high_risk_threats = self._catalog.get_high_risk_threats()
        # 未対処脅威数を計算する
        open_threats = self._catalog.get_open_threats()
        # RiskSummary を生成する
        summary = RiskSummary(
            axis_name=self._axis_name,
            total_assets=len(self._catalog.all_assets()),
            total_threats=len(self._catalog.all_threats()),
            total_mitigations=len(self._catalog.all_mitigations()),
            open_threat_count=len(open_threats),
            high_risk_count=len(high_risk_threats),
            risk_by_category=risk_by_cat,
            overall_risk_score=round(overall_score, 3),
        )
        # 脅威詳細リストを生成する（CVSS 降順）
        threat_details = self._build_threat_details()
        # 緩和策ステータスリストを生成する
        mitigation_status = self._build_mitigation_status()
        # 緩和策の検証を実行する
        validation = self._catalog.validate_mitigations()
        # RiskReport を生成して返す
        report = RiskReport(
            summary=summary,
            threat_details=threat_details,
            mitigation_status=mitigation_status,
            validation=validation,
        )
        # レポート生成をログに記録する
        logger.info(
            "リスクレポート生成完了: axis=%s overall=%.3f ci_pass=%s",
            self._axis_name,
            overall_score,
            summary.passes_ci_gate(),
        )
        # レポートを返す
        return report

    def _build_threat_details(self) -> List[ThreatDetail]:
        """全脅威エントリから ThreatDetail リストを生成して返す（CVSS 降順）。"""
        # 脅威詳細リストを初期化する
        details: List[ThreatDetail] = []
        # 全脅威エントリをイテレートする
        for threat in self._catalog.all_threats():
            # CVSS 重大度ラベルを計算する
            severity = CvssCalculator.classify_severity(threat.cvss_score)
            # ThreatDetail を生成する
            detail = ThreatDetail(
                threat_id=threat.threat_id,
                category=threat.category.value,
                asset_id=threat.asset_id,
                cvss_score=threat.cvss_score,
                severity=severity,
                mitigation_status=threat.mitigation_status,
                mitigation_ids=threat.mitigation_ids,
                description=threat.description,
            )
            # 詳細リストに追加する
            details.append(detail)
        # CVSS スコアの降順でソートする
        details.sort(key=lambda d: d.cvss_score, reverse=True)
        # 脅威詳細リストを返す
        return details

    def _build_mitigation_status(self) -> List[Dict[str, Any]]:
        """全緩和策エントリのステータスリストを生成して返す。"""
        # ステータスリストを初期化する
        status_list: List[Dict[str, Any]] = []
        # 全緩和策エントリをイテレートする
        for mit in self._catalog.all_mitigations():
            # ステータス辞書を生成する
            status = {
                "mitigation_id": mit.mitigation_id,
                "technique": mit.technique,
                "implementation_axis": mit.implementation_axis,
                "effectiveness": mit.effectiveness,
                "is_implemented": mit.is_implemented,
                "evidence": mit.evidence,
            }
            # ステータスリストに追加する
            status_list.append(status)
        # ステータスリストを返す
        return status_list

    def export_json(self, output_path: Optional[Path] = None) -> str:
        """リスクレポートを JSON 文字列として生成して返す。

        Args:
            output_path: JSON ファイルの出力先パス（省略時はファイル出力しない）

        Returns:
            JSON 形式のレポート文字列
        """
        # レポートを生成する
        report = self.build_report()
        # 辞書形式に変換する
        report_dict = report.to_dict()
        # JSON 文字列に変換する（インデント付き、日本語文字はそのまま）
        json_str = json.dumps(report_dict, indent=2, ensure_ascii=False)
        # 出力先が指定されている場合はファイルに書き込む
        if output_path is not None:
            # 親ディレクトリを作成する（存在しない場合）
            output_path.parent.mkdir(parents=True, exist_ok=True)
            # JSON ファイルに書き込む
            output_path.write_text(json_str, encoding="utf-8")
            # ファイル出力をログに記録する
            logger.info("JSON レポート出力: %s", output_path)
        # JSON 文字列を返す
        return json_str

    def export_yaml(self, output_path: Optional[Path] = None) -> str:
        """リスクレポートを YAML 文字列として生成して返す。

        Args:
            output_path: YAML ファイルの出力先パス（省略時はファイル出力しない）

        Returns:
            YAML 形式のレポート文字列

        Raises:
            ImportError: PyYAML がインストールされていない場合
        """
        # PyYAML が利用可能かどうかを確認する
        if not _HAS_YAML:
            # PyYAML が存在しない場合はエラーを送出する
            raise ImportError("PyYAML が必要です。pip install pyyaml を実行してください。")
        # レポートを生成して辞書形式に変換する
        report = self.build_report()
        # 辞書形式に変換する
        report_dict = report.to_dict()
        # YAML 文字列に変換する（allow_unicode=True で日本語を維持）
        yaml_str = yaml.dump(
            report_dict,
            allow_unicode=True,
            default_flow_style=False,
            sort_keys=True,
        )
        # 出力先が指定されている場合はファイルに書き込む
        if output_path is not None:
            # 親ディレクトリを作成する（存在しない場合）
            output_path.parent.mkdir(parents=True, exist_ok=True)
            # YAML ファイルに書き込む
            output_path.write_text(yaml_str, encoding="utf-8")
            # ファイル出力をログに記録する
            logger.info("YAML レポート出力: %s", output_path)
        # YAML 文字列を返す
        return yaml_str

    def export_text_summary(self) -> str:
        """人間が読みやすいテキスト形式のサマリーを生成して返す。"""
        # レポートを生成する
        report = self.build_report()
        # サマリーを取得する
        s = report.summary
        # テキストサマリーの行リストを初期化する
        lines: List[str] = []
        # ヘッダーを追加する
        lines.append("=" * 60)
        # タイトルを追加する
        lines.append(f"STRIDE リスクレポート: {s.axis_name}")
        # 生成時刻を追加する
        lines.append(f"生成時刻: {s.generated_at}")
        # 区切り線を追加する
        lines.append("=" * 60)
        # 資産・脅威・緩和策のサマリーを追加する
        lines.append(f"資産数: {s.total_assets}")
        # 脅威数を追加する
        lines.append(f"脅威数: {s.total_threats}")
        # 緩和策数を追加する
        lines.append(f"緩和策数: {s.total_mitigations}")
        # 未対処脅威数を追加する
        lines.append(f"未対処脅威数: {s.open_threat_count}")
        # 高リスク脅威数を追加する
        lines.append(f"高リスク脅威数: {s.high_risk_count}")
        # 全体リスクスコアを追加する
        lines.append(f"全体リスクスコア: {s.overall_risk_score:.3f} [{s.risk_level()}]")
        # CI ゲートの判定結果を追加する
        ci_result = "PASS" if s.passes_ci_gate() else "FAIL"
        # CI ゲートの判定結果を追加する
        lines.append(f"CI ゲート: {ci_result}")
        # 区切り線を追加する
        lines.append("-" * 60)
        # カテゴリ別リスクスコアを追加する
        lines.append("カテゴリ別リスクスコア:")
        # 各カテゴリのスコアを追加する
        for cat, score in sorted(s.risk_by_category.items()):
            # カテゴリとスコアを追加する
            lines.append(f"  {cat}: {score:.3f}")
        # 区切り線を追加する
        lines.append("-" * 60)
        # 高リスク脅威の詳細を追加する
        high_risk_details = [d for d in report.threat_details if d.cvss_score >= 7.0]
        # 高リスク脅威のセクションを追加する
        if high_risk_details:
            # 高リスク脅威のヘッダーを追加する
            lines.append("高リスク脅威（CVSS >= 7.0）:")
            # 各高リスク脅威の詳細を追加する
            for detail in high_risk_details:
                # 脅威の詳細行を追加する
                lines.append(
                    f"  [{detail.severity}] {detail.threat_id} "
                    f"(CVSS={detail.cvss_score:.1f}, "
                    f"status={detail.mitigation_status})"
                )
                # 説明が存在する場合は追加する
                if detail.description:
                    # 説明を 50 文字で折り返す
                    wrapped = textwrap.fill(
                        f"    {detail.description}",
                        width=70,
                        subsequent_indent="    ",
                    )
                    # 折り返した説明を追加する
                    lines.append(wrapped)
        # フッターを追加する
        lines.append("=" * 60)
        # テキストサマリーを改行で結合して返す
        return "\n".join(lines)

    def check_ci_gate(self) -> Tuple[bool, List[str]]:
        """CI ゲートの判定を実行して (通過フラグ, 失敗理由リスト) を返す。

        Returns:
            (CI ゲート通過の場合 True, 失敗理由のリスト) のタプル
        """
        # レポートを生成する
        report = self.build_report()
        # 失敗理由のリストを初期化する
        failures: List[str] = []
        # 高リスク未対処脅威を確認する
        for detail in report.threat_details:
            # 高リスクかつ未対処の場合は失敗理由に追加する
            if detail.cvss_score >= _CI_FAIL_THRESHOLD_CVSS and detail.mitigation_status == "open":
                # 失敗理由を追加する
                failures.append(
                    f"高リスク未対処脅威: {detail.threat_id} "
                    f"(CVSS={detail.cvss_score:.1f}, "
                    f"category={detail.category})"
                )
        # 検証エラーを確認する
        if report.validation:
            # 検証エラーがある場合は失敗理由に追加する
            for error in report.validation.errors:
                # 検証エラーを失敗理由に追加する
                failures.append(f"検証エラー: {error}")
        # 失敗理由が存在する場合は CI ゲート失敗を返す
        ci_passed = len(failures) == 0
        # CI ゲート判定をログに記録する
        logger.info(
            "CI ゲート判定: %s (%d 件の失敗理由)",
            "PASS" if ci_passed else "FAIL",
            len(failures),
        )
        # (通過フラグ, 失敗理由リスト) のタプルを返す
        return ci_passed, failures

    def generate_delta_report(
        self,
        previous_report_path: Path,
    ) -> Dict[str, Any]:
        """前回のレポートとの差分を計算して差分レポートを返す。

        Args:
            previous_report_path: 前回のレポート JSON ファイルパス

        Returns:
            差分レポートの辞書
        """
        # 前回のレポートファイルが存在するかを確認する
        if not previous_report_path.exists():
            # 前回のレポートが存在しない場合は差分なしを返す
            logger.info("前回のレポートが存在しません。初回レポートとして扱います。")
            # 初回レポートの差分辞書を返す
            return {
                "is_first_run": True,
                "previous_path": str(previous_report_path),
                "delta": {},
            }
        # 前回のレポート JSON を読み込む
        try:
            # ファイルを UTF-8 で読み込む
            prev_raw = previous_report_path.read_text(encoding="utf-8")
            # JSON をパースする
            prev_data = json.loads(prev_raw)
        except (json.JSONDecodeError, OSError) as exc:
            # 読み込みエラーをログに記録する
            logger.error("前回レポート読み込みエラー: %s (%s)", previous_report_path, exc)
            # エラーの場合は差分なしを返す
            return {"error": str(exc)}
        # 現在のレポートを生成する
        current_report = self.build_report()
        # 現在のレポートを辞書形式に変換する
        current_data = current_report.to_dict()
        # 前回サマリーを取得する
        prev_summary = prev_data.get("summary", {})
        # 現在サマリーを取得する
        curr_summary = current_data.get("summary", {})
        # 差分を計算する
        delta: Dict[str, Any] = {}
        # 未対処脅威数の差分を計算する
        prev_open = prev_summary.get("open_threat_count", 0)
        # 現在の未対処脅威数を取得する
        curr_open = curr_summary.get("open_threat_count", 0)
        # 未対処脅威数の差分を記録する
        delta["open_threat_delta"] = curr_open - prev_open
        # 高リスク脅威数の差分を計算する
        prev_high = prev_summary.get("high_risk_count", 0)
        # 現在の高リスク脅威数を取得する
        curr_high = curr_summary.get("high_risk_count", 0)
        # 高リスク脅威数の差分を記録する
        delta["high_risk_delta"] = curr_high - prev_high
        # 全体リスクスコアの差分を計算する
        prev_score = float(prev_summary.get("overall_risk_score", 0.0))
        # 現在の全体リスクスコアを取得する
        curr_score = float(curr_summary.get("overall_risk_score", 0.0))
        # 全体リスクスコアの差分を記録する
        delta["overall_risk_delta"] = round(curr_score - prev_score, 3)
        # 差分レポートを構築して返す
        return {
            "is_first_run": False,
            "previous_path": str(previous_report_path),
            "generated_at": current_report.summary.generated_at,
            "delta": delta,
            "current_summary": curr_summary,
            "previous_summary": prev_summary,
        }

    def export_prometheus_metrics(self) -> str:
        """Prometheus テキスト形式のメトリクス文字列を生成して返す。"""
        # レポートを生成する
        report = self.build_report()
        # サマリーを取得する
        s = report.summary
        # メトリクス行のリストを初期化する
        lines: List[str] = []
        # メトリクスのヘルプとタイプを定義する
        lines.append("# HELP stride_total_threats STRIDE 脅威モデルの総脅威数")
        # メトリクスのタイプを定義する
        lines.append("# TYPE stride_total_threats gauge")
        # メトリクス値を追加する
        lines.append(f'stride_total_threats{{axis="{s.axis_name}"}} {s.total_threats}')
        # 未対処脅威数のメトリクスを追加する
        lines.append("# HELP stride_open_threats 未対処の STRIDE 脅威数")
        # メトリクスのタイプを定義する
        lines.append("# TYPE stride_open_threats gauge")
        # メトリクス値を追加する
        lines.append(f'stride_open_threats{{axis="{s.axis_name}"}} {s.open_threat_count}')
        # 高リスク脅威数のメトリクスを追加する
        lines.append("# HELP stride_high_risk_threats 高リスク（CVSS >= 7.0）の脅威数")
        # メトリクスのタイプを定義する
        lines.append("# TYPE stride_high_risk_threats gauge")
        # メトリクス値を追加する
        lines.append(f'stride_high_risk_threats{{axis="{s.axis_name}"}} {s.high_risk_count}')
        # 全体リスクスコアのメトリクスを追加する
        lines.append("# HELP stride_overall_risk_score 全体リスクスコア")
        # メトリクスのタイプを定義する
        lines.append("# TYPE stride_overall_risk_score gauge")
        # メトリクス値を追加する
        lines.append(
            f'stride_overall_risk_score{{axis="{s.axis_name}"}} {s.overall_risk_score}'
        )
        # カテゴリ別リスクスコアのメトリクスを追加する
        lines.append("# HELP stride_category_risk_score カテゴリ別リスクスコア")
        # メトリクスのタイプを定義する
        lines.append("# TYPE stride_category_risk_score gauge")
        # 各カテゴリのスコアをメトリクスに追加する
        for cat, score in s.risk_by_category.items():
            # カテゴリ別スコアのメトリクス値を追加する
            lines.append(
                f'stride_category_risk_score{{axis="{s.axis_name}",category="{cat}"}} {score}'
            )
        # CI ゲートのメトリクスを追加する
        lines.append("# HELP stride_ci_gate_passed CI ゲートを通過したかどうか (1=pass, 0=fail)")
        # メトリクスのタイプを定義する
        lines.append("# TYPE stride_ci_gate_passed gauge")
        # CI ゲートの判定値を追加する
        ci_value = 1 if s.passes_ci_gate() else 0
        # CI ゲートメトリクス値を追加する
        lines.append(f'stride_ci_gate_passed{{axis="{s.axis_name}"}} {ci_value}')
        # メトリクス文字列を改行で結合して返す
        return "\n".join(lines) + "\n"
