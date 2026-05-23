"""src/infra/disk_monitor/monitor.py

ディスク遅延・使用率モニター。
各ノードのディスク I/O レイテンシと使用率を定期的に収集し、
閾値超過時にアラートを発火する。

仕様: docs/04_詳細設計/01_適合仕様/08_infra適合仕様.md §disk_integrity
"""

from __future__ import annotations

import datetime
import json
import os
import pathlib
import statistics
import time
from dataclasses import dataclass, field
from typing import Any


# ディスク I/O サンプルを表すデータクラス
@dataclass
class DiskIOSample:
    """単一ディスク I/O 計測値。"""
    # サンプル採取時刻 (Unix epoch 秒)
    timestamp: float
    # 対象デバイス名（例: sda, nvme0n1）
    device: str
    # 読み取りレイテンシ (マイクロ秒)
    read_latency_us: float
    # 書き込みレイテンシ (マイクロ秒)
    write_latency_us: float
    # 読み取りスループット (bytes/s)
    read_bytes_per_sec: float
    # 書き込みスループット (bytes/s)
    write_bytes_per_sec: float
    # I/O wait 率 (0.0–1.0)
    iowait_ratio: float


# ディスク使用率サンプルを表すデータクラス
@dataclass
class DiskUsageSample:
    """マウントポイントごとのディスク使用率。"""
    # サンプル採取時刻 (Unix epoch 秒)
    timestamp: float
    # マウントポイント (例: /, /var/lib/etcd)
    mountpoint: str
    # 総容量 (bytes)
    total_bytes: int
    # 使用済み容量 (bytes)
    used_bytes: int
    # 使用率 (0.0–1.0)
    usage_ratio: float


# ディスク統計サマリーを表すデータクラス
@dataclass
class DiskStats:
    """複数サンプルから算出したディスク統計。"""
    # 対象デバイス名
    device: str
    # サンプル数
    sample_count: int
    # 読み取りレイテンシ中央値 (マイクロ秒)
    read_latency_p50_us: float
    # 読み取りレイテンシ 99 パーセンタイル (マイクロ秒)
    read_latency_p99_us: float
    # 書き込みレイテンシ中央値 (マイクロ秒)
    write_latency_p50_us: float
    # 書き込みレイテンシ 99 パーセンタイル (マイクロ秒)
    write_latency_p99_us: float
    # 平均 I/O wait 率
    avg_iowait_ratio: float
    # 最大 I/O wait 率
    max_iowait_ratio: float


# アラート条件の設定を表すデータクラス
@dataclass
class DiskAlertPolicy:
    """ディスクアラート発火条件。"""
    # 読み取りレイテンシ p99 閾値 (マイクロ秒)
    read_latency_p99_limit_us: float = 10_000.0
    # 書き込みレイテンシ p99 閾値 (マイクロ秒)
    write_latency_p99_limit_us: float = 20_000.0
    # I/O wait 率上限
    iowait_ratio_limit: float = 0.70
    # ディスク使用率上限
    usage_ratio_limit: float = 0.85
    # アラート発火に必要な連続違反回数
    consecutive_violations_required: int = 3


# アラートイベントを表すデータクラス
@dataclass
class DiskAlert:
    """発火したディスクアラート。"""
    # アラート発火時刻
    fired_at: float
    # 対象デバイスまたはマウントポイント
    target: str
    # アラート種別
    alert_kind: str
    # 実測値
    observed_value: float
    # 閾値
    threshold: float
    # アラートの詳細メッセージ
    message: str


# /proc/diskstats の単一行を表すデータクラス
@dataclass
class _ProcDiskstatsRow:
    """proc/diskstats の解析結果。"""
    # デバイス名
    device: str
    # 読み取り完了数
    reads_completed: int
    # 読み取りセクタ数
    sectors_read: int
    # 読み取りにかかった時間 (ミリ秒)
    read_time_ms: int
    # 書き込み完了数
    writes_completed: int
    # 書き込みセクタ数
    sectors_written: int
    # 書き込みにかかった時間 (ミリ秒)
    write_time_ms: int
    # I/O 処理中の時間 (ミリ秒)
    io_time_ms: int


