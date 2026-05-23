"""src/security/threat_model/stride_catalog.py

STRIDE 脅威カタログ管理モジュール。
製造業プラットフォーム k1s0 の全資産に対する脅威エントリを YAML から読み込み、
リスクスコア計算・緩和策検証を提供する。
仕様: docs/04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md
"""

# 将来の注釈構文互換のため annotations をインポートする
from __future__ import annotations

# 列挙型定義に使用する enum をインポートする
import enum
# ハッシュ計算に使用する hashlib をインポートする
import hashlib
# JSON 処理に使用する json をインポートする
import json
# ログ出力に使用する logging をインポートする
import logging
# 数学関数（対数・指数）に使用する math をインポートする
import math
# OS 操作に使用する os をインポートする
import os
# システム操作に使用する sys をインポートする
import sys
# 型安全なデータクラスに使用する dataclasses をインポートする
from dataclasses import dataclass, field
# パス操作に使用する pathlib をインポートする
from pathlib import Path
# 型ヒント定義に使用する typing をインポートする
from typing import Any, Dict, List, Optional

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
# 列挙型定義
# ===========================================================================


class ThreatCategory(enum.Enum):
    """STRIDE 脅威モデルの 6 カテゴリを表す列挙型。"""

    # なりすまし（Spoofing）: 正当な主体になりすます攻撃
    SPOOFING = "Spoofing"
    # 改ざん（Tampering）: データや機能を不正に変更する攻撃
    TAMPERING = "Tampering"
    # 否認（Repudiation）: 実行した操作を否定できる状態
    REPUDIATION = "Repudiation"
    # 情報漏洩（Information Disclosure）: 機密情報への不正アクセス
    INFO_DISCLOSURE = "InformationDisclosure"
    # サービス拒否（Denial of Service）: サービスの可用性を妨害する攻撃
    DENIAL_OF_SERVICE = "DenialOfService"
    # 特権昇格（Elevation of Privilege）: 不正に高い権限を取得する攻撃
    ELEVATION_OF_PRIVILEGE = "ElevationOfPrivilege"


# ===========================================================================
# データクラス定義
# ===========================================================================


@dataclass
class Asset:
    """保護対象の資産を表すデータクラス。"""

    # 資産の一意識別子（例: "asset_auth_service"）
    asset_id: str
    # 資産の可読名称（例: "認証サービス"）
    asset_name: str
    # 資産の種別（例: "service", "database", "api_endpoint"）
    asset_type: str
    # 機密レベル（1=低, 2=中, 3=高, 4=極秘）
    sensitivity_level: int
    # 資産を所有する軸の識別子（例: "tier1", "data"）
    owner_axis: str

    def validate(self) -> bool:
        """資産データが有効かどうかを検証する。"""
        # asset_id が空でないことを確認する
        if not self.asset_id:
            # asset_id が空の場合は無効とする
            return False
        # sensitivity_level が 1〜4 の範囲内であることを確認する
        if not (1 <= self.sensitivity_level <= 4):
            # 範囲外の場合は無効とする
            return False
        # 全検証が通過した場合は有効とする
        return True


@dataclass
class ThreatEntry:
    """個別の脅威エントリを表すデータクラス。"""

    # 脅威エントリの一意識別子（例: "threat_001"）
    threat_id: str
    # 脅威が属する STRIDE カテゴリ
    category: ThreatCategory
    # 対象資産の識別子
    asset_id: str
    # 攻撃ベクター（例: "network", "local", "physical"）
    attack_vector: str
    # 発生可能性（0.0〜1.0）
    likelihood: float
    # 影響度（0.0〜1.0）
    impact: float
    # CVSS スコア（0.0〜10.0、-1.0 は未計算）
    cvss_score: float = -1.0
    # 関連する緩和策 ID のリスト
    mitigation_ids: List[str] = field(default_factory=list)
    # 脅威の詳細説明
    description: str = ""
    # 緩和ステータス（"open", "mitigated", "accepted"）
    mitigation_status: str = "open"

    def is_open(self) -> bool:
        """脅威がオープン（未対処）状態かどうかを返す。"""
        # mitigation_status が "open" かどうかを確認する
        return self.mitigation_status == "open"

    def is_high_risk(self) -> bool:
        """脅威が高リスク（CVSS >= 7.0）かどうかを返す。"""
        # CVSS スコアが 7.0 以上であれば高リスクとみなす
        return self.cvss_score >= 7.0

    def compute_inherent_risk(self) -> float:
        """発生可能性と影響度から固有リスクスコアを計算する。"""
        # 固有リスク = 発生可能性 × 影響度 × 10（0〜10 のスケールに正規化）
        return self.likelihood * self.impact * 10.0


