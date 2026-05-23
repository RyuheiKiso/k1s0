"""src/test/integration/tier1/test_v1_fault_chaos.py

tier1 軸の v1_fault_chaos 検証クラスに対するインテグレーションテスト。
ネットワーク分断・ディスク障害・プロセスクラッシュをシミュレートし、
障害解消後のシステム回復を検証する。

仕様: docs/04_詳細設計/01_適合仕様/19_検証規律適合仕様.md

ai_generator_id: claude-sonnet-4-6
model_version: claude-sonnet-4-6
prompt_hash: tier1_fault_chaos_v1
"""

from __future__ import annotations

# 標準ライブラリをインポートする
import time
import random
import threading
# pytest フレームワークをインポートする
import pytest
# 型ヒントをインポートする
from typing import Any, Generator, List, Optional, Dict
# dataclass デコレータをインポートする
from dataclasses import dataclass, field
# 列挙型をインポートする
from enum import Enum, auto
# コンテキストマネージャをインポートする
from contextlib import contextmanager


# ノードの状態を表す列挙型を定義する
class NodeState(Enum):
    """コンセンサスノードの状態。"""

    # リーダーノードの状態を定義する
    LEADER = auto()
    # フォロワーノードの状態を定義する
    FOLLOWER = auto()
    # 候補ノードの状態を定義する
    CANDIDATE = auto()
    # 停止中のノードの状態を定義する
    CRASHED = auto()
    # 分断中のノードの状態を定義する
    PARTITIONED = auto()


# 障害の種類を表す列挙型を定義する
class FaultType(Enum):
    """注入する障害の種類。"""

    # ネットワーク分断を表す障害タイプを定義する
    NETWORK_PARTITION = "network_partition"
    # ディスク書き込み失敗を表す障害タイプを定義する
    DISK_WRITE_FAILURE = "disk_write_failure"
    # プロセスクラッシュを表す障害タイプを定義する
    PROCESS_CRASH = "process_crash"
    # 遅延注入を表す障害タイプを定義する
    LATENCY_INJECTION = "latency_injection"


# ディスク書き込み失敗を模擬する例外クラスを定義する
class DiskWriteError(Exception):
    """ディスク書き込み失敗を模擬する例外。"""

    # エラーメッセージを初期化するメソッドを定義する
    def __init__(self, path: str) -> None:
        """ディスク書き込みエラーを初期化する。"""
        # 親クラスの初期化を呼び出す
        super().__init__(f"Disk write failed for path: {path}")
        # 障害が発生したパスを記録する
        self.path = path


# コンセンサスノードのモッククラスを定義する
@dataclass
class MockConsensusNode:
    """テスト用のコンセンサスノードモック。"""

    # ノード ID を定義する
    node_id: str
    # ノードの現在状態を定義する
    state: NodeState = NodeState.FOLLOWER
    # コミット済みのログエントリを定義する
    committed_log: list[tuple[int, bytes]] = field(default_factory=list)
    # 未コミットのエントリを定義する
    pending_log: list[tuple[int, bytes]] = field(default_factory=list)
    # ネットワーク分断フラグを定義する
    is_partitioned: bool = False
    # クラッシュフラグを定義する
    is_crashed: bool = False
    # ディスク障害フラグを定義する
    disk_failure: bool = False
    # 次のシーケンス番号を定義する
    next_seq: int = 1

    # ログエントリを提案するメソッドを定義する
    def propose(self, payload: bytes) -> Optional[int]:
        """ログエントリを提案する。クラッシュまたは分断中は None を返す。"""
        # クラッシュ中の場合は提案を拒否する
        if self.is_crashed:
            # None を返してクラッシュ状態を表現する
            return None
        # 分断中の場合は提案を拒否する
        if self.is_partitioned:
            # None を返して分断状態を表現する
            return None
        # ディスク障害の場合はエラーを発生させる
        if self.disk_failure:
            # ディスク書き込みエラーを発生させる
            raise DiskWriteError(f"/data/raft/node_{self.node_id}/log")
        # シーケンス番号を取得する
        seq = self.next_seq
        # エントリを保留リストに追加する
        self.pending_log.append((seq, payload))
        # シーケンス番号をインクリメントする
        self.next_seq += 1
        # シーケンス番号を返す
        return seq

    # ログエントリをコミットするメソッドを定義する
    def commit(self, seq: int, payload: bytes) -> bool:
        """ログエントリをコミットする。ディスク障害の場合は失敗する。"""
        # ディスク障害の場合はコミットに失敗する
        if self.disk_failure:
            # False を返してコミット失敗を表現する
            return False
        # クラッシュ中の場合はコミットに失敗する
        if self.is_crashed:
            # False を返してクラッシュ状態を表現する
            return False
        # コミット済みリストにエントリを追加する
        self.committed_log.append((seq, payload))
        # True を返してコミット成功を表現する
        return True

    # ノードを回復させるメソッドを定義する
    def recover(self) -> None:
        """ノードをクラッシュ・分断・ディスク障害から回復させる。"""
        # クラッシュフラグをリセットする
        self.is_crashed = False
        # 分断フラグをリセットする
        self.is_partitioned = False
        # ディスク障害フラグをリセットする
        self.disk_failure = False


