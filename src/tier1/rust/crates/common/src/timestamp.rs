// 本ファイルは google.protobuf.Timestamp ↔ RFC 3339 文字列の双方向変換 helper。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/00_tier1_API共通規約.md
//     §「HTTP/JSON 互換インタフェース共通仕様」: protojson 直列化
//   protojson canonical encoding（https://protobuf.dev/programming-guides/proto3/#json）:
//     `google.protobuf.Timestamp` は **RFC 3339 文字列**（"YYYY-MM-DDTHH:MM:SS[.fff]Z" 等）
//     として直列化される。`{seconds, nanos}` の internal repr は wire format 専用で、
//     JSON では出さない。
//
// 役割:
//   tier1 Rust Pod の HTTP/JSON gateway は protojson 互換を維持する責務がある（schemathesis
//   が `format: date-time` を期待する）。本 helper は 3 Pod すべて（audit / decision / pii）
//   から共有される。

use prost_types::Timestamp;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// `google.protobuf.Timestamp` を RFC 3339 文字列に変換する。
/// nanos が 0 の場合は秒精度（例: `"2026-04-30T09:00:00Z"`）、それ以外は小数秒
/// （例: `"2026-04-30T09:00:00.123456789Z"`）として整形する。
///
/// epoch 範囲外（year < 1 / year > 9999 等）の異常値は `None` を返し、呼出側で
/// fallback する想定（tier1 の audit chain は通常 1970-2200 の範囲なので実質到達不能）。
pub fn timestamp_to_rfc3339(ts: &Timestamp) -> Option<String> {
    // i64 nanos は OffsetDateTime に直接渡せないため、seconds * 1e9 + nanos で
    // 計算してから unix_timestamp_nanos に渡す（オーバーフロー検査込み）。
    let total_nanos = (ts.seconds as i128).checked_mul(1_000_000_000)?;
    let total_nanos = total_nanos.checked_add(ts.nanos as i128)?;
    let dt = OffsetDateTime::from_unix_timestamp_nanos(total_nanos).ok()?;
    dt.format(&Rfc3339).ok()
}

/// RFC 3339 文字列を `google.protobuf.Timestamp` に変換する。
/// 不正な入力は `None` を返す。
pub fn rfc3339_to_timestamp(s: &str) -> Option<Timestamp> {
    let dt = OffsetDateTime::parse(s, &Rfc3339).ok()?;
    let nanos_total = dt.unix_timestamp_nanos();
    let seconds = (nanos_total / 1_000_000_000) as i64;
    let nanos = (nanos_total % 1_000_000_000) as i32;
    Some(Timestamp { seconds, nanos })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_seconds_only() {
        let ts = Timestamp {
            seconds: 1714464000,
            nanos: 0,
        };
        let s = timestamp_to_rfc3339(&ts).unwrap();
        assert!(s.ends_with("Z"));
        let back = rfc3339_to_timestamp(&s).unwrap();
        assert_eq!(back.seconds, ts.seconds);
        assert_eq!(back.nanos, 0);
    }

    #[test]
    fn round_trip_with_nanos() {
        let ts = Timestamp {
            seconds: 1714464000,
            nanos: 123_456_789,
        };
        let s = timestamp_to_rfc3339(&ts).unwrap();
        let back = rfc3339_to_timestamp(&s).unwrap();
        assert_eq!(back, ts);
    }

    #[test]
    fn invalid_string_returns_none() {
        assert!(rfc3339_to_timestamp("not-a-date").is_none());
        assert!(rfc3339_to_timestamp("2026-04-30").is_none()); // 時刻部分なし
    }

    #[test]
    fn epoch_range_safety() {
        // 異常値（巨大 seconds + 巨大 nanos）でも panic せず None を返すこと。
        let ts = Timestamp {
            seconds: i64::MAX,
            nanos: i32::MAX,
        };
        assert!(timestamp_to_rfc3339(&ts).is_none());
    }
}