def _parse_diskstats(path: str = "/proc/diskstats") -> list[_ProcDiskstatsRow]:
    """/proc/diskstats を解析して行リストを返す。"""
    # ファイルが存在しない場合は空リストを返す（テスト環境対応）
    if not os.path.exists(path):
        return []
    # diskstats ファイルを読み込む
    rows: list[_ProcDiskstatsRow] = []
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            # 空行をスキップする
            parts = line.split()
            if len(parts) < 14:
                continue
            # デバイス名を取得する（3列目）
            device = parts[2]
            # パーティション（数字で終わるもの）はスキップする
            if device[-1].isdigit():
                continue
            # 各フィールドを整数に変換する
            try:
                rows.append(_ProcDiskstatsRow(
                    device=device,
                    reads_completed=int(parts[3]),
                    sectors_read=int(parts[5]),
                    read_time_ms=int(parts[6]),
                    writes_completed=int(parts[7]),
                    sectors_written=int(parts[9]),
                    write_time_ms=int(parts[10]),
                    io_time_ms=int(parts[12]),
                ))
            except (ValueError, IndexError):
                # 解析エラーが発生した行はスキップする
                continue
    return rows


def _compute_latency_us(time_ms: int, ops: int) -> float:
    """合計時間とオペレーション数から平均レイテンシ (μs) を計算する。"""
    # オペレーション数が 0 の場合は 0 を返す
    if ops == 0:
        return 0.0
    # ミリ秒からマイクロ秒に変換して平均を算出する
    return (time_ms / ops) * 1000.0


def _read_iowait_ratio() -> float:
    """/proc/stat から CPU I/O wait 率を読み取る。"""
    # /proc/stat が存在しない場合は 0 を返す（テスト環境対応）
    if not os.path.exists("/proc/stat"):
        return 0.0
    with open("/proc/stat", encoding="utf-8") as fh:
        # 最初の cpu 集計行を読む
        line = fh.readline()
    # フィールドを取得する（user, nice, system, idle, iowait, ...）
    parts = line.split()
    if len(parts) < 6:
        return 0.0
    # 各 CPU 時間を整数に変換する
    try:
        user = int(parts[1])
        nice = int(parts[2])
        system = int(parts[3])
        idle = int(parts[4])
        iowait = int(parts[5])
    except (ValueError, IndexError):
        # 変換失敗時は 0 を返す
        return 0.0
    # 全 CPU 時間の合計を算出する
    total = user + nice + system + idle + iowait
    if total == 0:
        return 0.0
    # I/O wait 率を算出して返す
    return iowait / total


def _read_disk_usage(mountpoints: list[str]) -> list[DiskUsageSample]:
    """指定されたマウントポイントのディスク使用率を読み取る。"""
    # 現在のタイムスタンプを取得する
    now = time.time()
    # 結果リストを初期化する
    samples: list[DiskUsageSample] = []
    for mp in mountpoints:
        # マウントポイントが存在しない場合はスキップする
        if not os.path.exists(mp):
            continue
        try:
            # os.statvfs でディスク統計を取得する
            st = os.statvfs(mp)
            # 総容量をバイト単位で算出する
            total = st.f_blocks * st.f_frsize
            # 空き容量をバイト単位で算出する
            free = st.f_bfree * st.f_frsize
            # 使用済み容量を算出する
            used = total - free
            # 使用率を算出する（total が 0 の場合は 0）
            ratio = used / total if total > 0 else 0.0
            samples.append(DiskUsageSample(
                timestamp=now,
                mountpoint=mp,
                total_bytes=total,
                used_bytes=used,
                usage_ratio=ratio,
            ))
        except OSError:
            # アクセスエラーが発生したマウントポイントはスキップする
            continue
    return samples


def _percentile(values: list[float], p: float) -> float:
    """昇順ソート済みリストから p パーセンタイル値を返す。"""
    # 値リストが空の場合は 0 を返す
    if not values:
        return 0.0
    # ソート済みコピーを作成する
    sorted_vals = sorted(values)
    # インデックスを計算する（0-based）
    idx = int(len(sorted_vals) * p / 100)
    # 範囲内にクランプして値を返す
    idx = min(idx, len(sorted_vals) - 1)
    return sorted_vals[idx]