@dataclass
class MitigationEntry:
    """緩和策エントリを表すデータクラス。"""

    # 緩和策の一意識別子（例: "mit_001"）
    mitigation_id: str
    # 緩和技術の名称（例: "JWT 署名検証", "RLS ポリシー適用"）
    technique: str
    # 実装を担当する軸の識別子（例: "security", "data"）
    implementation_axis: str
    # 有効性スコア（0.0〜1.0）
    effectiveness: float
    # 実装済みかどうかのフラグ
    is_implemented: bool = False
    # 実装の証跡（コードパスや CI ジョブ名など）
    evidence: str = ""

    def validate(self) -> bool:
        """緩和策データが有効かどうかを検証する。"""
        # mitigation_id が空でないことを確認する
        if not self.mitigation_id:
            # mitigation_id が空の場合は無効とする
            return False
        # effectiveness が 0.0〜1.0 の範囲内であることを確認する
        if not (0.0 <= self.effectiveness <= 1.0):
            # 範囲外の場合は無効とする
            return False
        # 全検証が通過した場合は有効とする
        return True


@dataclass
class ValidationResult:
    """緩和策検証の結果を表すデータクラス。"""

    # 検証が成功したかどうかのフラグ
    is_valid: bool
    # 実装済みの緩和策数
    implemented_count: int
    # 未実装の緩和策数
    missing_count: int
    # 高リスクかつ未緩和の脅威リスト
    high_risk_open_threats: List[ThreatEntry] = field(default_factory=list)
    # 検証エラーメッセージのリスト
    errors: List[str] = field(default_factory=list)
    # 検証警告メッセージのリスト
    warnings: List[str] = field(default_factory=list)

    def summary(self) -> str:
        """検証結果のサマリー文字列を返す。"""
        # 有効・無効の状態文字列を決定する
        status = "VALID" if self.is_valid else "INVALID"
        # サマリー文字列をフォーマットして返す
        return (
            f"ValidationResult[{status}] "
            f"implemented={self.implemented_count} "
            f"missing={self.missing_count} "
            f"high_risk_open={len(self.high_risk_open_threats)} "
            f"errors={len(self.errors)}"
        )


# ===========================================================================
# CVSS 計算ロジック
# ===========================================================================


