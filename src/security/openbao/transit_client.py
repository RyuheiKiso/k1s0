"""src/security/openbao/transit_client.py

OpenBao Transit 暗号化クライアントモジュール。
製造業プラットフォームの機密データ（PII・KEK・署名鍵）を
OpenBao Transit エンジン経由で暗号化・復号・鍵ローテーションする。
仕様: src/security/openbao/transit_config.yaml
"""

# 将来の注釈構文互換のため annotations をインポートする
from __future__ import annotations

# Base64 エンコード・デコードに使用する base64 をインポートする
import base64
# ハッシュ計算（HMAC-SHA256）に使用する hashlib / hmac をインポートする
import hashlib
# HMAC 署名計算に使用する hmac をインポートする
import hmac
# JSON 処理に使用する json をインポートする
import json
# ログ出力に使用する logging をインポートする
import logging
# OS 環境変数の取得に使用する os をインポートする
import os
# システム操作に使用する sys をインポートする
import sys
# 時刻処理に使用する time をインポートする
import time
# 型安全なデータクラスに使用する dataclasses をインポートする
from dataclasses import dataclass, field
# 日時処理に使用する datetime をインポートする
from datetime import datetime, timezone
# パス操作に使用する pathlib をインポートする
from pathlib import Path
# 型ヒント定義に使用する typing をインポートする
from typing import Any, Dict, List, Optional, Union
# URL パース・エンコードに使用する urllib をインポートする
from urllib.parse import urljoin, urlparse

# urllib.request を使った HTTP リクエスト実行に使用する
import urllib.request
# urllib.error を使った HTTP エラー処理に使用する
import urllib.error

# モジュール専用ロガーを取得する
logger = logging.getLogger(__name__)

# OpenBao API バージョンのプレフィックス
_OPENBAO_API_V1 = "/v1/"
# OpenBao Transit エンジンのデフォルトマウントパス
_DEFAULT_TRANSIT_MOUNT = "transit"
# デフォルトの接続タイムアウト（秒）
_DEFAULT_TIMEOUT_SECONDS = 30
# 環境変数からの OpenBao アドレス取得キー
_ENV_OPENBAO_ADDR = "OPENBAO_ADDR"
# 環境変数からの OpenBao トークン取得キー
_ENV_OPENBAO_TOKEN = "OPENBAO_TOKEN"


# ===========================================================================
# データクラス定義
# ===========================================================================


@dataclass
class TransitKey:
    """OpenBao Transit エンジンの暗号化キーを表すデータクラス。"""

    # キーの名称（例: "pii-encryption-key"）
    key_name: str
    # キーの種別（例: "aes256-gcm96", "chacha20-poly1305", "ed25519"）
    key_type: str
    # キーのローテーション周期（例: "720h" = 30 日）
    rotation_period: str
    # 復号に使用できる最小バージョン
    min_decryption_version: int
    # 最新のキーバージョン
    latest_version: int = 1
    # キーが削除可能かどうか
    deletion_allowed: bool = False
    # キーがエクスポート可能かどうか（エクスポート禁止が推奨）
    exportable: bool = False
    # convergent_encryption が有効かどうか
    convergent_encryption: bool = False

    def is_rotation_needed(self, current_version: int) -> bool:
        """キーのローテーションが必要かどうかを返す。"""
        # 最新バージョンと現在バージョンを比較してローテーション要否を返す
        return current_version < self.latest_version


@dataclass
class EncryptRequest:
    """暗号化リクエストのパラメータを表すデータクラス。"""

    # 暗号化対象のプレーンテキスト（バイト列）
    plaintext: bytes
    # 使用するキー名
    key_name: str
    # コンテキスト値（Derived Key の派生に使用、省略可能）
    context: Optional[bytes] = None
    # キーバージョン（0 は最新バージョンを使用）
    key_version: int = 0

    def to_api_payload(self) -> Dict[str, Any]:
        """OpenBao API リクエストのペイロード辞書を生成して返す。"""
        # プレーンテキストを Base64 エンコードする（API 仕様準拠）
        plaintext_b64 = base64.b64encode(self.plaintext).decode("ascii")
        # ペイロード辞書を初期化する
        payload: Dict[str, Any] = {"plaintext": plaintext_b64}
        # コンテキストが指定されている場合は追加する
        if self.context:
            # コンテキストを Base64 エンコードして追加する
            payload["context"] = base64.b64encode(self.context).decode("ascii")
        # キーバージョンが指定されている場合は追加する
        if self.key_version > 0:
            # キーバージョンを追加する
            payload["key_version"] = self.key_version
        # ペイロード辞書を返す
        return payload


