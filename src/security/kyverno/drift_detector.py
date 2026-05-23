"""src/security/kyverno/drift_detector.py

Kyverno ポリシードリフト検出モジュール。
実際にクラスタに適用されているポリシーと、カタログで期待されるポリシーの
差分（追加・削除・変更）を検出し、ドリフトレポートを生成する。
仕様: docs/04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md
"""

# 将来の注釈構文互換のため annotations をインポートする
from __future__ import annotations

# ハッシュ計算に使用する hashlib をインポートする
import hashlib
# JSON 処理に使用する json をインポートする
import json
# ログ出力に使用する logging をインポートする
import logging
# 正規表現に使用する re をインポートする
import re
# サブプロセス実行に使用する subprocess をインポートする
import subprocess
# システム操作に使用する sys をインポートする
import sys
# 型安全なデータクラスに使用する dataclasses をインポートする
from dataclasses import dataclass, field
# 日時処理に使用する datetime をインポートする
from datetime import datetime, timezone
# パス操作に使用する pathlib をインポートする
from pathlib import Path
# 型ヒント定義に使用する typing をインポートする
from typing import Any, Dict, List, Optional, Set

# PyYAML ライブラリのインポートを試みる
try:
    # YAML ファイルの読み込みに使用する yaml をインポートする
    import yaml
    # yaml が利用可能であることを示すフラグを設定する
    _HAS_YAML = True
except ImportError:
    # yaml がインストールされていない場合はエラーメッセージを出力する
    print("ERROR: PyYAML が必要です。pip install pyyaml を実行してください。", file=sys.stderr)
    # 前提条件不足のため終了する
    sys.exit(1)

# モジュール専用ロガーを取得する
logger = logging.getLogger(__name__)


# ===========================================================================
# データクラス定義
# ===========================================================================


@dataclass
class PolicySpec:
    """Kyverno ポリシーの仕様を表すデータクラス。"""

    # ポリシーの名称（例: "require-image-signature"）
    policy_name: str
    # ポリシーのバージョン文字列（例: "1.2.0"）
    version: str
    # ポリシーに含まれるルールの数
    rules_count: int
    # ポリシー内容の SHA-256 ハッシュ（正規化後）
    spec_hash: str
    # ポリシーの種別（"ClusterPolicy" or "Policy"）
    policy_kind: str = "ClusterPolicy"
    # ポリシーが有効化されているかどうか
    is_active: bool = True
    # ポリシーが適用される名前空間（空文字はクラスタスコープ）
    namespace: str = ""
    # ポリシーのラベル辞書
    labels: Dict[str, str] = field(default_factory=dict)
    # ポリシーの説明
    description: str = ""

    def matches_name(self, other: "PolicySpec") -> bool:
        """他のポリシースペックと名称が一致するかどうかを返す。"""
        # ポリシー名が一致するかを確認する
        return self.policy_name == other.policy_name

    def has_drift_from(self, other: "PolicySpec") -> bool:
        """指定したポリシースペックと差分があるかどうかを返す。"""
        # ハッシュが異なればドリフトがある
        if self.spec_hash != other.spec_hash:
            # ハッシュ差分があるため True を返す
            return True
        # ルール数が異なればドリフトがある
        if self.rules_count != other.rules_count:
            # ルール数差分があるため True を返す
            return True
        # バージョンが異なればドリフトがある
        if self.version != other.version:
            # バージョン差分があるため True を返す
            return True
        # 差分がない場合は False を返す
        return False


