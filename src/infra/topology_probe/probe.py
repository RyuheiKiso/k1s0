"""src/infra/topology_probe/probe.py

クラスタートポロジープローブ。
仕様: 12_クラスタ位相適合仕様.md §topology_class

- NodeInfo: ノード名・ゾーン・リージョン・CPU 数・メモリ容量を保持する
- TopologyGraph: クラスタートポロジーを表すグラフ構造
- kubeconfig / kubectl / 環境変数からクラスター構成を探索する
- probe_connectivity: ノード間の接続性を確認する
- detect_partition: ネットワーク分断（パーティション）を検出する
- validate_placement: pod 配置が AZ 間 HLC 追跡なしに同期書き込みを行わないかを確認する
- 不変条件: AZ をまたぐ同期書き込みは必ず HLC タグを付与しなければならない
"""

from __future__ import annotations

import json
import logging
import os
import subprocess
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

# モジュールロガーを初期化する
logger = logging.getLogger(__name__)

# kubectl コマンドのタイムアウト秒数
_KUBECTL_TIMEOUT_S = 30
# デフォルト kubeconfig パス
_DEFAULT_KUBECONFIG = Path(os.environ.get("KUBECONFIG", str(Path.home() / ".kube" / "config")))
# デフォルト k1s0 名前空間
_K1S0_NAMESPACE = os.environ.get("K1S0_NAMESPACE", "k1s0")
# 接続性チェックのタイムアウト（秒）
_CONNECTIVITY_TIMEOUT_S = 5
# AZ ラベルのキー（k8s の標準ラベル）
_TOPOLOGY_ZONE_LABEL = "topology.kubernetes.io/zone"
# リージョンラベルのキー（k8s の標準ラベル）
_TOPOLOGY_REGION_LABEL = "topology.kubernetes.io/region"


# ---------------------------------------------------------------------------
# NodeInfo データクラス
# ---------------------------------------------------------------------------

@dataclass
class NodeInfo:
    """クラスター内の 1 ノードの物理情報を保持するデータクラス。"""

    # ノード名（k8s node name）
    node_name: str
    # アベイラビリティゾーン名
    zone: str
    # リージョン名
    region: str
    # CPU コア数
    cpu_count: int
    # メモリ容量（GiB）
    memory_gb: float
    # ノードの役割（"control-plane" / "worker" / "edge"）
    role: str = "worker"
    # ノード IP アドレス（内部 IP）
    internal_ip: str = ""
    # ノードが Ready 状態かどうか
    ready: bool = True
    # ノードのラベル辞書
    labels: dict[str, str] = field(default_factory=dict)
    # ノードのアノテーション辞書
    annotations: dict[str, str] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        """NodeInfo を辞書形式にシリアライズする。"""
        return {
            "node_name": self.node_name,
            "zone": self.zone,
            "region": self.region,
            "cpu_count": self.cpu_count,
            "memory_gb": self.memory_gb,
            "role": self.role,
            "internal_ip": self.internal_ip,
            "ready": self.ready,
        }


# ---------------------------------------------------------------------------
# ConnectivityResult データクラス
# ---------------------------------------------------------------------------

@dataclass
class ConnectivityResult:
    """ノード間の接続性確認結果を保持するデータクラス。"""

    # 送信元ノード名
    src_node: str
    # 送信先ノード名
    dst_node: str
    # 接続可能かどうか
    reachable: bool
    # RTT（ミリ秒）、接続不可の場合は -1
    rtt_ms: float
    # 確認方法（"ping" / "tcp_probe" / "simulated"）
    method: str
    # エラーメッセージ（接続可能の場合は空文字列）
    error: str = ""
    # 確認実行時刻
    checked_at: float = field(default_factory=time.time)

    def to_dict(self) -> dict[str, Any]:
        """ConnectivityResult を辞書形式にシリアライズする。"""
        return {
            "src_node": self.src_node,
            "dst_node": self.dst_node,
            "reachable": self.reachable,
            "rtt_ms": self.rtt_ms,
            "method": self.method,
            "error": self.error,
            "checked_at": self.checked_at,
        }


# ---------------------------------------------------------------------------
# Partition データクラス
# ---------------------------------------------------------------------------

@dataclass
class Partition:
    """検出されたネットワーク分断（パーティション）を表すデータクラス。"""

    # 分断 ID（自動採番）
    partition_id: int
    # 分断に含まれるノード名リスト
    nodes: list[str]
    # 分断が影響するゾーンリスト
    affected_zones: list[str]
    # 分断が影響するリージョンリスト
    affected_regions: list[str]
    # 分断の深刻度（"critical" / "warning" / "info"）
    severity: str

    def to_dict(self) -> dict[str, Any]:
        """Partition を辞書形式にシリアライズする。"""
        return {
            "partition_id": self.partition_id,
            "nodes": self.nodes,
            "affected_zones": self.affected_zones,
            "affected_regions": self.affected_regions,
            "severity": self.severity,
        }


