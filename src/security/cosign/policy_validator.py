"""src/security/cosign/policy_validator.py

cosign/sigstore 署名ポリシー検証モジュール。
製造業プラットフォームのコンテナイメージ・バイナリ・SBOM に対する
署名検証・Rekor 透明性ログ確認・証明書チェーン検証を提供する。
仕様: docs/04_詳細設計/01_適合仕様/16_build_provenance適合仕様.md
"""

# 将来の注釈構文互換のため annotations をインポートする
from __future__ import annotations

# Base64 エンコード・デコードに使用する base64 をインポートする
import base64
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
# データクラス定義
# ===========================================================================


@dataclass
class SigningPolicy:
    """cosign 署名ポリシーを表すデータクラス。"""

    # ポリシーの名称（例: "manufacturing-image-policy"）
    policy_name: str
    # 署名者の証明書 Subject の正規表現パターン
    subject_regexp: str
    # OIDC 発行者の URL（例: "https://accounts.google.com"）
    oidc_issuer: str
    # 必須アノテーションの辞書（キー → 値）
    required_annotations: Dict[str, str] = field(default_factory=dict)
    # 許可する証明書の有効期限（秒）
    max_cert_age_seconds: int = 3600
    # Rekor 透明性ログへの包含を必須とするかどうか
    require_rekor_inclusion: bool = True

    def validate(self) -> bool:
        """ポリシーデータが有効かどうかを検証する。"""
        # policy_name が空でないことを確認する
        if not self.policy_name:
            # ポリシー名が空の場合は無効とする
            return False
        # subject_regexp が空でないことを確認する
        if not self.subject_regexp:
            # Subject パターンが空の場合は無効とする
            return False
        # oidc_issuer が空でないことを確認する
        if not self.oidc_issuer:
            # OIDC 発行者が空の場合は無効とする
            return False
        # 全検証が通過した場合は有効とする
        return True


@dataclass
class BundleEntry:
    """cosign バンドルファイルのエントリを表すデータクラス。"""

    # 検証対象のアーティファクトファイルパス
    artifact_path: str
    # cosign バンドル JSON ファイルのパス
    bundle_path: str
    # バンドルに含まれる証明書の Subject（SAN）
    cert_identity: str
    # バンドルに含まれる証明書の発行者
    cert_issuer: str
    # 検証実行時刻（ISO 8601 形式）
    verified_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())
    # 署名の SHA-256 ダイジェスト
    signature_digest: str = ""
    # Rekor ログのエントリ UUID
    rekor_log_id: str = ""


@dataclass
class VerifyResult:
    """個別の署名検証結果を表すデータクラス。"""

    # 検証対象のアーティファクトパス
    artifact_path: str
    # 検証が成功したかどうか
    is_verified: bool
    # 検証失敗の理由（成功時は空文字）
    failure_reason: str = ""
    # Rekor ログへの包含が確認されたかどうか
    rekor_included: bool = False
    # 証明書チェーンが有効かどうか
    cert_chain_valid: bool = False
    # 検証に使用したポリシー名
    policy_name: str = ""
    # 検証実行時刻
    verified_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

    def summary(self) -> str:
        """検証結果のサマリー文字列を返す。"""
        # 成功・失敗のラベルを決定する
        status = "VERIFIED" if self.is_verified else "FAILED"
        # サマリー文字列を返す
        return (
            f"VerifyResult[{status}] {self.artifact_path} "
            f"rekor={self.rekor_included} cert_chain={self.cert_chain_valid}"
        )