def compute_disk_stats(samples: list[DiskIOSample]) -> list[DiskStats]:
    """I/O サンプルリストからデバイスごとの統計を計算する。"""
    # デバイス別にサンプルをグループ化する
    by_device: dict[str, list[DiskIOSample]] = {}
    for s in samples:
        by_device.setdefault(s.device, []).append(s)
    # デバイスごとに統計を算出する
    result: list[DiskStats] = []
    for device, devsamples in by_device.items():
        # 読み取りレイテンシリストを取得する
        reads = [s.read_latency_us for s in devsamples]
        # 書き込みレイテンシリストを取得する
        writes = [s.write_latency_us for s in devsamples]
        # I/O wait 率リストを取得する
        iowait_vals = [s.iowait_ratio for s in devsamples]
        result.append(DiskStats(
            device=device,
            sample_count=len(devsamples),
            read_latency_p50_us=_percentile(reads, 50),
            read_latency_p99_us=_percentile(reads, 99),
            write_latency_p50_us=_percentile(writes, 50),
            write_latency_p99_us=_percentile(writes, 99),
            avg_iowait_ratio=statistics.mean(iowait_vals) if iowait_vals else 0.0,
            max_iowait_ratio=max(iowait_vals) if iowait_vals else 0.0,
        ))
    return result