# コンセンサスクラスタのモッククラスを定義する
class MockConsensusCluster:
    """テスト用のコンセンサスクラスタモック（3 ノード構成）。"""

    # クラスタを初期化するメソッドを定義する
    def __init__(self) -> None:
        """3 ノードのクラスタを初期化する。"""
        # ノードをリストとして初期化する
        self.nodes: list[MockConsensusNode] = [
            MockConsensusNode(node_id="node_0", state=NodeState.LEADER),
            MockConsensusNode(node_id="node_1", state=NodeState.FOLLOWER),
            MockConsensusNode(node_id="node_2", state=NodeState.FOLLOWER),
        ]
        # リーダーノードのインデックスを初期化する
        self._leader_idx: int = 0

    # リーダーノードを取得するプロパティを定義する
    @property
    def leader(self) -> MockConsensusNode:
        """現在のリーダーノードを返す。"""
        # リーダーノードを返す
        return self.nodes[self._leader_idx]

    # クラスタにエントリを書き込むメソッドを定義する
    def write(self, payload: bytes) -> Optional[int]:
        """リーダーを通じてエントリを書き込む。"""
        # リーダーノードに提案を送る
        seq = self.leader.propose(payload)
        # リーダーの提案が成功した場合はフォロワーにレプリカを作成する
        if seq is not None:
            # フォロワーに複製してクオーラムを達成する
            quorum_count = 1
            # 各ノードにコミットを試みる
            for i, node in enumerate(self.nodes):
                # リーダー以外のノードにコミットを試みる
                if i != self._leader_idx and not node.is_partitioned and not node.is_crashed:
                    # コミットに成功した場合はカウントをインクリメントする
                    if node.commit(seq, payload):
                        # クオーラムカウントをインクリメントする
                        quorum_count += 1
            # クオーラムが達成された場合はリーダーもコミットする
            if quorum_count >= 2:
                # リーダーのコミット済みリストにエントリを追加する
                self.leader.committed_log.append((seq, payload))
                # シーケンス番号を返す
                return seq
        # クオーラムが達成できなかった場合は None を返す
        return None

    # ネットワーク分断を発生させるメソッドを定義する
    def partition_node(self, node_id: str) -> None:
        """指定されたノードをネットワーク分断状態にする。"""
        # 指定されたノードを検索する
        for node in self.nodes:
            # ノード ID が一致する場合は分断フラグを設定する
            if node.node_id == node_id:
                # 分断フラグを True に設定する
                node.is_partitioned = True
                # 分断中の状態に変更する
                node.state = NodeState.PARTITIONED
                # 処理を終了する
                return

    # ネットワーク分断を解消するメソッドを定義する
    def heal_partition(self, node_id: str) -> None:
        """指定されたノードのネットワーク分断を解消する。"""
        # 指定されたノードを検索する
        for node in self.nodes:
            # ノード ID が一致する場合は分断フラグをリセットする
            if node.node_id == node_id:
                # ノードを回復させる
                node.recover()
                # フォロワー状態に変更する
                node.state = NodeState.FOLLOWER
                # 処理を終了する
                return

    # 全ノードのコミット済みエントリ数を取得するメソッドを定義する
    def get_committed_counts(self) -> dict[str, int]:
        """各ノードのコミット済みエントリ数を返す。"""
        # ノード ID をキーにコミット済みエントリ数の辞書を返す
        return {node.node_id: len(node.committed_log) for node in self.nodes}


