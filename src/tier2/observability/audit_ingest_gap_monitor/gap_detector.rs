// gap_detector.rs — audit ingest pipeline の gap/drop を別経路で検出する detector
// spec 10 §audit_ingest_gap_monitor: heartbeat insert + gap 検出でデータロスを発見する
// chain_sequence の連続性と heartbeat の staleness を独立した 2 経路で監視する

// 時刻関連の型をインポートする
use std::time::{Duration, SystemTime};

// gap 検出の結果を表す構造体
#[derive(Debug, PartialEq)]
pub struct GapDetectionResult {
    // gap が検出されたかを示すフラグ
    pub has_gap: bool,
    // 最後に確認された chain_sequence（空の場合は 0）
    pub last_sequence: i64,
    // 期待される次の chain_sequence
    pub expected_next: i64,
    // gap のサイズ（欠落件数、gap がない場合は 0）
    pub gap_size: i64,
}

// chain_sequence の連続性を検査して gap を検出する関数
// sequences: DB から取得した chain_sequence の昇順リスト
pub fn detect_sequence_gap(sequences: &[i64]) -> GapDetectionResult {
    // シーケンスが空の場合は gap なしと判断する（データ未取得状態）
    if sequences.is_empty() {
        return GapDetectionResult {
            // 空の場合は gap なしとして扱う
            has_gap: false,
            // 最後のシーケンスは 0 として初期化する
            last_sequence: 0,
            // 次に期待するシーケンスは 1 から始まる
            expected_next: 1,
            // gap サイズは 0
            gap_size: 0,
        };
    }
    // 最初のシーケンスを基準値として設定する
    let mut expected = sequences[0];
    // 各シーケンスを順番に確認して連続性を検証する
    for &seq in sequences {
        // 期待値と実際の値が異なる場合は gap を検出したと判断する
        if seq != expected {
            return GapDetectionResult {
                // gap を検出したことを示す
                has_gap: true,
                // gap が始まる直前の最後の正常シーケンスを記録する
                last_sequence: expected - 1,
                // gap が始まる位置（欠落が始まった期待値）を記録する
                expected_next: expected,
                // 欠落している件数を計算する（実際の値 - 期待値）
                gap_size: seq - expected,
            };
        }
        // 期待値を次の連番に更新して次のループに進む
        expected += 1;
    }
    // すべてのシーケンスが連続している場合は gap なしを返す
    GapDetectionResult {
        // gap なし
        has_gap: false,
        // 最後に確認したシーケンスを記録する
        last_sequence: *sequences.last().unwrap(),
        // 次に期待するシーケンス番号を設定する
        expected_next: expected,
        // gap サイズは 0
        gap_size: 0,
    }
}

// heartbeat タイムスタンプから staleness を検出する関数
// last_heartbeat: 最後の heartbeat が記録された SystemTime
// threshold: この Duration を超えていたら stale と判断する閾値
pub fn is_heartbeat_stale(last_heartbeat: SystemTime, threshold: Duration) -> bool {
    // 現在時刻との経過差分を計算する
    match last_heartbeat.elapsed() {
        // 経過時間が閾値を超えていたら stale と判断する
        Ok(elapsed) => elapsed > threshold,
        // SystemTime が未来を指している（時刻巻き戻り等）場合は安全側に stale と判断する
        Err(_) => true,
    }
}

// AuditIngestGapMonitor: gap 検出と heartbeat staleness を統合して監視する構造体
#[derive(Debug)]
pub struct AuditIngestGapMonitor {
    // Prometheus メトリクス名: audit_ingest_last_event_ts
    pub metric_name: String,
    // gap 検出の閾値（この秒数を超えたら page_immediate アラートを発行する）
    pub gap_threshold_seconds: u64,
}

impl AuditIngestGapMonitor {
    // AuditIngestGapMonitor を生成する
    pub fn new(metric_name: String, gap_threshold_seconds: u64) -> Self {
        // 設定値を保持するインスタンスを返す
        Self {
            metric_name,
            gap_threshold_seconds,
        }
    }

    // gap 検出結果と heartbeat staleness をまとめて評価してアラート要否を返す関数
    // sequences: DB から取得した chain_sequence リスト
    // last_heartbeat: 最後の heartbeat の記録時刻
    pub fn should_alert(
        &self,
        sequences: &[i64],
        last_heartbeat: SystemTime,
    ) -> bool {
        // chain_sequence の連続性を検査する
        let gap_result = detect_sequence_gap(sequences);
        // gap が検出された場合はアラートを発行する
        if gap_result.has_gap {
            return true;
        }
        // heartbeat の staleness を閾値と比較する
        let threshold = Duration::from_secs(self.gap_threshold_seconds);
        // heartbeat が stale な場合もアラートを発行する
        is_heartbeat_stale(last_heartbeat, threshold)
    }
}

// テスト: sequence gap 検出と heartbeat staleness の単体テスト
#[cfg(test)]
mod tests {
    use super::*;