class CvssCalculator:
    """CVSS v3.1 Base Score 計算クラス。製造業の資産に対して適用する。"""

    # 攻撃ベクター係数の定義（CVSS v3.1 仕様準拠）
    _ATTACK_VECTOR_WEIGHT: Dict[str, float] = {
        # ネットワーク経由の攻撃（最も高い係数）
        "network": 0.85,
        # 隣接ネットワーク経由の攻撃
        "adjacent": 0.62,
        # ローカルアクセスが必要な攻撃
        "local": 0.55,
        # 物理アクセスが必要な攻撃（最も低い係数）
        "physical": 0.20,
    }

    # 脅威カテゴリ別の影響度増幅係数を定義する
    _CATEGORY_MULTIPLIER: Dict[str, float] = {
        # なりすましは機密性に中程度の影響
        "Spoofing": 1.0,
        # 改ざんは完全性に高い影響
        "Tampering": 1.1,
        # 否認は追跡可能性に中程度の影響
        "Repudiation": 0.9,
        # 情報漏洩は機密性に高い影響
        "InformationDisclosure": 1.15,
        # サービス拒否は可用性に高い影響
        "DenialOfService": 1.05,
        # 特権昇格は全面的に最高の影響
        "ElevationOfPrivilege": 1.2,
    }

    @classmethod
    def calculate_base_score(
        cls,
        attack_vector: str,
        likelihood: float,
        impact: float,
        category: ThreatCategory,
    ) -> float:
        """CVSS ベーススコア（0.0〜10.0）を計算して返す。"""
        # 攻撃ベクター係数を取得する（未知の場合はデフォルト 0.5 を使用）
        av_weight = cls._ATTACK_VECTOR_WEIGHT.get(attack_vector.lower(), 0.5)
        # カテゴリ乗数を取得する（未知の場合はデフォルト 1.0 を使用）
        cat_mult = cls._CATEGORY_MULTIPLIER.get(category.value, 1.0)
        # 悪用可能性スコアを計算する（攻撃ベクター × 発生可能性）
        exploitability = av_weight * likelihood * 8.22
        # 影響度スコアを計算する（影響度 × カテゴリ乗数 × 6.42）
        impact_score = impact * cat_mult * 6.42
        # 影響度が 0 の場合はスコア 0 を返す
        if impact_score <= 0:
            # 影響がない脅威にはスコア 0 を割り当てる
            return 0.0
        # CVSS v3.1 の ISCBase 計算式を適用する
        isc_base = 1.0 - (1.0 - impact) * (1.0 - impact * 0.5) * (1.0 - impact * 0.7)
        # 影響度サブスコアを計算する
        iss = 7.52 * isc_base - 3.25 * (isc_base - 0.02) ** 15
        # 総合スコアを計算する（Scope Unchanged の式）
        raw_score = min(exploitability + iss, 10.0)
        # スコアが負の場合は 0 に補正する
        raw_score = max(raw_score, 0.0)
        # 小数点第 1 位に丸めて返す
        return round(raw_score, 1)

    @classmethod
    def classify_severity(cls, cvss_score: float) -> str:
        """CVSS スコアを重大度ラベルに変換して返す。"""
        # スコアが 9.0 以上は Critical
        if cvss_score >= 9.0:
            # 最高重大度ラベルを返す
            return "Critical"
        # スコアが 7.0 以上は High
        if cvss_score >= 7.0:
            # 高重大度ラベルを返す
            return "High"
        # スコアが 4.0 以上は Medium
        if cvss_score >= 4.0:
            # 中重大度ラベルを返す
            return "Medium"
        # スコアが 0.1 以上は Low
        if cvss_score >= 0.1:
            # 低重大度ラベルを返す
            return "Low"
        # スコアが 0 に近い場合は None
        return "None"


# ===========================================================================
# STRIDECatalog クラス
# ===========================================================================