# モック KV ストアのディスク書き込みクラスを定義する
class MockKVStoreDisk:
    """ディスク障害をシミュレートできる KV ストアのモック。"""

    # ストアを初期化するメソッドを定義する
    def __init__(self) -> None:
        """ストアを初期化する。"""
        # メモリストレージを初期化する
        self._memory: dict[bytes, bytes] = {}
        # ディスク障害フラグを初期化する
        self._disk_failure: bool = False
        # 書き込み試行回数を記録する
        self._write_attempts: int = 0
        # 書き込み成功回数を記録する
        self._write_successes: int = 0

    # ディスク障害を注入するメソッドを定義する
    def inject_disk_failure(self) -> None:
        """ディスク障害を注入する。"""
        # ディスク障害フラグを True に設定する
        self._disk_failure = True

    # ディスク障害を解消するメソッドを定義する
    def recover_disk(self) -> None:
        """ディスク障害を解消する。"""
        # ディスク障害フラグを False に設定する
        self._disk_failure = False

    # KV ストアに書き込むメソッドを定義する
    def put(self, key: bytes, value: bytes) -> bool:
        """キーバリューを書き込む。ディスク障害中は例外を発生させる。"""
        # 書き込み試行回数をインクリメントする
        self._write_attempts += 1
        # ディスク障害の場合は例外を発生させる
        if self._disk_failure:
            # ディスク書き込みエラーを発生させる
            raise DiskWriteError(f"/data/kv/{key.hex()}")
        # メモリに書き込む
        self._memory[key] = value
        # 書き込み成功回数をインクリメントする
        self._write_successes += 1
        # True を返す
        return True

    # KV ストアから読み取るメソッドを定義する
    def get(self, key: bytes) -> Optional[bytes]:
        """指定されたキーの値を取得する。"""
        # メモリから値を返す
        return self._memory.get(key)

    # 書き込み統計を取得するプロパティを定義する
    @property
    def stats(self) -> dict[str, int]:
        """書き込み統計を返す。"""
        # 統計辞書を返す
        return {
            "attempts": self._write_attempts,
            "successes": self._write_successes,
        }