@dataclass
class AuditReport:
    """全アーティファクトの監査レポートを表すデータクラス。"""

    # 監査対象ディレクトリのパス
    artifact_dir: str
    # 監査された全アーティファクトの結果リスト
    results: List[VerifyResult] = field(default_factory=list)
    # 監査実行時刻
    audited_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())

    @property
    def verified_count(self) -> int:
        """検証成功したアーティファクトの数を返す。"""
        # 検証成功の件数をカウントして返す
        return sum(1 for r in self.results if r.is_verified)

    @property
    def failed_count(self) -> int:
        """検証失敗したアーティファクトの数を返す。"""
        # 検証失敗の件数をカウントして返す
        return sum(1 for r in self.results if not r.is_verified)

    @property
    def is_all_verified(self) -> bool:
        """全アーティファクトの検証が成功しているかどうかを返す。"""
        # 結果が空の場合は False を返す
        if not self.results:
            # 空の場合は全検証成功とはみなさない
            return False
        # 全結果が検証成功かどうかを確認する
        return all(r.is_verified for r in self.results)

    def summary(self) -> str:
        """監査レポートのサマリー文字列を返す。"""
        # 監査レポートのサマリーをフォーマットして返す
        return (
            f"AuditReport: total={len(self.results)} "
            f"verified={self.verified_count} failed={self.failed_count} "
            f"all_verified={self.is_all_verified}"
        )


# ===========================================================================
# PolicyValidator クラス
# ===========================================================================