# ---------------------------------------------------------------------------
# PlacementViolation データクラス
# ---------------------------------------------------------------------------

@dataclass
class PlacementViolation:
    """pod 配置違反を表すデータクラス。"""

    # 違反した pod 名
    pod_name: str
    # 違反の種類
    violation_type: str
    # 違反の説明
    description: str
    # 推奨する修正方法
    remediation: str

    def to_dict(self) -> dict[str, Any]:
        """PlacementViolation を辞書形式にシリアライズする。"""
        return {
            "pod_name": self.pod_name,
            "violation_type": self.violation_type,
            "description": self.description,
            "remediation": self.remediation,
        }


# ---------------------------------------------------------------------------
# PlacementReport データクラス
# ---------------------------------------------------------------------------

@dataclass
class PlacementReport:
    """pod 配置検証レポートを保持するデータクラス。"""

    # 検証した pod 数
    pod_count: int
    # 違反リスト
    violations: list[PlacementViolation]
    # 全ての配置が有効かどうか
    valid: bool
    # レポート生成時刻
    generated_at: float = field(default_factory=time.time)

    def to_dict(self) -> dict[str, Any]:
        """PlacementReport を辞書形式にシリアライズする。"""
        return {
            "pod_count": self.pod_count,
            "violation_count": len(self.violations),
            "violations": [v.to_dict() for v in self.violations],
            "valid": self.valid,
            "generated_at": self.generated_at,
        }


# ---------------------------------------------------------------------------
# TopologyGraph クラス
# ---------------------------------------------------------------------------

class TopologyGraph:
    """クラスタートポロジーをグラフ構造で表現するクラス。"""

    def __init__(self) -> None:
        """TopologyGraph を初期化する。"""
        # ノード名 → NodeInfo のマッピングを初期化する
        self._nodes: dict[str, NodeInfo] = {}
        # 隣接リスト（ノード名 → 接続先ノード名の集合）を初期化する
        self._edges: dict[str, set[str]] = {}

    def add_node(self, node: NodeInfo) -> None:
        """ノードをトポロジーグラフに追加する。

        Args:
            node: 追加する NodeInfo
        """
        # ノードを辞書に追加する
        self._nodes[node.node_name] = node
        # 隣接リストにエントリを初期化する
        if node.node_name not in self._edges:
            self._edges[node.node_name] = set()

    def add_edge(self, src_name: str, dst_name: str) -> None:
        """2 ノード間の接続エッジをグラフに追加する（無向グラフ）。

        Args:
            src_name: 接続元ノード名
            dst_name: 接続先ノード名
        """
        # src → dst エッジを追加する
        if src_name not in self._edges:
            self._edges[src_name] = set()
        self._edges[src_name].add(dst_name)
        # dst → src エッジを追加する（無向グラフ）
        if dst_name not in self._edges:
            self._edges[dst_name] = set()
        self._edges[dst_name].add(src_name)

    def get_node(self, node_name: str) -> NodeInfo | None:
        """指定名のノードを返す。存在しない場合は None を返す。"""
        return self._nodes.get(node_name)

    def get_all_nodes(self) -> list[NodeInfo]:
        """グラフ内の全ノードリストを返す。"""
        return list(self._nodes.values())

    def get_neighbors(self, node_name: str) -> list[str]:
        """指定ノードの隣接ノード名リストを返す。"""
        return list(self._edges.get(node_name, set()))

    def node_count(self) -> int:
        """グラフ内のノード数を返す。"""
        return len(self._nodes)

    def get_zones(self) -> list[str]:
        """グラフ内に存在するゾーン名の一覧を返す（重複なし）。"""
        # 全ノードのゾーンを収集して重複を除去する
        return list({node.zone for node in self._nodes.values()})

    def get_nodes_in_zone(self, zone: str) -> list[NodeInfo]:
        """指定ゾーンに属するノードリストを返す。"""
        # ゾーンが一致するノードをフィルタリングする
        return [n for n in self._nodes.values() if n.zone == zone]

    def get_nodes_in_region(self, region: str) -> list[NodeInfo]:
        """指定リージョンに属するノードリストを返す。"""
        # リージョンが一致するノードをフィルタリングする
        return [n for n in self._nodes.values() if n.region == region]

    def is_connected(self, src_name: str, dst_name: str) -> bool:
        """2 ノード間がエッジで接続されているか確認する（直接接続のみ）。"""
        # 隣接リストに dst_name が存在するか確認する
        return dst_name in self._edges.get(src_name, set())

    def to_dict(self) -> dict[str, Any]:
        """TopologyGraph を辞書形式にシリアライズする。"""
        # ノードリストとエッジリストを構築する
        nodes_list = [n.to_dict() for n in self._nodes.values()]
        edges_list = [
            {"src": src, "dst": dst}
            for src, dsts in self._edges.items()
            for dst in dsts
            if src < dst
        ]
        return {
            "node_count": self.node_count(),
            "nodes": nodes_list,
            "edge_count": len(edges_list),
            "edges": edges_list,
            "zones": self.get_zones(),
        }