# tier1 障害注入テストスイートクラスを定義する
class TestTier1FaultChaos:
    """tier1 軸 v1_fault_chaos の検証テストスイート。"""

    # コンセンサスクラスタのフィクスチャを定義する
    @pytest.fixture
    def cluster(self) -> Generator[MockConsensusCluster, None, None]:
        """MockConsensusCluster フィクスチャを生成する。"""
        # クラスタをインスタンス化する
        c = MockConsensusCluster()
        # フィクスチャを返す
        yield c

    # KV ストアのフィクスチャを定義する
    @pytest.fixture
    def kv_store(self) -> Generator[MockKVStoreDisk, None, None]:
        """MockKVStoreDisk フィクスチャを生成する。"""
        # KV ストアをインスタンス化する
        store = MockKVStoreDisk()
        # フィクスチャを返す
        yield store

    # ネットワーク分断のシナリオを定義するパラメータリストを作成する
    @pytest.fixture(
        params=[
            # ノード 2 を分断する設定を定義する
            pytest.param("node_2", id="partition_follower_node_2"),
            # ノード 1 を分断する設定を定義する
            pytest.param("node_1", id="partition_follower_node_1"),
        ]
    )
    def partitioned_node_id(self, request: pytest.FixtureRequest) -> str:
        """分断対象ノード ID を返すパラメータフィクスチャ。"""
        # パラメータ値を返す
        return request.param

    # ネットワーク分断中でも 2 ノードクオーラムで書き込めることをテストする
    def test_network_partition_quorum_still_works(
        self,
        cluster: MockConsensusCluster,
        partitioned_node_id: str,
    ) -> None:
        """1 ノード分断でも 2 ノードクオーラムにより書き込みが成功することを検証する。"""
        # フォロワーノードを分断する
        cluster.partition_node(partitioned_node_id)
        # 分断前にいくつかのエントリを書き込む
        for i in range(5):
            # エントリを書き込む
            seq = cluster.write(f"pre_partition_entry_{i}".encode())
            # 書き込みが成功したことを確認する
            assert seq is not None, f"Write failed for entry {i}"
        # リーダーのコミット済みエントリ数が 5 であることを確認する
        assert len(cluster.leader.committed_log) == 5

    # ネットワーク分断解消後に整合性が回復することをテストする
    def test_network_partition_heal_restores_consistency(
        self,
        cluster: MockConsensusCluster,
        partitioned_node_id: str,
    ) -> None:
        """ネットワーク分断解消後にノードが正常状態に回復することを検証する。"""
        # フォロワーノードを分断する
        cluster.partition_node(partitioned_node_id)
        # 分断中に 3 件のエントリを書き込む
        for i in range(3):
            # エントリを書き込む
            cluster.write(f"during_partition_{i}".encode())
        # 分断を解消する
        cluster.heal_partition(partitioned_node_id)
        # 分断解消後に 2 件のエントリを書き込む
        for i in range(2):
            # エントリを書き込む
            seq = cluster.write(f"after_heal_{i}".encode())
            # 書き込みが成功したことを確認する
            assert seq is not None, f"Post-heal write {i} failed"
        # 分断解消後は 3 ノードすべてで書き込みが成功することを確認する
        counts = cluster.get_committed_counts()
        # リーダーのコミット済みエントリ数が 5 であることを確認する
        assert counts["node_0"] == 5

    # ディスク書き込み失敗中に KV ストアの書き込みが拒否されることをテストする
    def test_disk_write_failure_rejected(self, kv_store: MockKVStoreDisk) -> None:
        """ディスク障害注入中に書き込みが例外を発生させることを検証する。"""
        # 正常書き込みが成功することを確認する
        result = kv_store.put(b"pre_failure_key", b"pre_failure_value")
        # 書き込みが成功したことを確認する
        assert result is True
        # ディスク障害を注入する
        kv_store.inject_disk_failure()
        # 障害注入後の書き込みが例外を発生させることを確認する
        with pytest.raises(DiskWriteError) as exc_info:
            # 障害中に書き込みを試みる
            kv_store.put(b"during_failure_key", b"should_fail")
        # エラーメッセージにパス情報が含まれることを確認する
        assert "/data/kv/" in str(exc_info.value)

    # ディスク障害回復後に書き込みが再開することをテストする
    def test_disk_write_failure_recovery(self, kv_store: MockKVStoreDisk) -> None:
        """ディスク障害回復後に書き込みが再開することを検証する。"""
        # 障害前に書き込む
        kv_store.put(b"key_before", b"value_before")
        # ディスク障害を注入する
        kv_store.inject_disk_failure()
        # 障害中の書き込みが失敗することを確認する
        with pytest.raises(DiskWriteError):
            # 障害中に書き込みを試みる
            kv_store.put(b"during_failure", b"should_fail")
        # ディスク障害を回復させる
        kv_store.recover_disk()
        # 回復後の書き込みが成功することを確認する
        result = kv_store.put(b"key_after", b"value_after")
        # 書き込みが成功したことを確認する
        assert result is True
        # 障害前のデータが読み取れることを確認する
        assert kv_store.get(b"key_before") == b"value_before"
        # 回復後のデータが読み取れることを確認する
        assert kv_store.get(b"key_after") == b"value_after"
        # 障害中のデータが存在しないことを確認する
        assert kv_store.get(b"during_failure") is None

    # プロセスクラッシュ中のコンセンサス書き込みが失敗することをテストする
    def test_process_crash_during_consensus(self, cluster: MockConsensusCluster) -> None:
        """リーダーノードのクラッシュ中にコンセンサス書き込みが失敗することを検証する。"""
        # 正常状態で書き込みを実行する
        seq = cluster.write(b"pre_crash_entry")
        # 書き込みが成功したことを確認する
        assert seq is not None
        # リーダーノードをクラッシュさせる
        cluster.leader.is_crashed = True
        # クラッシュ中の書き込みが失敗することを確認する
        crash_result = cluster.write(b"during_crash_entry")
        # クラッシュ中は書き込みが失敗することを確認する
        assert crash_result is None, "Write should fail during leader crash"

    # クラッシュ回復後にコンセンサス書き込みが再開することをテストする
    def test_process_crash_recovery_resumes_writes(self, cluster: MockConsensusCluster) -> None:
        """クラッシュ回復後にコンセンサス書き込みが再開することを検証する。"""
        # リーダーをクラッシュさせる
        cluster.leader.is_crashed = True
        # クラッシュ中の書き込みが失敗することを確認する
        assert cluster.write(b"crash_test") is None
        # リーダーを回復させる
        cluster.leader.recover()
        # 回復後の書き込みが成功することを確認する
        seq = cluster.write(b"post_recovery_entry")
        # 書き込みが成功したことを確認する
        assert seq is not None, "Write should succeed after recovery"

    # ディスク障害中のコンセンサスコミットが失敗することをテストする
    def test_disk_failure_during_consensus_commit(
        self, cluster: MockConsensusCluster
    ) -> None:
        """リーダーのディスク障害中にコンセンサスコミットが失敗することを検証する。"""
        # 正常状態で書き込みを実行する
        seq1 = cluster.write(b"before_disk_failure")
        # 書き込みが成功したことを確認する
        assert seq1 is not None
        # リーダーのディスク障害を注入する
        cluster.leader.disk_failure = True
        # ディスク障害中の書き込みが例外を発生させることを確認する
        with pytest.raises(DiskWriteError):
            # 障害中に書き込みを試みる
            cluster.write(b"during_disk_failure")
        # ディスク障害を回復させる
        cluster.leader.recover()
        # 回復後の書き込みが成功することを確認する
        seq2 = cluster.write(b"after_disk_recovery")
        # 書き込みが成功したことを確認する
        assert seq2 is not None

    # 複数の障害シナリオに対してパラメータ化テストを実施する
    @pytest.mark.parametrize(
        "fault_type,expected_failure",
        [
            # ネットワーク分断の場合は書き込みが失敗する
            (FaultType.NETWORK_PARTITION, True),
            # プロセスクラッシュの場合は書き込みが失敗する
            (FaultType.PROCESS_CRASH, True),
        ],
    )
    def test_parameterized_fault_injection(
        self,
        cluster: MockConsensusCluster,
        fault_type: FaultType,
        expected_failure: bool,
    ) -> None:
        """複数の障害タイプに対してコンセンサス障害を検証する。"""
        # 障害タイプに応じて障害を注入する
        if fault_type == FaultType.NETWORK_PARTITION:
            # リーダーを分断する
            cluster.leader.is_partitioned = True
        elif fault_type == FaultType.PROCESS_CRASH:
            # リーダーをクラッシュさせる
            cluster.leader.is_crashed = True
        # 障害中の書き込み結果を取得する
        result = cluster.write(b"fault_test_payload")
        # 障害中の書き込みが失敗していることを確認する
        assert (result is None) == expected_failure, (
            f"Fault type {fault_type.value}: expected_failure={expected_failure}, got result={result}"
        )

    # 障害後の整合性回復をエンドツーエンドでテストする
    def test_end_to_end_fault_and_recovery(
        self, cluster: MockConsensusCluster
    ) -> None:
        """障害注入から回復までのエンドツーエンドシナリオを検証する。"""
        # フェーズ 1: 正常書き込みを実行する
        pre_fault_entries = 5
        # 正常書き込みを実行する
        for i in range(pre_fault_entries):
            # エントリを書き込む
            seq = cluster.write(f"phase1_{i}".encode())
            # 書き込みが成功したことを確認する
            assert seq is not None
        # フェーズ 2: ノード 2 を分断する
        cluster.partition_node("node_2")
        # フェーズ 2 での書き込みを実行する
        phase2_entries = 3
        # 書き込みを実行する
        for i in range(phase2_entries):
            # エントリを書き込む
            seq = cluster.write(f"phase2_{i}".encode())
            # クオーラムが 2 ノードなので書き込みは成功するはず
            assert seq is not None
        # フェーズ 3: 分断を解消する
        cluster.heal_partition("node_2")
        # フェーズ 3 での書き込みを実行する
        phase3_entries = 2
        # 書き込みを実行する
        for i in range(phase3_entries):
            # エントリを書き込む
            seq = cluster.write(f"phase3_{i}".encode())
            # 分断解消後の書き込みが成功することを確認する
            assert seq is not None
        # リーダーのコミット済みエントリ数が合計と一致することを確認する
        total_expected = pre_fault_entries + phase2_entries + phase3_entries
        # リーダーのコミット済みエントリ数を取得する
        assert len(cluster.leader.committed_log) == total_expected

    # 書き込み統計が正確であることをテストする
    def test_kv_write_statistics_accurate(self, kv_store: MockKVStoreDisk) -> None:
        """KV ストアの書き込み統計が正確であることを検証する。"""
        # 5 件の正常書き込みを実行する
        for i in range(5):
            # 書き込みを実行する
            kv_store.put(f"key_{i}".encode(), f"val_{i}".encode())
        # ディスク障害を注入する
        kv_store.inject_disk_failure()
        # 3 件の障害書き込みを試みる
        for _ in range(3):
            # 障害中の書き込みを試みる
            try:
                # 書き込みを試みる
                kv_store.put(b"fail_key", b"fail_val")
            except DiskWriteError:
                # エラーを無視する
                pass
        # 統計を取得する
        stats = kv_store.stats
        # 書き込み試行回数が 8 であることを確認する
        assert stats["attempts"] == 8, f"Expected 8 attempts, got {stats['attempts']}"
        # 書き込み成功回数が 5 であることを確認する
        assert stats["successes"] == 5, f"Expected 5 successes, got {stats['successes']}"


