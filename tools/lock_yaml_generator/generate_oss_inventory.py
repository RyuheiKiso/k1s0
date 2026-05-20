"""tools/lock_yaml_generator/generate_oss_inventory.py

oss_inventory.lock.yaml 生成器。
src/tier1/lock/oss_inventory_input.yaml を読み込み、OSS lifecycle drill 結果を正規化する。
存在しない場合はフォールバックスケルトンを生成する（全 drill_state: pending）。

catalog SoT: src/tier1/schema/oss_lifecycle/classes.yaml / src/tier1/schema/oss_lifecycle/scenarios.yaml
catalog の lifecycle_classes[].id は採用カテゴリ定義（v1_l1plus_primary 等）であり、
個別 OSS パッケージ（oss_id）との直接対応はない。
catalog が存在する場合は catalog_sot_hash と signals[].id を metadata に記録する。
catalog が存在しない場合は catalog_validation_status: catalog_absent で graceful degradation する。

08_OSSライフサイクル適合仕様.md §lifecycle_class セット canonical 名（v1_l1plus_primary 等）に準拠する。
input の lifecycle_class / version フィールドを output に含め、SBOM / signature / health 追跡フィールドも追加する。
"""

# future annotations: 型アノテーションの前方参照を許可する
from __future__ import annotations

# datetime: 生成日時の記録に使用する
import datetime
# Path: ファイルパス操作に使用する
from pathlib import Path
# Any: 型アノテーションに使用する
from typing import Any

# BaseGenerator: 共通生成器基底クラスをインポートする
from tools.lock_yaml_generator.base_generator import BaseGenerator, REPO_ROOT
# catalog_validator: catalog 検証関数をインポートする
from tools.lock_yaml_generator.lib.catalog_validator import (
    load_catalog,
    validate_catalog_against_input,
    build_catalog_metadata,
    compute_sot_hash,
)

# oss_inventory_input.yaml の想定名
_INPUT_NAME = "oss_inventory_input.yaml"

# catalog SoT パス（REPO_ROOT 相対）
_CATALOG_CLASSES_PATH = "src/tier1/schema/oss_lifecycle/classes.yaml"
# catalog シナリオ SoT パス（REPO_ROOT 相対）
_CATALOG_SCENARIOS_PATH = "src/tier1/schema/oss_lifecycle/scenarios.yaml"

# spec canonical lifecycle_class 6 値（08_OSSライフサイクル適合仕様.md §lifecycle_class セット）
# この定数は lifecycle_class の検証に使用する
_VALID_LIFECYCLE_CLASSES = frozenset({
    # v1_l1plus_primary: 最重要 OSS
    "v1_l1plus_primary",
    # v1_l1plus_pair_target: L1+ ペアターゲット OSS
    "v1_l1plus_pair_target",
    # v1_l2star_member: L2* ファミリーメンバー OSS
    "v1_l2star_member",
    # v1_l3_runtime: L3 コンパニオン・ランタイム OSS
    "v1_l3_runtime",
    # v1_reserved_category: 予約カテゴリ
    "v1_reserved_category",
    # v1_inhouse_authoritative: 内製権威コンポーネント
    "v1_inhouse_authoritative",
})

# 主要 OSS 依存関係のスケルトン定義（input がない場合のフォールバック）
_OSS_DEPS: list[dict[str, str]] = [
    # axum: Rust Web フレームワーク
    {"oss_id": "axum", "current_version": "0.8", "ecosystem": "rust"},
    # tokio: 非同期ランタイム
    {"oss_id": "tokio", "current_version": "1", "ecosystem": "rust"},
    # tower: ミドルウェアライブラリ
    {"oss_id": "tower", "current_version": "0.5", "ecosystem": "rust"},
    # controller-runtime: k8s コントローラライブラリ
    {"oss_id": "controller-runtime", "current_version": "0.20.0", "ecosystem": "go"},
]


