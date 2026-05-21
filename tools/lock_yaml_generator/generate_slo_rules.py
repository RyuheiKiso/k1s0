"""tools/lock_yaml_generator/generate_slo_rules.py

slo_rules.lock.yaml 生成器。
spec 07 §SLI 計測式は build artifact に準拠し、SLO monitoring rule を自動生成する。

出力内容:
  - Prometheus Recording Rule（record: job:slo_burn_rate:5m 等）
  - Perses Dashboard の stub エントリ
  - Litmus Chaos scenario の stub エントリ

入力 SoT: src/tier1/schema/slo/classes.yaml / src/tier1/schema/slo/scenarios.yaml
入力データ: src/tier1/lock/instruments.lock.yaml（drill 状態の参照用）
出力先: src/tier1/lock/slo_rules.lock.yaml
"""

# future annotations: 型アノテーションの前方参照を許可する
from __future__ import annotations

# datetime: 生成日時の記録に使用する
import datetime
# Path: ファイルパス操作に使用する
from pathlib import Path
# Any: 型アノテーションに使用する
from typing import Any

# yaml: YAML 読み書きに使用する
try:
    import yaml
except ImportError:
    raise ImportError("PyYAML required: pip install PyYAML")

# BaseGenerator: 共通生成器基底クラスをインポートする
from tools.lock_yaml_generator.base_generator import BaseGenerator, REPO_ROOT

# SLO classes catalog の想定パス（REPO_ROOT 相対）
_CATALOG_CLASSES_PATH = "src/tier1/schema/slo/classes.yaml"
# SLO scenarios catalog の想定パス（REPO_ROOT 相対）
_CATALOG_SCENARIOS_PATH = "src/tier1/schema/slo/scenarios.yaml"
# instruments.lock.yaml の想定パス（REPO_ROOT 相対）
_INSTRUMENTS_LOCK_PATH = "src/tier1/lock/instruments.lock.yaml"

# 6 slo_class セット（07_SLO適合仕様.md に準拠する）
_SLO_CLASSES: list[str] = [
    # HTTP/gRPC API 可用性（SLO 99.9%、monthly budget 43.2 分）
    "v1_request_availability",
    # request 経路 p99 latency（class 別 threshold_ms、30d window）
    "v1_request_latency_p99",
    # cross-region write p99 latency（典型 1500ms、v1_cross_region_replicated 経路必須）
    "v1_request_latency_p99_cross_region",
    # server-driven event 配信 freshness（p95 ≤ target、7d window）
    "v1_event_freshness",
    # Temporal Workflow / Saga 完了率（99.5%、30d window）
    "v1_workflow_completion",
    # tenant data 永続性（11 nines + freeze_on_any_loss、365d window）
    "v1_data_durability",
]

# Prometheus recording rule の burn_rate window ペア設定
# 07_SLO適合仕様.md §multi_window_burn_rate に準拠する
_BURN_RATE_WINDOWS = {
    # v1_request_availability: fast 2h + slow 24h
    "v1_request_availability": [
        {"window": "5m",  "record_suffix": "5m"},
        {"window": "30m", "record_suffix": "30m"},
        {"window": "1h",  "record_suffix": "1h"},
        {"window": "2h",  "record_suffix": "2h"},
        {"window": "6h",  "record_suffix": "6h"},
        {"window": "24h", "record_suffix": "24h"},
    ],
    # v1_request_latency_p99: fast 1h + slow 6h
    "v1_request_latency_p99": [
        {"window": "5m",  "record_suffix": "5m"},
        {"window": "30m", "record_suffix": "30m"},
        {"window": "1h",  "record_suffix": "1h"},
        {"window": "6h",  "record_suffix": "6h"},
    ],
    # v1_request_latency_p99_cross_region: fast 1h + slow 6h
    "v1_request_latency_p99_cross_region": [
        {"window": "5m",  "record_suffix": "5m"},
        {"window": "30m", "record_suffix": "30m"},
        {"window": "1h",  "record_suffix": "1h"},
        {"window": "6h",  "record_suffix": "6h"},
    ],
    # v1_event_freshness: fast 30m + slow 4h
    "v1_event_freshness": [
        {"window": "5m",  "record_suffix": "5m"},
        {"window": "30m", "record_suffix": "30m"},
        {"window": "1h",  "record_suffix": "1h"},
        {"window": "4h",  "record_suffix": "4h"},
    ],
    # v1_workflow_completion: fast 4h + slow 3d
    "v1_workflow_completion": [
        {"window": "5m",  "record_suffix": "5m"},
        {"window": "1h",  "record_suffix": "1h"},
        {"window": "4h",  "record_suffix": "4h"},
        {"window": "24h", "record_suffix": "24h"},
    ],
    # v1_data_durability: any_loss = immediate（特別 window）
    "v1_data_durability": [
        {"window": "5m",  "record_suffix": "5m"},
        {"window": "1h",  "record_suffix": "1h"},
    ],
}