@dataclass
class RewrapRequest:
    """キーローテーション後の ciphertext 再ラップリクエストを表すデータクラス。"""

    # 再ラップ対象の暗号文（vault:v1:... 形式）
    ciphertext: str
    # 使用するキー名
    key_name: str
    # コンテキスト値（元の暗号化時のコンテキストと一致させる必要がある）
    context: Optional[bytes] = None

    def to_api_payload(self) -> Dict[str, Any]:
        """OpenBao API リクエストのペイロード辞書を生成して返す。"""
        # ペイロード辞書を初期化する
        payload: Dict[str, Any] = {"ciphertext": self.ciphertext}
        # コンテキストが指定されている場合は追加する
        if self.context:
            # コンテキストを Base64 エンコードして追加する
            payload["context"] = base64.b64encode(self.context).decode("ascii")
        # ペイロード辞書を返す
        return payload


@dataclass
class TransitResponse:
    """OpenBao Transit API のレスポンスを表すデータクラス。"""

    # HTTP ステータスコード
    status_code: int
    # レスポンスデータの辞書
    data: Dict[str, Any] = field(default_factory=dict)
    # エラーメッセージ（成功時は空文字）
    errors: List[str] = field(default_factory=list)

    @property
    def is_success(self) -> bool:
        """レスポンスが成功かどうかを返す。"""
        # HTTP ステータスコードが 200 台であれば成功とみなす
        return 200 <= self.status_code < 300

    def get_ciphertext(self) -> str:
        """レスポンスデータから ciphertext を抽出して返す。"""
        # data.ciphertext フィールドを返す（存在しない場合は空文字）
        return self.data.get("ciphertext", "")

    def get_plaintext(self) -> bytes:
        """レスポンスデータから平文を Base64 デコードして返す。"""
        # data.plaintext フィールドを取得する
        plaintext_b64 = self.data.get("plaintext", "")
        # 空文字の場合は空バイト列を返す
        if not plaintext_b64:
            # 空文字の場合は空バイト列を返す
            return b""
        # Base64 デコードして返す
        return base64.b64decode(plaintext_b64)


# ===========================================================================
# TransitClient クラス
# ===========================================================================