@dataclass
class DriftReport:
    """ポリシードリフト検出の結果レポートを表すデータクラス。"""

    # ドリフトが検出されたポリシーの総数
    drift_count: int
    # 期待リストにはなく実際には追加されているポリシーのリスト
    added: List[PolicySpec] = field(default_factory=list)
    # 実際には削除されているが期待リストには存在するポリシーのリスト
    removed: List[PolicySpec] = field(default_factory=list)
    # 名称は一致するが内容が変更されているポリシーのリスト
    modified: List[PolicySpec] = field(default_factory=list)
    # レポート生成時刻（ISO 8601 形式）
    generated_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

    @property
    def is_clean(self) -> bool:
        """ドリフトが一切検出されていないかどうかを返す。"""
        # ドリフト数が 0 であれば clean とみなす
        return self.drift_count == 0

    def summary(self) -> str:
        """ドリフトレポートのサマリー文字列を返す。"""
        # クリーン状態の場合はその旨を返す
        if self.is_clean:
            # ドリフトなしのメッセージを返す
            return "DriftReport: CLEAN（ドリフトなし）"
        # ドリフトあり・詳細をフォーマットして返す
        return (
            f"DriftReport: DRIFT DETECTED "
            f"added={len(self.added)} "
            f"removed={len(self.removed)} "
            f"modified={len(self.modified)} "
            f"total={self.drift_count}"
        )

    def to_dict(self) -> Dict[str, Any]:
        """ドリフトレポートを辞書形式に変換して返す。"""
        # レポートを辞書形式に変換する
        return {
            "generated_at": self.generated_at,
            "is_clean": self.is_clean,
            "drift_count": self.drift_count,
            "added": [_policy_spec_to_dict(p) for p in self.added],
            "removed": [_policy_spec_to_dict(p) for p in self.removed],
            "modified": [_policy_spec_to_dict(p) for p in self.modified],
        }


def _policy_spec_to_dict(spec: PolicySpec) -> Dict[str, Any]:
    """PolicySpec を辞書形式に変換するヘルパー関数。"""
    # PolicySpec のフィールドを辞書に変換して返す
    return {
        "policy_name": spec.policy_name,
        "version": spec.version,
        "rules_count": spec.rules_count,
        "spec_hash": spec.spec_hash,
        "policy_kind": spec.policy_kind,
        "is_active": spec.is_active,
        "namespace": spec.namespace,
        "description": spec.description,
    }


# ===========================================================================
# ハッシュ計算ユーティリティ
# ===========================================================================


def _normalize_yaml_content(raw_yaml: str) -> str:
    """YAML コンテンツを正規化して比較可能な文字列を返す。

    メタデータ（creationTimestamp / resourceVersion / uid / generation）を
    除去してから正規化することで、Kubernetes が付与する可変フィールドを無視する。
    """
    # YAML を辞書にパースする
    try:
        # YAML 文字列を安全にパースする
        data = yaml.safe_load(raw_yaml)
    except yaml.YAMLError as exc:
        # パースエラーの場合はそのまま文字列を返す
        logger.warning("YAML パースエラー（正規化スキップ）: %s", exc)
        # 元の文字列を返す
        return raw_yaml
    # data が辞書でない場合は文字列変換して返す
    if not isinstance(data, dict):
        # 辞書でない場合は JSON シリアライズして返す
        return json.dumps(data, sort_keys=True)
    # metadata から可変フィールドを除去する
    metadata = data.get("metadata", {})
    # 除去するフィールドのリストを定義する
    volatile_fields = [
        "creationTimestamp",
        "resourceVersion",
        "uid",
        "generation",
        "managedFields",
        "annotations",
    ]
    # 可変フィールドを metadata から削除する
    for field_name in volatile_fields:
        # フィールドが存在する場合は削除する
        metadata.pop(field_name, None)
    # status フィールドを除去する（ランタイム状態は比較対象外）
    data.pop("status", None)
    # 正規化した辞書を JSON で文字列化して返す（ソートして決定論的にする）
    return json.dumps(data, sort_keys=True, ensure_ascii=False)


def _compute_spec_hash(raw_yaml: str) -> str:
    """YAML コンテンツの SHA-256 ハッシュ文字列を計算して返す。"""
    # YAML コンテンツを正規化する
    normalized = _normalize_yaml_content(raw_yaml)
    # SHA-256 ハッシュを計算して16進数文字列で返す
    return hashlib.sha256(normalized.encode("utf-8")).hexdigest()


