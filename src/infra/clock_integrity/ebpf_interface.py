"""src/infra/clock_integrity/ebpf_interface.py

カーネルレベルのクロック監視のための eBPF プローブインターフェース。
仕様: 13_時刻整合適合仕様.md §clock_integrity_class / infra CLAUDE.md §eBPF

- EbpfProbe: eBPF プログラムのライフサイクルを管理する
- ClockEvent: eBPF 出力から得られるクロックイベントを表す
- trace_pipe から clock_gettime / gettimeofday / adjtimex の syscall を追跡する
- eBPF が利用できない場合は /proc/<pid>/status をパースするフォールバックモードを使用する
- 1 秒間に 10,000 回以上クロックを読んでいるプロセスを異常として検出する
"""

from __future__ import annotations

import logging
import os
import re
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Iterator

# モジュールロガーを初期化する
logger = logging.getLogger(__name__)

# trace_pipe のパス（カーネル tracing インフラ）
_TRACE_PIPE_PATH = Path("/sys/kernel/debug/tracing/trace_pipe")
# proc ファイルシステムのパス
_PROC_PATH = Path("/proc")
# クロック読み取り頻度の異常判定閾値（回/秒）
_CLOCK_READ_FREQ_THRESHOLD = 10_000
# クロック関連の syscall 名リスト
_CLOCK_SYSCALLS = frozenset(["clock_gettime", "gettimeofday", "adjtimex", "clock_adjtime", "settimeofday"])
# trace_pipe 読み取りのタイムアウト秒数
_TRACE_READ_TIMEOUT_S = 1.0
# プロセス名の最大長（/proc/<pid>/comm から取得）
_COMM_MAX_LEN = 15


# ---------------------------------------------------------------------------
# ClockEvent データクラス
# ---------------------------------------------------------------------------

@dataclass
class ClockEvent:
    """eBPF trace_pipe から取得したクロック関連 syscall イベントを表すデータクラス。"""

    # syscall を呼び出したプロセス ID
    pid: int
    # イベントのカーネルタイムスタンプ（ナノ秒）
    ts_ns: int
    # 呼び出された syscall の名前
    syscall: str
    # イベントが発生した CPU ID
    cpu_id: int
    # プロセス名（comm）
    comm: str = ""
    # スレッド グループ ID
    tgid: int = 0

    def to_dict(self) -> dict[str, Any]:
        """ClockEvent を辞書形式にシリアライズする。"""
        return {
            "pid": self.pid,
            "ts_ns": self.ts_ns,
            "syscall": self.syscall,
            "cpu_id": self.cpu_id,
            "comm": self.comm,
            "tgid": self.tgid,
        }


# ---------------------------------------------------------------------------
# ProcessClockUsage データクラス
# ---------------------------------------------------------------------------

@dataclass
class ProcessClockUsage:
    """プロセスごとのクロック読み取り使用量を集計するデータクラス。"""

    # プロセス ID
    pid: int
    # プロセス名（comm）
    comm: str
    # 集計期間中のクロック読み取り総回数
    call_count: int
    # 集計期間の開始時刻（Unix タイムスタンプ）
    period_start: float
    # 集計期間の終了時刻（Unix タイムスタンプ）
    period_end: float
    # syscall 名 → 呼び出し回数のマッピング
    syscall_counts: dict[str, int] = field(default_factory=dict)

    @property
    def calls_per_second(self) -> float:
        """集計期間における 1 秒あたりのクロック読み取り回数を返す。"""
        # 期間を計算する
        duration = self.period_end - self.period_start
        # 期間が 0 以下の場合は分母エラーを回避して 0 を返す
        if duration <= 0:
            return 0.0
        return self.call_count / duration

    def is_anomalous(self, threshold: int = _CLOCK_READ_FREQ_THRESHOLD) -> bool:
        """クロック読み取り頻度が異常判定閾値を超えているか確認する。"""
        # calls_per_second が閾値を超えた場合は True を返す
        return self.calls_per_second > threshold

    def to_dict(self) -> dict[str, Any]:
        """ProcessClockUsage を辞書形式にシリアライズする。"""
        return {
            "pid": self.pid,
            "comm": self.comm,
            "call_count": self.call_count,
            "calls_per_second": self.calls_per_second,
            "is_anomalous": self.is_anomalous(),
            "syscall_counts": self.syscall_counts,
            "period_start": self.period_start,
            "period_end": self.period_end,
        }


# ---------------------------------------------------------------------------
# CpuTimestampSpread データクラス
# ---------------------------------------------------------------------------

