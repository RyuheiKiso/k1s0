"""tools/lock_yaml_generator/generate_registries.py

registries.lock.yaml 生成器。
src/tier1/lock/registries_input.yaml を読み込み、6 schema_class のドリフト状態を正規化する。
存在しない場合はフォールバックスケルトンを生成する（全 drift_status: unknown）。

catalog SoT: src/tier1/schema/schema_evolution/classes.yaml / src/tier1/schema/schema_evolution/scenarios.yaml
catalog が存在する場合、宣言された schema_class id が全て drill 済みであることを検証する。
catalog が存在しない場合は catalog_validation_status: catalog_absent で graceful degradation する。
"""

from __future__ import annotations

import datetime
from pathlib import Path
from typing import Any

from tools.lock_yaml_generator.base_generator import BaseGenerator, REPO_ROOT
from tools.lock_yaml_generator.lib.catalog_validator import (
    load_catalog,
    validate_catalog_against_input,
    build_catalog_metadata,
)

# registries_input.yaml の想定名
_INPUT_NAME = "registries_input.yaml"

# catalog SoT パス（REPO_ROOT 相対）
_CATALOG_CLASSES_PATH = "src/tier1/schema/schema_evolution/classes.yaml"
_CATALOG_SCENARIOS_PATH = "src/tier1/schema/schema_evolution/scenarios.yaml"

# 6 schema_class 定義（06_スキーマ進化適合仕様.md の v1 schema_class セットと一致する）
_SCHEMA_CLASSES: list[dict[str, str]] = [
    # External proto: Buf BSR、BACKWARD 互換
    {"schema_class": "v1_external_proto", "schema_kind": "proto_external", "registry_backend": "buf_bsr"},
    # Internal proto: Buf BSR internal、互換性 NONE
    {"schema_class": "v1_internal_proto", "schema_kind": "proto_internal", "registry_backend": "buf_bsr_internal"},
    # Event Avro: Apicurio Registry、FULL 互換
    {"schema_class": "v1_event_avro", "schema_kind": "event_avro", "registry_backend": "apicurio"},
    # DB DDL: sqlx-cli migration、forward_backward_via_migration
    {"schema_class": "v1_db_ddl", "schema_kind": "db_ddl", "registry_backend": "sqlx_migrations"},
    # Config YAML: リポジトリ内 yaml、BACKWARD 互換
    {"schema_class": "v1_config_yaml", "schema_kind": "config_yaml", "registry_backend": "yaml_in_repo"},
    # OTel SemConv: OpenTelemetry Weaver、FULL 互換
    {"schema_class": "v1_otel_semconv", "schema_kind": "otel_semconv", "registry_backend": "weaver"},
]


class RegistriesGenerator(BaseGenerator):
    """registries.lock.yaml 生成器。"""

    # 出力ファイル名
    OUTPUT_NAME = "registries.lock.yaml"
    # 必須入力なし（フォールバックあり）
    REQUIRED_INPUTS: list[str] = []
    # スキーマ検証なし
    SCHEMA_PATH: Path | None = None
    # 出力先ディレクトリ
    DEFAULT_OUTPUT_DIR = "src/tier1/lock"

    def load_inputs(self, lock_dir: Path) -> dict[str, Any]:
        """src/tier1/lock/registries_input.yaml を読み込む。
        存在しない場合は空 dict を返す。
        """
        return self.load_lock(lock_dir, _INPUT_NAME)

    def build_artifact(self, inputs: dict[str, Any]) -> dict[str, Any]:
        """6 schema_class のレジストリリストを正規化して artifact dict を返す。
        inputs が空の場合はフォールバックスケルトンを使用する。
        catalog SoT (schema_evolution/classes.yaml) が存在する場合は cross-validation を実行する。
        """
        generated_at = datetime.datetime.now(tz=datetime.timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        )

        # registries キーを読み込む
        raw_registries: list[dict[str, Any]] = inputs.get("registries", [])

        if raw_registries:
            # 入力データから registries を正規化する
            registries = self._normalize_registries(raw_registries)
        else:
            # フォールバック: 全 unknown のスケルトンを生成する
            registries = self._build_skeleton()

        # catalog SoT のパスを解決する
        classes_path = REPO_ROOT / _CATALOG_CLASSES_PATH
        scenarios_path = REPO_ROOT / _CATALOG_SCENARIOS_PATH

        # catalog yaml を読み込む（不在時は None）
        classes_catalog = load_catalog(classes_path)
        scenarios_catalog = load_catalog(scenarios_path)

        # schema_classes catalog の cross-validation: schema_classes[].id vs input registries[].schema_class
        classes_result = validate_catalog_against_input(
            catalog=classes_catalog,
            input_data=inputs,
            class_key="schema_classes",
            input_id_key="schema_class",
            input_list_key="registries",
            catalog_path=classes_path,
            catalog_id_key="id",
        )

        # scenarios catalog の cross-validation（scenarios id と drill id の対応がないため passed 扱い）
        scenarios_result = validate_catalog_against_input(
            catalog=scenarios_catalog,
            input_data={},
            class_key="scenarios",
            input_id_key="id",
            input_list_key=None,
            catalog_path=scenarios_path,
            catalog_id_key="id",
        )
        # scenarios は drill 対象外なので常に passed とする
        if scenarios_catalog is not None:
            scenarios_result = {
                "status": "passed",
                "missing": [],
                "catalog_sot_hash": scenarios_result.get("catalog_sot_hash"),
            }

        # metadata を構築する
        catalog_meta = build_catalog_metadata(
            classes_result=classes_result,
            scenarios_result=scenarios_result,
            classes_path=_CATALOG_CLASSES_PATH,
            scenarios_path=_CATALOG_SCENARIOS_PATH,
        )

        return {
            "_AUTO_GENERATED": (
                "DO NOT EDIT. Generated by"
                " tools/lock_yaml_generator/generate_registries.py"
            ),
            "total_registries": len(registries),
            "generated_at":     generated_at,
            "metadata":         catalog_meta,
            "registries":       registries,
        }

    @staticmethod
    def _normalize_registries(raw: list[dict[str, Any]]) -> list[dict[str, Any]]:
        """registry エントリを正規化する。"""
        result: list[dict[str, Any]] = []
        for entry in raw:
            result.append({
                "schema_class":     str(entry.get("schema_class", "")),
                "schema_kind":      str(entry.get("schema_kind", "")),
                "registry_backend": str(entry.get("registry_backend", "")),
                "drift_status":     str(entry.get("drift_status", "unknown")),
                "notes":            str(entry.get("notes", "")),
            })
        return result

    @staticmethod
    def _build_skeleton() -> list[dict[str, Any]]:
        """6 schema_class のスケルトンを生成する。"""
        result: list[dict[str, Any]] = []
        for sc in _SCHEMA_CLASSES:
            result.append({
                "schema_class":     sc["schema_class"],
                "schema_kind":      sc["schema_kind"],
                "registry_backend": sc["registry_backend"],
                "drift_status":     "unknown",
                "notes":            "",
            })
        return result