def _count_rules_in_yaml(data: Dict[str, Any]) -> int:
    """Kyverno ポリシー辞書からルール数を計算して返す。"""
    # spec.rules フィールドを取得する
    spec = data.get("spec", {})
    # rules フィールドがリストの場合はその長さを返す
    rules = spec.get("rules", [])
    # rules がリストでない場合は 0 を返す
    if not isinstance(rules, list):
        # ルールがリスト形式でない場合は 0 とする
        return 0
    # ルールの数を返す
    return len(rules)


def _extract_version_from_labels(labels: Dict[str, str]) -> str:
    """ポリシーのラベルからバージョン文字列を抽出して返す。"""
    # app.kubernetes.io/version ラベルを確認する
    if "app.kubernetes.io/version" in labels:
        # バージョンラベルが存在する場合はその値を返す
        return labels["app.kubernetes.io/version"]
    # version ラベルを確認する
    if "version" in labels:
        # version ラベルが存在する場合はその値を返す
        return labels["version"]
    # バージョン情報が見つからない場合はデフォルト値を返す
    return "0.0.0"


# ===========================================================================
# DriftDetector クラス
# ===========================================================================


class DriftDetector:
    """Kyverno ポリシードリフトを検出するクラス。

    実際のポリシーディレクトリまたはクラスタから取得したポリシーと
    カタログで期待されるポリシーを比較してドリフトを検出する。
    """

    def __init__(self, policies_dir: Optional[Path] = None, catalog_path: Optional[Path] = None) -> None:
        """DriftDetector を初期化する。

        Args:
            policies_dir: 実際のポリシー YAML ファイルが格納されているディレクトリ
            catalog_path: 期待ポリシーカタログの YAML ファイルパス
        """
        # 実際のポリシーディレクトリを設定する
        self._policies_dir = policies_dir
        # 期待ポリシーカタログのパスを設定する
        self._catalog_path = catalog_path
        # 実際のポリシーキャッシュを初期化する
        self._actual_policies: Optional[List[PolicySpec]] = None
        # 期待ポリシーキャッシュを初期化する
        self._expected_policies: Optional[List[PolicySpec]] = None

    def load_policies_from_dir(self, path: Path) -> List[PolicySpec]:
        """ディレクトリ内の Kyverno ポリシー YAML ファイルを読み込んで返す。

        Args:
            path: ポリシー YAML ファイルが格納されているディレクトリ

        Returns:
            読み込んだ PolicySpec のリスト
        """
        # ディレクトリが存在するかどうかを確認する
        if not path.exists():
            # ディレクトリが存在しない場合は空リストを返す
            logger.warning("ポリシーディレクトリが存在しません: %s", path)
            # 空リストを返す
            return []
        # 読み込んだポリシーリストを初期化する
        policies: List[PolicySpec] = []
        # ディレクトリ内の全 YAML ファイルを検索する
        yaml_files = list(path.glob("*.yaml")) + list(path.glob("*.yml"))
        # YAML ファイルが存在しない場合はログに記録する
        if not yaml_files:
            # YAML ファイルが見つからない旨をログに記録する
            logger.warning("ポリシーディレクトリに YAML ファイルが見つかりません: %s", path)
        # 各 YAML ファイルを読み込んで PolicySpec に変換する
        for yaml_file in sorted(yaml_files):
            # YAML ファイルを読み込む
            try:
                # ファイルを UTF-8 で読み込む
                raw_content = yaml_file.read_text(encoding="utf-8")
                # YAML をパースする
                data = yaml.safe_load(raw_content)
                # data が辞書でない場合はスキップする
                if not isinstance(data, dict):
                    # 辞書形式でないファイルはスキップする
                    logger.debug("非辞書 YAML をスキップ: %s", yaml_file)
                    # 次のファイルに進む
                    continue
                # kind フィールドを確認する（ClusterPolicy または Policy のみ処理）
                kind = data.get("kind", "")
                # Kyverno ポリシーでない場合はスキップする
                if kind not in ("ClusterPolicy", "Policy"):
                    # Kyverno ポリシーでないのでスキップする
                    continue
                # メタデータを取得する
                metadata = data.get("metadata", {})
                # ポリシー名を取得する
                policy_name = metadata.get("name", yaml_file.stem)
                # ラベルを取得する
                labels = metadata.get("labels", {})
                # バージョンをラベルから抽出する
                version = _extract_version_from_labels(labels)
                # ルール数を計算する
                rules_count = _count_rules_in_yaml(data)
                # スペックハッシュを計算する
                spec_hash = _compute_spec_hash(raw_content)
                # 説明を取得する
                annotations = metadata.get("annotations", {})
                # 説明アノテーションを取得する
                description = annotations.get("policies.kyverno.io/description", "")
                # PolicySpec を生成してリストに追加する
                spec = PolicySpec(
                    policy_name=policy_name,
                    version=version,
                    rules_count=rules_count,
                    spec_hash=spec_hash,
                    policy_kind=kind,
                    labels=labels,
                    description=description,
                )
                # ポリシーリストに追加する
                policies.append(spec)
                # 読み込み成功をログに記録する
                logger.debug("ポリシー読み込み: %s (rules=%d)", policy_name, rules_count)
            except (yaml.YAMLError, OSError, KeyError) as exc:
                # 読み込みエラーをログに記録してスキップする
                logger.warning("ポリシーファイル読み込みエラー: %s (%s)", yaml_file, exc)
        # 読み込んだポリシーリストをキャッシュに保存する
        self._actual_policies = policies
        # ポリシー読み込み完了をログに記録する
        logger.info("ポリシーディレクトリ読み込み完了: %d 件 (%s)", len(policies), path)
        # 読み込んだポリシーリストを返す
        return policies

    def load_expected_policies_from_catalog(self, catalog_path: Path) -> List[PolicySpec]:
        """カタログ YAML ファイルから期待するポリシーリストを読み込んで返す。

        Args:
            catalog_path: 期待ポリシーカタログの YAML ファイルパス

        Returns:
            期待する PolicySpec のリスト
        """
        # カタログファイルが存在するかどうかを確認する
        if not catalog_path.exists():
            # カタログファイルが存在しない場合は空リストを返す
            logger.warning("カタログファイルが存在しません: %s", catalog_path)
            # 空リストを返す
            return []
        # カタログ YAML ファイルを読み込む
        try:
            # ファイルを UTF-8 で読み込む
            raw_content = catalog_path.read_text(encoding="utf-8")
            # YAML をパースする
            catalog_data = yaml.safe_load(raw_content)
        except (yaml.YAMLError, OSError) as exc:
            # 読み込みエラーをログに記録する
            logger.error("カタログファイル読み込みエラー: %s (%s)", catalog_path, exc)
            # 空リストを返す
            return []
        # catalog_data が辞書でない場合は空リストを返す
        if not isinstance(catalog_data, dict):
            # 不正な構造の場合は空リストを返す
            logger.error("カタログ YAML のルートは辞書でなければなりません: %s", catalog_path)
            # 空リストを返す
            return []
        # 期待ポリシーリストを初期化する
        expected: List[PolicySpec] = []
        # policies セクションをイテレートする
        for policy_raw in catalog_data.get("policies", []):
            # 辞書でない場合はスキップする
            if not isinstance(policy_raw, dict):
                # 辞書でないエントリはスキップする
                continue
            # ポリシー名を取得する（必須）
            policy_name = policy_raw.get("name", "")
            # ポリシー名が空の場合はスキップする
            if not policy_name:
                # 名前がないエントリはスキップする
                continue
            # バージョンを取得する
            version = policy_raw.get("version", "0.0.0")
            # 期待ルール数を取得する
            rules_count = int(policy_raw.get("rules_count", 0))
            # 期待ハッシュを取得する（空文字は「チェックしない」を意味する）
            spec_hash = policy_raw.get("spec_hash", "")
            # ポリシー種別を取得する
            policy_kind = policy_raw.get("kind", "ClusterPolicy")
            # 説明を取得する
            description = policy_raw.get("description", "")
            # PolicySpec を生成して期待リストに追加する
            spec = PolicySpec(
                policy_name=policy_name,
                version=version,
                rules_count=rules_count,
                spec_hash=spec_hash,
                policy_kind=policy_kind,
                description=description,
            )
            # 期待ポリシーリストに追加する
            expected.append(spec)
        # 読み込んだ期待ポリシーリストをキャッシュに保存する
        self._expected_policies = expected
        # 期待ポリシー読み込み完了をログに記録する
        logger.info("期待ポリシーカタログ読み込み完了: %d 件 (%s)", len(expected), catalog_path)
        # 期待ポリシーリストを返す
        return expected

    def report_added_policies(
        self, actual: List[PolicySpec], expected: List[PolicySpec]
    ) -> List[PolicySpec]:
        """期待リストにはなく実際には追加されているポリシーのリストを返す。

        Args:
            actual: 実際に存在するポリシーのリスト
            expected: 期待されるポリシーのリスト

        Returns:
            期待リストにない追加ポリシーのリスト
        """
        # 期待ポリシー名のセットを作成する
        expected_names: Set[str] = {p.policy_name for p in expected}
        # 期待リストにない実際のポリシーをフィルタリングして返す
        return [p for p in actual if p.policy_name not in expected_names]

    def report_removed_policies(
        self, actual: List[PolicySpec], expected: List[PolicySpec]
    ) -> List[PolicySpec]:
        """実際には削除されているが期待リストには存在するポリシーのリストを返す。

        Args:
            actual: 実際に存在するポリシーのリスト
            expected: 期待されるポリシーのリスト

        Returns:
            実際に存在しない削除済みポリシーのリスト
        """
        # 実際のポリシー名のセットを作成する
        actual_names: Set[str] = {p.policy_name for p in actual}
        # 実際には存在しない期待ポリシーをフィルタリングして返す
        return [p for p in expected if p.policy_name not in actual_names]

    def report_modified_policies(
        self, actual: List[PolicySpec], expected: List[PolicySpec]
    ) -> List[PolicySpec]:
        """名称は一致するが内容が変更されているポリシーのリストを返す。

        Args:
            actual: 実際に存在するポリシーのリスト
            expected: 期待されるポリシーのリスト

        Returns:
            ドリフトが検出された変更済みポリシーのリスト（実際のスペックを返す）
        """
        # 期待ポリシーの名前 → スペック辞書を作成する
        expected_by_name: Dict[str, PolicySpec] = {p.policy_name: p for p in expected}
        # 変更されたポリシーのリストを初期化する
        modified: List[PolicySpec] = []
        # 実際のポリシーをイテレートして変更を検出する
        for actual_spec in actual:
            # 期待リストに同名のポリシーが存在するか確認する
            if actual_spec.policy_name not in expected_by_name:
                # 期待リストにない場合はスキップ（追加扱い）
                continue
            # 期待ポリシーを取得する
            expected_spec = expected_by_name[actual_spec.policy_name]
            # 期待ハッシュが空の場合はハッシュ比較をスキップする
            if expected_spec.spec_hash and actual_spec.has_drift_from(expected_spec):
                # ドリフトが検出されたので変更リストに追加する
                modified.append(actual_spec)
            # ルール数のみチェックする（ハッシュが空の場合）
            elif not expected_spec.spec_hash and actual_spec.rules_count != expected_spec.rules_count:
                # ルール数が異なる場合も変更リストに追加する
                modified.append(actual_spec)
        # 変更済みポリシーのリストを返す
        return modified

    def detect_drift(self) -> DriftReport:
        """ドリフト検出を実行して DriftReport を返す。

        事前に load_policies_from_dir と load_expected_policies_from_catalog を
        呼び出してキャッシュを設定しておく必要がある。

        Returns:
            ドリフト検出レポート

        Raises:
            RuntimeError: ポリシーが読み込まれていない場合
        """
        # 実際のポリシーが読み込まれているか確認する
        if self._actual_policies is None:
            # 実際のポリシーが未読み込みの場合は RuntimeError を送出する
            if self._policies_dir is not None:
                # ディレクトリが設定されている場合は自動読み込みする
                self.load_policies_from_dir(self._policies_dir)
            else:
                # ディレクトリが設定されていない場合はエラーを送出する
                raise RuntimeError(
                    "実際のポリシーが読み込まれていません。"
                    "load_policies_from_dir() を先に呼び出してください。"
                )
        # 期待ポリシーが読み込まれているか確認する
        if self._expected_policies is None:
            # 期待ポリシーが未読み込みの場合は RuntimeError を送出する
            if self._catalog_path is not None:
                # カタログパスが設定されている場合は自動読み込みする
                self.load_expected_policies_from_catalog(self._catalog_path)
            else:
                # カタログパスが設定されていない場合はエラーを送出する
                raise RuntimeError(
                    "期待ポリシーが読み込まれていません。"
                    "load_expected_policies_from_catalog() を先に呼び出してください。"
                )
        # 追加・削除・変更されたポリシーを検出する
        actual = self._actual_policies or []
        # 期待ポリシーリストを取得する
        expected = self._expected_policies or []
        # 追加されたポリシーを検出する
        added = self.report_added_policies(actual, expected)
        # 削除されたポリシーを検出する
        removed = self.report_removed_policies(actual, expected)
        # 変更されたポリシーを検出する
        modified = self.report_modified_policies(actual, expected)
        # ドリフト総数を計算する
        drift_count = len(added) + len(removed) + len(modified)
        # DriftReport を生成して返す
        report = DriftReport(
            drift_count=drift_count,
            added=added,
            removed=removed,
            modified=modified,
        )
        # ドリフト検出結果をログに記録する
        logger.info("ドリフト検出完了: %s", report.summary())
        # ドリフトレポートを返す
        return report

    def detect_drift_from_paths(
        self, policies_dir: Path, catalog_path: Path
    ) -> DriftReport:
        """ディレクトリとカタログパスを指定して直接ドリフト検出を実行して返す。

        Args:
            policies_dir: 実際のポリシーディレクトリ
            catalog_path: 期待ポリシーカタログ YAML

        Returns:
            ドリフト検出レポート
        """
        # 実際のポリシーをディレクトリから読み込む
        self.load_policies_from_dir(policies_dir)
        # 期待ポリシーをカタログから読み込む
        self.load_expected_policies_from_catalog(catalog_path)
        # ドリフト検出を実行して返す
        return self.detect_drift()

    def generate_remediation_plan(self, report: DriftReport) -> List[str]:
        """ドリフトレポートから修正計画コマンドのリストを生成して返す。

        Args:
            report: ドリフト検出レポート

        Returns:
            修正のための kubectl コマンドのリスト
        """
        # 修正コマンドのリストを初期化する
        commands: List[str] = []
        # ドリフトがない場合は空リストを返す
        if report.is_clean:
            # ドリフトなしのためコマンドなし
            return commands
        # 削除されたポリシーを再適用するコマンドを追加する
        for removed_policy in report.removed:
            # 削除されたポリシーの再適用コマンドを追加する
            commands.append(
                f"# 削除されたポリシーを再適用する: {removed_policy.policy_name}"
            )
            # kubectl apply コマンドを追加する
            commands.append(
                f"kubectl apply -f src/security/kyverno/policies/{removed_policy.policy_name}.yaml"
            )
        # 変更されたポリシーを更新するコマンドを追加する
        for modified_policy in report.modified:
            # 変更されたポリシーの更新コマンドを追加する
            commands.append(
                f"# 変更されたポリシーを更新する: {modified_policy.policy_name}"
            )
            # kubectl apply コマンドを追加する
            commands.append(
                f"kubectl apply -f src/security/kyverno/policies/{modified_policy.policy_name}.yaml"
            )
        # 追加されたポリシーを確認するコマンドを追加する
        for added_policy in report.added:
            # 追加されたポリシーの確認コマンドを追加する
            commands.append(
                f"# 未期待ポリシーを確認する: {added_policy.policy_name}"
            )
            # kubectl get コマンドを追加する
            commands.append(
                f"kubectl get clusterpolicy {added_policy.policy_name} -o yaml"
            )
        # 修正計画コマンドのリストを返す
        return commands