class DiskMonitor:
    """ディスク I/O レイテンシおよび使用率のモニター。"""

    # デフォルトのモニタリング対象マウントポイント
    DEFAULT_MOUNTPOINTS: list[str] = ["/", "/var/lib/etcd", "/var/lib/postgresql"]
    # time-series ファイルを保存するデフォルトディレクトリ
    DEFAULT_STORE_DIR: str = "/var/log/k1s0/disk_monitor"

    def __init__(
        self,
        policy: DiskAlertPolicy | None = None,
        mountpoints: list[str] | None = None,
        store_dir: str | None = None,
    ) -> None:
        """モニターを初期化する。"""
        # アラートポリシーを設定する（未指定時はデフォルトを使用）
        self._policy = policy or DiskAlertPolicy()
        # モニタリング対象マウントポイントを設定する
        self._mountpoints = mountpoints or self.DEFAULT_MOUNTPOINTS
        # time-series 保存ディレクトリを設定する
        self._store_dir = pathlib.Path(store_dir or self.DEFAULT_STORE_DIR)
        # I/O サンプルのバッファを初期化する
        self._io_samples: list[DiskIOSample] = []
        # 使用率サンプルのバッファを初期化する
        self._usage_samples: list[DiskUsageSample] = []
        # アラートリストを初期化する
        self._alerts: list[DiskAlert] = []
        # 前回の diskstats スナップショットを保持する変数
        self._prev_diskstats: dict[str, _ProcDiskstatsRow] | None = None
        # 前回のスナップショット採取時刻を保持する変数
        self._prev_ts: float | None = None
        # 連続違反カウンタを初期化する
        self._violation_counters: dict[str, int] = {}

    def collect_once(self) -> list[DiskIOSample]:
        """diskstats を1回読み取り、前回差分から I/O サンプルを生成する。"""
        # 現在時刻を取得する
        now = time.time()
        # 現在の diskstats を読み取る
        current_rows = {r.device: r for r in _parse_diskstats()}
        # 前回スナップショットが存在しない場合は初回として記録する
        if self._prev_diskstats is None or self._prev_ts is None:
            self._prev_diskstats = current_rows
            self._prev_ts = now
            return []
        # 経過時間を算出する（秒）
        elapsed = now - self._prev_ts
        if elapsed <= 0:
            return []
        # I/O wait 率を読み取る
        iowait = _read_iowait_ratio()
        # 差分からサンプルを生成する
        samples: list[DiskIOSample] = []
        for device, cur in current_rows.items():
            prev = self._prev_diskstats.get(device)
            if prev is None:
                continue
            # 読み取りオペレーション差分を算出する
            d_reads = max(cur.reads_completed - prev.reads_completed, 0)
            # 書き込みオペレーション差分を算出する
            d_writes = max(cur.writes_completed - prev.writes_completed, 0)
            # 読み取り時間差分を算出する（ミリ秒）
            d_read_ms = max(cur.read_time_ms - prev.read_time_ms, 0)
            # 書き込み時間差分を算出する（ミリ秒）
            d_write_ms = max(cur.write_time_ms - prev.write_time_ms, 0)
            # 読み取りスループット (bytes/s) を算出する
            d_read_sectors = max(cur.sectors_read - prev.sectors_read, 0)
            read_bps = (d_read_sectors * 512) / elapsed
            # 書き込みスループット (bytes/s) を算出する
            d_write_sectors = max(cur.sectors_written - prev.sectors_written, 0)
            write_bps = (d_write_sectors * 512) / elapsed
            # サンプルを追加する
            samples.append(DiskIOSample(
                timestamp=now,
                device=device,
                read_latency_us=_compute_latency_us(d_read_ms, d_reads),
                write_latency_us=_compute_latency_us(d_write_ms, d_writes),
                read_bytes_per_sec=read_bps,
                write_bytes_per_sec=write_bps,
                iowait_ratio=iowait,
            ))
        # スナップショットを更新する
        self._prev_diskstats = current_rows
        self._prev_ts = now
        # バッファに追加する（最大 3600 サンプル保持）
        self._io_samples.extend(samples)
        if len(self._io_samples) > 3600:
            self._io_samples = self._io_samples[-3600:]
        return samples

    def collect_usage(self) -> list[DiskUsageSample]:
        """マウントポイントのディスク使用率を収集する。"""
        # 使用率を読み取る
        samples = _read_disk_usage(self._mountpoints)
        # バッファに追加する
        self._usage_samples.extend(samples)
        if len(self._usage_samples) > 1440:
            self._usage_samples = self._usage_samples[-1440:]
        return samples

    def check_alerts(self, stats: list[DiskStats]) -> list[DiskAlert]:
        """統計値をポリシーと照合してアラートを発火する。"""
        # アラートリストを初期化する
        new_alerts: list[DiskAlert] = []
        for s in stats:
            # 読み取りレイテンシ p99 を確認する
            if s.read_latency_p99_us > self._policy.read_latency_p99_limit_us:
                key = f"{s.device}:read_latency"
                self._violation_counters[key] = self._violation_counters.get(key, 0) + 1
                # 連続違反回数が閾値を超えた場合のみアラートを発火する
                if self._violation_counters[key] >= self._policy.consecutive_violations_required:
                    new_alerts.append(DiskAlert(
                        fired_at=time.time(),
                        target=s.device,
                        alert_kind="read_latency_p99_exceeded",
                        observed_value=s.read_latency_p99_us,
                        threshold=self._policy.read_latency_p99_limit_us,
                        message=(
                            f"disk {s.device}: read p99={s.read_latency_p99_us:.1f}µs "
                            f"> limit={self._policy.read_latency_p99_limit_us:.1f}µs"
                        ),
                    ))
            else:
                # 違反がなければカウンタをリセットする
                self._violation_counters.pop(f"{s.device}:read_latency", None)
            # 書き込みレイテンシ p99 を確認する
            if s.write_latency_p99_us > self._policy.write_latency_p99_limit_us:
                key = f"{s.device}:write_latency"
                self._violation_counters[key] = self._violation_counters.get(key, 0) + 1
                if self._violation_counters[key] >= self._policy.consecutive_violations_required:
                    new_alerts.append(DiskAlert(
                        fired_at=time.time(),
                        target=s.device,
                        alert_kind="write_latency_p99_exceeded",
                        observed_value=s.write_latency_p99_us,
                        threshold=self._policy.write_latency_p99_limit_us,
                        message=(
                            f"disk {s.device}: write p99={s.write_latency_p99_us:.1f}µs "
                            f"> limit={self._policy.write_latency_p99_limit_us:.1f}µs"
                        ),
                    ))
            else:
                self._violation_counters.pop(f"{s.device}:write_latency", None)
            # I/O wait 率を確認する
            if s.avg_iowait_ratio > self._policy.iowait_ratio_limit:
                key = f"{s.device}:iowait"
                self._violation_counters[key] = self._violation_counters.get(key, 0) + 1
                if self._violation_counters[key] >= self._policy.consecutive_violations_required:
                    new_alerts.append(DiskAlert(
                        fired_at=time.time(),
                        target=s.device,
                        alert_kind="iowait_ratio_exceeded",
                        observed_value=s.avg_iowait_ratio,
                        threshold=self._policy.iowait_ratio_limit,
                        message=(
                            f"disk {s.device}: avg_iowait={s.avg_iowait_ratio:.1%} "
                            f"> limit={self._policy.iowait_ratio_limit:.1%}"
                        ),
                    ))
            else:
                self._violation_counters.pop(f"{s.device}:iowait", None)
        # アラートリストに追加する
        self._alerts.extend(new_alerts)
        return new_alerts

    def check_usage_alerts(self, samples: list[DiskUsageSample]) -> list[DiskAlert]:
        """使用率サンプルをポリシーと照合してアラートを発火する。"""
        # アラートリストを初期化する
        new_alerts: list[DiskAlert] = []
        for s in samples:
            # 使用率上限を確認する
            if s.usage_ratio > self._policy.usage_ratio_limit:
                key = f"{s.mountpoint}:usage"
                self._violation_counters[key] = self._violation_counters.get(key, 0) + 1
                if self._violation_counters[key] >= self._policy.consecutive_violations_required:
                    new_alerts.append(DiskAlert(
                        fired_at=time.time(),
                        target=s.mountpoint,
                        alert_kind="disk_usage_exceeded",
                        observed_value=s.usage_ratio,
                        threshold=self._policy.usage_ratio_limit,
                        message=(
                            f"mountpoint {s.mountpoint}: usage={s.usage_ratio:.1%} "
                            f"> limit={self._policy.usage_ratio_limit:.1%}"
                        ),
                    ))
            else:
                self._violation_counters.pop(f"{s.mountpoint}:usage", None)
        # アラートリストに追加する
        self._alerts.extend(new_alerts)
        return new_alerts

    def persist_timeseries(self, samples: list[DiskIOSample]) -> None:
        """I/O サンプルを時系列 JSONL ファイルに書き出す。"""
        # サンプルが空の場合はスキップする
        if not samples:
            return
        # 保存ディレクトリを作成する
        self._store_dir.mkdir(parents=True, exist_ok=True)
        # 現在時刻から時間単位のファイル名を生成する
        hour_str = datetime.datetime.utcfromtimestamp(samples[0].timestamp).strftime("%Y%m%d_%H")
        fpath = self._store_dir / f"disk_io_{hour_str}.jsonl"
        # JSONL 形式で追記する
        with fpath.open("a", encoding="utf-8") as fh:
            for s in samples:
                fh.write(json.dumps({
                    "ts": s.timestamp,
                    "device": s.device,
                    "read_lat_us": s.read_latency_us,
                    "write_lat_us": s.write_latency_us,
                    "read_bps": s.read_bytes_per_sec,
                    "write_bps": s.write_bytes_per_sec,
                    "iowait": s.iowait_ratio,
                }) + "\n")

    def run_once(self) -> dict[str, Any]:
        """I/O 収集・使用率収集・アラートチェックを一括実行する。"""
        # I/O サンプルを収集する
        io_samples = self.collect_once()
        # ディスク使用率を収集する
        usage_samples = self.collect_usage()
        # I/O 統計を算出する
        stats = compute_disk_stats(io_samples) if io_samples else []
        # I/O アラートを確認する
        io_alerts = self.check_alerts(stats)
        # 使用率アラートを確認する
        usage_alerts = self.check_usage_alerts(usage_samples)
        # サンプルを永続化する
        self.persist_timeseries(io_samples)
        # 結果サマリーを返す
        return {
            "io_samples": len(io_samples),
            "usage_samples": len(usage_samples),
            "new_alerts": len(io_alerts) + len(usage_alerts),
            "total_alerts": len(self._alerts),
            "stats": [
                {
                    "device": s.device,
                    "read_p99_us": s.read_latency_p99_us,
                    "write_p99_us": s.write_latency_p99_us,
                    "iowait_avg": s.avg_iowait_ratio,
                }
                for s in stats
            ],
        }

    def get_alerts(self) -> list[DiskAlert]:
        """蓄積されたアラートリストを返す。"""
        # アラートリストのコピーを返す
        return list(self._alerts)

    def clear_alerts(self) -> None:
        """アラートリストをクリアする。"""
        # アラートリストを空にする
        self._alerts.clear()