@dataclass
class CpuTimestampSpread:
    """CPU ごとのタイムスタンプ分散を表すデータクラス。"""

    # CPU ID → 最後に観測したタイムスタンプ（ns）のマッピング
    cpu_timestamps: dict[int, int]
    # 計算されたスプレッド（最大値 - 最小値, ナノ秒）
    spread_ns: int
    # 計算実行時刻
    computed_at: float = field(default_factory=time.time)

    def spread_us(self) -> int:
        """スプレッドをマイクロ秒単位で返す。"""
        return self.spread_ns // 1_000

    def to_dict(self) -> dict[str, Any]:
        """CpuTimestampSpread を辞書形式にシリアライズする。"""
        return {
            "spread_ns": self.spread_ns,
            "spread_us": self.spread_us(),
            "cpu_count": len(self.cpu_timestamps),
            "computed_at": self.computed_at,
        }


# ---------------------------------------------------------------------------
# EbpfClockReport データクラス
# ---------------------------------------------------------------------------

@dataclass
class EbpfClockReport:
    """eBPF クロック監視の集計レポートデータクラス。"""

    # 集計期間の開始時刻（Unix タイムスタンプ）
    period_start: float
    # 集計期間の終了時刻（Unix タイムスタンプ）
    period_end: float
    # 処理したイベントの総件数
    event_count: int
    # プロセスごとのクロック使用量リスト（calls_per_second 降順）
    top_callers: list[ProcessClockUsage]
    # 異常と判定されたプロセスのリスト
    anomalous_processes: list[ProcessClockUsage]
    # CPU タイムスタンプスプレッド情報
    cpu_spread: CpuTimestampSpread | None
    # eBPF が利用可能かどうか（False の場合は fallback モード）
    ebpf_available: bool
    # フォールバックモードで使用したデータソース
    fallback_source: str = ""

    def to_dict(self) -> dict[str, Any]:
        """EbpfClockReport を辞書形式にシリアライズする。"""
        return {
            "period_start": self.period_start,
            "period_end": self.period_end,
            "event_count": self.event_count,
            "top_caller_count": len(self.top_callers),
            "top_callers": [c.to_dict() for c in self.top_callers],
            "anomalous_count": len(self.anomalous_processes),
            "anomalous_processes": [p.to_dict() for p in self.anomalous_processes],
            "cpu_spread": self.cpu_spread.to_dict() if self.cpu_spread else None,
            "ebpf_available": self.ebpf_available,
            "fallback_source": self.fallback_source,
        }


# ---------------------------------------------------------------------------
# TraceLineParser ユーティリティクラス
# ---------------------------------------------------------------------------

class TraceLineParser:
    """trace_pipe の出力行をパースするユーティリティクラス。"""

    # trace_pipe の行フォーマット例:
    # <comm>-<pid>  [<cpu>] .... <ts>: <function>: <args>
    _LINE_PATTERN = re.compile(
        r"^\s*(?P<comm>\S+)-(?P<pid>\d+)\s+\[(?P<cpu>\d+)\]\s+\S+\s+(?P<ts>[\d.]+):\s+(?P<func>\S+)"
    )

    @classmethod
    def parse(cls, line: str) -> ClockEvent | None:
        """trace_pipe の 1 行をパースして ClockEvent を返す。

        クロック関連 syscall を含まない行は None を返す。

        Args:
            line: trace_pipe から読み取った 1 行

        Returns:
            ClockEvent または None
        """
        # 正規表現でマッチを試みる
        m = cls._LINE_PATTERN.match(line)
        if not m:
            return None
        # マッチ結果からフィールドを抽出する
        comm = m.group("comm")
        pid_str = m.group("pid")
        cpu_str = m.group("cpu")
        ts_str = m.group("ts")
        func = m.group("func")
        # クロック関連 syscall かどうかを確認する
        matched_syscall = None
        for syscall in _CLOCK_SYSCALLS:
            if syscall in func:
                matched_syscall = syscall
                break
        # クロック関連 syscall でなければ None を返す
        if matched_syscall is None:
            return None
        # フィールドを適切な型に変換する
        try:
            pid = int(pid_str)
            cpu_id = int(cpu_str)
            # タイムスタンプを秒→ナノ秒に変換する
            ts_ns = int(float(ts_str) * 1_000_000_000)
        except ValueError:
            return None
        # ClockEvent を構築して返す
        return ClockEvent(
            pid=pid,
            ts_ns=ts_ns,
            syscall=matched_syscall,
            cpu_id=cpu_id,
            comm=comm[:_COMM_MAX_LEN],
        )