class STRIDECatalog:
    """STRIDE 脅威カタログを管理するクラス。

    YAML ファイルから脅威エントリ・資産・緩和策を読み込み、
    リスク評価・検証機能を提供する。
    """

    def __init__(self) -> None:
        """カタログを空の状態で初期化する。"""
        # 資産の辞書（asset_id → Asset）を初期化する
        self._assets: Dict[str, Asset] = {}
        # 脅威エントリのリストを初期化する
        self._threats: List[ThreatEntry] = []
        # 緩和策の辞書（mitigation_id → MitigationEntry）を初期化する
        self._mitigations: Dict[str, MitigationEntry] = {}
        # カタログのメタデータを初期化する
        self._metadata: Dict[str, Any] = {}

    @classmethod
    def load_from_yaml(cls, path: Path) -> "STRIDECatalog":
        """YAML ファイルから STRIDE カタログを読み込んで返す。

        Args:
            path: カタログ YAML ファイルのパス

        Returns:
            読み込んだ STRIDECatalog インスタンス

        Raises:
            FileNotFoundError: 指定したパスにファイルが存在しない場合
            ValueError: YAML の構造が不正な場合
        """
        # パスが存在するかどうかを確認する
        if not path.exists():
            # ファイルが存在しない場合は FileNotFoundError を送出する
            raise FileNotFoundError(f"カタログファイルが見つかりません: {path}")
        # YAML ファイルを UTF-8 で読み込む
        with open(path, encoding="utf-8") as fh:
            # YAML を安全にパースして辞書に変換する
            raw_data = yaml.safe_load(fh)
        # raw_data が辞書でない場合はエラーを送出する
        if not isinstance(raw_data, dict):
            # 不正な構造の場合は ValueError を送出する
            raise ValueError(f"カタログ YAML のルートは辞書でなければなりません: {path}")
        # 新しいカタログインスタンスを作成する
        catalog = cls()
        # メタデータを取得してカタログに設定する
        catalog._metadata = raw_data.get("metadata", {})
        # 資産セクションを読み込む
        for asset_raw in raw_data.get("assets", []):
            # 資産エントリをパースしてカタログに追加する
            asset = cls._parse_asset(asset_raw)
            # 資産を辞書に格納する
            catalog._assets[asset.asset_id] = asset
        # 緩和策セクションを読み込む
        for mit_raw in raw_data.get("mitigations", []):
            # 緩和策エントリをパースしてカタログに追加する
            mit = cls._parse_mitigation(mit_raw)
            # 緩和策を辞書に格納する
            catalog._mitigations[mit.mitigation_id] = mit
        # 脅威エントリセクションを読み込む
        for threat_raw in raw_data.get("threats", []):
            # 脅威エントリをパースしてカタログに追加する
            threat = cls._parse_threat(threat_raw)
            # 脅威をリストに追加する
            catalog._threats.append(threat)
        # 脅威の CVSS スコアを計算して設定する
        catalog._enrich_cvss_scores()
        # 読み込んだカタログをログに記録する
        logger.info(
            "STRIDEカタログを読み込みました: 資産=%d 脅威=%d 緩和策=%d",
            len(catalog._assets),
            len(catalog._threats),
            len(catalog._mitigations),
        )
        # 初期化済みのカタログを返す
        return catalog

    @staticmethod
    def _parse_asset(raw: Dict[str, Any]) -> Asset:
        """辞書から Asset データクラスを生成して返す。"""
        # asset_id フィールドを取得する（必須）
        asset_id = raw.get("asset_id", "")
        # asset_name フィールドを取得する
        asset_name = raw.get("asset_name", "")
        # asset_type フィールドを取得する
        asset_type = raw.get("asset_type", "unknown")
        # sensitivity_level フィールドを整数で取得する
        sensitivity_level = int(raw.get("sensitivity_level", 1))
        # owner_axis フィールドを取得する
        owner_axis = raw.get("owner_axis", "unknown")
        # Asset データクラスを生成して返す
        return Asset(
            asset_id=asset_id,
            asset_name=asset_name,
            asset_type=asset_type,
            sensitivity_level=sensitivity_level,
            owner_axis=owner_axis,
        )

    @staticmethod
    def _parse_mitigation(raw: Dict[str, Any]) -> MitigationEntry:
        """辞書から MitigationEntry データクラスを生成して返す。"""
        # mitigation_id フィールドを取得する（必須）
        mitigation_id = raw.get("mitigation_id", "")
        # technique フィールドを取得する
        technique = raw.get("technique", "")
        # implementation_axis フィールドを取得する
        implementation_axis = raw.get("implementation_axis", "unknown")
        # effectiveness フィールドを浮動小数点で取得する
        effectiveness = float(raw.get("effectiveness", 0.5))
        # is_implemented フィールドを取得する
        is_implemented = bool(raw.get("is_implemented", False))
        # evidence フィールドを取得する
        evidence = raw.get("evidence", "")
        # MitigationEntry データクラスを生成して返す
        return MitigationEntry(
            mitigation_id=mitigation_id,
            technique=technique,
            implementation_axis=implementation_axis,
            effectiveness=effectiveness,
            is_implemented=is_implemented,
            evidence=evidence,
        )

    @staticmethod
    def _parse_threat(raw: Dict[str, Any]) -> ThreatEntry:
        """辞書から ThreatEntry データクラスを生成して返す。"""
        # threat_id フィールドを取得する（必須）
        threat_id = raw.get("threat_id", "")
        # category 文字列を ThreatCategory 列挙型に変換する
        category_str = raw.get("category", "Spoofing")
        # ThreatCategory の値と一致するメンバーを探す
        category = ThreatCategory(category_str)
        # asset_id フィールドを取得する
        asset_id = raw.get("asset_id", "")
        # attack_vector フィールドを取得する
        attack_vector = raw.get("attack_vector", "network")
        # likelihood フィールドを浮動小数点で取得する
        likelihood = float(raw.get("likelihood", 0.5))
        # impact フィールドを浮動小数点で取得する
        impact = float(raw.get("impact", 0.5))
        # cvss_score フィールドを取得する（-1.0 は未計算を示す）
        cvss_score = float(raw.get("cvss_score", -1.0))
        # mitigation_ids フィールドをリストで取得する
        mitigation_ids = list(raw.get("mitigation_ids", []))
        # description フィールドを取得する
        description = raw.get("description", "")
        # mitigation_status フィールドを取得する
        mitigation_status = raw.get("mitigation_status", "open")
        # ThreatEntry データクラスを生成して返す
        return ThreatEntry(
            threat_id=threat_id,
            category=category,
            asset_id=asset_id,
            attack_vector=attack_vector,
            likelihood=likelihood,
            impact=impact,
            cvss_score=cvss_score,
            mitigation_ids=mitigation_ids,
            description=description,
            mitigation_status=mitigation_status,
        )

    def _enrich_cvss_scores(self) -> None:
        """CVSS スコアが未計算の脅威エントリにスコアを計算して設定する。"""
        # 全脅威エントリをイテレートする
        for threat in self._threats:
            # CVSS スコアが未計算（-1.0）の場合のみ計算する
            if threat.cvss_score < 0:
                # CVSS 計算クラスを使ってスコアを計算する
                threat.cvss_score = CvssCalculator.calculate_base_score(
                    attack_vector=threat.attack_vector,
                    likelihood=threat.likelihood,
                    impact=threat.impact,
                    category=threat.category,
                )

    def get_threats_by_category(self, cat: ThreatCategory) -> List[ThreatEntry]:
        """指定したカテゴリに属する脅威エントリのリストを返す。

        Args:
            cat: 絞り込む STRIDE カテゴリ

        Returns:
            指定カテゴリに属する ThreatEntry のリスト
        """
        # カテゴリが一致する脅威エントリをフィルタリングして返す
        return [t for t in self._threats if t.category == cat]

    def get_open_threats(self) -> List[ThreatEntry]:
        """未対処（open）状態の脅威エントリのリストを返す。"""
        # mitigation_status が "open" の脅威エントリをフィルタリングする
        return [t for t in self._threats if t.is_open()]

    def get_high_risk_threats(self, cvss_threshold: float = 7.0) -> List[ThreatEntry]:
        """CVSS スコアが閾値以上の高リスク脅威エントリを返す。

        Args:
            cvss_threshold: リスク判定の閾値（デフォルト 7.0）

        Returns:
            高リスクな ThreatEntry のリスト（CVSS 降順ソート）
        """
        # CVSS スコアが閾値以上の脅威をフィルタリングする
        high_risk = [t for t in self._threats if t.cvss_score >= cvss_threshold]
        # CVSS スコアの降順でソートして返す
        return sorted(high_risk, key=lambda t: t.cvss_score, reverse=True)

    def compute_risk_score(self) -> Dict[ThreatCategory, float]:
        """カテゴリ別の集計リスクスコアを計算して返す。

        各カテゴリの「固有リスク合計 × 未緩和率」を計算する。
        Returns:
            カテゴリ → 集計リスクスコアの辞書
        """
        # 結果辞書を初期化する（全カテゴリを 0.0 で初期化）
        result: Dict[ThreatCategory, float] = {cat: 0.0 for cat in ThreatCategory}
        # カテゴリごとに集計する
        for cat in ThreatCategory:
            # 当該カテゴリの全脅威を取得する
            threats_in_cat = self.get_threats_by_category(cat)
            # 脅威が存在しない場合はスキップする
            if not threats_in_cat:
                # スコアは 0.0 のままにする
                continue
            # 全脅威の固有リスク合計を計算する
            total_inherent = sum(t.compute_inherent_risk() for t in threats_in_cat)
            # 未対処脅威の固有リスク合計を計算する
            open_inherent = sum(
                t.compute_inherent_risk() for t in threats_in_cat if t.is_open()
            )
            # 未緩和率を計算する（全リスクに対する未対処リスクの割合）
            unmitigated_ratio = open_inherent / total_inherent if total_inherent > 0 else 0.0
            # カテゴリのリスクスコア = 固有リスク合計 × 未緩和率
            result[cat] = round(total_inherent * unmitigated_ratio, 3)
        # カテゴリ別リスクスコアの辞書を返す
        return result

    def validate_mitigations(self) -> ValidationResult:
        """全緩和策の実装状態を検証して ValidationResult を返す。"""
        # 実装済み緩和策数をカウントする
        implemented_count = sum(1 for m in self._mitigations.values() if m.is_implemented)
        # 未実装緩和策数をカウントする
        missing_count = sum(1 for m in self._mitigations.values() if not m.is_implemented)
        # 高リスクかつ未対処の脅威リストを取得する
        high_risk_open = [
            t for t in self._threats if t.is_open() and t.is_high_risk()
        ]
        # エラーリストを初期化する
        errors: List[str] = []
        # 警告リストを初期化する
        warnings: List[str] = []
        # 高リスクかつ未対処の脅威が存在する場合はエラーとして記録する
        for threat in high_risk_open:
            # 高リスク未対処脅威をエラーリストに追加する
            errors.append(
                f"高リスク未対処脅威: {threat.threat_id} "
                f"(CVSS={threat.cvss_score}, category={threat.category.value})"
            )
        # 脅威が参照する緩和策が存在するかどうかを確認する
        for threat in self._threats:
            # 脅威に関連する緩和策 ID をイテレートする
            for mit_id in threat.mitigation_ids:
                # 緩和策が辞書に存在しない場合は警告を記録する
                if mit_id not in self._mitigations:
                    # 未定義の緩和策への参照を警告リストに追加する
                    warnings.append(
                        f"脅威 {threat.threat_id} が未定義の緩和策 {mit_id} を参照しています"
                    )
        # エラーが 0 件であれば検証成功とする
        is_valid = len(errors) == 0
        # ValidationResult を生成して返す
        return ValidationResult(
            is_valid=is_valid,
            implemented_count=implemented_count,
            missing_count=missing_count,
            high_risk_open_threats=high_risk_open,
            errors=errors,
            warnings=warnings,
        )

    def get_asset(self, asset_id: str) -> Optional[Asset]:
        """資産 ID から Asset データクラスを返す。存在しない場合は None を返す。"""
        # 資産辞書から指定した asset_id を検索して返す
        return self._assets.get(asset_id)

    def get_mitigation(self, mitigation_id: str) -> Optional[MitigationEntry]:
        """緩和策 ID から MitigationEntry を返す。存在しない場合は None を返す。"""
        # 緩和策辞書から指定した mitigation_id を検索して返す
        return self._mitigations.get(mitigation_id)

    def all_assets(self) -> List[Asset]:
        """全資産エントリのリストを返す。"""
        # 資産辞書の値リストを返す
        return list(self._assets.values())

    def all_threats(self) -> List[ThreatEntry]:
        """全脅威エントリのリストを返す。"""
        # 脅威リストのコピーを返す
        return list(self._threats)

    def all_mitigations(self) -> List[MitigationEntry]:
        """全緩和策エントリのリストを返す。"""
        # 緩和策辞書の値リストを返す
        return list(self._mitigations.values())

    def summary_dict(self) -> Dict[str, Any]:
        """カタログのサマリー情報を辞書で返す。"""
        # 未対処脅威の件数を集計する
        open_count = len(self.get_open_threats())
        # 高リスク脅威の件数を集計する
        high_risk_count = len(self.get_high_risk_threats())
        # カテゴリ別リスクスコアを計算する
        risk_scores = {cat.value: score for cat, score in self.compute_risk_score().items()}
        # サマリー辞書を構築して返す
        return {
            "total_assets": len(self._assets),
            "total_threats": len(self._threats),
            "total_mitigations": len(self._mitigations),
            "open_threat_count": open_count,
            "high_risk_threat_count": high_risk_count,
            "risk_scores_by_category": risk_scores,
        }

    def export_to_dict(self) -> Dict[str, Any]:
        """カタログ全体を辞書形式にシリアライズして返す。"""
        # 資産リストをシリアライズする
        assets_list = [
            {
                "asset_id": a.asset_id,
                "asset_name": a.asset_name,
                "asset_type": a.asset_type,
                "sensitivity_level": a.sensitivity_level,
                "owner_axis": a.owner_axis,
            }
            for a in self._assets.values()
        ]
        # 脅威エントリリストをシリアライズする
        threats_list = [
            {
                "threat_id": t.threat_id,
                "category": t.category.value,
                "asset_id": t.asset_id,
                "attack_vector": t.attack_vector,
                "likelihood": t.likelihood,
                "impact": t.impact,
                "cvss_score": t.cvss_score,
                "mitigation_ids": t.mitigation_ids,
                "description": t.description,
                "mitigation_status": t.mitigation_status,
            }
            for t in self._threats
        ]
        # 緩和策リストをシリアライズする
        mitigations_list = [
            {
                "mitigation_id": m.mitigation_id,
                "technique": m.technique,
                "implementation_axis": m.implementation_axis,
                "effectiveness": m.effectiveness,
                "is_implemented": m.is_implemented,
                "evidence": m.evidence,
            }
            for m in self._mitigations.values()
        ]
        # 全データをまとめた辞書を返す
        return {
            "metadata": self._metadata,
            "assets": assets_list,
            "threats": threats_list,
            "mitigations": mitigations_list,
        }


