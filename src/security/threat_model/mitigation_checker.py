"""src/security/threat_model/mitigation_checker.py

緩和策の実装状態を検証するモジュール。
コードベース内のパターン検索・Kyverno ポリシー確認・RLS 検証を通じて
STRIDE カタログの緩和策が物理的に実装されているかどうかを判定する。
仕様: docs/04_詳細設計/01_適合仕様/15_脅威モデル適合仕様.md
"""

# 将来の注釈構文互換のため annotations をインポートする
from __future__ import annotations

# 列挙型定義に使用する enum をインポートする
import enum
# ハッシュ計算に使用する hashlib をインポートする
import hashlib
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
from typing import Any, Dict, List, Optional, Tuple

# stride_catalog モジュールから必要な型をインポートする
from .stride_catalog import MitigationEntry, STRIDECatalog, ThreatCategory

# モジュール専用ロガーを取得する
logger = logging.getLogger(__name__)


# ===========================================================================
# 列挙型定義
# ===========================================================================


class MitigationStatus(enum.Enum):
    """緩和策の実装状態を表す列挙型。"""

    # 完全に実装済みで稼働中の状態
    IMPLEMENTED = "implemented"
    # 部分的に実装されているが未完全な状態
    PARTIAL = "partial"
    # 未実装の状態（対処が必要）
    MISSING = "missing"
    # より新しい緩和策に置き換えられて廃止された状態
    SUPERSEDED = "superseded"


# ===========================================================================
# データクラス定義
# ===========================================================================


@dataclass
class EffectivenessScore:
    """緩和策の有効性評価スコアを表すデータクラス。"""

    # 有効性スコア（0.0〜1.0）
    score_0_to_1: float
    # 評価の信頼度（0.0〜1.0）
    confidence: float
    # 評価に使用した証拠の件数
    evidence_count: int
    # 評価に使用した証拠の詳細リスト
    evidence_details: List[str] = field(default_factory=list)

    def is_sufficient(self, threshold: float = 0.7) -> bool:
        """スコアが指定した閾値以上かどうかを返す。"""
        # スコアが閾値以上であれば十分とみなす
        return self.score_0_to_1 >= threshold


@dataclass
class CheckResult:
    """個別の緩和策チェック結果を表すデータクラス。"""

    # チェック対象の緩和策 ID
    mitigation_id: str
    # 検出された実装状態
    status: MitigationStatus
    # チェックで発見した証拠のリスト
    evidence_found: List[str] = field(default_factory=list)
    # チェック実行時刻（ISO 8601 形式）
    checked_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    # チェック失敗時のエラーメッセージ
    error_message: str = ""

    def is_ok(self) -> bool:
        """実装済みまたは部分実装の状態かどうかを返す。"""
        # IMPLEMENTED または PARTIAL であれば OK とみなす
        return self.status in (MitigationStatus.IMPLEMENTED, MitigationStatus.PARTIAL)


@dataclass
class OverallEffectivenessReport:
    """全緩和策の有効性評価レポートを表すデータクラス。"""

    # 緩和策 ID → EffectivenessScore の辞書
    scores: Dict[str, EffectivenessScore] = field(default_factory=dict)
    # 評価実行時刻
    evaluated_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    # 全体の有効性スコア（全緩和策の加重平均）
    overall_score: float = 0.0
    # 不十分な緩和策 ID のリスト
    insufficient_mitigations: List[str] = field(default_factory=list)

    def summary(self) -> str:
        """レポートのサマリー文字列を返す。"""
        # 緩和策の総数を取得する
        total = len(self.scores)
        # 不十分な緩和策の数を取得する
        insufficient = len(self.insufficient_mitigations)
        # サマリー文字列をフォーマットして返す
        return (
            f"OverallEffectivenessReport: "
            f"total={total} insufficient={insufficient} "
            f"overall_score={self.overall_score:.3f}"
        )


# ===========================================================================
# パターン定義
# ===========================================================================


