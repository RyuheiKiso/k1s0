"""tests for base_generator.py — 共通基底クラスと utility 関数の検証。"""
from __future__ import annotations

import hashlib
import sys
from pathlib import Path

import pytest
import yaml

sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent))
from tools.lock_yaml_generator.base_generator import (
    BaseGenerator,
    dump_deterministic,
    get_max_mtime,
    sha256_of_text,
    REPO_ROOT,
)


# ---------------------------------------------------------------------------
# dump_deterministic
# ---------------------------------------------------------------------------

def test_dump_deterministic_returns_str():
    result = dump_deterministic({"a": 1, "b": 2})
    assert isinstance(result, str)


def test_dump_deterministic_unicode_not_escaped():
    result = dump_deterministic({"message": "日本語テキスト"})
    assert "日本語テキスト" in result


def test_dump_deterministic_idempotent():
    data = {"x": [1, 2, 3], "y": {"z": "value"}}
    out1 = dump_deterministic(data)
    out2 = dump_deterministic(data)
    assert hashlib.sha256(out1.encode()).hexdigest() == hashlib.sha256(out2.encode()).hexdigest()


def test_dump_deterministic_no_flow_style():
    result = dump_deterministic({"items": [1, 2, 3]})
    assert "{" not in result  # flow style が使われていないこと


# ---------------------------------------------------------------------------
# get_max_mtime
# ---------------------------------------------------------------------------

def test_get_max_mtime_empty_dir(tmp_path):
    result = get_max_mtime(tmp_path)
    assert result == "1970-01-01T00:00:00Z"


def test_get_max_mtime_with_files(tmp_path):
    (tmp_path / "foo.lock.yaml").write_text("x: 1")
    result = get_max_mtime(tmp_path)
    assert result != "1970-01-01T00:00:00Z"
    assert "T" in result and result.endswith("Z")


def test_get_max_mtime_excludes_release_gate(tmp_path):
    (tmp_path / "release_gate.lock.yaml").write_text("x: 1")
    result = get_max_mtime(tmp_path)
    assert result == "1970-01-01T00:00:00Z"


def test_get_max_mtime_excludes_cosign_trigger(tmp_path):
    (tmp_path / "release_gate.cosign_trigger.lock.yaml").write_text("x: 1")
    result = get_max_mtime(tmp_path)
    assert result == "1970-01-01T00:00:00Z"


# ---------------------------------------------------------------------------
# sha256_of_text
# ---------------------------------------------------------------------------

def test_sha256_of_text_format():
    result = sha256_of_text("hello")
    assert result.startswith("sha256:")
    assert len(result) == 7 + 64


def test_sha256_of_text_deterministic():
    r1 = sha256_of_text("test data")
    r2 = sha256_of_text("test data")
    assert r1 == r2


def test_sha256_of_text_different():
    r1 = sha256_of_text("a")
    r2 = sha256_of_text("b")
    assert r1 != r2


# ---------------------------------------------------------------------------
# BaseGenerator — 具体サブクラスでテスト
# ---------------------------------------------------------------------------

class _SimpleGenerator(BaseGenerator):
    OUTPUT_NAME = "simple.lock.yaml"
    REQUIRED_INPUTS = []
    SCHEMA_PATH = None
    DEFAULT_OUTPUT_DIR = "."

    def load_inputs(self, lock_dir: Path) -> dict:
        return {"source": "test"}

    def build_artifact(self, inputs: dict) -> dict:
        return {
            "_AUTO_GENERATED": "test",
            "key": inputs.get("source", ""),
            "count": 42,
        }


def test_base_generator_emit(tmp_path):
    gen = _SimpleGenerator()
    out = gen.emit(tmp_path)
    assert out.exists()
    data = yaml.safe_load(out.read_text())
    assert data["key"] == "test"
    assert data["count"] == 42


def test_base_generator_emit_deterministic(tmp_path):
    gen = _SimpleGenerator()
    out1 = gen.emit(tmp_path)
    out2 = gen.emit(tmp_path)
    assert hashlib.sha256(out1.read_bytes()).hexdigest() == hashlib.sha256(out2.read_bytes()).hexdigest()


def test_base_generator_load_lock_missing(tmp_path):
    gen = _SimpleGenerator()
    result = gen.load_lock(tmp_path, "nonexistent.lock.yaml")
    assert result == {}


def test_base_generator_load_lock_exists(tmp_path):
    (tmp_path / "input.lock.yaml").write_text(yaml.safe_dump({"value": 99}))
    gen = _SimpleGenerator()
    result = gen.load_lock(tmp_path, "input.lock.yaml")
    assert result["value"] == 99


def test_base_generator_creates_output_dir(tmp_path):
    nested = tmp_path / "deep" / "path"
    gen = _SimpleGenerator()
    gen.emit(nested)
    assert (nested / "simple.lock.yaml").exists()


# ---------------------------------------------------------------------------
# REPO_ROOT sanity check
# ---------------------------------------------------------------------------

def test_repo_root_has_claude_md():
    assert (REPO_ROOT / "CLAUDE.md").exists(), f"CLAUDE.md not found at {REPO_ROOT}"