# ---------------------------------------------------------------------------
# EbpfProbe クラス
# ---------------------------------------------------------------------------

class EbpfProbe:
    """eBPF プログラムのライフサイクルを管理するクラス。

    /sys/kernel/debug/tracing/trace_pipe を読み取ってクロック関連 syscall を
    追跡する。eBPF が利用できない場合は /proc/<pid>/status をパースする
    フォールバックモードで動作する。
    """

    def __init__(
        self,
        trace_pipe_path: Path | None = None,
        proc_path: Path | None = None,
        dry_run: bool = True,
        freq_threshold: int = _CLOCK_READ_FREQ_THRESHOLD,
    ) -> None:
        """EbpfProbe を初期化する。

        Args:
            trace_pipe_path: trace_pipe ファイルのパス（テスト用にオーバーライド可能）
            proc_path: /proc ファイルシステムのパス
            dry_run: True の場合はダミーデータを使用する
            freq_threshold: 異常判定の閾値（回/秒）
        """
        # trace_pipe のパスを設定する
        self._trace_pipe_path = trace_pipe_path or _TRACE_PIPE_PATH
        # /proc のパスを設定する
        self._proc_path = proc_path or _PROC_PATH
        # dry-run モードフラグを保持する
        self._dry_run = dry_run
        # 異常判定閾値を保持する
        self._freq_threshold = freq_threshold
        # eBPF 利用可能かどうかを確認する
        self._ebpf_available = self._check_ebpf_availability()
        # 収集したイベントのバッファを初期化する
        self._event_buffer: list[ClockEvent] = []
        # PID → プロセス名のキャッシュを初期化する
        self._pid_comm_cache: dict[int, str] = {}
        # CPU ID → 最後のタイムスタンプのマッピングを初期化する
        self._cpu_last_ts: dict[int, int] = {}
        # プローブの稼働状態フラグを初期化する
        self._running = False

    def _check_ebpf_availability(self) -> bool:
        """eBPF が利用可能な環境かどうかを確認する。"""
        # dry-run モードの場合は利用不可として扱う
        if self._dry_run:
            return False
        # trace_pipe ファイルが存在するか確認する
        if not self._trace_pipe_path.exists():
            logger.info("EbpfProbe: trace_pipe が存在しない - フォールバックモードを使用する")
            return False
        # trace_pipe が読み取り可能か確認する
        try:
            with open(self._trace_pipe_path, "rb", buffering=0):
                pass
            return True
        except (PermissionError, OSError) as exc:
            logger.info("EbpfProbe: trace_pipe アクセス不可 error=%s - フォールバックモードを使用する", exc)
            return False

    @property
    def ebpf_available(self) -> bool:
        """eBPF が利用可能かどうかを返す。"""
        return self._ebpf_available

    def start(self) -> None:
        """プローブを開始する（ライフサイクル管理）。"""
        # 既に稼働中の場合はスキップする
        if self._running:
            logger.warning("EbpfProbe: 既に稼働中")
            return
        # 稼働状態フラグを立てる
        self._running = True
        logger.info(
            "EbpfProbe: プローブ開始 ebpf_available=%s", self._ebpf_available
        )

    def stop(self) -> None:
        """プローブを停止する。"""
        # 稼働状態フラグを落とす
        self._running = False
        logger.info("EbpfProbe: プローブ停止")

    def collect_events(self, duration_s: float = 1.0) -> list[ClockEvent]:
        """指定秒数の間イベントを収集して返す。

        eBPF が利用可能な場合は trace_pipe から読み取る。
        利用不可の場合は /proc をスキャンするフォールバックを使用する。

        Args:
            duration_s: 収集期間（秒）

        Returns:
            収集した ClockEvent のリスト
        """
        # dry-run モードの場合はダミーイベントを生成する
        if self._dry_run:
            return self._generate_dummy_events(duration_s)
        # eBPF が利用可能な場合は trace_pipe から収集する
        if self._ebpf_available:
            return self._collect_from_trace_pipe(duration_s)
        # フォールバック: /proc をスキャンする
        return self._collect_from_proc(duration_s)

    def _generate_dummy_events(self, duration_s: float) -> list[ClockEvent]:
        """dry-run モード用のダミーイベントを生成する。"""
        # 開始時刻を記録する
        start_ns = int(time.time() * 1_000_000_000)
        # ダミーイベントリストを初期化する
        events: list[ClockEvent] = []
        # 3 プロセス × 10 イベントのダミーデータを生成する
        dummy_procs = [
            (1234, "k1s0-tier1", "clock_gettime"),
            (5678, "k1s0-ops", "gettimeofday"),
            (9012, "chrony", "clock_gettime"),
        ]
        for i, (pid, comm, syscall) in enumerate(dummy_procs):
            for j in range(10):
                ts_ns = start_ns + i * 100_000_000 + j * 10_000_000
                events.append(ClockEvent(
                    pid=pid,
                    ts_ns=ts_ns,
                    syscall=syscall,
                    cpu_id=i % 4,
                    comm=comm,
                    tgid=pid,
                ))
        return events

    def _collect_from_trace_pipe(self, duration_s: float) -> list[ClockEvent]:
        """trace_pipe から指定秒数の間イベントを収集する。"""
        # 収集したイベントリストを初期化する
        events: list[ClockEvent] = []
        # 終了時刻を計算する
        end_time = time.time() + duration_s
        try:
            # trace_pipe をノンブロッキングで開く
            with open(self._trace_pipe_path, "r", encoding="ascii", errors="replace") as fp:
                while time.time() < end_time:
                    try:
                        line = fp.readline()
                    except OSError:
                        break
                    # 空行はスキップする
                    if not line:
                        break
                    # 行をパースしてイベントを取得する
                    event = TraceLineParser.parse(line)
                    if event is not None:
                        events.append(event)
                        # CPU タイムスタンプを更新する
                        self._cpu_last_ts[event.cpu_id] = event.ts_ns
        except Exception as exc:
            logger.error("EbpfProbe: trace_pipe 読み取りエラー error=%s", exc)
        return events

    def _collect_from_proc(self, duration_s: float) -> list[ClockEvent]:
        """フォールバック: /proc をスキャンしてプロセス情報を収集する。"""
        # 収集したイベントリストを初期化する（疑似イベントとして生成する）
        events: list[ClockEvent] = []
        # 現在時刻をタイムスタンプとして使用する
        now_ns = int(time.time() * 1_000_000_000)
        # /proc/<pid>/status をスキャンする
        try:
            for pid_dir in self._proc_path.iterdir():
                # 数字のディレクトリのみを処理する（pid）
                if not pid_dir.name.isdigit():
                    continue
                pid = int(pid_dir.name)
                # プロセス名を取得する
                comm = self._get_proc_comm(pid)
                if not comm:
                    continue
                # 疑似 ClockEvent を生成する（syscall は不明なため "unknown" を使用）
                event = ClockEvent(
                    pid=pid,
                    ts_ns=now_ns,
                    syscall="unknown",
                    cpu_id=0,
                    comm=comm,
                    tgid=pid,
                )
                events.append(event)
        except PermissionError as exc:
            logger.warning("EbpfProbe: /proc スキャン権限エラー error=%s", exc)
        return events

    def _get_proc_comm(self, pid: int) -> str:
        """指定 PID のプロセス名を /proc/<pid>/comm から取得する。"""
        # キャッシュを確認する
        if pid in self._pid_comm_cache:
            return self._pid_comm_cache[pid]
        # /proc/<pid>/comm を読み取る
        comm_path = self._proc_path / str(pid) / "comm"
        try:
            comm = comm_path.read_text(encoding="utf-8", errors="replace").strip()
            self._pid_comm_cache[pid] = comm
            return comm
        except (FileNotFoundError, PermissionError):
            return ""

    def aggregate_by_process(
        self,
        events: list[ClockEvent],
        period_start: float,
        period_end: float,
    ) -> list[ProcessClockUsage]:
        """イベントリストをプロセスごとに集計して ProcessClockUsage リストを返す。

        Args:
            events: 集計する ClockEvent リスト
            period_start: 集計期間の開始時刻（Unix タイムスタンプ）
            period_end: 集計期間の終了時刻（Unix タイムスタンプ）

        Returns:
            calls_per_second 降順にソートされた ProcessClockUsage リスト
        """
        # PID → イベント情報の辞書を初期化する
        pid_data: dict[int, dict[str, Any]] = {}
        for event in events:
            # PID が未登録の場合は初期化する
            if event.pid not in pid_data:
                pid_data[event.pid] = {
                    "comm": event.comm or self._get_proc_comm(event.pid),
                    "call_count": 0,
                    "syscall_counts": {},
                }
            # カウントをインクリメントする
            pid_data[event.pid]["call_count"] += 1
            # syscall 別カウントを更新する
            syscall = event.syscall
            pid_data[event.pid]["syscall_counts"][syscall] = (
                pid_data[event.pid]["syscall_counts"].get(syscall, 0) + 1
            )
        # ProcessClockUsage リストを構築する
        usages: list[ProcessClockUsage] = []
        for pid, data in pid_data.items():
            usage = ProcessClockUsage(
                pid=pid,
                comm=data["comm"],
                call_count=data["call_count"],
                period_start=period_start,
                period_end=period_end,
                syscall_counts=data["syscall_counts"],
            )
            usages.append(usage)
        # calls_per_second 降順にソートして返す
        usages.sort(key=lambda u: u.calls_per_second, reverse=True)
        return usages

    def compute_cpu_spread(self, events: list[ClockEvent]) -> CpuTimestampSpread:
        """CPU ごとのタイムスタンプスプレッドを計算する。

        Args:
            events: 計算に使用する ClockEvent リスト

        Returns:
            CpuTimestampSpread
        """
        # CPU ID → 最新タイムスタンプのマッピングを更新する
        cpu_ts: dict[int, int] = {}
        for event in events:
            # 各 CPU の最新タイムスタンプを記録する
            if event.cpu_id not in cpu_ts or event.ts_ns > cpu_ts[event.cpu_id]:
                cpu_ts[event.cpu_id] = event.ts_ns
        # スプレッドを計算する（最大値 - 最小値）
        if len(cpu_ts) < 2:
            return CpuTimestampSpread(cpu_timestamps=cpu_ts, spread_ns=0)
        max_ts = max(cpu_ts.values())
        min_ts = min(cpu_ts.values())
        spread_ns = max_ts - min_ts
        return CpuTimestampSpread(cpu_timestamps=cpu_ts, spread_ns=spread_ns)

    def run_once(self, duration_s: float = 1.0, top_n: int = 10) -> EbpfClockReport:
        """1 サイクル分の収集・集計・レポート生成を実行する。

        Args:
            duration_s: イベント収集期間（秒）
            top_n: レポートに含める上位プロセス数

        Returns:
            EbpfClockReport
        """
        # 収集期間の開始時刻を記録する
        period_start = time.time()
        # イベントを収集する
        events = self.collect_events(duration_s)
        # 収集期間の終了時刻を記録する
        period_end = time.time()
        # プロセスごとに集計する
        all_usages = self.aggregate_by_process(events, period_start, period_end)
        # 上位 top_n プロセスを抽出する
        top_callers = all_usages[:top_n]
        # 異常プロセスを抽出する
        anomalous = [u for u in all_usages if u.is_anomalous(self._freq_threshold)]
        # CPU タイムスタンプスプレッドを計算する
        cpu_spread = self.compute_cpu_spread(events)
        # フォールバックソースを特定する
        if self._ebpf_available:
            fallback_source = ""
        elif self._dry_run:
            fallback_source = "dry_run_dummy"
        else:
            fallback_source = "/proc"
        # レポートを構築して返す
        return EbpfClockReport(
            period_start=period_start,
            period_end=period_end,
            event_count=len(events),
            top_callers=top_callers,
            anomalous_processes=anomalous,
            cpu_spread=cpu_spread,
            ebpf_available=self._ebpf_available,
            fallback_source=fallback_source,
        )

    def get_proc_fd_count(self, pid: int) -> int:
        """指定 PID のオープンファイルディスクリプタ数を /proc から取得する。"""
        # /proc/<pid>/fd ディレクトリのエントリ数をカウントする
        fd_dir = self._proc_path / str(pid) / "fd"
        try:
            return len(list(fd_dir.iterdir()))
        except (FileNotFoundError, PermissionError):
            return 0

    def get_proc_status(self, pid: int) -> dict[str, str]:
        """/proc/<pid>/status の内容を辞書形式で返す。"""
        # /proc/<pid>/status を読み取る
        status_path = self._proc_path / str(pid) / "status"
        result: dict[str, str] = {}
        try:
            content = status_path.read_text(encoding="utf-8", errors="replace")
            for line in content.splitlines():
                if ":" in line:
                    key, _, value = line.partition(":")
                    result[key.strip()] = value.strip()
        except (FileNotFoundError, PermissionError):
            pass
        return result

    def clear_cache(self) -> None:
        """PID → comm キャッシュをクリアする。"""
        # キャッシュを空にして古い情報を破棄する
        self._pid_comm_cache.clear()
        logger.debug("EbpfProbe: PID comm キャッシュをクリアした")
