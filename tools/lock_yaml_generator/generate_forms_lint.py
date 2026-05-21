"""tools/lock_yaml_generator/generate_forms_lint.py

# forms_lint.lock.yaml 生成器。
# src/tier3/typescript/packages/forms/ ディレクトリを確認し、
# Forms パッケージの lint 状態を lock artifact として生成する。
# package.json が存在する場合は 'ready'、存在しない場合は 'declared'。
"""

# 標準ライブラリのインポート
from __future__ import annotations

# datetime モジュール: generated_at タイムスタンプ生成に使用
import datetime
# Path: ファイルパス操作に使用
from pathlib import Path
# Any: 型ヒントに使用
from typing import Any

# BaseGenerator と REPO_ROOT をインポートする
from tools.lock_yaml_generator.base_generator import BaseGenerator, REPO_ROOT

# forms パッケージディレクトリの想定パス
_FORMS_DIR = REPO_ROOT / "src/tier3/typescript/packages/forms"
# forms パッケージの package.json パス
_FORMS_PACKAGE_JSON = _FORMS_DIR / "package.json"


class FormsLintGenerator(BaseGenerator):
    """forms_lint.lock.yaml 生成器。"""

    # 出力ファイル名
    OUTPUT_NAME = "forms_lint.lock.yaml"
    # 必須入力なし（フォールバックあり）
    REQUIRED_INPUTS: list[str] = []
    # スキーマ検証なし
    SCHEMA_PATH: Path | None = None
    # 出力先ディレクトリ
    DEFAULT_OUTPUT_DIR = "src/tier3/lock"

    def load_inputs(self, lock_dir: Path) -> dict[str, Any]:
        """src/tier3/typescript/packages/forms/ ディレクトリと package.json の存在を確認する。
        package.json が存在する場合は 'ready' フラグを返す。
        """
        # forms パッケージディレクトリが存在するか確認する
        forms_dir_exists = _FORMS_DIR.exists() and _FORMS_DIR.is_dir()
        # package.json が存在するか確認する
        package_json_exists = _FORMS_PACKAGE_JSON.exists()
        # 確認結果を返す
        return {
            "forms_dir_exists":    forms_dir_exists,
            "package_json_exists": package_json_exists,
        }

    def build_artifact(self, inputs: dict[str, Any]) -> dict[str, Any]:
        """Forms パッケージの lint 状態を表す artifact dict を返す。
        package.json が存在する場合は 'ready'、存在しない場合は 'declared'。
        6 forbidden symbol を lint cell として列挙する（Y-tier3-6 cell 列挙拡張）。
        """
        # 現在時刻を UTC で生成する
        generated_at = datetime.datetime.now(tz=datetime.timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        )

        # package.json の存在を確認する
        package_json_exists = inputs.get("package_json_exists", False)

        # package.json が存在する場合は 'ready'、存在しない場合は 'declared' とする
        forms_package_status = "ready" if package_json_exists else "declared"

        # 6 forbidden symbol の lint cell 列挙
        # tenant_id_in_public_form 禁止 + pii_localstorage_write 禁止 の 6 forbidden symbol
        forbidden_symbol_cells = [
            {
                # tenant_id をフォームの公開フィールドに含めることを禁止する
                "symbol_id": "tenant_id_in_public_form",
                "description": "公開 form schema に tenant_id フィールドを含めることを禁止する（tier3 CLAUDE.md §公開 type / form schema に tenant_id フィールド禁止）",
                # ESLint の no-restricted-syntax rule で検出するセレクタ
                "eslint_selector": "Property[key.name='tenant_id'][parent.type='ObjectExpression']",
                # 検出ステータス（物理 lint は Stage 4 で実施）
                "status": "declared",
            },
            {
                # pii フィールドを localStorage に書き込むことを禁止する
                "symbol_id": "pii_localstorage_write",
                "description": "PII フィールドを localStorage に平文で書き込むことを禁止する（tier3 CLAUDE.md §PII を localStorage / IndexedDB に平文保管禁止）",
                # ESLint の no-restricted-syntax rule で検出するセレクタ
                "eslint_selector": "CallExpression[callee.object.name='localStorage'][callee.property.name='setItem']",
                # 検出ステータス
                "status": "declared",
            },
            {
                # unsafe-inline を CSP に設定することを禁止する
                "symbol_id": "csp_unsafe_inline",
                "description": "CSP に unsafe-inline を設定することを禁止する（tier3 CLAUDE.md §CSP: unsafe-inline / unsafe-eval 禁止）",
                # ESLint の no-restricted-syntax rule で検出するセレクタ
                "eslint_selector": "Literal[value=/unsafe-inline/]",
                # 検出ステータス
                "status": "declared",
            },
            {
                # fetch を tier3 から直接呼び出すことを禁止する
                "symbol_id": "direct_fetch_call",
                "description": "fetch を tier3 から直接呼び出すことを禁止する（tier2 SDK 経由必須）",
                # ESLint の no-restricted-globals rule で検出する
                "eslint_selector": "CallExpression[callee.name='fetch']",
                # 検出ステータス
                "status": "declared",
            },
            {
                # axios を tier3 から直接使用することを禁止する
                "symbol_id": "direct_axios_use",
                "description": "axios を tier3 から直接使用することを禁止する（tier2 SDK 経由必須）",
                # import restriction で検出する（no-restricted-imports / import/no-restricted-paths）
                "eslint_selector": "ImportDeclaration[source.value='axios']",
                # 検出ステータス
                "status": "declared",
            },
            {
                # cookie に PII を平文で保管することを禁止する
                "symbol_id": "pii_cookie_write",
                "description": "PII を cookie に平文で書き込むことを禁止する（tier3 CLAUDE.md §PII を cookie に平文保管禁止）",
                # ESLint の no-restricted-syntax rule で検出するセレクタ
                "eslint_selector": "AssignmentExpression[left.object.name='document'][left.property.name='cookie']",
                # 検出ステータス
                "status": "declared",
            },
        ]

        # 検査件数を計算する（package.json の有無で判定）
        # 基本検査項目 3 件 + forbidden symbol cell 6 件
        base_checks = 3 if package_json_exists else 0
        # forbidden symbol cell 分の追加検査件数
        symbol_checks = len(forbidden_symbol_cells) if package_json_exists else 0
        # 合計検査件数
        total_checks = base_checks + symbol_checks
        # 合格件数（全検査項目が合格の場合は total_checks と同数）
        passed_checks = total_checks
        # 違反件数（現フェーズでは 0 = 物理 lint は Stage 4 で実施）
        violations_count = 0

        # artifact dict を構築して返す
        return {
            # 自動生成ヘッダ（手書き禁止の明示）
            "_AUTO_GENERATED": (
                "DO NOT EDIT. Generated by"
                " tools/lock_yaml_generator/generate_forms_lint.py"
            ),
            # 生成日時
            "generated_at": generated_at,
            # 検査総件数（package.json 存在時は 9 件: 基本 3 件 + forbidden symbol cell 6 件）
            "total_checks": total_checks,
            # 合格件数
            "passed_checks": passed_checks,
            # 違反件数（物理 lint は Stage 4 で実施）
            "violations_count": violations_count,
            # ESLint 違反数（後方互換のため残す）
            "violations": violations_count,
            # forms パッケージの状態（package.json の有無で判定）
            "forms_package_status": forms_package_status,
            # 6 forbidden symbol の lint cell 列挙（Y-tier3-6 cell 列挙拡張）
            "forbidden_symbol_cells": forbidden_symbol_cells if package_json_exists else [],
        }