    // 連続したシーケンスでは gap なしを返すことを確認する
    #[test]
    fn test_no_gap_in_continuous_sequence() {
        // 1 から 5 の連続したシーケンスを用意する
        let seqs = vec![1, 2, 3, 4, 5];
        // gap 検出を実行する
        let result = detect_sequence_gap(&seqs);
        // gap なしであることを確認する
        assert!(!result.has_gap);
        // 最後のシーケンスが 5 であることを確認する
        assert_eq!(result.last_sequence, 5);
        // 次の期待シーケンスが 6 であることを確認する
        assert_eq!(result.expected_next, 6);
        // gap サイズが 0 であることを確認する
        assert_eq!(result.gap_size, 0);
    }

    // gap がある場合に正しく検出することを確認する（3 が欠落）
    #[test]
    fn test_gap_detected() {
        // 1, 2, 4, 5 と 3 が欠落したシーケンスを用意する
        let seqs = vec![1, 2, 4, 5];
        // gap 検出を実行する
        let result = detect_sequence_gap(&seqs);
        // gap ありであることを確認する
        assert!(result.has_gap);
        // gap サイズが 1 であることを確認する（3 が 1 件欠落）
        assert_eq!(result.gap_size, 1);
        // 期待シーケンスが 3 であることを確認する
        assert_eq!(result.expected_next, 3);
        // 最後の正常シーケンスが 2 であることを確認する
        assert_eq!(result.last_sequence, 2);
    }

    // 複数連続 gap がある場合に正しく検出することを確認する（3, 4 が欠落）
    #[test]
    fn test_multiple_gap_detected() {
        // 1, 2, 5, 6 と 3, 4 が欠落したシーケンスを用意する
        let seqs = vec![1, 2, 5, 6];
        // gap 検出を実行する
        let result = detect_sequence_gap(&seqs);
        // gap ありであることを確認する
        assert!(result.has_gap);
        // gap サイズが 2 であることを確認する（3, 4 が 2 件欠落）
        assert_eq!(result.gap_size, 2);
    }

    // 空のシーケンスでは gap なしを返すことを確認する
    #[test]
    fn test_empty_sequence_no_gap() {
        // 空のシーケンスを用意する
        let seqs: Vec<i64> = vec![];
        // gap 検出を実行する
        let result = detect_sequence_gap(&seqs);
        // gap なしであることを確認する
        assert!(!result.has_gap);
        // 初期 expected_next が 1 であることを確認する
        assert_eq!(result.expected_next, 1);
    }

    // heartbeat が閾値内に収まっている場合は stale でないことを確認する
    #[test]
    fn test_heartbeat_not_stale_within_threshold() {
        // 現在時刻（stale でない heartbeat を模擬する）
        let now = SystemTime::now();
        // 10 秒の閾値で stale でないことを確認する
        let threshold = Duration::from_secs(10);
        // 現在時刻は閾値内のため stale でないことを確認する
        assert!(!is_heartbeat_stale(now, threshold));
    }

    // 古い heartbeat は stale と判断することを確認する
    #[test]
    fn test_heartbeat_stale_past_threshold() {
        // 十分に古い時刻を生成する（60 秒前）
        let old_time = SystemTime::now() - Duration::from_secs(60);
        // 10 秒の閾値で stale であることを確認する
        let threshold = Duration::from_secs(10);
        // 60 秒前の heartbeat は 10 秒閾値を超えているため stale であることを確認する
        assert!(is_heartbeat_stale(old_time, threshold));
    }

    // should_alert が gap 検出時に true を返すことを確認する
    #[test]
    fn test_should_alert_on_sequence_gap() {
        // 3 分閾値の monitor を生成する（heartbeat.yaml の expected_max_gap_seconds = 180 と一致）
        let monitor = AuditIngestGapMonitor::new(
            "audit_ingest_last_event_ts".to_string(),
            180,
        );
        // gap のあるシーケンス（3 が欠落）を用意する
        let seqs = vec![1, 2, 4];
        // 現在時刻の heartbeat を用意する（staleness は問わない）
        let heartbeat = SystemTime::now();
        // gap 検出によりアラートが発行されることを確認する
        assert!(monitor.should_alert(&seqs, heartbeat));
    }

    // should_alert が全て正常な場合に false を返すことを確認する
    #[test]
    fn test_should_not_alert_when_all_ok() {
        // 3 分閾値の monitor を生成する
        let monitor = AuditIngestGapMonitor::new(
            "audit_ingest_last_event_ts".to_string(),
            180,
        );
        // 連続したシーケンスを用意する（gap なし）
        let seqs = vec![1, 2, 3, 4, 5];
        // 現在時刻の heartbeat を用意する（stale でない）
        let heartbeat = SystemTime::now();
        // 正常状態ではアラートが発行されないことを確認する
        assert!(!monitor.should_alert(&seqs, heartbeat));
    }
}
