"""tools/lock_yaml_generator/generate_fsm.py

# fsm.lock.yaml 生成器。
# src/tier2/fsm/fsm_spec.yaml を読み込み、
# FSM 定義（状態遷移パターン 13）の宣言状態を lock artifact として生成する。
# 4 言語 typestate ファイルの物理存在チェックを含む。
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

# fsm_spec.yaml の想定パス（SoT）
_FSM_SPEC_YAML = REPO_ROOT / "src/tier2/fsm/fsm_spec.yaml"

# 4 言語 typestate ファイルのパス定義（存在チェック対象）
_TYPESTATE_FILES: dict[str, str] = {
    # Rust 言語の typestate ファイル
    "rust": "src/tier2/fsm/rust_phantom.rs",
    # Go 言語の typestate ファイル
    "go": "src/tier2/fsm/go_typestate.go",
    # C# 言語の typestate ファイル
    "csharp": "src/tier2/fsm/csharp_sealed_record.cs",
    # TypeScript 言語の typestate ファイル
    "typescript": "src/tier2/fsm/typescript_branded.ts",
}


class FsmGenerator(BaseGenerator):
    """fsm.lock.yaml 生成器。"""

    # 出力ファイル名
    OUTPUT_NAME = "fsm.lock.yaml"
    # 必須入力なし（SoT は別パスから直読み）
    REQUIRED_INPUTS: list[str] = []
    # スキーマ検証なし
    SCHEMA_PATH: Path | None = None
    # 出力先ディレクトリ（tier2/lock）
    DEFAULT_OUTPUT_DIR = "src/tier2/lock"

    def load_inputs(self, lock_dir: Path) -> dict[str, Any]:
        """fsm_spec.yaml を読み込む。
        存在しない場合は空 dict を返す。
        """
        # fsm_spec.yaml が存在する場合のみ読み込む
        if _FSM_SPEC_YAML.exists():
            # PyYAML をインポートする
            import yaml  # type: ignore
            # テキストを読み込んで YAML パースする
            raw = yaml.safe_load(_FSM_SPEC_YAML.read_text(encoding="utf-8"))
            # dict であれば返す、そうでなければ空 dict を返す
            return raw if isinstance(raw, dict) else {}
        # ファイルが存在しない場合は空 dict を返す
        return {}

    def build_artifact(self, inputs: dict[str, Any]) -> dict[str, Any]:
        """FSM 定義の宣言状態を表す artifact dict を返す。
        inputs が空の場合は state_machines を空にして生成する。
        4 言語 typestate ファイルの物理存在チェックを行う。
        """
        # 現在時刻を UTC で生成する
        generated_at = datetime.datetime.now(tz=datetime.timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        )

        # metadata から spec_reference を取り出す
        metadata: dict[str, Any] = inputs.get("metadata", {})
        # spec_reference を取り出す（存在しない場合はデフォルト値を使う）
        spec_ref: str = str(
            metadata.get(
                "spec_reference",
                "docs/04_詳細設計/01_適合仕様/13_状態遷移パターン適合仕様.md",
            )
        )

        # 4 言語 typestate ファイルの物理存在を確認する
        typestate_files_all_exist = all(
            # 各 typestate ファイルの物理存在をチェックする
            (REPO_ROOT / path).exists()
            for path in _TYPESTATE_FILES.values()
        )

        # fsm_spec.yaml の state_machines リストを取り出す
        raw_state_machines: list[dict[str, Any]] = inputs.get("state_machines", [])

        # 各 state_machine エントリを整形する
        state_machines: list[dict[str, Any]] = []

        # raw_state_machines の各エントリを処理する
        for raw_sm in raw_state_machines:
            # FSM の id を取り出す
            sm_id = str(raw_sm.get("id", ""))
            # 初期状態を取り出す
            initial_state = str(raw_sm.get("initial_state", ""))

            # states リストを取り出す
            raw_states: list[dict[str, Any]] = raw_sm.get("states", [])
            # state id のリストを構築する
            states: list[str] = [str(s.get("id", "")) for s in raw_states]
            # 状態総数
            total_states = len(states)

            # transitions リストを取り出す
            raw_transitions: list[dict[str, Any]] = raw_sm.get("transitions", [])
            # 整形済み transition エントリのリストを構築する
            transitions: list[dict[str, str]] = []
            # raw_transitions の各エントリを処理する
            for raw_tr in raw_transitions:
                # 遷移エントリを整形する（from / to / event のみ抽出）
                transitions.append(
                    {
                        # 遷移元状態
                        "from": str(raw_tr.get("from", "")),
                        # 遷移先状態
                        "to": str(raw_tr.get("to", "")),
                        # 遷移イベント
                        "event": str(raw_tr.get("event", "")),
                    }
                )
            # 遷移総数
            total_transitions = len(transitions)

            # 整形済み state_machine エントリをリストに追加する
            state_machines.append(
                {
                    # FSM 識別子
                    "id": sm_id,
                    # 初期状態
                    "initial_state": initial_state,
                    # 状態一覧
                    "states": states,
                    # 状態総数
                    "total_states": total_states,
                    # 遷移一覧
                    "transitions": transitions,
                    # 遷移総数
                    "total_transitions": total_transitions,
                }
            )

        # artifact dict を構築して返す
        return {
            # 自動生成ヘッダ（手書き禁止の明示）
            "_AUTO_GENERATED": (
                "DO NOT EDIT. Generated by"
                " tools/lock_yaml_generator/generate_fsm.py"
            ),
            # 参照仕様
            "spec_ref": spec_ref,
            # 生成日時
            "generated_at": generated_at,
            # 4 言語 typestate ファイルのパス定義
            "typestate_files": _TYPESTATE_FILES,
            # 4 言語 typestate ファイルの物理存在フラグ
            "typestate_files_all_exist": typestate_files_all_exist,
            # FSM 定義一覧
            "state_machines": state_machines,
            # FSM 総数
            "total_state_machines": len(state_machines),
        }