# ---------------------------------------------------------------------------
# TopologyProbe クラス
# ---------------------------------------------------------------------------

class TopologyProbe:
    """クラスタートポロジーを探索・検証するプロービングクラス。"""

    def __init__(
        self,
        kubeconfig: Path | None = None,
        namespace: str = _K1S0_NAMESPACE,
        dry_run: bool = True,
    ) -> None:
        """TopologyProbe を初期化する。

        Args:
            kubeconfig: kubeconfig ファイルのパス
            namespace: 対象 k8s 名前空間
            dry_run: True の場合は kubectl をスキップしてダミーデータを使用する
        """
        # kubeconfig パスを設定する
        self._kubeconfig = kubeconfig or _DEFAULT_KUBECONFIG
        # 対象名前空間を保持する
        self._namespace = namespace
        # dry-run モードフラグを保持する
        self._dry_run = dry_run
        # 現在のトポロジーグラフを初期化する
        self._graph: TopologyGraph | None = None

    def _run_kubectl(self, args: list[str]) -> dict[str, Any] | None:
        """kubectl コマンドを実行して JSON 出力を返す。

        Args:
            args: kubectl に渡す引数リスト（"kubectl" を含まない）

        Returns:
            JSON パース結果の辞書、失敗時は None
        """
        # kubectl コマンドを組み立てる
        cmd = ["kubectl"] + args + ["-o", "json"]
        # kubeconfig が存在する場合は指定する
        if self._kubeconfig.exists():
            cmd = ["kubectl", "--kubeconfig", str(self._kubeconfig)] + args + ["-o", "json"]
        # コマンドを実行する
        try:
            result = subprocess.run(
                cmd, capture_output=True, text=True, timeout=_KUBECTL_TIMEOUT_S
            )
            # コマンドが失敗した場合は None を返す
            if result.returncode != 0:
                logger.error("TopologyProbe: kubectl 失敗 args=%s stderr=%s", args, result.stderr[:200])
                return None
            # JSON をパースして返す
            return json.loads(result.stdout)
        except json.JSONDecodeError as exc:
            logger.error("TopologyProbe: JSON パースエラー error=%s", exc)
            return None
        except Exception as exc:
            logger.error("TopologyProbe: kubectl 実行エラー error=%s", exc)
            return None

    def discover(self) -> TopologyGraph:
        """クラスタートポロジーを探索してグラフを構築する。

        dry-run モードの場合はダミートポロジーを生成する。

        Returns:
            構築した TopologyGraph
        """
        # dry-run モードの場合はダミートポロジーを返す
        if self._dry_run:
            return self._build_dummy_topology()
        # kubectl で node リストを取得する
        node_data = self._run_kubectl(["get", "nodes"])
        if node_data is None:
            logger.warning("TopologyProbe: ノード情報取得失敗 - ダミートポロジーを使用する")
            return self._build_dummy_topology()
        # TopologyGraph を構築する
        graph = TopologyGraph()
        for item in node_data.get("items", []):
            # ノード情報を抽出する
            node_info = self._extract_node_info(item)
            if node_info is not None:
                graph.add_node(node_info)
        # 同一クラスター内のノードは全て接続済みとみなす（complete graph）
        all_nodes = graph.get_all_nodes()
        for i in range(len(all_nodes)):
            for j in range(i + 1, len(all_nodes)):
                graph.add_edge(all_nodes[i].node_name, all_nodes[j].node_name)
        # 構築したグラフを保持する
        self._graph = graph
        logger.info("TopologyProbe: トポロジー探索完了 nodes=%d", graph.node_count())
        return graph

    def _extract_node_info(self, item: dict[str, Any]) -> NodeInfo | None:
        """kubectl node オブジェクトから NodeInfo を抽出する。"""
        # メタデータを取得する
        metadata = item.get("metadata", {})
        node_name = metadata.get("name", "unknown")
        labels = metadata.get("labels", {})
        annotations = metadata.get("annotations", {})
        # ゾーンとリージョンをラベルから取得する
        zone = labels.get(_TOPOLOGY_ZONE_LABEL, "unknown-zone")
        region = labels.get(_TOPOLOGY_REGION_LABEL, "unknown-region")
        # ノード役割を確認する
        role = "worker"
        if "node-role.kubernetes.io/control-plane" in labels:
            role = "control-plane"
        elif "node-role.kubernetes.io/master" in labels:
            role = "control-plane"
        # ステータスから CPU・メモリ・IP を取得する
        status = item.get("status", {})
        capacity = status.get("capacity", {})
        # CPU 数を取得する（"4" または "4000m" 形式）
        cpu_str = capacity.get("cpu", "0")
        try:
            cpu_count = int(cpu_str.rstrip("m")) if "m" not in cpu_str else max(1, int(cpu_str.rstrip("m")) // 1000)
        except ValueError:
            cpu_count = 0
        # メモリを GiB で取得する（"16Gi" 等の形式）
        memory_str = capacity.get("memory", "0Ki")
        memory_gb = self._parse_memory_gib(memory_str)
        # 内部 IP を取得する
        internal_ip = ""
        for addr in status.get("addresses", []):
            if addr.get("type") == "InternalIP":
                internal_ip = addr.get("address", "")
                break
        # Ready 状態を確認する
        ready = False
        for cond in status.get("conditions", []):
            if cond.get("type") == "Ready" and cond.get("status") == "True":
                ready = True
                break
        return NodeInfo(
            node_name=node_name,
            zone=zone,
            region=region,
            cpu_count=cpu_count,
            memory_gb=memory_gb,
            role=role,
            internal_ip=internal_ip,
            ready=ready,
            labels=labels,
            annotations=annotations,
        )

    @staticmethod
    def _parse_memory_gib(memory_str: str) -> float:
        """Kubernetes メモリ文字列を GiB の float に変換する。"""
        # サフィックスと数値部分を分離する
        if memory_str.endswith("Ki"):
            return int(memory_str[:-2]) / (1024 * 1024)
        elif memory_str.endswith("Mi"):
            return int(memory_str[:-2]) / 1024
        elif memory_str.endswith("Gi"):
            return float(memory_str[:-2])
        elif memory_str.endswith("Ti"):
            return float(memory_str[:-2]) * 1024
        try:
            return int(memory_str) / (1024 ** 3)
        except ValueError:
            return 0.0

    def _build_dummy_topology(self) -> TopologyGraph:
        """dry-run モード用のダミートポロジーを構築する。"""
        # ダミーグラフを作成する
        graph = TopologyGraph()
        # 3 ゾーン × 2 ノードのダミー構成を生成する
        dummy_nodes = [
            NodeInfo("node-az1-control", "ap-northeast-1a", "ap-northeast-1", 8, 32.0, "control-plane", "10.0.0.1"),
            NodeInfo("node-az1-worker", "ap-northeast-1a", "ap-northeast-1", 16, 64.0, "worker", "10.0.0.2"),
            NodeInfo("node-az2-worker-1", "ap-northeast-1b", "ap-northeast-1", 16, 64.0, "worker", "10.0.1.1"),
            NodeInfo("node-az2-worker-2", "ap-northeast-1b", "ap-northeast-1", 16, 64.0, "worker", "10.0.1.2"),
            NodeInfo("node-az3-worker-1", "ap-northeast-1c", "ap-northeast-1", 16, 64.0, "worker", "10.0.2.1"),
            NodeInfo("node-az3-worker-2", "ap-northeast-1c", "ap-northeast-1", 8, 32.0, "worker", "10.0.2.2"),
        ]
        # 全ノードを追加する
        for node in dummy_nodes:
            graph.add_node(node)
        # 全ノード間をエッジで接続する
        for i in range(len(dummy_nodes)):
            for j in range(i + 1, len(dummy_nodes)):
                graph.add_edge(dummy_nodes[i].node_name, dummy_nodes[j].node_name)
        logger.debug("TopologyProbe: ダミートポロジー構築完了 nodes=%d", graph.node_count())
        return graph

    def probe_connectivity(self, src_node: str, dst_node: str) -> ConnectivityResult:
        """2 ノード間の接続性を確認する。

        dry-run モードの場合はシミュレーション結果を返す。

        Args:
            src_node: 送信元ノード名
            dst_node: 送信先ノード名

        Returns:
            ConnectivityResult
        """
        # dry-run モードの場合はシミュレーション結果を返す
        if self._dry_run:
            return ConnectivityResult(
                src_node=src_node,
                dst_node=dst_node,
                reachable=True,
                rtt_ms=1.5,
                method="simulated",
            )
        # グラフからノード情報を取得する
        if self._graph is None:
            self.discover()
        src_info = self._graph.get_node(src_node) if self._graph else None
        dst_info = self._graph.get_node(dst_node) if self._graph else None
        # ノードが存在しない場合はエラーを返す
        if src_info is None or dst_info is None:
            return ConnectivityResult(
                src_node=src_node,
                dst_node=dst_node,
                reachable=False,
                rtt_ms=-1,
                method="none",
                error="ノードが見つからない",
            )
        # 送信先 IP に ping を送信する
        target_ip = dst_info.internal_ip
        if not target_ip:
            return ConnectivityResult(
                src_node=src_node,
                dst_node=dst_node,
                reachable=False,
                rtt_ms=-1,
                method="ping",
                error="送信先 IP が不明",
            )
        # ping を実行して RTT を計測する
        try:
            result = subprocess.run(
                ["ping", "-c", "3", "-W", str(_CONNECTIVITY_TIMEOUT_S), target_ip],
                capture_output=True, text=True, timeout=_CONNECTIVITY_TIMEOUT_S * 3 + 2
            )
            # 成功の場合は RTT を抽出する
            if result.returncode == 0:
                rtt_ms = self._extract_avg_rtt(result.stdout)
                return ConnectivityResult(
                    src_node=src_node, dst_node=dst_node,
                    reachable=True, rtt_ms=rtt_ms, method="ping"
                )
            return ConnectivityResult(
                src_node=src_node, dst_node=dst_node,
                reachable=False, rtt_ms=-1, method="ping",
                error=f"ping 失敗 returncode={result.returncode}"
            )
        except Exception as exc:
            return ConnectivityResult(
                src_node=src_node, dst_node=dst_node,
                reachable=False, rtt_ms=-1, method="ping",
                error=str(exc)
            )

    @staticmethod
    def _extract_avg_rtt(ping_output: str) -> float:
        """ping 出力の統計行から平均 RTT を抽出する。"""
        # "rtt min/avg/max/mdev = X/Y/Z/W ms" 形式の行を探す
        for line in ping_output.splitlines():
            if "avg" in line and "/" in line:
                try:
                    # スラッシュ区切りで分割して avg（2 番目）を取得する
                    parts = line.split("=")[1].strip().split("/")
                    return float(parts[1])
                except (IndexError, ValueError):
                    pass
        return -1.0

    def detect_partition(self, graph: TopologyGraph) -> list[Partition]:
        """ネットワーク分断（パーティション）を検出する。

        depth-first search を使って連結成分を見つけ、複数の連結成分が
        存在する場合は分断と判定する。

        Args:
            graph: 検査対象の TopologyGraph

        Returns:
            検出された Partition のリスト（分断がない場合は空リスト）
        """
        # 全ノード名を取得する
        all_nodes = [n.node_name for n in graph.get_all_nodes()]
        # DFS で連結成分を見つける
        visited: set[str] = set()
        components: list[list[str]] = []
        for start_node in all_nodes:
            # 未訪問のノードから DFS を開始する
            if start_node in visited:
                continue
            component: list[str] = []
            stack = [start_node]
            while stack:
                node = stack.pop()
                if node in visited:
                    continue
                visited.add(node)
                component.append(node)
                for neighbor in graph.get_neighbors(node):
                    if neighbor not in visited:
                        stack.append(neighbor)
            components.append(component)
        # 連結成分が 1 つの場合はパーティションなし
        if len(components) <= 1:
            return []
        # 複数の連結成分をパーティションとして返す
        partitions: list[Partition] = []
        for partition_id, component in enumerate(components):
            # 影響を受けるゾーンとリージョンを収集する
            affected_zones: set[str] = set()
            affected_regions: set[str] = set()
            for node_name in component:
                node_info = graph.get_node(node_name)
                if node_info:
                    affected_zones.add(node_info.zone)
                    affected_regions.add(node_info.region)
            # パーティションの深刻度を判定する
            if len(affected_regions) > 1:
                severity = "critical"
            elif len(affected_zones) > 1:
                severity = "warning"
            else:
                severity = "info"
            partition = Partition(
                partition_id=partition_id,
                nodes=component,
                affected_zones=sorted(affected_zones),
                affected_regions=sorted(affected_regions),
                severity=severity,
            )
            partitions.append(partition)
            logger.warning(
                "TopologyProbe: パーティション検出 id=%d nodes=%d zones=%s severity=%s",
                partition_id, len(component), list(affected_zones), severity
            )
        return partitions

    def validate_placement(
        self,
        pods: list[dict[str, Any]],
        topology: TopologyGraph,
    ) -> PlacementReport:
        """pod 配置が AZ 間 HLC 追跡なしに同期書き込みを行わないかを検証する。

        AZ をまたぐ配置の pod が "hlc_tracking" アノテーションを持たない場合を違反と判定する。
        製造業 k1s0 プラットフォームでは全クロス AZ 書き込みに HLC タグが必須。

        Args:
            pods: 検証する pod 情報の辞書リスト
            topology: クラスタートポロジー

        Returns:
            PlacementReport
        """
        # 違反リストを初期化する
        violations: list[PlacementViolation] = []
        # ゾーンリストを取得する
        zones = topology.get_zones()
        # AZ が 1 つ以下の場合はチェック不要
        if len(zones) <= 1:
            return PlacementReport(pod_count=len(pods), violations=[], valid=True)
        # 各 pod の配置を検証する
        for pod in pods:
            pod_name = pod.get("metadata", {}).get("name", "unknown")
            annotations = pod.get("metadata", {}).get("annotations", {})
            labels = pod.get("metadata", {}).get("labels", {})
            # pod が配置されているノードを特定する
            node_name = pod.get("spec", {}).get("nodeName", "")
            node_info = topology.get_node(node_name)
            # ノード情報が取得できない場合はスキップする
            if node_info is None:
                continue
            # クロス AZ 書き込みを行う可能性のある pod かどうかを確認する
            # k1s0-tier1 または k1s0-data ラベルが付いた pod が対象
            tier_label = labels.get("k1s0-tier", "")
            is_writer = tier_label in ("tier1", "data")
            if not is_writer:
                continue
            # HLC トラッキングアノテーションが存在するか確認する
            has_hlc_annotation = "k1s0.io/hlc-tracking" in annotations
            # HLC アノテーションがない場合は違反と判定する
            if not has_hlc_annotation:
                violation = PlacementViolation(
                    pod_name=pod_name,
                    violation_type="missing_hlc_tracking",
                    description=(
                        f"Pod {pod_name} は {node_info.zone} ゾーンに配置されているが "
                        "k1s0.io/hlc-tracking アノテーションが欠落している。"
                        "AZ 間同期書き込みには HLC タグが必須。"
                    ),
                    remediation=(
                        "Pod spec に annotations: {k1s0.io/hlc-tracking: 'enabled'} を追加し、"
                        "全クロス AZ 書き込みに HLC タイムスタンプを付与すること。"
                    ),
                )
                violations.append(violation)
                logger.warning(
                    "TopologyProbe: 配置違反 pod=%s zone=%s violation=missing_hlc_tracking",
                    pod_name, node_info.zone
                )
        # 違反がない場合は valid=True を返す
        valid = len(violations) == 0
        return PlacementReport(
            pod_count=len(pods),
            violations=violations,
            valid=valid,
        )

    def get_zone_topology_summary(self, graph: TopologyGraph) -> dict[str, Any]:
        """ゾーン別トポロジーサマリーを生成する。"""
        # ゾーン別のノード数・CPU 合計・メモリ合計を集計する
        zones = graph.get_zones()
        summary: dict[str, Any] = {}
        for zone in zones:
            nodes_in_zone = graph.get_nodes_in_zone(zone)
            total_cpu = sum(n.cpu_count for n in nodes_in_zone)
            total_memory_gb = sum(n.memory_gb for n in nodes_in_zone)
            ready_count = sum(1 for n in nodes_in_zone if n.ready)
            summary[zone] = {
                "node_count": len(nodes_in_zone),
                "ready_count": ready_count,
                "total_cpu": total_cpu,
                "total_memory_gb": total_memory_gb,
                "nodes": [n.node_name for n in nodes_in_zone],
            }
        return {
            "zone_count": len(zones),
            "total_node_count": graph.node_count(),
            "zones": summary,
        }

    def run_full_probe(self) -> dict[str, Any]:
        """トポロジー探索・接続性検証・パーティション検出を一括実行する。

        Returns:
            全検証結果をまとめた辞書
        """
        # トポロジーを探索する
        graph = self.discover()
        # ゾーンサマリーを生成する
        zone_summary = self.get_zone_topology_summary(graph)
        # パーティションを検出する
        partitions = self.detect_partition(graph)
        # 抜き取りで接続性を確認する
        connectivity_results: list[dict[str, Any]] = []
        all_nodes = graph.get_all_nodes()
        if len(all_nodes) >= 2:
            # 最初の 3 ペアのみ確認する（全ペアは高コストのため）
            check_pairs = [
                (all_nodes[0].node_name, all_nodes[1].node_name),
            ]
            if len(all_nodes) >= 3:
                check_pairs.append((all_nodes[0].node_name, all_nodes[2].node_name))
            for src_name, dst_name in check_pairs:
                result = self.probe_connectivity(src_name, dst_name)
                connectivity_results.append(result.to_dict())
        # 結果をまとめて返す
        return {
            "probed_at": time.time(),
            "topology": graph.to_dict(),
            "zone_summary": zone_summary,
            "partition_count": len(partitions),
            "partitions": [p.to_dict() for p in partitions],
            "connectivity_samples": connectivity_results,
            "healthy": len(partitions) == 0,
        }


# ---------------------------------------------------------------------------
# AffinityRule データクラス
# ---------------------------------------------------------------------------

@dataclass
class AffinityRule:
    """pod 配置のアフィニティルールを表すデータクラス。"""

    # ルール ID
    rule_id: str
    # ルール種別（"required" / "preferred"）
    rule_type: str
    # 対象 pod セレクター（ラベルキー → 値の辞書）
    pod_selector: dict[str, str]
    # トポロジーキー（"kubernetes.io/hostname" / "topology.kubernetes.io/zone" 等）
    topology_key: str
    # アンチアフィニティルールかどうか（True の場合は配置を避ける）
    anti_affinity: bool = False

    def to_dict(self) -> dict[str, Any]:
        """AffinityRule を辞書形式にシリアライズする。"""
        return {
            "rule_id": self.rule_id,
            "rule_type": self.rule_type,
            "pod_selector": self.pod_selector,
            "topology_key": self.topology_key,
            "anti_affinity": self.anti_affinity,
        }


# ---------------------------------------------------------------------------
# AffinityValidator クラス
# ---------------------------------------------------------------------------

class AffinityValidator:
    """pod アフィニティルールの遵守を検証するバリデーター。"""

    def __init__(self, topology: TopologyGraph) -> None:
        """AffinityValidator を初期化する。

        Args:
            topology: 検証に使用するトポロジーグラフ
        """
        # トポロジーグラフを保持する
        self._topology = topology
        # 登録されたアフィニティルールのリストを初期化する
        self._rules: list[AffinityRule] = []

    def add_rule(self, rule: AffinityRule) -> None:
        """アフィニティルールを追加する。"""
        self._rules.append(rule)

    def add_k1s0_default_rules(self) -> None:
        """k1s0 製造業プラットフォーム用のデフォルトアフィニティルールを追加する。"""
        # tier1 レプリカは異なるゾーンに分散配置することを要求する
        self.add_rule(AffinityRule(
            rule_id="tier1-zone-spread",
            rule_type="required",
            pod_selector={"k1s0-tier": "tier1"},
            topology_key=_TOPOLOGY_ZONE_LABEL,
            anti_affinity=True,
        ))
        # ops エンジンは tier1 と同一ゾーンに配置することを推奨する
        self.add_rule(AffinityRule(
            rule_id="ops-tier1-colocate",
            rule_type="preferred",
            pod_selector={"k1s0-tier": "ops"},
            topology_key=_TOPOLOGY_ZONE_LABEL,
            anti_affinity=False,
        ))
        # data レプリカは異なるゾーンに分散配置することを要求する
        self.add_rule(AffinityRule(
            rule_id="data-zone-spread",
            rule_type="required",
            pod_selector={"k1s0-tier": "data"},
            topology_key=_TOPOLOGY_ZONE_LABEL,
            anti_affinity=True,
        ))

    def validate(self, pods: list[dict[str, Any]]) -> list[PlacementViolation]:
        """登録済みルールに対して pod 配置を検証する。

        Args:
            pods: 検証する pod 情報の辞書リスト

        Returns:
            検出された PlacementViolation のリスト
        """
        # 違反リストを初期化する
        violations: list[PlacementViolation] = []
        # 各ルールについて検証する
        for rule in self._rules:
            # required ルールのみ違反チェックを実行する（preferred は推奨のみ）
            if rule.rule_type != "required":
                continue
            # ルールの対象 pod を抽出する
            target_pods = self._filter_pods_by_selector(pods, rule.pod_selector)
            # 対象 pod が 2 未満の場合はチェック不要
            if len(target_pods) < 2:
                continue
            # anti_affinity ルールの場合は同一トポロジーへの重複配置をチェックする
            if rule.anti_affinity:
                violations.extend(
                    self._check_anti_affinity(target_pods, rule)
                )
        return violations

    def _filter_pods_by_selector(
        self,
        pods: list[dict[str, Any]],
        selector: dict[str, str],
    ) -> list[dict[str, Any]]:
        """ラベルセレクターにマッチする pod を抽出する。"""
        # セレクターの全ラベルが pod に存在する場合のみ含める
        result = []
        for pod in pods:
            labels = pod.get("metadata", {}).get("labels", {})
            if all(labels.get(k) == v for k, v in selector.items()):
                result.append(pod)
        return result

    def _check_anti_affinity(
        self,
        pods: list[dict[str, Any]],
        rule: AffinityRule,
    ) -> list[PlacementViolation]:
        """アンチアフィニティルールの違反を検出する。"""
        # トポロジーキー値 → pod リストのマッピングを構築する
        topology_groups: dict[str, list[str]] = {}
        for pod in pods:
            pod_name = pod.get("metadata", {}).get("name", "unknown")
            node_name = pod.get("spec", {}).get("nodeName", "")
            node_info = self._topology.get_node(node_name)
            if node_info is None:
                continue
            # トポロジーキーに対応する値を取得する
            if rule.topology_key == _TOPOLOGY_ZONE_LABEL:
                topo_val = node_info.zone
            elif rule.topology_key == _TOPOLOGY_REGION_LABEL:
                topo_val = node_info.region
            else:
                topo_val = node_info.labels.get(rule.topology_key, "unknown")
            if topo_val not in topology_groups:
                topology_groups[topo_val] = []
            topology_groups[topo_val].append(pod_name)
        # 同一トポロジー値に 2 つ以上の pod が存在する場合は違反
        violations: list[PlacementViolation] = []
        for topo_val, pod_names in topology_groups.items():
            if len(pod_names) >= 2:
                for pod_name in pod_names[1:]:
                    violation = PlacementViolation(
                        pod_name=pod_name,
                        violation_type="anti_affinity_violation",
                        description=(
                            f"Pod {pod_name} がルール {rule.rule_id} に違反。"
                            f"同一 {rule.topology_key}={topo_val} に複数の同種 pod が存在する。"
                        ),
                        remediation=(
                            f"pod を異なる {rule.topology_key} の node に再配置すること。"
                            "PodAntiAffinity の requiredDuringSchedulingIgnoredDuringExecution を設定する。"
                        ),
                    )
                    violations.append(violation)
        return violations


# ---------------------------------------------------------------------------
# NodeCapacityChecker クラス
# ---------------------------------------------------------------------------

class NodeCapacityChecker:
    """クラスター内のノードキャパシティを検証するクラス。"""

    def __init__(self, topology: TopologyGraph) -> None:
        """NodeCapacityChecker を初期化する。"""
        # トポロジーグラフを保持する
        self._topology = topology

    def check_zone_balance(self) -> dict[str, Any]:
        """ゾーン間の CPU・メモリ容量のバランスを確認する。

        Returns:
            ゾーンバランスチェック結果の辞書
        """
        # ゾーンリストを取得する
        zones = self._topology.get_zones()
        # ゾーンが 1 つ以下の場合はバランスチェック不要
        if len(zones) <= 1:
            return {"balanced": True, "zone_count": len(zones), "reason": "single_zone"}
        # ゾーンごとの容量を集計する
        zone_cpu: dict[str, int] = {}
        zone_mem: dict[str, float] = {}
        for zone in zones:
            nodes = self._topology.get_nodes_in_zone(zone)
            zone_cpu[zone] = sum(n.cpu_count for n in nodes)
            zone_mem[zone] = sum(n.memory_gb for n in nodes)
        # CPU バランスを確認する（最大/最小比が 2 以上の場合は不均衡）
        max_cpu = max(zone_cpu.values())
        min_cpu = min(zone_cpu.values())
        cpu_ratio = max_cpu / min_cpu if min_cpu > 0 else float("inf")
        # メモリバランスを確認する
        max_mem = max(zone_mem.values())
        min_mem = min(zone_mem.values())
        mem_ratio = max_mem / min_mem if min_mem > 0 else float("inf")
        # バランス判定を行う
        balanced = cpu_ratio < 2.0 and mem_ratio < 2.0
        return {
            "balanced": balanced,
            "zone_count": len(zones),
            "cpu_ratio": cpu_ratio,
            "mem_ratio": mem_ratio,
            "zone_cpu": zone_cpu,
            "zone_mem_gb": zone_mem,
        }

    def check_ready_node_count(self, min_per_zone: int = 1) -> dict[str, Any]:
        """各ゾーンに最小限の Ready ノードが存在するか確認する。

        Args:
            min_per_zone: ゾーンごとに必要な最小 Ready ノード数

        Returns:
            確認結果の辞書
        """
        # ゾーンリストを取得する
        zones = self._topology.get_zones()
        # 各ゾーンの Ready ノード数を集計する
        zone_ready: dict[str, int] = {}
        for zone in zones:
            nodes = self._topology.get_nodes_in_zone(zone)
            zone_ready[zone] = sum(1 for n in nodes if n.ready)
        # 最小要件を満たさないゾーンを特定する
        insufficient_zones = [z for z, count in zone_ready.items() if count < min_per_zone]
        return {
            "sufficient": len(insufficient_zones) == 0,
            "min_per_zone": min_per_zone,
            "zone_ready_counts": zone_ready,
            "insufficient_zones": insufficient_zones,
        }