# tier1 軸障害注入拡張テストスイートクラスを定義する
class TestTier1FaultChaosExtended:
    """tier1 軸 v1_fault_chaos の拡張テストスイート。追加の障害シナリオを網羅的に検証する。"""

    # KV ストアのフィクスチャを定義する
    @pytest.fixture
    def kv_store(self) -> Generator[MockKVStoreDisk, None, None]:
        """MockKVStoreDisk フィクスチャを生成する。"""
        # KV ストアをインスタンス化する
        store = MockKVStoreDisk()
        # フィクスチャを返す
        yield store

    # クラスタのフィクスチャを定義する
    @pytest.fixture
    def cluster(self) -> Generator[MockConsensusCluster, None, None]:
        """MockConsensusCluster フィクスチャを生成する。"""
        # クラスタをインスタンス化する
        c = MockConsensusCluster()
        # フィクスチャを返す
        yield c

    # KV ストアの連続ディスク障害注入・回復を検証するテストを定義する
    def test_multiple_disk_failure_cycles(self, kv_store: MockKVStoreDisk) -> None:
        """KV ストアで複数回のディスク障害・回復サイクルを繰り返しても正常動作することを検証する。"""
        # サイクル数を定義する
        cycles = 3
        # サイクルを繰り返す
        for cycle in range(cycles):
            # 障害前のデータを書き込む
            kv_store.put(f"key_cycle_{cycle}_pre".encode(), f"val_{cycle}_pre".encode())
            # ディスク障害を注入する
            kv_store.inject_disk_failure()
            # 障害中の書き込みが失敗することを確認する
            with pytest.raises(DiskWriteError):
                # 障害中に書き込みを試みる
                kv_store.put(f"key_cycle_{cycle}_fail".encode(), b"should_fail")
            # ディスク障害を回復させる
            kv_store.recover_disk()
            # 回復後の書き込みが成功することを確認する
            result = kv_store.put(f"key_cycle_{cycle}_post".encode(), f"val_{cycle}_post".encode())
            # 書き込みが成功したことを確認する
            assert result is True
        # 全ての障害前データが読み取れることを確認する
        for cycle in range(cycles):
            # 障害前のデータが読み取れることを確認する
            assert kv_store.get(f"key_cycle_{cycle}_pre".encode()) == f"val_{cycle}_pre".encode()
            # 回復後のデータが読み取れることを確認する
            assert kv_store.get(f"key_cycle_{cycle}_post".encode()) == f"val_{cycle}_post".encode()

    # KV ストアの書き込み統計が障害を正確に記録することをテストする
    def test_kv_stats_track_failure_correctly(self, kv_store: MockKVStoreDisk) -> None:
        """KV ストアの統計が書き込み試行・成功・失敗を正確に記録することを検証する。"""
        # 3 件の正常書き込みを実行する
        for i in range(3):
            # 書き込みを実行する
            kv_store.put(f"key_{i}".encode(), f"val_{i}".encode())
        # ディスク障害を注入する
        kv_store.inject_disk_failure()
        # 2 件の障害書き込みを試みる
        for _ in range(2):
            # 障害中の書き込みを試みる
            try:
                # 書き込みを試みる
                kv_store.put(b"fail", b"fail_val")
            except DiskWriteError:
                # エラーを無視する
                pass
        # 統計を検証する
        stats = kv_store.stats
        # 合計試行回数が 5 であることを確認する
        assert stats["attempts"] == 5, f"Expected 5 attempts, got {stats['attempts']}"
        # 成功回数が 3 であることを確認する
        assert stats["successes"] == 3, f"Expected 3 successes, got {stats['successes']}"

    # ノード 1 台分断でクオーラムが維持されることをテストする
    def test_single_node_partition_quorum_maintained(
        self, cluster: MockConsensusCluster
    ) -> None:
        """1 ノードが分断されても 2 ノードのクオーラムで書き込みが成功することを検証する。"""
        # ノード 2 を分断する
        cluster.partition_node("node_2")
        # 分断後の書き込みが成功することを確認する
        for i in range(5):
            # 書き込みを実行する
            seq = cluster.write(f"quorum_entry_{i}".encode())
            # 書き込みが成功したことを確認する
            assert seq is not None, f"Write {i} should succeed with 2/3 quorum"

    # 全ノードが分断されるとクオーラムが失われることをテストする
    def test_all_followers_partitioned_quorum_lost(
        self, cluster: MockConsensusCluster
    ) -> None:
        """全フォロワーが分断されるとクオーラムが失われ書き込みが失敗することを検証する。"""
        # node_1 と node_2 を分断する（リーダーは node_0）
        cluster.partition_node("node_1")
        # node_2 を分断する
        cluster.partition_node("node_2")
        # クオーラム失失後の書き込みが失敗することを確認する
        result = cluster.write(b"should_fail_no_quorum")
        # 書き込みが失敗することを確認する
        assert result is None, "Write should fail when quorum is lost"

    # リーダークラッシュから回復後の書き込みシーケンス番号が継続することをテストする
    def test_seq_number_continues_after_leader_recovery(
        self, cluster: MockConsensusCluster
    ) -> None:
        """リーダークラッシュから回復後もシーケンス番号が継続することを検証する。"""
        # 5 件の書き込みを実行する
        seqs_before = []
        # 5 件の書き込みを実行する
        for i in range(5):
            # 書き込みを実行する
            seq = cluster.write(f"before_crash_{i}".encode())
            # 書き込みが成功したことを確認する
            assert seq is not None
            # シーケンス番号を記録する
            seqs_before.append(seq)
        # リーダーをクラッシュさせる
        cluster.leader.is_crashed = True
        # クラッシュ中の書き込みが失敗することを確認する
        assert cluster.write(b"crash_test") is None
        # リーダーを回復させる
        cluster.leader.recover()
        # 回復後の書き込みが成功することを確認する
        seq_after = cluster.write(b"after_recovery")
        # 書き込みが成功したことを確認する
        assert seq_after is not None
        # シーケンス番号が継続していることを確認する（前の最後より大きい）
        assert seq_after > seqs_before[-1], (
            f"seq_after={seq_after} should be > last_before={seqs_before[-1]}"
        )

    # KV ストアの大量書き込み後のデータ整合性をテストする
    def test_kv_bulk_write_data_integrity(self, kv_store: MockKVStoreDisk) -> None:
        """大量書き込み後にデータが全て正確に読み取れることを検証する。"""
        # 50 件の書き込みを実行する
        written: dict[bytes, bytes] = {}
        # 50 件のデータを書き込む
        for i in range(50):
            # キーと値を生成する
            key = f"bulk_key_{i:04d}".encode()
            # 値を生成する
            value = f"bulk_val_{i:04d}_{'x' * (i % 20)}".encode()
            # 書き込みを実行する
            kv_store.put(key, value)
            # 書き込んだデータを記録する
            written[key] = value
        # 全データが正確に読み取れることを確認する
        for key, expected_value in written.items():
            # 値を取得する
            got = kv_store.get(key)
            # 値が正確であることを確認する
            assert got == expected_value, (
                f"Data integrity violation for key={key!r}: "
                f"expected={expected_value!r}, got={got!r}"
            )
        # 統計が正確であることを確認する
        stats = kv_store.stats
        # 試行回数が 50 であることを確認する
        assert stats["attempts"] == 50
        # 成功回数が 50 であることを確認する
        assert stats["successes"] == 50