class TransitClient:
    """OpenBao Transit エンジンクライアントクラス。

    OpenBao の Transit シークレットエンジンに対して暗号化・復号・
    キーローテーション・キートリム・再ラップ操作を提供する。
    製造業 PII データの暗号化に特化した実装。
    """

    def __init__(
        self,
        addr: Optional[str] = None,
        token: Optional[str] = None,
        transit_mount: str = _DEFAULT_TRANSIT_MOUNT,
        timeout: int = _DEFAULT_TIMEOUT_SECONDS,
        verify_tls: bool = True,
    ) -> None:
        """TransitClient を初期化する。

        Args:
            addr: OpenBao サーバーのアドレス（省略時は OPENBAO_ADDR 環境変数を使用）
            token: OpenBao のアクセストークン（省略時は OPENBAO_TOKEN 環境変数を使用）
            transit_mount: Transit エンジンのマウントパス（デフォルト: "transit"）
            timeout: HTTP タイムアウト秒数
            verify_tls: TLS 証明書を検証するかどうか
        """
        # OpenBao アドレスを設定する（引数 → 環境変数 → デフォルト の順）
        self._addr = addr or os.environ.get(_ENV_OPENBAO_ADDR, "http://localhost:8200")
        # トークンを設定する（引数 → 環境変数 の順）
        self._token = token or os.environ.get(_ENV_OPENBAO_TOKEN, "")
        # Transit エンジンのマウントパスを設定する
        self._transit_mount = transit_mount.strip("/")
        # HTTP タイムアウト秒数を設定する
        self._timeout = timeout
        # TLS 検証フラグを設定する
        self._verify_tls = verify_tls
        # アドレスの末尾スラッシュを除去する
        self._addr = self._addr.rstrip("/")
        # トークンが設定されているかどうかを確認する
        if not self._token:
            # トークンが未設定の場合はログに警告を出力する
            logger.warning(
                "OpenBao トークンが設定されていません。"
                "OPENBAO_TOKEN 環境変数を設定してください。"
            )

    def _build_url(self, path: str) -> str:
        """OpenBao API の完全 URL を構築して返す。

        Args:
            path: API エンドポイントのパス（先頭スラッシュなし）

        Returns:
            完全 URL 文字列
        """
        # Transit エンジンのベースパスを構築する
        base_path = f"{_OPENBAO_API_V1}{self._transit_mount}/{path}"
        # アドレスとパスを結合して完全 URL を返す
        return f"{self._addr}{base_path}"

    def _make_request(
        self,
        method: str,
        url: str,
        payload: Optional[Dict[str, Any]] = None,
    ) -> TransitResponse:
        """OpenBao API に HTTP リクエストを送信して TransitResponse を返す。

        Args:
            method: HTTP メソッド（"GET", "POST", "PUT", "DELETE"）
            url: リクエスト先の完全 URL
            payload: リクエストボディの辞書（None の場合は送信しない）

        Returns:
            TransitResponse インスタンス
        """
        # リクエストボディをバイト列に変換する
        body_bytes: Optional[bytes] = None
        # ペイロードが存在する場合は JSON に変換する
        if payload is not None:
            # JSON 文字列に変換してバイト列にエンコードする
            body_bytes = json.dumps(payload).encode("utf-8")
        # HTTP リクエストを生成する
        req = urllib.request.Request(url, data=body_bytes, method=method)
        # Content-Type ヘッダーを設定する
        req.add_header("Content-Type", "application/json")
        # OpenBao トークンを X-Vault-Token ヘッダーで送信する（互換性のため）
        req.add_header("X-Vault-Token", self._token)
        # リクエスト識別子として X-Request-ID ヘッダーを設定する
        req.add_header("X-Request-ID", self._generate_request_id())
        # HTTP リクエストを実行する
        try:
            # urllib で HTTP リクエストを送信する
            with urllib.request.urlopen(req, timeout=self._timeout) as resp:
                # レスポンスのステータスコードを取得する
                status_code = resp.status
                # レスポンスボディを読み込む
                raw_body = resp.read()
                # レスポンスボディが存在する場合は JSON デコードする
                if raw_body:
                    # JSON をパースしてデータ辞書を生成する
                    resp_data = json.loads(raw_body.decode("utf-8"))
                    # data フィールドを取得する
                    data = resp_data.get("data", {})
                else:
                    # ボディが空の場合は空辞書を設定する
                    data = {}
                # 成功レスポンスを返す
                return TransitResponse(status_code=status_code, data=data)
        except urllib.error.HTTPError as exc:
            # HTTP エラーをログに記録する
            logger.error("OpenBao HTTP エラー: %s %s → %s", method, url, exc.code)
            # エラーレスポンスボディを読み込む
            error_body = exc.read().decode("utf-8", errors="ignore")
            # エラーレスポンスを JSON デコードする
            try:
                # JSON デコードを試みる
                error_data = json.loads(error_body)
                # errors フィールドを取得する
                errors = error_data.get("errors", [str(exc)])
            except json.JSONDecodeError:
                # JSON デコード失敗の場合は文字列をそのままリストに変換する
                errors = [error_body[:500]]
            # エラーレスポンスを返す
            return TransitResponse(status_code=exc.code, errors=errors)
        except urllib.error.URLError as exc:
            # URL エラーをログに記録する（接続失敗・タイムアウト等）
            logger.error("OpenBao 接続エラー: %s %s → %s", method, url, exc)
            # 接続エラーレスポンスを返す
            return TransitResponse(
                status_code=0,
                errors=[f"接続エラー: {exc.reason}"],
            )

    @staticmethod
    def _generate_request_id() -> str:
        """一意のリクエスト ID を生成して返す。"""
        # 現在時刻のナノ秒を使って一意 ID を生成する
        timestamp = str(time.time_ns())
        # SHA-256 ハッシュを計算して先頭 16 文字を返す
        return hashlib.sha256(timestamp.encode()).hexdigest()[:16]

    def encrypt(self, plaintext: bytes, key_name: str, context: Optional[bytes] = None) -> str:
        """プレーンテキストを OpenBao Transit で暗号化して ciphertext を返す。

        Args:
            plaintext: 暗号化対象のバイト列
            key_name: 使用する Transit キー名
            context: Derived Key のコンテキスト（省略可能）

        Returns:
            "vault:v1:..." 形式の ciphertext 文字列

        Raises:
            ValueError: 暗号化が失敗した場合
        """
        # 暗号化リクエストを生成する
        request = EncryptRequest(plaintext=plaintext, key_name=key_name, context=context)
        # API エンドポイント URL を構築する
        url = self._build_url(f"encrypt/{key_name}")
        # ペイロードを生成する
        payload = request.to_api_payload()
        # OpenBao API にリクエストを送信する
        response = self._make_request("POST", url, payload)
        # リクエストが失敗した場合はエラーを送出する
        if not response.is_success:
            # エラーメッセージを結合してエラーを送出する
            error_msg = "; ".join(response.errors) or "不明なエラー"
            # ValueError を送出する
            raise ValueError(f"暗号化失敗 (key={key_name}): {error_msg}")
        # ciphertext を取得する
        ciphertext = response.get_ciphertext()
        # ciphertext が空の場合はエラーを送出する
        if not ciphertext:
            # 空の ciphertext の場合はエラーを送出する
            raise ValueError(f"暗号化レスポンスに ciphertext が含まれていません (key={key_name})")
        # 暗号化成功をログに記録する（平文はログに出力しない）
        logger.debug("暗号化成功: key=%s ciphertext_prefix=%s", key_name, ciphertext[:20])
        # ciphertext を返す
        return ciphertext

    def decrypt(self, ciphertext: str, key_name: str, context: Optional[bytes] = None) -> bytes:
        """ciphertext を OpenBao Transit で復号してプレーンテキストを返す。

        Args:
            ciphertext: "vault:v1:..." 形式の暗号文文字列
            key_name: 使用する Transit キー名
            context: Derived Key のコンテキスト（暗号化時と同一値が必要）

        Returns:
            復号されたバイト列

        Raises:
            ValueError: 復号が失敗した場合
        """
        # ciphertext が vault: プレフィックスを持つか確認する
        if not ciphertext.startswith("vault:"):
            # 不正な形式の場合はエラーを送出する
            raise ValueError(f"不正な ciphertext 形式: {ciphertext[:30]}")
        # ペイロードを構築する
        payload: Dict[str, Any] = {"ciphertext": ciphertext}
        # コンテキストが指定されている場合は追加する
        if context:
            # コンテキストを Base64 エンコードして追加する
            payload["context"] = base64.b64encode(context).decode("ascii")
        # API エンドポイント URL を構築する
        url = self._build_url(f"decrypt/{key_name}")
        # OpenBao API にリクエストを送信する
        response = self._make_request("POST", url, payload)
        # リクエストが失敗した場合はエラーを送出する
        if not response.is_success:
            # エラーメッセージを結合してエラーを送出する
            error_msg = "; ".join(response.errors) or "不明なエラー"
            # ValueError を送出する
            raise ValueError(f"復号失敗 (key={key_name}): {error_msg}")
        # plaintext を Base64 デコードして返す
        plaintext = response.get_plaintext()
        # 復号成功をログに記録する（復号結果はログに出力しない）
        logger.debug("復号成功: key=%s plaintext_length=%d", key_name, len(plaintext))
        # 復号されたバイト列を返す
        return plaintext

    def rotate_key(self, key_name: str) -> bool:
        """指定した Transit キーをローテーションして成否を返す。

        Args:
            key_name: ローテーションするキー名

        Returns:
            ローテーションが成功すれば True
        """
        # API エンドポイント URL を構築する
        url = self._build_url(f"keys/{key_name}/rotate")
        # POST リクエストでキーローテーションを実行する
        response = self._make_request("POST", url, {})
        # リクエストの成否を確認する
        if response.is_success:
            # ローテーション成功をログに記録する
            logger.info("キーローテーション成功: key=%s", key_name)
            # 成功を返す
            return True
        else:
            # ローテーション失敗をログに記録する
            logger.error(
                "キーローテーション失敗: key=%s errors=%s",
                key_name,
                response.errors,
            )
            # 失敗を返す
            return False

    def trim_key(self, key_name: str, min_version: int) -> bool:
        """Transit キーの古いバージョンを刈り取って成否を返す。

        古いバージョンのキーを削除することでキーストアのサイズを削減する。

        Args:
            key_name: トリム対象のキー名
            min_version: 保持する最小バージョン番号

        Returns:
            トリムが成功すれば True
        """
        # min_version が 1 以上であることを確認する
        if min_version < 1:
            # 無効なバージョン指定の場合はエラーをログに記録する
            logger.error("trim_key: min_version は 1 以上でなければなりません: %d", min_version)
            # False を返す
            return False
        # API エンドポイント URL を構築する
        url = self._build_url(f"keys/{key_name}/trim")
        # ペイロードを構築する
        payload = {"min_available_version": min_version}
        # POST リクエストでキートリムを実行する
        response = self._make_request("POST", url, payload)
        # リクエストの成否を確認する
        if response.is_success:
            # トリム成功をログに記録する
            logger.info("キートリム成功: key=%s min_version=%d", key_name, min_version)
            # 成功を返す
            return True
        else:
            # トリム失敗をログに記録する
            logger.error(
                "キートリム失敗: key=%s min_version=%d errors=%s",
                key_name,
                min_version,
                response.errors,
            )
            # 失敗を返す
            return False

    def rewrap(self, ciphertext: str, key_name: str, context: Optional[bytes] = None) -> str:
        """古いキーバージョンで暗号化された ciphertext を最新バージョンで再ラップして返す。

        Args:
            ciphertext: 再ラップ対象の "vault:v1:..." 形式の暗号文
            key_name: 使用するキー名
            context: Derived Key のコンテキスト（元の暗号化時と同一値が必要）

        Returns:
            最新キーバージョンで再ラップされた ciphertext 文字列

        Raises:
            ValueError: 再ラップが失敗した場合
        """
        # 再ラップリクエストを生成する
        request = RewrapRequest(ciphertext=ciphertext, key_name=key_name, context=context)
        # API エンドポイント URL を構築する
        url = self._build_url(f"rewrap/{key_name}")
        # ペイロードを生成する
        payload = request.to_api_payload()
        # OpenBao API にリクエストを送信する
        response = self._make_request("POST", url, payload)
        # リクエストが失敗した場合はエラーを送出する
        if not response.is_success:
            # エラーメッセージを結合してエラーを送出する
            error_msg = "; ".join(response.errors) or "不明なエラー"
            # ValueError を送出する
            raise ValueError(f"再ラップ失敗 (key={key_name}): {error_msg}")
        # 新しい ciphertext を取得する
        new_ciphertext = response.get_ciphertext()
        # 新しい ciphertext が空の場合はエラーを送出する
        if not new_ciphertext:
            # 空の ciphertext の場合はエラーを送出する
            raise ValueError(f"再ラップレスポンスに ciphertext が含まれていません (key={key_name})")
        # 再ラップ成功をログに記録する
        logger.debug("再ラップ成功: key=%s new_prefix=%s", key_name, new_ciphertext[:20])
        # 新しい ciphertext を返す
        return new_ciphertext

    def get_key_info(self, key_name: str) -> Optional[TransitKey]:
        """指定した Transit キーの情報を取得して TransitKey を返す。

        Args:
            key_name: 情報を取得するキー名

        Returns:
            キー情報を含む TransitKey インスタンス、失敗時は None
        """
        # API エンドポイント URL を構築する
        url = self._build_url(f"keys/{key_name}")
        # GET リクエストでキー情報を取得する
        response = self._make_request("GET", url)
        # リクエストが失敗した場合は None を返す
        if not response.is_success:
            # キー情報取得失敗をログに記録する
            logger.warning("キー情報取得失敗: key=%s errors=%s", key_name, response.errors)
            # None を返す
            return None
        # レスポンスデータからキー情報を抽出する
        data = response.data
        # TransitKey を生成して返す
        return TransitKey(
            key_name=key_name,
            key_type=data.get("type", "aes256-gcm96"),
            rotation_period=data.get("auto_rotate_period", "0s"),
            min_decryption_version=int(data.get("min_decryption_version", 1)),
            latest_version=int(data.get("latest_version", 1)),
            deletion_allowed=bool(data.get("deletion_allowed", False)),
            exportable=bool(data.get("exportable", False)),
            convergent_encryption=bool(data.get("convergent_encryption", False)),
        )

    def batch_encrypt(
        self, plaintexts: List[bytes], key_name: str
    ) -> List[str]:
        """複数のプレーンテキストをバッチ暗号化して ciphertext のリストを返す。

        Args:
            plaintexts: 暗号化対象のバイト列のリスト
            key_name: 使用するキー名

        Returns:
            ciphertext 文字列のリスト（順序は入力と同一）

        Raises:
            ValueError: バッチ暗号化が失敗した場合
        """
        # バッチ暗号化の入力リストを構築する
        batch_input = [
            {"plaintext": base64.b64encode(pt).decode("ascii")}
            for pt in plaintexts
        ]
        # ペイロードを構築する
        payload = {"batch_input": batch_input}
        # API エンドポイント URL を構築する
        url = self._build_url(f"encrypt/{key_name}")
        # OpenBao API にリクエストを送信する
        response = self._make_request("POST", url, payload)
        # リクエストが失敗した場合はエラーを送出する
        if not response.is_success:
            # エラーメッセージを結合してエラーを送出する
            error_msg = "; ".join(response.errors) or "不明なエラー"
            # ValueError を送出する
            raise ValueError(f"バッチ暗号化失敗 (key={key_name}): {error_msg}")
        # バッチ結果を取得する
        batch_results = response.data.get("batch_results", [])
        # ciphertext のリストを抽出して返す
        ciphertexts: List[str] = []
        # バッチ結果をイテレートする
        for idx, result in enumerate(batch_results):
            # result が辞書でない場合はエラーとする
            if not isinstance(result, dict):
                # 不正な形式の場合はエラーを送出する
                raise ValueError(f"バッチ暗号化: インデックス {idx} の結果が不正です")
            # error フィールドが存在する場合はエラーとする
            if "error" in result:
                # バッチ内の個別エラーを送出する
                raise ValueError(
                    f"バッチ暗号化: インデックス {idx} でエラー: {result['error']}"
                )
            # ciphertext を取得して追加する
            ciphertexts.append(result.get("ciphertext", ""))
        # バッチ暗号化成功をログに記録する
        logger.debug(
            "バッチ暗号化成功: key=%s count=%d", key_name, len(ciphertexts)
        )
        # ciphertext のリストを返す
        return ciphertexts

    def generate_data_key(
        self, key_name: str, key_type: str = "plaintext", bits: int = 256
    ) -> Dict[str, str]:
        """Transit を使ってデータ暗号化キー（DEK）を生成して返す。

        Args:
            key_name: DEK の生成に使用する Transit キー名
            key_type: "plaintext"（平文を含む）または "wrapped"（ラップのみ）
            bits: DEK のビット長（128 または 256）

        Returns:
            "plaintext" (平文 DEK, Base64) と "ciphertext" (ラップ済み DEK) の辞書

        Raises:
            ValueError: DEK 生成が失敗した場合
        """
        # bits が有効な値（128 または 256）であることを確認する
        if bits not in (128, 256):
            # 無効なビット長の場合はエラーを送出する
            raise ValueError(f"bits は 128 または 256 でなければなりません: {bits}")
        # API エンドポイント URL を構築する（datakey サブパス）
        url = self._build_url(f"datakey/{key_type}/{key_name}")
        # ペイロードを構築する
        payload = {"bits": bits}
        # OpenBao API にリクエストを送信する
        response = self._make_request("POST", url, payload)
        # リクエストが失敗した場合はエラーを送出する
        if not response.is_success:
            # エラーメッセージを結合してエラーを送出する
            error_msg = "; ".join(response.errors) or "不明なエラー"
            # ValueError を送出する
            raise ValueError(f"DEK 生成失敗 (key={key_name}): {error_msg}")
        # レスポンスデータを返す（plaintext と ciphertext を含む）
        result = {
            "ciphertext": response.data.get("ciphertext", ""),
            "plaintext": response.data.get("plaintext", ""),
            "key_version": str(response.data.get("key_version", 1)),
        }
        # DEK 生成成功をログに記録する（平文 DEK はログに出力しない）
        logger.info("DEK 生成成功: key=%s bits=%d", key_name, bits)
        # 結果辞書を返す
        return result
