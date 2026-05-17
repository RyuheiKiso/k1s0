// clock_skew_probe.bpf.c: カーネルレベルの時刻ずれ検知 BPF プログラム
// kprobe で clock_gettime をフックして時刻値を監視し、perf_event_output でユーザ空間に通知する
// SPDX-License-Identifier: GPL-2.0

// Linux BPF ヘルパー関数の定義をインクルードする
#include <linux/bpf.h>
// BPF ヘルパーマクロ群（bpf_printk, bpf_map_lookup_elem 等）をインクルードする
#include <bpf/bpf_helpers.h>
// BPF トレーシング用ヘルパー（BPF_KRETPROBE 等）をインクルードする
#include <bpf/bpf_tracing.h>
// Linux カーネルの time 構造体定義をインクルードする
#include <linux/time.h>

// ライセンス宣言: GPL v2 でないとカーネル内の GPL-only シンボルにアクセスできない
char LICENSE[] SEC("license") = "GPL";

// ============================================================
// 定数定義: 時刻ずれ検知のしきい値と設定値を宣言する
// ============================================================
// 時刻ずれを検知するしきい値（ナノ秒単位）: 1ms = 1,000,000 ns
#define CLOCK_SKEW_THRESHOLD_NS 1000000ULL
// perf_event ringbuf の最大エントリ数を定義する
#define MAX_ENTRIES 8192
// CPU 数の上限を定義する（perf_event array のサイズに使用する）
#define MAX_CPUS 256

// ============================================================
// イベント構造体: ユーザ空間に通知するデータ形式を定義する
// ============================================================
// clock skew 検知イベントのデータ構造を定義する
struct clock_skew_event {
    // イベントが発生した時刻（カーネル単調時刻、ナノ秒）
    __u64 ktime_ns;
    // 前回の clock_gettime 呼び出し時刻（ナノ秒）
    __u64 prev_time_ns;
    // 今回の clock_gettime 呼び出し時刻（ナノ秒）
    __u64 curr_time_ns;
    // 計算された時刻ずれ量（ナノ秒、負値は逆行を示す）
    __s64 skew_ns;
    // イベントが発生した CPU 番号
    __u32 cpu_id;
    // clock_gettime を呼び出したプロセスの PID
    __u32 pid;
    // 検知済みであることを示すフラグ（1=detected, 0=normal）
    __u8  detected;
};

// ============================================================
// BPF マップ定義: カーネル-ユーザ空間間のデータ共有領域を宣言する
// ============================================================
// 前回の時刻値を CPU ごとに保持するハッシュマップを定義する
struct {
    // BPF マップタイプを PERCPU_HASH に設定する（CPU ごとに独立した値を持つ）
    __uint(type, BPF_MAP_TYPE_PERCPU_HASH);
    // キー: CPU 番号（u32）
    __uint(key_size, sizeof(__u32));
    // 値: 前回の時刻（u64、ナノ秒）
    __uint(value_size, sizeof(__u64));
    // マップの最大エントリ数を定義する
    __uint(max_entries, MAX_CPUS);
} prev_time_map SEC(".maps");

// perf_event を介してユーザ空間にイベントを送信するリングバッファを定義する
struct {
    // BPF マップタイプを PERF_EVENT_ARRAY に設定する
    __uint(type, BPF_MAP_TYPE_PERF_EVENT_ARRAY);
    // キー: CPU 番号（u32）
    __uint(key_size, sizeof(__u32));
    // 値: ファイルディスクリプタ（u32）
    __uint(value_size, sizeof(__u32));
    // perf event エントリ数（CPU 数に対応する）
    __uint(max_entries, MAX_CPUS);
} perf_events SEC(".maps");

// 検知した時刻ずれの統計カウンタを定義する
struct {
    // BPF マップタイプを ARRAY に設定する（単純なカウンタ配列）
    __uint(type, BPF_MAP_TYPE_ARRAY);
    // キー: カウンタ ID（u32）
    __uint(key_size, sizeof(__u32));
    // 値: カウント値（u64）
    __uint(value_size, sizeof(__u64));
    // カウンタの種類数（0=total_calls, 1=skew_detected）
    __uint(max_entries, 2);
} stats_map SEC(".maps");

