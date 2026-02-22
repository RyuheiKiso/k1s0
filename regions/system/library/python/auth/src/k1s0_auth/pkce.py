"""OAuth2 PKCE フロー実装"""

from __future__ import annotations

import base64
import hashlib
import os


def generate_code_verifier(length: int = 64) -> str:
    """PKCE コードベリファイアを生成する（RFC 7636）。

    Args:
        length: ランダムバイト数（最小 32、最大 96 推奨）

    Returns:
        URL-safe base64 エンコードされたコードベリファイア
    """
    return base64.urlsafe_b64encode(os.urandom(length)).rstrip(b"=").decode("ascii")


def generate_code_challenge(verifier: str) -> str:
    """コードベリファイアから S256 コードチャレンジを生成する。

    Args:
        verifier: generate_code_verifier で生成したコードベリファイア

    Returns:
        SHA-256 ハッシュを URL-safe base64 エンコードしたコードチャレンジ
    """
    digest = hashlib.sha256(verifier.encode("ascii")).digest()
    return base64.urlsafe_b64encode(digest).rstrip(b"=").decode("ascii")