# 実装パターンの辞書: 緩和技術名 → 検索パターンのリスト
_IMPLEMENTATION_PATTERNS: Dict[str, List[str]] = {
    # JWT 署名検証の実装パターン
    "JWT RS256 署名検証 + 有効期限チェック": [
        r"jsonwebtoken",
        r"verify_jwt",
        r"decode_token",
        r"RS256",
        r"jwt\.decode",
    ],
    # RLS ポリシーの実装パターン
    "PostgreSQL RLS FORCE ポリシー": [
        r"ROW SECURITY",
        r"ENABLE ROW LEVEL SECURITY",
        r"CREATE POLICY",
        r"app\.current_tenant",
        r"current_setting",
    ],
    # Kyverno ポリシーの実装パターン
    "Kyverno Admission Controller ポリシー": [
        r"ClusterPolicy",
        r"kyverno\.io",
        r"admission\.k8s\.io",
        r"validate\.cel",
    ],
    # cosign 署名の実装パターン
    "cosign/sigstore によるサプライチェーン署名": [
        r"cosign",
        r"sigstore",
        r"rekor",
        r"fulcio",
        r"bundle\.json",
    ],
    # pgaudit の実装パターン
    "pgaudit + append-only audit_event テーブル": [
        r"pgaudit",
        r"audit_event",
        r"pg_audit",
        r"audit_trail",
    ],
    # OpenBao Transit 暗号化の実装パターン
    "OpenBao Transit 暗号化": [
        r"transit/encrypt",
        r"transit/decrypt",
        r"openbao",
        r"vault/v1/transit",
    ],
    # mTLS 相互認証の実装パターン
    "mTLS 相互認証": [
        r"client_cert",
        r"mutual_tls",
        r"mtls",
        r"peer_certificate",
        r"TLSConfig",
    ],
    # RBAC ポリシーの実装パターン
    "RBAC ポリシー": [
        r"ClusterRole",
        r"RoleBinding",
        r"rbac\.authorization\.k8s\.io",
        r"rules:",
    ],
}


# ===========================================================================
# MitigationChecker クラス
# ===========================================================================