class SloRulesGenerator(BaseGenerator):
    """slo_rules.lock.yaml 生成器。

    spec 07 §SLI 計測式は build artifact に準拠し、SLO monitoring rule を自動生成する。
    入力 SoT は src/tier1/schema/slo/classes.yaml と scenarios.yaml。
    出力は Prometheus recording rules + Perses dashboard stub + Litmus scenario stub。
    """

    # 出力ファイル名
    OUTPUT_NAME = "slo_rules.lock.yaml"
    # 必須入力なし（catalog は直接 REPO_ROOT から読み込む）
    REQUIRED_INPUTS: list[str] = []
    # スキーマ検証なし
    SCHEMA_PATH: Path | None = None
    # 出力先ディレクトリ
    DEFAULT_OUTPUT_DIR = "src/tier1/lock"

    def load_inputs(self, lock_dir: Path) -> dict[str, Any]:
        """SLO classes / scenarios catalog と instruments.lock.yaml を読み込む。

        catalog ファイルが存在しない場合は空 dict で graceful degradation する。
        """
        # SLO classes catalog を読み込む（SoT）
        classes_path = REPO_ROOT / _CATALOG_CLASSES_PATH
        # SLO scenarios catalog を読み込む（SoT）
        scenarios_path = REPO_ROOT / _CATALOG_SCENARIOS_PATH
        # instruments.lock.yaml を読み込む（drill 状態の参照用）
        instruments_path = REPO_ROOT / _INSTRUMENTS_LOCK_PATH

        # classes catalog を読み込む
        classes_data: dict[str, Any] = {}
        if classes_path.exists():
            # classes catalog が存在する場合は読み込む
            raw = yaml.safe_load(classes_path.read_text(encoding="utf-8"))
            # None の場合は空 dict を使用する
            classes_data = raw or {}

        # scenarios catalog を読み込む
        scenarios_data: dict[str, Any] = {}
        if scenarios_path.exists():
            # scenarios catalog が存在する場合は読み込む
            raw = yaml.safe_load(scenarios_path.read_text(encoding="utf-8"))
            # None の場合は空 dict を使用する
            scenarios_data = raw or {}

        # instruments.lock.yaml を読み込む
        instruments_data: dict[str, Any] = {}
        if instruments_path.exists():
            # instruments.lock.yaml が存在する場合は読み込む
            raw = yaml.safe_load(instruments_path.read_text(encoding="utf-8"))
            # None の場合は空 dict を使用する
            instruments_data = raw or {}

        # 読み込んだデータを dict として返す
        return {
            # SLO classes catalog データ
            "classes": classes_data,
            # SLO scenarios catalog データ
            "scenarios": scenarios_data,
            # instruments drill 状態データ
            "instruments": instruments_data,
        }

    def build_artifact(self, inputs: dict[str, Any]) -> dict[str, Any]:
        """SLO recording rules / dashboard stub / chaos scenario stub を生成して artifact dict を返す。"""
        # 生成日時を UTC で取得する
        generated_at = datetime.datetime.now(tz=datetime.timezone.utc).strftime(
            "%Y-%m-%dT%H:%M:%SZ"
        )

        # SLO classes データを取得する
        classes_data: dict[str, Any] = inputs.get("classes", {})
        # SLO scenarios データを取得する
        scenarios_data: dict[str, Any] = inputs.get("scenarios", {})
        # instruments データを取得する
        instruments_data: dict[str, Any] = inputs.get("instruments", {})

        # slo_classes リストを取得する（catalog が存在しない場合は定数を使用する）
        slo_class_list: list[dict[str, Any]] = classes_data.get("slo_classes", [])
        # catalog が空の場合は定数リストからデフォルトを生成する
        if not slo_class_list:
            # 定数からデフォルト slo_class 情報を構築する
            slo_class_list = [{"id": sc} for sc in _SLO_CLASSES]

        # scenarios リストを取得する
        scenarios_list: list[dict[str, Any]] = scenarios_data.get("scenarios", [])

        # instruments drills リストを取得する
        drills_list: list[dict[str, Any]] = instruments_data.get("drills", [])
        # drill 状態を slo_class でインデックス化する
        drill_state_by_class: dict[str, str] = {
            # slo_class をキーにして drill_state を値にする
            d.get("slo_class", ""): d.get("drill_state", "pending")
            for d in drills_list
        }

        # Prometheus Recording Rules を生成する
        prometheus_rules = self._build_prometheus_rules(slo_class_list, drill_state_by_class)

        # Perses Dashboard stub を生成する
        perses_dashboard_stub = self._build_perses_dashboard_stub(slo_class_list)

        # Litmus Chaos scenario stub を生成する
        litmus_scenarios_stub = self._build_litmus_scenarios_stub(scenarios_list)

        # metadata を構築する
        metadata = {
            # catalog が存在するかどうかを記録する
            "catalog_validation_status": "passed" if slo_class_list else "catalog_absent",
            # classes catalog の参照パスを記録する
            "catalog_classes_path": _CATALOG_CLASSES_PATH,
            # scenarios catalog の参照パスを記録する
            "catalog_scenarios_path": _CATALOG_SCENARIOS_PATH,
            # 生成した recording rules の数を記録する
            "total_recording_rules": len(prometheus_rules),
            # 生成した dashboard stub の数を記録する
            "total_dashboard_panels": len(perses_dashboard_stub),
            # 生成した chaos scenario の数を記録する
            "total_chaos_scenarios": len(litmus_scenarios_stub),
        }

        # artifact dict を返す
        return {
            # 自動生成ヘッダー
            "_AUTO_GENERATED": (
                "DO NOT EDIT. Generated by"
                " tools/lock_yaml_generator/generate_slo_rules.py"
            ),
            # 生成日時
            "generated_at": generated_at,
            # metadata セクション
            "metadata": metadata,
            # Prometheus Recording Rules セクション
            "prometheus_recording_rules": prometheus_rules,
            # Perses Dashboard stub セクション
            "perses_dashboard_stub": perses_dashboard_stub,
            # Litmus Chaos scenario stub セクション
            "litmus_chaos_scenarios": litmus_scenarios_stub,
        }

    def _build_prometheus_rules(
        self,
        slo_class_list: list[dict[str, Any]],
        drill_state_by_class: dict[str, str],
    ) -> list[dict[str, Any]]:
        """Prometheus Recording Rule のリストを生成する。

        各 slo_class に対して burn_rate recording rule を生成する。
        record 名は "job:slo_burn_rate_{window}:{slo_class_short}" 形式とする。
        """
        # recording rules のリストを初期化する
        rules: list[dict[str, Any]] = []

        # 各 slo_class に対して recording rule を生成する
        for slo_class_entry in slo_class_list:
            # slo_class の ID を取得する
            slo_class_id: str = slo_class_entry.get("id", "")
            # slo_class_id が空の場合はスキップする
            if not slo_class_id:
                continue

            # SLI kind を取得する（catalog にある場合）
            sli_kind: str = slo_class_entry.get("sli_kind", "unknown")
            # target を取得する
            target: str = str(slo_class_entry.get("target", ""))
            # window を取得する
            window: str = str(slo_class_entry.get("window", "30d"))

            # drill 状態を取得する（未登録の場合は pending）
            drill_state: str = drill_state_by_class.get(slo_class_id, "pending")

            # この slo_class のウィンドウ設定を取得する（デフォルトは標準 5 window）
            burn_windows = _BURN_RATE_WINDOWS.get(
                slo_class_id,
                # デフォルトウィンドウ設定
                [
                    {"window": "5m",  "record_suffix": "5m"},
                    {"window": "1h",  "record_suffix": "1h"},
                    {"window": "6h",  "record_suffix": "6h"},
                ],
            )

            # 各ウィンドウに対して recording rule を生成する
            for bw in burn_windows:
                # ウィンドウ文字列を取得する
                bw_window: str = bw.get("window", "5m")
                # record suffix を取得する
                bw_suffix: str = bw.get("record_suffix", "5m")

                # slo_class_id を Prometheus ラベル名として正規化する（: と _ は許可）
                # slo_class_id は "v1_request_availability" のような形式なので変換不要
                prometheus_label = slo_class_id.replace("-", "_")

                # Prometheus Recording Rule の PromQL 式を生成する
                # availability の場合は 1 - error_rate 形式を使用する
                if sli_kind == "availability":
                    # 可用性 SLI: 1 - (error_count / total_count) で計算する
                    expr = (
                        f"1 - ("
                        f"sum(rate(k1s0_request_errors_total[{bw_window}])) "
                        f"/ "
                        f"sum(rate(k1s0_requests_total[{bw_window}]))"
                        f")"
                    )
                elif sli_kind == "latency":
                    # レイテンシ SLI: histogram_quantile(0.99, ...) で計算する
                    expr = (
                        f"histogram_quantile(0.99, "
                        f"sum(rate(k1s0_request_duration_seconds_bucket[{bw_window}])) "
                        f"by (le))"
                    )
                elif sli_kind == "freshness":
                    # フレッシュネス SLI: histogram_quantile(0.95, ...) で計算する
                    expr = (
                        f"histogram_quantile(0.95, "
                        f"sum(rate(k1s0_event_freshness_seconds_bucket[{bw_window}])) "
                        f"by (le))"
                    )
                elif sli_kind == "completion_rate":
                    # 完了率 SLI: 完了したワークフロー数 / 全ワークフロー数 で計算する
                    expr = (
                        f"sum(rate(k1s0_workflow_completions_total[{bw_window}])) "
                        f"/ "
                        f"sum(rate(k1s0_workflow_starts_total[{bw_window}]))"
                    )
                elif sli_kind == "durability":
                    # 永続性 SLI: データ損失イベントが 0 であることを確認する
                    expr = (
                        f"absent(k1s0_data_loss_events_total) "
                        f"or "
                        f"sum(increase(k1s0_data_loss_events_total[{bw_window}])) == 0"
                    )
                else:
                    # 不明な SLI kind の場合はプレースホルダー式を使用する
                    expr = f"# TODO: SLI式を実装する（sli_kind={sli_kind}）"

                # Recording Rule エントリを構築する
                rule: dict[str, Any] = {
                    # record: recording rule 名（job:slo_burn_rate_{window}:{slo_class} 形式）
                    "record": f"job:slo_burn_rate_{bw_suffix}:{prometheus_label}",
                    # expr: PromQL 計算式
                    "expr": expr,
                    # labels: slo_class ラベルを付与する
                    "labels": {
                        # slo_class ラベル
                        "slo_class": slo_class_id,
                        # sli_kind ラベル
                        "sli_kind": sli_kind,
                        # target ラベル
                        "target": target,
                        # window ラベル（SLO window）
                        "slo_window": window,
                        # burn_rate_window ラベル
                        "burn_rate_window": bw_window,
                        # drill_state ラベル（instruments.lock.yaml から取得する）
                        "drill_state": drill_state,
                    },
                }
                # recording rules リストに追加する
                rules.append(rule)

        # Prometheus Recording Rules のリストを返す
        return rules

    def _build_perses_dashboard_stub(
        self,
        slo_class_list: list[dict[str, Any]],
    ) -> list[dict[str, Any]]:
        """Perses Dashboard の stub パネルリストを生成する。

        各 slo_class に対して dashboard panel stub を生成する。
        実際の Perses Dashboard 定義は別途 CI pipeline で生成する（ここは stub のみ）。
        """
        # dashboard panel stub のリストを初期化する
        panels: list[dict[str, Any]] = []

        # 各 slo_class に対して panel stub を生成する
        for slo_class_entry in slo_class_list:
            # slo_class の ID を取得する
            slo_class_id: str = slo_class_entry.get("id", "")
            # slo_class_id が空の場合はスキップする
            if not slo_class_id:
                continue

            # SLI kind を取得する
            sli_kind: str = slo_class_entry.get("sli_kind", "unknown")
            # target を取得する
            target: str = str(slo_class_entry.get("target", ""))

            # Perses Dashboard panel stub を構築する
            panel: dict[str, Any] = {
                # panel_id: ダッシュボードパネルの識別子
                "panel_id": f"slo_panel__{slo_class_id}",
                # title: パネルタイトル
                "title": f"SLO: {slo_class_id}",
                # description: パネルの説明
                "description": f"SLI kind={sli_kind}, target={target}",
                # metrics: 参照する recording rule 名のリスト
                "metrics": [
                    # 5m burn rate recording rule を参照する
                    f"job:slo_burn_rate_5m:{slo_class_id}",
                    # 1h burn rate recording rule を参照する
                    f"job:slo_burn_rate_1h:{slo_class_id}",
                ],
                # status: stub（実際のダッシュボード JSON は別途 CI で生成する）
                "status": "stub",
                # dashboard_json_ref: 実際の Perses Dashboard 定義ファイルへの参照
                "dashboard_json_ref": f"src/ops/perses_dashboards/{slo_class_id}.json",
            }
            # panels リストに追加する
            panels.append(panel)

        # Perses Dashboard stub のリストを返す
        return panels

    def _build_litmus_scenarios_stub(
        self,
        scenarios_list: list[dict[str, Any]],
    ) -> list[dict[str, Any]]:
        """Litmus Chaos scenario の stub リストを生成する。

        scenarios.yaml の各 scenario に対して Litmus scenario stub を生成する。
        実際の Litmus Workflow YAML は別途 src/test/chaos/ に配置する（ここは stub のみ）。
        """
        # Litmus scenario stub のリストを初期化する
        litmus_stubs: list[dict[str, Any]] = []

        # 各 scenario に対して Litmus scenario stub を生成する
        for scenario in scenarios_list:
            # scenario の ID を取得する
            scenario_id: str = scenario.get("id", "")
            # scenario_id が空の場合はスキップする
            if not scenario_id:
                continue

            # 対象 slo_class リストを取得する
            target_classes: list[str] = scenario.get("classes", [])

            # Litmus Chaos scenario stub を構築する
            stub: dict[str, Any] = {
                # scenario_id: Litmus scenario の識別子
                "scenario_id": f"litmus__{scenario_id}",
                # chaos_type: Litmus ChaosEngine タイプ（pod-network-latency / pod-delete 等）
                "chaos_type": "stub",
                # target_slo_classes: この scenario が検証する slo_class リスト
                "target_slo_classes": target_classes,
                # litmus_workflow_yaml_ref: 実際の Litmus Workflow YAML への参照
                "litmus_workflow_yaml_ref": f"src/test/chaos/{scenario_id}.yaml",
                # status: stub（実際の Litmus Workflow YAML は別途作成する）
                "status": "stub",
                # description: scenario の説明
                "description": (
                    f"Litmus chaos scenario for SLO validation: {scenario_id}. "
                    f"Target SLO classes: {', '.join(target_classes)}. "
                    f"Actual ChaosEngine YAML defined in litmus_workflow_yaml_ref."
                ),
            }
            # litmus_stubs リストに追加する
            litmus_stubs.append(stub)

        # scenarios_list が空の場合はデフォルト stub を返す
        if not litmus_stubs:
            # デフォルト stub を生成する（catalog が存在しない場合の graceful degradation）
            litmus_stubs = [
                {
                    # scenario_id: デフォルト stub
                    "scenario_id": "litmus__default_availability_chaos",
                    # chaos_type: デフォルト stub タイプ
                    "chaos_type": "pod-network-latency",
                    # target_slo_classes: 全 slo_class を対象にする
                    "target_slo_classes": _SLO_CLASSES,
                    # litmus_workflow_yaml_ref: デフォルト stub への参照
                    "litmus_workflow_yaml_ref": "src/test/chaos/default_availability_chaos.yaml",
                    # status: stub
                    "status": "stub",
                    # description: デフォルト stub の説明
                    "description": "Default Litmus chaos scenario stub (scenarios catalog absent).",
                }
            ]

        # Litmus Chaos scenario stub のリストを返す
        return litmus_stubs