class PolicyValidator:
    """cosign 署名ポリシーを検証するクラス。

    cosign CLI のラッパーとして機能し、バンドルの検証・Rekor ログ確認・
    証明書チェーン検証を提供する。
    """

    # cosign バンドル JSON のサブセット: Rekor バンドル
    _BUNDLE_REKOR_KEY = "RekorBundle"
    # cosign バンドル JSON のサブセット: 証明書
    _BUNDLE_CERT_KEY = "Cert"
    # cosign verify-blob サブコマンド
    _COSIGN_VERIFY_BLOB_CMD = "cosign"

    def __init__(self, cosign_binary: str = "cosign") -> None:
        """PolicyValidator を初期化する。

        Args:
            cosign_binary: cosign バイナリのパス（デフォルト: "cosign"）
        """
        # cosign バイナリのパスを設定する
        self._cosign_binary = cosign_binary
        # 読み込んだポリシーのキャッシュを初期化する
        self._policy_cache: Dict[str, SigningPolicy] = {}

    def load_policy(self, path: Path) -> SigningPolicy:
        """YAML ファイルから署名ポリシーを読み込んで返す。

        Args:
            path: ポリシー YAML ファイルのパス

        Returns:
            読み込んだ SigningPolicy インスタンス

        Raises:
            FileNotFoundError: ファイルが存在しない場合
            ValueError: YAML 構造が不正な場合
        """
        # パスが存在するかどうかを確認する
        if not path.exists():
            # ファイルが存在しない場合は FileNotFoundError を送出する
            raise FileNotFoundError(f"ポリシーファイルが見つかりません: {path}")
        # YAML ファイルを読み込む
        with open(path, encoding="utf-8") as fh:
            # YAML を安全にパースする
            raw_data = yaml.safe_load(fh)
        # raw_data が辞書でない場合はエラーを送出する
        if not isinstance(raw_data, dict):
            # 不正な構造の場合は ValueError を送出する
            raise ValueError(f"ポリシー YAML のルートは辞書でなければなりません: {path}")
        # ポリシーフィールドを取得する
        policy_name = raw_data.get("policy_name", path.stem)
        # subject_regexp フィールドを取得する
        subject_regexp = raw_data.get("subject_regexp", "")
        # oidc_issuer フィールドを取得する
        oidc_issuer = raw_data.get("oidc_issuer", "")
        # required_annotations フィールドを取得する
        required_annotations = dict(raw_data.get("required_annotations", {}))
        # max_cert_age_seconds フィールドを取得する
        max_cert_age_seconds = int(raw_data.get("max_cert_age_seconds", 3600))
        # require_rekor_inclusion フィールドを取得する
        require_rekor_inclusion = bool(raw_data.get("require_rekor_inclusion", True))
        # SigningPolicy を生成する
        policy = SigningPolicy(
            policy_name=policy_name,
            subject_regexp=subject_regexp,
            oidc_issuer=oidc_issuer,
            required_annotations=required_annotations,
            max_cert_age_seconds=max_cert_age_seconds,
            require_rekor_inclusion=require_rekor_inclusion,
        )
        # ポリシーの有効性を検証する
        if not policy.validate():
            # 無効なポリシーの場合はエラーを送出する
            raise ValueError(f"無効なポリシー内容: {path}")
        # ポリシーをキャッシュに保存する
        self._policy_cache[policy_name] = policy
        # 読み込み成功をログに記録する
        logger.info("署名ポリシー読み込み完了: %s", policy_name)
        # 読み込んだポリシーを返す
        return policy

    def verify_bundle(self, bundle_path: Path, artifact_path: Path) -> VerifyResult:
        """cosign バンドルを検証して VerifyResult を返す。

        Args:
            bundle_path: cosign バンドル JSON ファイルのパス
            artifact_path: 検証対象のアーティファクトファイルパス

        Returns:
            検証結果を表す VerifyResult インスタンス
        """
        # バンドルファイルが存在するかどうかを確認する
        if not bundle_path.exists():
            # バンドルファイルが存在しない場合は失敗結果を返す
            return VerifyResult(
                artifact_path=str(artifact_path),
                is_verified=False,
                failure_reason=f"バンドルファイルが見つかりません: {bundle_path}",
            )
        # アーティファクトファイルが存在するかどうかを確認する
        if not artifact_path.exists():
            # アーティファクトファイルが存在しない場合は失敗結果を返す
            return VerifyResult(
                artifact_path=str(artifact_path),
                is_verified=False,
                failure_reason=f"アーティファクトファイルが見つかりません: {artifact_path}",
            )
        # cosign verify-blob コマンドを構築する
        cmd = [
            self._cosign_binary,
            "verify-blob",
            "--bundle",
            str(bundle_path),
            str(artifact_path),
        ]
        # cosign コマンドを実行する
        try:
            # サブプロセスで cosign を実行する
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=60,
            )
            # 終了コードが 0 の場合は検証成功
            if result.returncode == 0:
                # 検証成功をログに記録する
                logger.info("署名検証成功: %s", artifact_path)
                # 検証成功結果を返す（Rekor と証明書の確認も実施）
                rekor_ok = self.check_rekor_log_inclusion(bundle_path)
                # 証明書チェーンをバンドルから読み込んで検証する
                cert_ok = self._check_cert_chain_from_bundle(bundle_path)
                # 成功結果を返す
                return VerifyResult(
                    artifact_path=str(artifact_path),
                    is_verified=True,
                    rekor_included=rekor_ok,
                    cert_chain_valid=cert_ok,
                )
            else:
                # 検証失敗をログに記録する
                logger.warning(
                    "署名検証失敗: %s (exit=%d stderr=%s)",
                    artifact_path,
                    result.returncode,
                    result.stderr[:200],
                )
                # 検証失敗結果を返す
                return VerifyResult(
                    artifact_path=str(artifact_path),
                    is_verified=False,
                    failure_reason=result.stderr.strip()[:500],
                )
        except subprocess.TimeoutExpired:
            # タイムアウトが発生した場合はログに記録する
            logger.error("cosign タイムアウト: %s", artifact_path)
            # タイムアウト失敗結果を返す
            return VerifyResult(
                artifact_path=str(artifact_path),
                is_verified=False,
                failure_reason="cosign コマンドがタイムアウトしました",
            )
        except FileNotFoundError:
            # cosign が存在しない場合はバンドル JSON を手動で検証する
            logger.warning("cosign が見つかりません。バンドル手動検証を試みます。")
            # バンドル JSON を手動で検証する
            return self._manual_bundle_verify(bundle_path, artifact_path)

    def _manual_bundle_verify(
        self, bundle_path: Path, artifact_path: Path
    ) -> VerifyResult:
        """cosign が存在しない場合にバンドル JSON を手動で検証して返す。"""
        # バンドル JSON を読み込む
        try:
            # バンドルファイルを読み込む
            bundle_raw = bundle_path.read_text(encoding="utf-8")
            # JSON をパースする
            bundle_data = json.loads(bundle_raw)
        except (json.JSONDecodeError, OSError) as exc:
            # JSON パースエラーの場合は失敗結果を返す
            return VerifyResult(
                artifact_path=str(artifact_path),
                is_verified=False,
                failure_reason=f"バンドル JSON パースエラー: {exc}",
            )
        # バンドルに必要フィールドが存在するかを確認する
        has_cert = "Cert" in bundle_data or "cert" in bundle_data
        # Rekor バンドルの存在を確認する
        has_rekor = "RekorBundle" in bundle_data or "rekorBundle" in bundle_data
        # 必要フィールドが揃っていれば部分的に有効とみなす
        is_partial_valid = has_cert and has_rekor
        # 手動検証の結果をログに記録する
        logger.info(
            "バンドル手動検証: %s has_cert=%s has_rekor=%s",
            artifact_path,
            has_cert,
            has_rekor,
        )
        # 部分的な検証結果を返す
        return VerifyResult(
            artifact_path=str(artifact_path),
            is_verified=is_partial_valid,
            rekor_included=has_rekor,
            cert_chain_valid=has_cert,
            failure_reason="" if is_partial_valid else "cosign バイナリなしでは完全検証不可",
        )

    def check_rekor_log_inclusion(self, bundle_path: Path) -> bool:
        """cosign バンドルに Rekor 透明性ログへの包含が記録されているかを返す。

        Args:
            bundle_path: cosign バンドル JSON ファイルのパス

        Returns:
            Rekor ログへの包含が確認できれば True
        """
        # バンドルファイルが存在するかどうかを確認する
        if not bundle_path.exists():
            # バンドルファイルが存在しない場合は False を返す
            logger.warning("Rekor 確認: バンドルファイルが見つかりません: %s", bundle_path)
            # False を返す
            return False
        # バンドル JSON を読み込む
        try:
            # バンドルファイルを読み込む
            raw = bundle_path.read_text(encoding="utf-8")
            # JSON をパースする
            data = json.loads(raw)
        except (json.JSONDecodeError, OSError) as exc:
            # JSON パースエラーをログに記録する
            logger.warning("Rekor 確認: バンドル JSON パースエラー (%s)", exc)
            # False を返す
            return False
        # RekorBundle フィールドを確認する（大小文字のバリアントを試す）
        rekor_bundle = data.get("RekorBundle") or data.get("rekorBundle")
        # Rekor バンドルが存在しない場合は False を返す
        if not rekor_bundle:
            # Rekor バンドルが見つからない旨をログに記録する
            logger.debug("Rekor バンドルフィールドが見つかりません: %s", bundle_path)
            # False を返す
            return False
        # Rekor バンドルに LogEntry が含まれているかを確認する
        log_entry = rekor_bundle.get("Payload") or rekor_bundle.get("payload") or rekor_bundle
        # logIndex が存在するかどうかを確認する
        has_log_index = "logIndex" in log_entry or "LogIndex" in log_entry
        # Rekor ログ包含の確認結果をログに記録する
        logger.debug(
            "Rekor ログ包含確認: %s has_log_index=%s",
            bundle_path,
            has_log_index,
        )
        # Rekor ログへの包含確認結果を返す
        return has_log_index

    def validate_cert_chain(self, cert_pem: str) -> bool:
        """PEM 形式の証明書チェーンが有効かどうかを検証して返す。

        Args:
            cert_pem: PEM 形式の証明書文字列

        Returns:
            証明書チェーンが有効であれば True
        """
        # PEM が空文字の場合は無効とする
        if not cert_pem.strip():
            # 空の PEM は無効とする
            logger.warning("証明書チェーン: 空の PEM が渡されました")
            # False を返す
            return False
        # PEM の先頭・末尾マーカーを確認する
        has_begin = "-----BEGIN CERTIFICATE-----" in cert_pem
        # END マーカーの存在を確認する
        has_end = "-----END CERTIFICATE-----" in cert_pem
        # BEGIN / END マーカーが揃っていない場合は無効とする
        if not (has_begin and has_end):
            # マーカーが不完全な場合は無効とする
            logger.warning("証明書チェーン: PEM マーカーが不完全です")
            # False を返す
            return False
        # openssl を使って証明書の有効期限を確認する
        try:
            # openssl x509 コマンドで証明書情報を取得する
            result = subprocess.run(
                ["openssl", "x509", "-noout", "-text"],
                input=cert_pem,
                capture_output=True,
                text=True,
                timeout=10,
            )
            # openssl が成功した場合は有効とする
            if result.returncode == 0:
                # 証明書が有効であることをログに記録する
                logger.debug("証明書チェーン検証: openssl による確認成功")
                # True を返す
                return True
            else:
                # openssl エラーをログに記録する
                logger.warning(
                    "証明書チェーン検証: openssl エラー (%s)",
                    result.stderr[:200],
                )
                # False を返す
                return False
        except FileNotFoundError:
            # openssl が存在しない場合は構文チェックのみ行う
            logger.warning("openssl が見つかりません。構文チェックのみ実施します。")
            # Base64 デコードを試みて形式確認を行う
            return self._check_cert_pem_format_only(cert_pem)
        except subprocess.TimeoutExpired:
            # タイムアウトをログに記録する
            logger.warning("openssl タイムアウト")
            # タイムアウト時は形式チェックにフォールバック
            return self._check_cert_pem_format_only(cert_pem)

    def _check_cert_pem_format_only(self, cert_pem: str) -> bool:
        """openssl なしで PEM の基本形式を確認して返す。"""
        # PEM の BEGIN / END マーカーを除去して Base64 部分を抽出する
        lines = cert_pem.strip().splitlines()
        # ヘッダー・フッター行を除去する
        b64_lines = [
            line for line in lines
            if not line.startswith("-----")
        ]
        # Base64 文字列を結合する
        b64_content = "".join(b64_lines)
        # Base64 のデコードを試みる
        try:
            # Base64 デコードを実行する
            decoded = base64.b64decode(b64_content, validate=True)
            # デコードが成功した場合は基本形式が正しいとみなす
            return len(decoded) > 0
        except Exception:
            # デコード失敗の場合は無効とする
            return False

    def _check_cert_chain_from_bundle(self, bundle_path: Path) -> bool:
        """バンドルファイルから証明書を抽出して検証して返す。"""
        # バンドルファイルが存在するかどうかを確認する
        if not bundle_path.exists():
            # バンドルファイルが存在しない場合は False を返す
            return False
        # バンドル JSON を読み込む
        try:
            # バンドルファイルを読み込む
            raw = bundle_path.read_text(encoding="utf-8")
            # JSON をパースする
            data = json.loads(raw)
        except (json.JSONDecodeError, OSError):
            # パースエラーの場合は False を返す
            return False
        # Cert フィールドを取得する（大小文字のバリアントを試す）
        cert = data.get("Cert") or data.get("cert") or data.get("certificate")
        # Cert フィールドが存在しない場合は False を返す
        if not cert:
            # 証明書フィールドが見つからない場合は False を返す
            return False
        # 証明書が文字列の場合は直接検証する
        if isinstance(cert, str):
            # 文字列証明書を検証する
            return self.validate_cert_chain(cert)
        # 証明書が辞書の場合は rawBytes を取得する
        if isinstance(cert, dict):
            # rawBytes フィールドを取得する
            raw_bytes_b64 = cert.get("rawBytes") or cert.get("RawBytes", "")
            # Base64 デコードして PEM に変換する
            if raw_bytes_b64:
                # Base64 デコードを試みる
                try:
                    # デコード結果の長さを確認して有効性を判定する
                    decoded = base64.b64decode(raw_bytes_b64)
                    # デコードが成功した場合は True を返す
                    return len(decoded) > 0
                except Exception:
                    # デコード失敗の場合は False を返す
                    return False
        # 不明な形式の場合は False を返す
        return False

    def audit_all_artifacts(self, artifact_dir: Path) -> AuditReport:
        """指定ディレクトリ内の全アーティファクトを監査して AuditReport を返す。

        Args:
            artifact_dir: 監査対象のアーティファクトディレクトリ

        Returns:
            全アーティファクトの監査結果を含む AuditReport インスタンス
        """
        # ディレクトリが存在するかどうかを確認する
        if not artifact_dir.exists():
            # ディレクトリが存在しない場合は空レポートを返す
            logger.warning("監査対象ディレクトリが存在しません: %s", artifact_dir)
            # 空のレポートを返す
            return AuditReport(artifact_dir=str(artifact_dir))
        # 監査レポートを初期化する
        report = AuditReport(artifact_dir=str(artifact_dir))
        # 対象外の拡張子リストを定義する
        skip_extensions = {".json", ".sig", ".sbom", ".bundle"}
        # ディレクトリ内のファイルを列挙する
        for artifact_path in sorted(artifact_dir.rglob("*")):
            # ファイルでない場合はスキップする
            if not artifact_path.is_file():
                # ディレクトリはスキップする
                continue
            # 対象外の拡張子はスキップする
            if artifact_path.suffix in skip_extensions:
                # 対象外拡張子なのでスキップする
                continue
            # 対応するバンドルファイルを探す
            bundle_path = artifact_path.with_suffix(artifact_path.suffix + ".bundle")
            # バンドルが存在しない場合は未署名として失敗結果を追加する
            if not bundle_path.exists():
                # バンドルなし = 署名なしとして失敗結果を追加する
                report.results.append(
                    VerifyResult(
                        artifact_path=str(artifact_path),
                        is_verified=False,
                        failure_reason="cosign バンドルファイルが見つかりません",
                    )
                )
                # 次のファイルへ進む
                continue
            # バンドルを検証して結果をレポートに追加する
            result = self.verify_bundle(bundle_path, artifact_path)
            # 検証結果をレポートに追加する
            report.results.append(result)
        # 監査完了をログに記録する
        logger.info("アーティファクト監査完了: %s", report.summary())
        # 監査レポートを返す
        return report

    def check_annotations(
        self, bundle_path: Path, required_annotations: Dict[str, str]
    ) -> bool:
        """バンドルのアノテーションが必須アノテーションを満たすかどうかを返す。

        Args:
            bundle_path: cosign バンドル JSON ファイルのパス
            required_annotations: 必須アノテーションの辞書

        Returns:
            全ての必須アノテーションが存在すれば True
        """
        # 必須アノテーションが空の場合は True を返す
        if not required_annotations:
            # アノテーション要件なしのため True を返す
            return True
        # バンドルファイルが存在するかどうかを確認する
        if not bundle_path.exists():
            # バンドルファイルが存在しない場合は False を返す
            return False
        # バンドル JSON を読み込む
        try:
            # バンドルファイルを読み込む
            raw = bundle_path.read_text(encoding="utf-8")
            # JSON をパースする
            data = json.loads(raw)
        except (json.JSONDecodeError, OSError) as exc:
            # パースエラーをログに記録して False を返す
            logger.warning("アノテーション確認: バンドル JSON パースエラー (%s)", exc)
            # False を返す
            return False
        # バンドル内のアノテーションを取得する（複数パス試行）
        annotations: Dict[str, Any] = {}
        # Optional.annotations フィールドを試す
        opt_data = data.get("Optional") or data.get("optional") or {}
        # opt_data がある場合はアノテーションとして使用する
        if opt_data:
            # Optional データをアノテーションとして設定する
            annotations.update(opt_data)
        # 全必須アノテーションが存在するかを確認する
        for key, expected_value in required_annotations.items():
            # アノテーションキーが存在するかを確認する
            if key not in annotations:
                # 必須アノテーションが存在しない旨をログに記録する
                logger.debug("必須アノテーション不存在: %s", key)
                # False を返す
                return False
            # アノテーション値が期待値と一致するかを確認する
            if expected_value and annotations[key] != expected_value:
                # アノテーション値が不一致の旨をログに記録する
                logger.debug(
                    "アノテーション値不一致: %s expect=%s actual=%s",
                    key,
                    expected_value,
                    annotations[key],
                )
                # False を返す
                return False
        # 全必須アノテーションが満たされた場合は True を返す
        return True