class MitigationChecker:
    """緩和策の実装状態を検証するクラス。

    コードベース検索・Kyverno API・データベース接続を通じて
    各緩和策が物理的に実装されているかどうかを判定する。
    """

    def __init__(self, codebase_root: Optional[Path] = None) -> None:
        """MitigationChecker を初期化する。

        Args:
            codebase_root: コードベースのルートディレクトリ（省略時は cwd）
        """
        # コードベースのルートを設定する（省略時はカレントディレクトリ）
        self._codebase_root = codebase_root or Path.cwd()
        # チェック結果のキャッシュを初期化する
        self._cache: Dict[str, CheckResult] = {}

    def check_implementation(
        self,
        mitigation: MitigationEntry,
        codebase_path: Optional[Path] = None,
    ) -> MitigationStatus:
        """緩和策がコードベースに実装されているかどうかを検証して返す。

        Args:
            mitigation: 検証対象の緩和策エントリ
            codebase_path: 検索対象のコードベースパス（省略時は初期化時のルート）

        Returns:
            検出された MitigationStatus 値
        """
        # 検索対象パスを決定する
        search_path = codebase_path or self._codebase_root
        # キャッシュに検索結果がある場合はキャッシュを返す
        cache_key = f"{mitigation.mitigation_id}:{search_path}"
        # キャッシュキーが存在する場合はキャッシュの結果を返す
        if cache_key in self._cache:
            # キャッシュヒットをログに記録する
            logger.debug("キャッシュヒット: %s", cache_key)
            # キャッシュから結果ステータスを返す
            return self._cache[cache_key].status
        # コードベース内でパターンを検索する
        evidence_found = self._search_patterns_in_codebase(
            technique=mitigation.technique,
            search_path=search_path,
        )
        # 証拠が十分見つかった場合は IMPLEMENTED を返す
        if len(evidence_found) >= 3:
            # 十分な証拠が見つかったため IMPLEMENTED とする
            status = MitigationStatus.IMPLEMENTED
        # 証拠が 1 件以上見つかった場合は PARTIAL を返す
        elif len(evidence_found) >= 1:
            # 証拠が不十分なため PARTIAL とする
            status = MitigationStatus.PARTIAL
        else:
            # 証拠が見つからなかったため MISSING とする
            status = MitigationStatus.MISSING
        # チェック結果をキャッシュに保存する
        self._cache[cache_key] = CheckResult(
            mitigation_id=mitigation.mitigation_id,
            status=status,
            evidence_found=evidence_found,
        )
        # 検証結果をログに記録する
        logger.debug(
            "緩和策チェック完了: %s → %s (証拠=%d件)",
            mitigation.mitigation_id,
            status.value,
            len(evidence_found),
        )
        # 検出した実装状態を返す
        return status

    def _search_patterns_in_codebase(
        self,
        technique: str,
        search_path: Path,
    ) -> List[str]:
        """コードベース内で緩和技術に対応するパターンを検索して証拠リストを返す。"""
        # 緩和技術に対応する検索パターンリストを取得する
        patterns = _IMPLEMENTATION_PATTERNS.get(technique, [])
        # パターンが定義されていない場合はファイル存在確認にフォールバックする
        if not patterns:
            # パターンが未定義の場合はファイル名ベースの検索を試みる
            return self._fallback_file_search(technique, search_path)
        # 発見した証拠ファイルのリストを初期化する
        evidence: List[str] = []
        # 各パターンに対してコードベースを検索する
        for pattern in patterns:
            # コードベース内でパターンを再帰的に検索する
            found_files = self._grep_codebase(pattern, search_path)
            # 見つかったファイルを証拠リストに追加する（重複を除去）
            for found in found_files:
                # まだ証拠リストにない場合のみ追加する
                if found not in evidence:
                    # 証拠リストにファイルパスを追加する
                    evidence.append(found)
        # 発見した証拠リストを返す
        return evidence

    def _grep_codebase(self, pattern: str, search_path: Path) -> List[str]:
        """指定したパターンでコードベースを grep して一致ファイルのリストを返す。"""
        # 検索対象パスが存在しない場合は空リストを返す
        if not search_path.exists():
            # パスが存在しない場合はログに警告を出力する
            logger.warning("検索対象パスが存在しません: %s", search_path)
            # 空リストを返す
            return []
        # 一致したファイルパスのリストを初期化する
        matched: List[str] = []
        # 検索対象の拡張子リストを定義する
        target_extensions = {".py", ".rs", ".go", ".ts", ".yaml", ".yml", ".sql", ".hcl"}
        # コードベース内の全ファイルを再帰的に探索する
        for file_path in search_path.rglob("*"):
            # ファイルでない場合はスキップする
            if not file_path.is_file():
                # ディレクトリなのでスキップする
                continue
            # 対象外の拡張子はスキップする
            if file_path.suffix not in target_extensions:
                # 対象外拡張子なのでスキップする
                continue
            # node_modules はスキップする
            if "node_modules" in file_path.parts:
                # node_modules 内のファイルはスキップする
                continue
            # ファイルの内容を読み込んでパターンを検索する
            try:
                # ファイルを UTF-8 で読み込む
                content = file_path.read_text(encoding="utf-8", errors="ignore")
                # パターンが見つかった場合は証拠リストに追加する
                if re.search(pattern, content):
                    # 相対パスで証拠リストに追加する
                    matched.append(str(file_path.relative_to(search_path)))
            except (PermissionError, OSError) as exc:
                # ファイル読み込みエラーをログに記録する
                logger.debug("ファイル読み込みエラー: %s (%s)", file_path, exc)
        # 一致したファイルパスのリストを返す
        return matched

    def _fallback_file_search(self, technique: str, search_path: Path) -> List[str]:
        """パターンが未定義の場合に技術名のキーワードでファイルを検索して返す。"""
        # 技術名を小文字に変換してキーワードを抽出する
        keywords = technique.lower().replace(" ", "_").split("_")
        # 発見したファイルリストを初期化する
        found: List[str] = []
        # コードベース内のファイルをキーワードで検索する
        for file_path in search_path.rglob("*"):
            # ファイルでない場合はスキップする
            if not file_path.is_file():
                # ディレクトリはスキップする
                continue
            # ファイル名にキーワードが含まれているか確認する
            file_name_lower = file_path.stem.lower()
            # いずれかのキーワードがファイル名に含まれている場合は証拠として追加する
            if any(kw in file_name_lower for kw in keywords if len(kw) > 3):
                # 相対パスで証拠リストに追加する
                found.append(str(file_path.relative_to(search_path)))
        # 発見したファイルリストを返す
        return found

    def verify_kyverno_policy_active(self, policy_name: str) -> bool:
        """指定した Kyverno ポリシーがクラスタで有効かどうかを確認して返す。

        Args:
            policy_name: 確認する Kyverno ポリシー名

        Returns:
            ポリシーが有効であれば True、そうでなければ False
        """
        # kubectl コマンドで Kyverno ポリシーを取得する
        cmd = ["kubectl", "get", "clusterpolicy", policy_name, "-o", "json"]
        # サブプロセスを実行してポリシー情報を取得する
        try:
            # kubectl を実行してポリシー JSON を取得する
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=30,
            )
            # 終了コードが 0 でない場合はポリシーが存在しない
            if result.returncode != 0:
                # ポリシーが見つからない旨をログに記録する
                logger.warning("Kyverno ポリシーが見つかりません: %s", policy_name)
                # False を返す
                return False
            # JSON 出力を解析してポリシーの状態を確認する
            import json
            # JSON 文字列を辞書にパースする
            policy_data = json.loads(result.stdout)
            # ポリシーの status.ready フィールドを確認する
            status = policy_data.get("status", {})
            # ready フィールドが True であればポリシーは有効
            is_ready = status.get("ready", False)
            # ポリシーの有効状態をログに記録する
            logger.info("Kyverno ポリシー %s の状態: ready=%s", policy_name, is_ready)
            # ポリシーの有効状態を返す
            return bool(is_ready)
        except subprocess.TimeoutExpired:
            # タイムアウトが発生した場合はログに記録して False を返す
            logger.error("kubectl タイムアウト: ポリシー %s の取得", policy_name)
            # タイムアウトのため False を返す
            return False
        except FileNotFoundError:
            # kubectl が存在しない場合はログに記録して False を返す
            logger.warning("kubectl が見つかりません。コードベース検索にフォールバックします。")
            # フォールバックとしてコードベースでポリシー名を検索する
            found = self._grep_codebase(policy_name, self._codebase_root)
            # 1 件以上見つかれば有効とみなす
            return len(found) > 0

    def verify_cosign_signing_active(self) -> bool:
        """cosign 署名が CI パイプラインで有効かどうかを確認して返す。

        Returns:
            cosign 署名が有効であれば True、そうでなければ False
        """
        # CI ワークフローファイルのパスを定義する
        ci_paths = [
            self._codebase_root / ".github" / "workflows",
            self._codebase_root / ".gitlab-ci.yml",
        ]
        # cosign 関連のパターンを定義する
        cosign_patterns = ["cosign sign", "cosign attest", "sigstore", "COSIGN_"]
        # 各 CI パスを確認する
        for ci_path in ci_paths:
            # パスが存在しない場合はスキップする
            if not ci_path.exists():
                # パスが存在しないのでスキップする
                continue
            # CI パスが YAML ファイルの場合は直接読み込む
            if ci_path.is_file():
                # CI ファイルを読み込む
                try:
                    # ファイルを UTF-8 で読み込む
                    content = ci_path.read_text(encoding="utf-8")
                    # cosign パターンが存在するか確認する
                    if any(p in content for p in cosign_patterns):
                        # cosign が使用されていることをログに記録する
                        logger.info("cosign 署名が CI ファイルで確認されました: %s", ci_path)
                        # True を返す
                        return True
                except OSError as exc:
                    # 読み込みエラーをログに記録する
                    logger.debug("CI ファイル読み込みエラー: %s (%s)", ci_path, exc)
            # CI パスがディレクトリの場合は内部を検索する
            elif ci_path.is_dir():
                # ディレクトリ内の全 YAML ファイルを検索する
                for yml_file in ci_path.glob("*.yml"):
                    # YAML ファイルを読み込む
                    try:
                        # ファイルを UTF-8 で読み込む
                        content = yml_file.read_text(encoding="utf-8")
                        # cosign パターンが存在するか確認する
                        if any(p in content for p in cosign_patterns):
                            # cosign が使用されていることをログに記録する
                            logger.info(
                                "cosign 署名が CI ワークフローで確認されました: %s",
                                yml_file,
                            )
                            # True を返す
                            return True
                    except OSError as exc:
                        # 読み込みエラーをログに記録する
                        logger.debug("ワークフロー読み込みエラー: %s (%s)", yml_file, exc)
        # cosign の証拠が見つからなかったことをログに記録する
        logger.warning("cosign 署名の証拠が CI ファイルで見つかりませんでした")
        # False を返す
        return False

    def verify_rls_enforced(self, table: str, db_url: str) -> bool:
        """指定したテーブルに RLS が強制適用されているかどうかを確認して返す。

        Args:
            table: 確認するテーブル名
            db_url: PostgreSQL 接続 URL

        Returns:
            RLS が強制適用されていれば True、そうでなければ False
        """
        # psycopg2 のインポートを試みる
        try:
            # PostgreSQL 接続ドライバをインポートする
            import psycopg2
            # psycopg2 の型変換モジュールをインポートする
            import psycopg2.extras
        except ImportError:
            # psycopg2 が存在しない場合はコードベース検索にフォールバックする
            logger.warning("psycopg2 が見つかりません。コードベース検索にフォールバックします。")
            # コードベースで RLS 関連のパターンを検索する
            rls_patterns = [f"'{table}'", "ROW LEVEL SECURITY", "FORCE"]
            # パターンが見つかった場合は True を返す
            return any(
                len(self._grep_codebase(p, self._codebase_root)) > 0
                for p in rls_patterns
            )
        # PostgreSQL に接続して RLS 設定を確認する
        try:
            # データベースに接続する
            conn = psycopg2.connect(db_url)
            # カーソルを取得する（辞書形式でフェッチ）
            with conn.cursor(cursor_factory=psycopg2.extras.DictCursor) as cur:
                # pg_tables カタログから RLS 設定を確認する SQL を実行する
                cur.execute(
                    """
                    SELECT relrowsecurity, relforcerowsecurity
                    FROM pg_class
                    WHERE relname = %s
                    """,
                    (table,),
                )
                # クエリ結果を取得する
                row = cur.fetchone()
                # テーブルが存在しない場合は False を返す
                if row is None:
                    # テーブルが見つからない旨をログに記録する
                    logger.warning("テーブルが見つかりません: %s", table)
                    # False を返す
                    return False
                # RLS が有効かつ FORCE が適用されているかを確認する
                rls_enabled = bool(row["relrowsecurity"])
                # FORCE RLS が適用されているかを確認する
                rls_forced = bool(row["relforcerowsecurity"])
                # RLS の状態をログに記録する
                logger.info(
                    "テーブル %s の RLS: enabled=%s forced=%s",
                    table,
                    rls_enabled,
                    rls_forced,
                )
                # RLS が有効かつ FORCE されている場合のみ True を返す
                return rls_enabled and rls_forced
        except Exception as exc:
            # データベース接続エラーをログに記録する
            logger.error("RLS 確認中にエラーが発生しました: %s", exc)
            # エラーの場合は False を返す
            return False
        finally:
            # 接続が確立されている場合はクローズする
            try:
                # データベース接続をクローズする
                conn.close()
            except Exception:
                # クローズ中のエラーは無視する
                pass

    def compute_overall_effectiveness(
        self, catalog: STRIDECatalog
    ) -> Dict[str, EffectivenessScore]:
        """カタログ内の全緩和策の有効性スコアを計算して返す。

        Args:
            catalog: 評価対象の STRIDE カタログ

        Returns:
            緩和策 ID → EffectivenessScore の辞書
        """
        # 結果辞書を初期化する
        result: Dict[str, EffectivenessScore] = {}
        # カタログ内の全緩和策をイテレートする
        for mitigation in catalog.all_mitigations():
            # 緩和策の実装状態を確認する
            status = self.check_implementation(mitigation, self._codebase_root)
            # キャッシュから証拠リストを取得する
            cache_key = f"{mitigation.mitigation_id}:{self._codebase_root}"
            # キャッシュエントリが存在する場合は証拠リストを取得する
            evidence = []
            # キャッシュにエントリが存在する場合は証拠を取得する
            if cache_key in self._cache:
                # キャッシュから証拠リストを取得する
                evidence = self._cache[cache_key].evidence_found
            # 実装状態に基づいてスコアを計算する
            if status == MitigationStatus.IMPLEMENTED:
                # 完全実装の場合は設定済みの有効性スコアを使用する
                base_score = mitigation.effectiveness
                # 信頼度は証拠件数に基づいて計算する
                confidence = min(1.0, len(evidence) / 5.0)
            elif status == MitigationStatus.PARTIAL:
                # 部分実装の場合は有効性スコアを半減する
                base_score = mitigation.effectiveness * 0.5
                # 信頼度は低めに設定する
                confidence = min(0.6, len(evidence) / 5.0)
            else:
                # 未実装の場合はスコア 0 を設定する
                base_score = 0.0
                # 信頼度は最低値に設定する
                confidence = 0.1
            # EffectivenessScore を生成して結果辞書に追加する
            result[mitigation.mitigation_id] = EffectivenessScore(
                score_0_to_1=base_score,
                confidence=confidence,
                evidence_count=len(evidence),
                evidence_details=evidence[:10],
            )
        # 全緩和策の有効性スコア辞書を返す
        return result

    def batch_check_all(
        self, catalog: STRIDECatalog
    ) -> Tuple[List[CheckResult], List[CheckResult]]:
        """カタログ内の全緩和策をチェックして (成功リスト, 失敗リスト) を返す。"""
        # 成功したチェック結果のリストを初期化する
        passed: List[CheckResult] = []
        # 失敗したチェック結果のリストを初期化する
        failed: List[CheckResult] = []
        # カタログ内の全緩和策をイテレートする
        for mitigation in catalog.all_mitigations():
            # 緩和策の実装状態を確認する
            status = self.check_implementation(mitigation, self._codebase_root)
            # キャッシュキーを生成する
            cache_key = f"{mitigation.mitigation_id}:{self._codebase_root}"
            # キャッシュからチェック結果を取得する
            if cache_key in self._cache:
                # キャッシュからチェック結果を取得する
                check_result = self._cache[cache_key]
            else:
                # キャッシュにない場合は新規に作成する
                check_result = CheckResult(
                    mitigation_id=mitigation.mitigation_id,
                    status=status,
                )
            # チェック結果を成功・失敗リストに振り分ける
            if check_result.is_ok():
                # 成功リストに追加する
                passed.append(check_result)
            else:
                # 失敗リストに追加する
                failed.append(check_result)
        # 成功・失敗リストのタプルを返す
        return passed, failed

    def generate_report_dict(self, catalog: STRIDECatalog) -> Dict[str, Any]:
        """全緩和策チェックの結果をレポート辞書として生成して返す。"""
        # 全緩和策のチェックを実行する
        passed, failed = self.batch_check_all(catalog)
        # 有効性スコアを計算する
        effectiveness = self.compute_overall_effectiveness(catalog)
        # 全体の平均有効性スコアを計算する
        if effectiveness:
            # 全スコアの平均を計算する
            avg_score = sum(e.score_0_to_1 for e in effectiveness.values()) / len(effectiveness)
        else:
            # 緩和策が存在しない場合はスコア 0 とする
            avg_score = 0.0
        # レポート辞書を構築して返す
        return {
            "checked_at": datetime.now(timezone.utc).isoformat(),
            "total_mitigations": len(passed) + len(failed),
            "passed_count": len(passed),
            "failed_count": len(failed),
            "average_effectiveness": round(avg_score, 3),
            "failed_mitigations": [r.mitigation_id for r in failed],
            "effectiveness_by_mitigation": {
                mid: {
                    "score": e.score_0_to_1,
                    "confidence": e.confidence,
                    "evidence_count": e.evidence_count,
                }
                for mid, e in effectiveness.items()
            },
        }