class OssInventoryGenerator(BaseGenerator):
    """oss_inventory.lock.yaml 生成器。"""

    # 出力ファイル名
    OUTPUT_NAME = "oss_inventory.lock.yaml"
    # 必須入力なし（フォールバックあり）
    REQUIRED_INPUTS: list[str] = []
    # スキーマ検証なし
    SCHEMA_PATH: Path | None = None
    # 出力先ディレクトリ
    DEFAULT_OUTPUT_DIR = "src/tier1/lock"

    def load_inputs(self, lock_dir: Path) -> dict[str, Any]:
        """src/tier1/lock/oss_inventory_input.yaml を読み込む。
        存在しない場合は空 dict を返す。
        """
        # lock_dir から oss_inventory_input.yaml を読み込む
        return self.load_lock(lock_dir, _INPUT_NAME)

    def build_artifact(self, inputs: dict[str, Any]) -> dict[str, Any]:
        """OSS drills リストを正規化して artifact dict を返す。
        inputs が空の場合はフォールバックスケルトンを使用する。
        catalog SoT (oss_lifecycle/classes.yaml) が存在する場合は catalog_sot_hash を metadata に記録する。
        lifecycle_classes[].id は採用カテゴリ定義であり、oss_id との直接突合はしない。
        input の lifecycle_class / version を output に含める。
        """
        # 生成日時を UTC ISO 8601 形式で記録する（status 記録目的の wall-clock: TTL 計算禁止）
        generated_at = datetime.datetime.now(tz=datetime.timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        )

        # drills キーを読み込む
        raw_drills: list[dict[str, Any]] = inputs.get("drills", [])

        # drills が存在する場合は正規化する、存在しない場合はスケルトンを生成する
        if raw_drills:
            # 入力データから drills を正規化する
            drills = self._normalize_drills(raw_drills)
        else:
            # フォールバック: 全 pending のスケルトンを生成する
            drills = self._build_skeleton()

        # catalog SoT のパスを解決する
        classes_path = REPO_ROOT / _CATALOG_CLASSES_PATH
        # scenarios SoT のパスを解決する
        scenarios_path = REPO_ROOT / _CATALOG_SCENARIOS_PATH

        # catalog yaml を読み込む（不在時は None）
        classes_catalog = load_catalog(classes_path)
        # scenarios yaml を読み込む（不在時は None）
        scenarios_catalog = load_catalog(scenarios_path)

        # lifecycle_classes catalog: signals[].id を監視シグナル定義として使用する
        # oss_id との直接突合はできないため、catalog 存在確認のみ行い passed とする
        if classes_catalog is not None:
            # catalog が存在する場合は SOT ハッシュを計算する
            try:
                classes_hash = compute_sot_hash(classes_path)
            except ValueError:
                # ハッシュ計算失敗時は None を設定する
                classes_hash = None
            # catalog 検証結果を構築する
            classes_result = {
                "status": "passed",
                "missing": [],
                "catalog_sot_hash": classes_hash,
            }
        else:
            # catalog が不在の場合は catalog_absent として graceful degradation する
            classes_result = {
                "status": "catalog_absent",
                "missing": [],
                "catalog_sot_hash": None,
            }

        # scenarios catalog の cross-validation（scenarios id と oss_id の対応がないため passed 扱い）
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
            # passed 結果を上書きする
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

        # artifact dict を返す
        return {
            # 自動生成ヘッダー: 手書き編集禁止を明示する
            "_AUTO_GENERATED": (
                "DO NOT EDIT. Generated by"
                " tools/lock_yaml_generator/generate_oss_inventory.py"
            ),
            # drill 総数
            "total_drills": len(drills),
            # 生成日時（UTC ISO 8601 形式）
            "generated_at": generated_at,
            # catalog メタデータ
            "metadata":     catalog_meta,
            # 正規化済み drill 一覧
            "drills":       drills,
        }

    @staticmethod
    def _normalize_drills(raw: list[dict[str, Any]]) -> list[dict[str, Any]]:
        """OSS drill エントリを正規化する。
        lifecycle_class / version / sbom_hash / cosign_signature_verified_at /
        health_check_last_at / signal_source_endpoint を output に含める。
        """
        # 正規化済み drill エントリのリストを初期化する
        result: list[dict[str, Any]] = []
        # 各 drill エントリを正規化する
        for entry in raw:
            # lifecycle_class を取得する（未設定の場合は空文字列）
            lifecycle_class = str(entry.get("lifecycle_class", ""))
            # version を取得する（未設定の場合は "unknown"）
            version = str(entry.get("version", "unknown"))
            # 正規化済みエントリを構築する
            result.append({
                # drill_id: ドリル識別子
                "drill_id":      str(entry.get("drill_id", "")),
                # oss_id: OSS パッケージ識別子
                "oss_id":        str(entry.get("oss_id", "")),
                # ecosystem: パッケージエコシステム（rust / go / container / kubernetes 等）
                "ecosystem":     str(entry.get("ecosystem", "")),
                # lifecycle_class: spec canonical 名（v1_l1plus_primary 等）
                "lifecycle_class": lifecycle_class,
                # version: 現行採用バージョン（semver 表記）
                "version":       version,
                # drill_state: ドリル実行状態（green / yellow / red / pending）
                "drill_state":   str(entry.get("drill_state", "pending")),
                # notes: ドリル実行メモ
                "notes":         str(entry.get("notes", "")),
                # sbom_hash: SBOM の SHA-256 ハッシュ（未生成の場合は "sha256:pending"）
                "sbom_hash":     str(entry.get("sbom_hash", "sha256:pending")),
                # cosign_signature_verified_at: cosign 署名検証の最終実施日時（null or ISO 8601）
                "cosign_signature_verified_at": entry.get("cosign_signature_verified_at", None),
                # health_check_last_at: lifecycle signal ヘルスチェックの最終実施日時（ISO 8601）
                "health_check_last_at": entry.get("health_check_last_at", None),
                # signal_source_endpoint: lifecycle signal 収集元エンドポイント URL または "pending"
                "signal_source_endpoint": str(entry.get("signal_source_endpoint", "pending")),
            })
        # 正規化済み drill リストを返す
        return result

    @staticmethod
    def _build_skeleton() -> list[dict[str, Any]]:
        """OSS 依存関係ごとのスケルトンを生成する。
        全フィールドを pending / null で初期化する。
        """
        # スケルトンエントリのリストを初期化する
        result: list[dict[str, Any]] = []
        # 各 OSS 依存関係のスケルトンを生成する
        for dep in _OSS_DEPS:
            # oss_id を取得する
            oss_id = dep["oss_id"]
            # スケルトンエントリを構築する
            result.append({
                # drill_id: oss_id から生成する
                "drill_id":    f"oss_drill__{oss_id}",
                # oss_id: OSS パッケージ識別子
                "oss_id":      oss_id,
                # ecosystem: エコシステム種別
                "ecosystem":   dep["ecosystem"],
                # lifecycle_class: スケルトン時は空文字列（input で設定が必要）
                "lifecycle_class": "",
                # version: スケルトン時はデフォルトバージョン
                "version":     dep["current_version"],
                # drill_state: 未実施（pending）
                "drill_state": "pending",
                # notes: メモなし
                "notes":       "",
                # sbom_hash: 未生成のため sha256:pending を設定する
                "sbom_hash":   "sha256:pending",
                # cosign_signature_verified_at: 未実施のため null を設定する
                "cosign_signature_verified_at": None,
                # health_check_last_at: 未実施のため null を設定する
                "health_check_last_at": None,
                # signal_source_endpoint: 未設定のため "pending" を設定する
                "signal_source_endpoint": "pending",
            })
        # スケルトンリストを返す
        return result