// ============================================================
// ヘルパー関数: 統計カウンタをアトミックにインクリメントする
// ============================================================
// 指定したカウンタ ID の値を 1 増やすヘルパー関数
static __always_inline void increment_counter(__u32 counter_id) {
    // マップから現在のカウンタ値を取得する
    __u64 *val = bpf_map_lookup_elem(&stats_map, &counter_id);
    // 値が存在する場合のみインクリメントする（NULL ポインタを防ぐ）
    if (val) {
        // アトミック加算でカウンタを 1 増やす
        __sync_fetch_and_add(val, 1);
    }
}

// ============================================================
// kprobe ハンドラ: clock_gettime のエントリポイントをフックする
// ============================================================
// clock_gettime システムコールの開始時に呼び出される BPF プログラムを定義する
SEC("kprobe/__x64_sys_clock_gettime")
int BPF_KPROBE(probe_clock_gettime)
{
    // 現在の CPU 番号を取得する
    __u32 cpu = bpf_get_smp_processor_id();
    // 現在の単調時刻をナノ秒で取得する（カーネルブート後の経過時間）
    __u64 curr_time = bpf_ktime_get_ns();
    // 現在のプロセス ID を取得する（上位 32bit が PID）
    __u32 pid = bpf_get_current_pid_tgid() >> 32;

    // total_calls カウンタをインクリメントする
    increment_counter(0);

    // 前回の時刻値をマップから取得する（初回は NULL になる）
    __u64 *prev_time_ptr = bpf_map_lookup_elem(&prev_time_map, &cpu);
    // 前回の時刻が存在する場合のみ時刻ずれを計算する
    if (prev_time_ptr) {
        // 前回の時刻値をローカル変数にコピーする
        __u64 prev_time = *prev_time_ptr;
        // 今回と前回の差分をナノ秒で計算する
        __s64 delta = (__s64)(curr_time - prev_time);

        // 時刻が逆行している場合（delta < 0）または閾値を超えた場合に検知する
        if (delta < 0 || (__u64)delta > CLOCK_SKEW_THRESHOLD_NS) {
            // clock skew 検知イベント構造体をスタック上に初期化する
            struct clock_skew_event event = {};
            // イベント発生時刻（カーネル単調時刻）を記録する
            event.ktime_ns = curr_time;
            // 前回の時刻値を記録する
            event.prev_time_ns = prev_time;
            // 今回の時刻値を記録する
            event.curr_time_ns = curr_time;
            // 計算された時刻ずれを記録する
            event.skew_ns = delta;
            // イベントが発生した CPU 番号を記録する
            event.cpu_id = cpu;
            // clock_gettime を呼び出したプロセスの PID を記録する
            event.pid = pid;
            // 検知フラグを 1 に設定する
            event.detected = 1;

            // perf_event_output でユーザ空間にイベントを送信する
            bpf_perf_event_output(ctx, &perf_events, BPF_F_CURRENT_CPU,
                                  &event, sizeof(event));
            // skew_detected カウンタをインクリメントする
            increment_counter(1);
        }
    }

    // 今回の時刻値を次回比較用にマップに保存する
    bpf_map_update_elem(&prev_time_map, &cpu, &curr_time, BPF_ANY);
    // eBPF プログラムの戻り値（0 は正常終了を示す）
    return 0;
}

// ============================================================
// kretprobe ハンドラ: clock_gettime のリターンポイントをフックする
// ============================================================
// clock_gettime システムコールの戻り時に呼び出される BPF プログラムを定義する
SEC("kretprobe/__x64_sys_clock_gettime")
int BPF_KRETPROBE(probe_clock_gettime_ret, long ret)
{
    // 返り値が 0（成功）の場合のみ処理する
    if (ret != 0) {
        // clock_gettime が失敗した場合は何も処理しない
        return 0;
    }
    // kretprobe での後処理（必要であれば追加の統計を記録できる）
    // 現時点では kprobe 側での処理のみを行う
    return 0;
}
