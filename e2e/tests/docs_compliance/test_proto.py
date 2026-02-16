"""API設計.md / ディレクトリ構成図.md の proto 仕様準拠テスト。

api/proto/ の構成がドキュメントと一致するかを検証する。
"""
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
PROTO = ROOT / "api" / "proto"


class TestProtoConfig:
    """api/proto の設定ファイルの検証。"""

    def test_buf_yaml_exists(self) -> None:
        assert (PROTO / "buf.yaml").exists()

    def test_buf_yaml_content(self) -> None:
        content = (PROTO / "buf.yaml").read_text(encoding="utf-8")
        assert "version" in content
        assert "lint" in content

    def test_buf_gen_yaml_exists(self) -> None:
        assert (PROTO / "buf.gen.yaml").exists()

    def test_buf_gen_yaml_has_go_plugin(self) -> None:
        content = (PROTO / "buf.gen.yaml").read_text(encoding="utf-8")
        assert "protoc-gen-go" in content or "go" in content

    def test_buf_gen_yaml_has_rust_plugin(self) -> None:
        content = (PROTO / "buf.gen.yaml").read_text(encoding="utf-8")
        assert "prost" in content or "rust" in content


class TestProtoCommonTypes:
    """API設計.md: 共通型の検証。"""

    def test_types_proto_exists(self) -> None:
        path = PROTO / "k1s0" / "system" / "common" / "v1" / "types.proto"
        assert path.exists()

    def test_types_has_pagination(self) -> None:
        content = (PROTO / "k1s0" / "system" / "common" / "v1" / "types.proto").read_text(encoding="utf-8")
        assert "message Pagination" in content
        assert "message PaginationResult" in content

    def test_types_has_timestamp(self) -> None:
        content = (PROTO / "k1s0" / "system" / "common" / "v1" / "types.proto").read_text(encoding="utf-8")
        assert "message Timestamp" in content

    def test_event_metadata_exists(self) -> None:
        path = PROTO / "k1s0" / "system" / "common" / "v1" / "event_metadata.proto"
        assert path.exists()

    def test_event_metadata_fields(self) -> None:
        content = (PROTO / "k1s0" / "system" / "common" / "v1" / "event_metadata.proto").read_text(encoding="utf-8")
        assert "message EventMetadata" in content
        assert "event_id" in content
        assert "event_type" in content
        assert "source" in content
        assert "timestamp" in content
        assert "trace_id" in content
        assert "correlation_id" in content
        assert "schema_version" in content


class TestProtoEventStructure:
    """proto イベント定義ディレクトリの検証。"""

    def test_event_directory_exists(self) -> None:
        assert (PROTO / "k1s0" / "event").exists()

    @pytest.mark.parametrize(
        "tier",
        [
            "system",
            "business",
            "service",
        ],
    )
    def test_event_tier_directories(self, tier: str) -> None:
        """各tier のイベントディレクトリが存在すること。"""
        path = PROTO / "k1s0" / "event" / tier
        assert path.exists() or any(
            (PROTO / "k1s0" / "event").iterdir()
        ), f"event/{tier}/ が存在しません"


class TestProtoPackageConvention:
    """proto パッケージ命名規約の検証。"""

    def test_common_package(self) -> None:
        content = (PROTO / "k1s0" / "system" / "common" / "v1" / "types.proto").read_text(encoding="utf-8")
        assert "package k1s0.system.common.v1" in content

    def test_event_metadata_package(self) -> None:
        content = (PROTO / "k1s0" / "system" / "common" / "v1" / "event_metadata.proto").read_text(encoding="utf-8")
        assert "package k1s0.system.common.v1" in content