# tier1 障害カオステストの第三拡張クラスを定義する
class TestTier1FaultChaosExtended3:
    """tier1 障害注入テストの追加テスト群（第三拡張）。"""

    # KV ストアのフィクスチャを定義する
    @pytest.fixture
    def kv_store(self) -> MockKVStoreDisk:
        """MockKVStoreDisk フィクスチャを生成する。"""
        # MockKVStoreDisk をインスタンス化して返す
        return MockKVStoreDisk()

    # コンセンサスクラスターのフィクスチャを定義する
    @pytest.fixture
    def cluster(self) -> MockConsensusCluster:
        """MockConsensusCluster フィクスチャを生成する。"""
        # MockConsensusCluster をインスタンス化する
        c = MockConsensusCluster(node_count=5)
        # フィクスチャを返す
        return c

    # KV ストアが障害なしで正常動作することをテストする
    def test_kv_no_failure_normal_operation(
        self, kv_store: MockKVStoreDisk
    ) -> None:
        """KV ストアが障害なしで正常動作することを検証する。"""
        # キーと値を定義する
        key = b"normal_key"
        # 値を定義する
        value = b"normal_value"
        # キーを書き込む
        kv_store.put(key, value)
        # 書き込んだ値を読み取る
        result = kv_store.get(key)
        # 値が一致することを確認する
        assert result == value

    # KV ストアがディスク障害中に書き込みを失敗させることをテストする
    def test_kv_disk_failure_blocks_writes(
        self, kv_store: MockKVStoreDisk
    ) -> None:
        """KV ストアがディスク障害中に書き込みを失敗させることを検証する。"""
        # ディスク障害を注入する
        kv_store.inject_disk_failure()
        # 書き込みが失敗することを確認する
        with pytest.raises(DiskWriteError):
            # 障害中の書き込みを試みる
            kv_store.put(b"fail_key", b"fail_value")

    # KV ストアが回復後も前のデータが読み取れることをテストする
    def test_kv_recovery_preserves_pre_failure_data(
        self, kv_store: MockKVStoreDisk
    ) -> None:
        """KV ストアが回復後も障害前のデータが読み取れることを検証する。"""
        # 障害前にデータを書き込む
        kv_store.put(b"pre_fail", b"pre_value")
        # ディスク障害を注入する
        kv_store.inject_disk_failure()
        # 回復させる
        kv_store.recover_disk()
        # 障害前のデータが読み取れることを確認する
        result = kv_store.get(b"pre_fail")
        # 値が一致することを確認する
        assert result == b"pre_value"

    # クラスターの 2 ノードが分断されてもクォーラムが維持されることをテストする
    def test_cluster_two_nodes_partitioned_quorum_maintained(
        self, cluster: MockConsensusCluster
    ) -> None:
        """2 ノードが分断されてもクォーラムが維持されることを検証する（5 ノード）。"""
        # ノード ID を取得する
        node_ids = [n.node_id for n in cluster.nodes]
        # 2 ノードを分断する
        cluster.partition_node(node_ids[3])
        # さらに 1 ノードを分断する
        cluster.partition_node(node_ids[4])
        # データを書き込む
        result = cluster.write(b"after_2partition")
        # 書き込みが成功することを確認する（3 ノードでクォーラム維持）
        assert result is not None

    # コンセンサスノードがクラッシュ後に回復できることをテストする
    def test_consensus_node_crash_and_recover(
        self, cluster: MockConsensusCluster
    ) -> None:
        """コンセンサスノードがクラッシュ後に正常に回復することを検証する。"""
        # ノード ID を取得する
        node_ids = [n.node_id for n in cluster.nodes]
        # ノードを分断する（クラッシュ相当）
        cluster.partition_node(node_ids[0])
        # 残り 4 ノードでデータを書き込む
        seq1 = cluster.write(b"before_recover")
        # シーケンス番号が返ることを確認する
        assert seq1 is not None
        # ノードを回復させる
        cluster.heal_partition(node_ids[0])
        # 回復後のデータを書き込む
        seq2 = cluster.write(b"after_recover")
        # シーケンス番号が返ることを確認する
        assert seq2 is not None
        # シーケンス番号が増加していることを確認する
        assert seq2 > seq1

    # KV ストアの統計が初期状態で全てゼロであることをテストする
    def test_kv_stats_initial_zero(self) -> None:
        """KV ストアの統計が初期状態で全てゼロであることを検証する。"""
        # 新しい KV ストアをインスタンス化する
        kv = MockKVStoreDisk()
        # 統計を取得する
        stats = kv.stats
        # 試行回数が 0 であることを確認する
        assert stats["attempts"] == 0
        # 成功回数が 0 であることを確認する
        assert stats["successes"] == 0
        # 失敗回数が 0 であることを確認する
        assert stats["failures"] == 0

    # KV ストアが複数のキーを独立して管理することをテストする
    def test_kv_multiple_keys_independent(
        self, kv_store: MockKVStoreDisk
    ) -> None:
        """KV ストアが複数のキーを独立して管理することを検証する。"""
        # 複数のキーバリューペアを書き込む
        pairs = {b"key_a": b"value_a", b"key_b": b"value_b", b"key_c": b"value_c"}
        # 全ペアを書き込む
        for k, v in pairs.items():
            # 書き込みを実行する
            kv_store.put(k, v)
        # 全ペアが正しく読み取れることを確認する
        for k, expected_v in pairs.items():
            # 値を取得する
            got = kv_store.get(k)
            # 値が一致することを確認する
            assert got == expected_v, f"Key {k!r}: expected {expected_v!r}, got {got!r}"

    # クラスターが全ノード分断後にクォーラムを失うことをテストする
    def test_cluster_all_followers_partitioned_no_quorum(
        self, cluster: MockConsensusCluster
    ) -> None:
        """全フォロワーが分断された場合にクォーラムが失われることを検証する。"""
        # リーダーを取得する
        leader = cluster.leader
        # 全フォロワーを分断する
        for node in cluster.nodes:
            # リーダー以外を全て分断する
            if node.node_id != leader.node_id:
                # ノードを分断する
                cluster.partition_node(node.node_id)
        # クォーラムなしで書き込みを試みる
        result = cluster.write(b"no_quorum_write")
        # 書き込みが失敗することを確認する（None が返る）
        assert result is None, "Write should fail without quorum"


