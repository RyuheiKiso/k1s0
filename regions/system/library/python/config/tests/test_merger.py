"""deep_merge ユーティリティのユニットテスト"""

from k1s0_config.merger import deep_merge


def test_merge_simple_override() -> None:
    """シンプルなキーの上書き。"""
    base = {"a": 1, "b": 2}
    override = {"b": 99, "c": 3}
    result = deep_merge(base, override)
    assert result == {"a": 1, "b": 99, "c": 3}


def test_merge_nested_dict() -> None:
    """ネストされた辞書のマージ。"""
    base = {"server": {"host": "localhost", "port": 8080}}
    override = {"server": {"port": 9090}}
    result = deep_merge(base, override)
    assert result["server"]["host"] == "localhost"
    assert result["server"]["port"] == 9090


def test_merge_list_replacement() -> None:
    """リストは置換されること。"""
    base = {"topics": ["a", "b"]}
    override = {"topics": ["c"]}
    result = deep_merge(base, override)
    assert result["topics"] == ["c"]


def test_merge_does_not_mutate_base() -> None:
    """base が変更されないこと。"""
    base = {"a": {"b": 1}}
    override = {"a": {"c": 2}}
    deep_merge(base, override)
    assert "c" not in base["a"]


def test_merge_empty_override() -> None:
    """空の override でも正常動作すること。"""
    base = {"a": 1}
    result = deep_merge(base, {})
    assert result == {"a": 1}
