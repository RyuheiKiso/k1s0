"""TokenClaims モデルのユニットテスト"""

from k1s0_auth.models import TokenClaims


def test_has_scope_true() -> None:
    """スコープを持つ場合は True を返すこと。"""
    claims = TokenClaims(sub="u", iss="i", aud=["a"], exp=9, iat=0, scope="read write admin")
    assert claims.has_scope("read") is True
    assert claims.has_scope("write") is True


def test_has_scope_false() -> None:
    """スコープを持たない場合は False を返すこと。"""
    claims = TokenClaims(sub="u", iss="i", aud=["a"], exp=9, iat=0, scope="read")
    assert claims.has_scope("write") is False


def test_has_scope_empty() -> None:
    """空スコープの場合は False を返すこと。"""
    claims = TokenClaims(sub="u", iss="i", aud=["a"], exp=9, iat=0)
    assert claims.has_scope("read") is False


def test_has_role_true() -> None:
    """ロールを持つ場合は True を返すこと。"""
    claims = TokenClaims(sub="u", iss="i", aud=["a"], exp=9, iat=0, roles=["admin"])
    assert claims.has_role("admin") is True


def test_has_role_false() -> None:
    """ロールを持たない場合は False を返すこと。"""
    claims = TokenClaims(sub="u", iss="i", aud=["a"], exp=9, iat=0, roles=["viewer"])
    assert claims.has_role("admin") is False