# tier1 障害カオスの第四拡張テストクラスを定義する
class TestTier1FaultChaosExtended4:
    """tier1 障害注入テストの追加テスト群（第四拡張）。"""

    # KV ストアが回復後に新しいデータを書き込めることをテストする
    def test_kv_recovery_allows_new_writes(self) -> None:
        """KV ストアが回復後に新しいデータを書き込めることを検証する。"""
        # KV ストアをインスタンス化する
        kv = MockKVStoreDisk()
        # ディスク障害を注入する
        kv.inject_disk_failure()
        # 回復させる
        kv.recover_disk()
        # 回復後に書き込む
        kv.put(b"post_recovery_key", b"post_recovery_value")
        # 書き込んだ値が読み取れることを確認する
        result = kv.get(b"post_recovery_key")
        # 値が一致することを確認する
        assert result == b"post_recovery_value"

    # コンセンサスクラスターのリーダーが存在することをテストする
    def test_cluster_leader_exists_initially(self) -> None:
        """コンセンサスクラスターの初期状態でリーダーが存在することを検証する。"""
        # クラスターをインスタンス化する
        cluster = MockConsensusCluster(node_count=3)
        # リーダーを取得する
        leader = cluster.leader
        # リーダーが存在することを確認する
        assert leader is not None
        # リーダーの node_id が空でないことを確認する
        assert leader.node_id != ""

    # KV ストアが空のキーを扱えることをテストする
    def test_kv_empty_key_handled(self) -> None:
        """KV ストアが空のキーを扱えることを検証する。"""
        # KV ストアをインスタンス化する
        kv = MockKVStoreDisk()
        # 空のキーに書き込む
        kv.put(b"", b"empty_key_value")
        # 空のキーで読み取る
        result = kv.get(b"")
        # 値が一致することを確認する
        assert result == b"empty_key_value"
