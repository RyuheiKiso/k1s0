"""PKCE フロー実装のユニットテスト"""

import re

from k1s0_auth.pkce import generate_code_challenge, generate_code_verifier


def test_code_verifier_length() -> None:
    """コードベリファイアが適切な長さであること。"""
    verifier = generate_code_verifier()
    # base64url エンコードなので元の 64 バイトより長くなる（パディング除去後）
    assert len(verifier) > 40


def test_code_verifier_is_url_safe() -> None:
    """コードベリファイアが URL-safe 文字のみで構成されること。"""
    verifier = generate_code_verifier()
    assert re.match(r"^[A-Za-z0-9\-_]+$", verifier), f"Not URL-safe: {verifier}"


def test_code_verifier_unique() -> None:
    """連続呼び出しで一意の値が生成されること。"""
    verifiers = {generate_code_verifier() for _ in range(50)}
    assert len(verifiers) == 50


def test_code_challenge_format() -> None:
    """コードチャレンジが URL-safe base64 形式であること。"""
    verifier = generate_code_verifier()
    challenge = generate_code_challenge(verifier)
    assert re.match(r"^[A-Za-z0-9\-_]+$", challenge)


def test_code_challenge_deterministic() -> None:
    """同じベリファイアから同じチャレンジが生成されること。"""
    verifier = generate_code_verifier()
    challenge1 = generate_code_challenge(verifier)
    challenge2 = generate_code_challenge(verifier)
    assert challenge1 == challenge2


def test_code_challenge_different_for_different_verifiers() -> None:
    """異なるベリファイアから異なるチャレンジが生成されること。"""
    v1 = generate_code_verifier()
    v2 = generate_code_verifier()
    c1 = generate_code_challenge(v1)
    c2 = generate_code_challenge(v2)
    assert c1 != c2