# ===========================================================================
# デフォルトカタログのファクトリ関数
# ===========================================================================


def build_default_manufacturing_catalog() -> STRIDECatalog:
    """製造業プラットフォーム向けのデフォルト STRIDE カタログを生成して返す。

    YAML ファイルが存在しない環境での開発・テスト用に使用する。
    """
    # 空のカタログインスタンスを生成する
    catalog = STRIDECatalog()
    # 認証サービス資産を定義する
    auth_asset = Asset(
        asset_id="asset_auth_service",
        asset_name="認証サービス",
        asset_type="service",
        sensitivity_level=4,
        owner_axis="tier1",
    )
    # 生産データベース資産を定義する
    prod_db_asset = Asset(
        asset_id="asset_prod_database",
        asset_name="生産管理データベース",
        asset_type="database",
        sensitivity_level=4,
        owner_axis="data",
    )
    # PII クラスタ資産を定義する
    pii_asset = Asset(
        asset_id="asset_pii_cluster",
        asset_name="PII 専用クラスタ",
        asset_type="database",
        sensitivity_level=4,
        owner_axis="data",
    )
    # API ゲートウェイ資産を定義する
    api_gw_asset = Asset(
        asset_id="asset_api_gateway",
        asset_name="API ゲートウェイ",
        asset_type="api_endpoint",
        sensitivity_level=3,
        owner_axis="tier1",
    )
    # コンテナレジストリ資産を定義する
    registry_asset = Asset(
        asset_id="asset_container_registry",
        asset_name="コンテナレジストリ",
        asset_type="registry",
        sensitivity_level=3,
        owner_axis="security",
    )
    # 定義した資産をカタログに追加する
    for asset in [auth_asset, prod_db_asset, pii_asset, api_gw_asset, registry_asset]:
        # 資産を資産辞書に格納する
        catalog._assets[asset.asset_id] = asset
    # JWT 署名検証緩和策を定義する
    mit_jwt = MitigationEntry(
        mitigation_id="mit_jwt_verification",
        technique="JWT RS256 署名検証 + 有効期限チェック",
        implementation_axis="tier1",
        effectiveness=0.9,
        is_implemented=True,
        evidence="src/tier1/transport/auth_middleware.rs",
    )
    # RLS 適用緩和策を定義する
    mit_rls = MitigationEntry(
        mitigation_id="mit_rls_enforcement",
        technique="PostgreSQL RLS FORCE ポリシー",
        implementation_axis="data",
        effectiveness=0.95,
        is_implemented=True,
        evidence="src/data/pii_cluster/rls_manager.py",
    )
    # Kyverno ポリシー緩和策を定義する
    mit_kyverno = MitigationEntry(
        mitigation_id="mit_kyverno_policy",
        technique="Kyverno Admission Controller ポリシー",
        implementation_axis="security",
        effectiveness=0.88,
        is_implemented=True,
        evidence="src/security/kyverno/policies/",
    )
    # cosign 署名検証緩和策を定義する
    mit_cosign = MitigationEntry(
        mitigation_id="mit_cosign_signing",
        technique="cosign/sigstore によるサプライチェーン署名",
        implementation_axis="security",
        effectiveness=0.92,
        is_implemented=True,
        evidence="src/security/cosign/policy_validator.py",
    )
    # 監査ログ記録緩和策を定義する
    mit_audit = MitigationEntry(
        mitigation_id="mit_audit_logging",
        technique="pgaudit + append-only audit_event テーブル",
        implementation_axis="data",
        effectiveness=0.85,
        is_implemented=True,
        evidence="src/data/audit_chain/verify.py",
    )
    # 定義した緩和策をカタログに追加する
    for mit in [mit_jwt, mit_rls, mit_kyverno, mit_cosign, mit_audit]:
        # 緩和策を緩和策辞書に格納する
        catalog._mitigations[mit.mitigation_id] = mit
    # 認証なりすまし脅威エントリを定義する
    threat_spoof_auth = ThreatEntry(
        threat_id="threat_spoof_001",
        category=ThreatCategory.SPOOFING,
        asset_id="asset_auth_service",
        attack_vector="network",
        likelihood=0.6,
        impact=0.9,
        mitigation_ids=["mit_jwt_verification"],
        description="攻撃者が有効なトークンなしに認証サービスになりすます",
        mitigation_status="mitigated",
    )
    # DB 改ざん脅威エントリを定義する
    threat_tamper_db = ThreatEntry(
        threat_id="threat_tamper_001",
        category=ThreatCategory.TAMPERING,
        asset_id="asset_prod_database",
        attack_vector="local",
        likelihood=0.3,
        impact=0.95,
        mitigation_ids=["mit_rls_enforcement", "mit_audit_logging"],
        description="権限昇格後に生産データを不正に改ざんする",
        mitigation_status="mitigated",
    )
    # PII 情報漏洩脅威エントリを定義する
    threat_info_pii = ThreatEntry(
        threat_id="threat_info_001",
        category=ThreatCategory.INFO_DISCLOSURE,
        asset_id="asset_pii_cluster",
        attack_vector="network",
        likelihood=0.4,
        impact=0.98,
        mitigation_ids=["mit_rls_enforcement", "mit_audit_logging"],
        description="テナント分離の欠陥により他テナントの PII データが閲覧可能になる",
        mitigation_status="mitigated",
    )
    # コンテナイメージ改ざん脅威エントリを定義する
    threat_tamper_image = ThreatEntry(
        threat_id="threat_tamper_002",
        category=ThreatCategory.TAMPERING,
        asset_id="asset_container_registry",
        attack_vector="network",
        likelihood=0.35,
        impact=0.85,
        mitigation_ids=["mit_cosign_signing", "mit_kyverno_policy"],
        description="コンテナレジストリへの不正アクセスによりイメージが改ざんされる",
        mitigation_status="mitigated",
    )
    # API ゲートウェイ DoS 脅威エントリを定義する
    threat_dos_api = ThreatEntry(
        threat_id="threat_dos_001",
        category=ThreatCategory.DENIAL_OF_SERVICE,
        asset_id="asset_api_gateway",
        attack_vector="network",
        likelihood=0.5,
        impact=0.7,
        mitigation_ids=[],
        description="API ゲートウェイへの過負荷攻撃によりサービス停止が発生する",
        mitigation_status="open",
    )
    # 定義した脅威エントリをカタログに追加する
    for threat in [
        threat_spoof_auth,
        threat_tamper_db,
        threat_info_pii,
        threat_tamper_image,
        threat_dos_api,
    ]:
        # 脅威エントリをリストに追加する
        catalog._threats.append(threat)
    # CVSS スコアを計算して全脅威エントリに設定する
    catalog._enrich_cvss_scores()
    # 構築したカタログを返す
    return catalog
