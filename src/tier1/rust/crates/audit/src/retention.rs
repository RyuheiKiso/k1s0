// 本ファイルは Audit ログの 7 年保存階層ポリシーを定義する。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/10_Audit_Pii_API.md
//     - FR-T1-AUDIT-003（長期保存 7 年）
//   docs/03_要件定義/30_非機能要件/C_運用.md（NFR-C-NOP-003: Audit 7 年保存）
//
// 役割:
//   各階層の境界（90 日 / 1 年 / 7 年）を定数として固定し、ある entry の経過時間から
//   現在の階層と次の遷移先を判定するヘルパを提供する。
//   実際の移行は archival.rs の RetentionRunner が本ポリシーを参照して行う。
//
// 階層（FR-T1-AUDIT-003 受け入れ基準）:
//   - HotLoki:    0  〜 90 日   - Loki でホット検索（5 分以内 ingestion）
//   - WarmPg:     90 〜 365 日  - Kafka → PostgreSQL ウォーム保存
//   - ColdMinio:  365 〜 2555 日 (7 年) - MinIO + Object Lock アーカイブ
//   - Expired:    2555 日 〜    - 削除対象（削除操作も Audit に記録）

/// `RetentionTier` は audit entry が現在属する保存階層。
///
/// 階層は単方向にしか遷移しない（HotLoki → WarmPg → ColdMinio → Expired）。
/// 評価は entry の `timestamp_ms`（生成時刻）と現在時刻の差分で行う。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetentionTier {
    /// 0 〜 90 日: Loki ホット検索層。
    HotLoki,
    /// 90 〜 365 日: PostgreSQL ウォーム保存層。
    WarmPg,
    /// 365 〜 2555 日（7 年）: MinIO + Object Lock アーカイブ層。
    ColdMinio,
    /// 7 年経過: 削除対象。
    Expired,
}

/// `RetentionPolicy` は階層境界の閾値を保持する。
///
/// docs FR-T1-AUDIT-003 の受け入れ基準は固定値（90 日 / 1 年 / 7 年）だが、
/// 採用後の運用拡大時に「短期テナント」「監査強化テナント」で異なる保管期間を
/// 適用する余地を残すため、struct にして注入できる形にする。
#[derive(Debug, Clone, Copy)]
pub struct RetentionPolicy {
    /// HotLoki → WarmPg 遷移境界（既定 90 日 = 90 * 86_400_000 ms）。
    pub hot_to_warm_ms: i64,
    /// WarmPg → ColdMinio 遷移境界（既定 365 日）。
    pub warm_to_cold_ms: i64,
    /// ColdMinio → Expired 遷移境界（既定 2555 日 = 7 年）。
    pub cold_to_expired_ms: i64,
}

impl RetentionPolicy {
    /// 1 日のミリ秒（86,400,000 ms）。
    pub const ONE_DAY_MS: i64 = 86_400_000;

    /// docs 既定値（90 日 / 1 年 / 7 年）。
    pub const DEFAULT: Self = Self {
        hot_to_warm_ms: 90 * Self::ONE_DAY_MS,
        warm_to_cold_ms: 365 * Self::ONE_DAY_MS,
        cold_to_expired_ms: 2555 * Self::ONE_DAY_MS,
    };

    /// 経過ミリ秒から現在の階層を判定する。
    ///
    /// `age_ms` が負（未来 timestamp）の場合は HotLoki 扱い（時計ズレに対する保守的選択）。
    pub fn tier_for_age_ms(&self, age_ms: i64) -> RetentionTier {
        if age_ms < self.hot_to_warm_ms {
            RetentionTier::HotLoki
        } else if age_ms < self.warm_to_cold_ms {
            RetentionTier::WarmPg
        } else if age_ms < self.cold_to_expired_ms {
            RetentionTier::ColdMinio
        } else {
            RetentionTier::Expired
        }
    }

    /// `entry_timestamp_ms` の entry が `now_ms` 時点でどの階層に属するかを判定する。
    pub fn tier_for(&self, entry_timestamp_ms: i64, now_ms: i64) -> RetentionTier {
        // 経過時間を ms 単位で計算する（過去なら正、未来なら負）。
        let age_ms = now_ms.saturating_sub(entry_timestamp_ms);
        self.tier_for_age_ms(age_ms)
    }

    /// `now_ms` から見て「Postgres → MinIO 移行対象」となる cutoff timestamp を返す。
    /// この timestamp 以前の entry は WarmPg を抜け、ColdMinio へ移すべき対象。
    pub fn warm_to_cold_cutoff_ms(&self, now_ms: i64) -> i64 {
        now_ms.saturating_sub(self.warm_to_cold_ms)
    }

    /// `now_ms` から見て「MinIO 削除対象（7 年経過）」となる cutoff timestamp を返す。
    pub fn cold_to_expired_cutoff_ms(&self, now_ms: i64) -> i64 {
        now_ms.saturating_sub(self.cold_to_expired_ms)
    }
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_doc_thresholds() {
        let p = RetentionPolicy::default();
        assert_eq!(p.hot_to_warm_ms, 90 * RetentionPolicy::ONE_DAY_MS);
        assert_eq!(p.warm_to_cold_ms, 365 * RetentionPolicy::ONE_DAY_MS);
        assert_eq!(p.cold_to_expired_ms, 2555 * RetentionPolicy::ONE_DAY_MS);
    }

    #[test]
    fn tier_for_age_boundary_inclusive_lower_exclusive_upper() {
        let p = RetentionPolicy::default();
        // 0 day → HotLoki。
        assert_eq!(p.tier_for_age_ms(0), RetentionTier::HotLoki);
        // 89 day 23h → 依然 HotLoki。
        assert_eq!(
            p.tier_for_age_ms(p.hot_to_warm_ms - 1),
            RetentionTier::HotLoki
        );
        // 90 day ちょうど → WarmPg（境界の挙動）。
        assert_eq!(p.tier_for_age_ms(p.hot_to_warm_ms), RetentionTier::WarmPg);
        // 364 day → WarmPg。
        assert_eq!(
            p.tier_for_age_ms(p.warm_to_cold_ms - 1),
            RetentionTier::WarmPg
        );
        // 365 day → ColdMinio。
        assert_eq!(
            p.tier_for_age_ms(p.warm_to_cold_ms),
            RetentionTier::ColdMinio
        );
        // 7 年 - 1 ms → ColdMinio。
        assert_eq!(
            p.tier_for_age_ms(p.cold_to_expired_ms - 1),
            RetentionTier::ColdMinio
        );
        // 7 年ちょうど → Expired。
        assert_eq!(
            p.tier_for_age_ms(p.cold_to_expired_ms),
            RetentionTier::Expired
        );
    }

    #[test]
    fn future_timestamp_treated_as_hot() {
        // 時計ズレ（未来 timestamp）は age=負 → HotLoki 扱い。
        let p = RetentionPolicy::default();
        assert_eq!(p.tier_for(2000, 1000), RetentionTier::HotLoki);
    }

    #[test]
    fn cutoff_helpers_compute_correctly() {
        let p = RetentionPolicy::default();
        let now = 10_000_000_000_000_i64;
        assert_eq!(p.warm_to_cold_cutoff_ms(now), now - p.warm_to_cold_ms);
        assert_eq!(p.cold_to_expired_cutoff_ms(now), now - p.cold_to_expired_ms);
    }
}
