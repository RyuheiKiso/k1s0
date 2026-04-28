// 本ファイルは AuditService.Export の chunk 整形ヘルパを提供する。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/10_Audit_Pii_API.md（AuditService.Export）
//
// 役割:
//   AuditEntry のリストを CSV / NDJSON / JSON 配列のいずれかでバイナリに整形し、
//   chunk_bytes 上限ごとに mpsc::Sender に送出する。
//   end-of-stream は is_last=true を持つ最終 chunk で必ず 1 度だけ通知する。
//
// crate 配置の意図:
//   フォーマット整形は store / proto と独立しているため lib 側に切り出し、
//   main.rs（バイナリ）の行数を 500 行規約以内に収める（src/CLAUDE.md 準拠）。
//   将来 server-streaming のテストを lib 単体で追加するためにも有用。

// SDK 公開 API の Export 関連 proto 型。
use k1s0_sdk_proto::k1s0::tier1::audit::v1::{ExportAuditChunk, ExportFormat};
// 非同期 channel の送信側。
use tokio::sync::mpsc;
// tonic Status は send 経路でエラーを伝搬する場合の result 型に必要。
use tonic::Status;

// store の AuditEntry 型を再利用する。
use crate::store::AuditEntry;

/// FR-T1-AUDIT-002 Export の chunk 送出ループ本体。
/// is_last=true の chunk を必ず最後に 1 度だけ送る（受信側が終端を判別するため）。
pub async fn send_export_chunks(
    tx: mpsc::Sender<Result<ExportAuditChunk, Status>>,
    entries: Vec<AuditEntry>,
    format: ExportFormat,
    chunk_bytes: usize,
) {
    // chunk バッファと統計。chunk 連番 / event 件数。
    let mut buf: Vec<u8> = Vec::with_capacity(chunk_bytes);
    // chunk_seq は 0 始まりの連番。
    let mut chunk_seq: i64 = 0;
    // 現 chunk 内に詰め込んだ event 数（is_last 直前の chunk の event_count に詰める）。
    let mut events_in_chunk: i64 = 0;

    // CSV 時はヘッダ行を最初に流す。
    if matches!(format, ExportFormat::Csv) {
        buf.extend_from_slice(b"audit_id,timestamp_ms,tenant_id,actor,action,resource,outcome\n");
    }
    // JSON 配列の開きカッコを最初に流す。
    if matches!(format, ExportFormat::JsonArray) {
        buf.push(b'[');
    }

    // 各 entry を 1 行 / 1 オブジェクトに整形して buf に追記する。
    for (idx, e) in entries.iter().enumerate() {
        // フォーマット別に 1 entry を bytes 化。
        let line = format_entry(e, format, idx == 0);
        // chunk 上限を超えそうなら現 chunk を flush する。
        if !buf.is_empty() && buf.len() + line.len() > chunk_bytes {
            // 現 buf を chunk として送出する（buf を空に戻す）。
            let _ = tx
                .send(Ok(ExportAuditChunk {
                    data: std::mem::take(&mut buf),
                    sequence: chunk_seq,
                    event_count: events_in_chunk,
                    is_last: false,
                }))
                .await;
            // 送出後は次 chunk 用にカウンタをリセット。
            chunk_seq += 1;
            events_in_chunk = 0;
        }
        // 現 buf に line を追記する。
        buf.extend_from_slice(&line);
        events_in_chunk += 1;
    }

    // JSON 配列の閉じカッコを末尾に追加する。
    if matches!(format, ExportFormat::JsonArray) {
        buf.push(b']');
    }

    // 最終 chunk を必ず 1 件送る（buf が空でも is_last=true で送る）。
    let _ = tx
        .send(Ok(ExportAuditChunk {
            data: buf,
            sequence: chunk_seq,
            event_count: events_in_chunk,
            is_last: true,
        }))
        .await;
}

/// 1 entry を所定フォーマットで bytes に変換する。
/// JSON 配列形式の場合、2 件目以降は先頭に "," を付ける（valid な JSON 配列を構築）。
pub fn format_entry(e: &AuditEntry, format: ExportFormat, is_first: bool) -> Vec<u8> {
    match format {
        // CSV: RFC 4180 準拠で各フィールドを quote する。
        ExportFormat::Csv => format!(
            "{},{},{},{},{},{},{}\n",
            csv_field(&e.audit_id),
            e.timestamp_ms,
            csv_field(&e.tenant_id),
            csv_field(&e.actor),
            csv_field(&e.action),
            csv_field(&e.resource),
            csv_field(&e.outcome),
        )
        .into_bytes(),
        ExportFormat::Ndjson => {
            // 1 行 1 イベントの JSON。末尾に改行。
            let line = serde_json::json!({
                "audit_id": e.audit_id,
                "timestamp_ms": e.timestamp_ms,
                "tenant_id": e.tenant_id,
                "actor": e.actor,
                "action": e.action,
                "resource": e.resource,
                "outcome": e.outcome,
                "attributes": e.attributes,
            });
            // serde_json 失敗時は空 bytes（実用上 to_vec は失敗しない）。
            let mut s = serde_json::to_vec(&line).unwrap_or_default();
            s.push(b'\n');
            s
        }
        ExportFormat::JsonArray => {
            // JSON 配列要素。複数なら "," で連結する。
            let line = serde_json::json!({
                "audit_id": e.audit_id,
                "timestamp_ms": e.timestamp_ms,
                "tenant_id": e.tenant_id,
                "actor": e.actor,
                "action": e.action,
                "resource": e.resource,
                "outcome": e.outcome,
                "attributes": e.attributes,
            });
            let body = serde_json::to_vec(&line).unwrap_or_default();
            if is_first {
                body
            } else {
                // 2 件目以降は先頭に "," を付ける。
                let mut s = Vec::with_capacity(body.len() + 1);
                s.push(b',');
                s.extend_from_slice(&body);
                s
            }
        }
        // UNSPECIFIED は NDJSON にフォールバック（caller 側でも正規化済だが defensive）。
        _ => {
            let line = serde_json::json!({
                "audit_id": e.audit_id,
                "timestamp_ms": e.timestamp_ms,
                "tenant_id": e.tenant_id,
                "actor": e.actor,
                "action": e.action,
                "resource": e.resource,
                "outcome": e.outcome,
            });
            let mut s = serde_json::to_vec(&line).unwrap_or_default();
            s.push(b'\n');
            s
        }
    }
}

/// CSV フィールドを RFC 4180 に従って quote する。",", "\n", "\"", "\r" のいずれか含む場合は
/// "..." で囲み、内部の "\"" は "\"\"" にエスケープする。
pub fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        // 内部 " を "" にエスケープ。
        let escaped = s.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::AuditEntry;
    use std::collections::BTreeMap;

    /// テスト用の AuditEntry 雛形を生成する。
    fn make_entry(audit_id: &str, ts_ms: i64, tenant: &str, actor: &str) -> AuditEntry {
        AuditEntry {
            audit_id: audit_id.to_string(),
            timestamp_ms: ts_ms,
            tenant_id: tenant.to_string(),
            actor: actor.to_string(),
            action: "READ".into(),
            resource: "k1s0:tenant:T:resource:s/x".into(),
            outcome: "SUCCESS".into(),
            attributes: BTreeMap::new(),
            // hash chain 上の前 entry の audit_id。export 整形のテストでは未使用なので "GENESIS"。
            prev_id: "GENESIS".into(),
        }
    }

    #[test]
    fn csv_field_escapes_special_chars() {
        // カンマ含む値は quote される。
        assert_eq!(csv_field("a,b"), "\"a,b\"");
        // ダブルクォート含む値は quote + エスケープされる。
        assert_eq!(csv_field("a\"b"), "\"a\"\"b\"");
        // 改行含む値は quote される。
        assert_eq!(csv_field("a\nb"), "\"a\nb\"");
        // 通常文字列は素通し。
        assert_eq!(csv_field("normal"), "normal");
    }

    #[test]
    fn format_entry_csv_terminates_with_newline() {
        let e = make_entry("aid", 100, "T", "u");
        // CSV は 1 行で末尾に改行が付く。
        let line = format_entry(&e, ExportFormat::Csv, true);
        assert!(line.ends_with(b"\n"));
        let s = String::from_utf8(line).unwrap();
        // フィールドが 7 つ並んでいる。
        assert_eq!(s.matches(',').count(), 6);
    }

    #[test]
    fn format_entry_ndjson_one_object_per_line() {
        let e = make_entry("aid", 100, "T", "u");
        let line = format_entry(&e, ExportFormat::Ndjson, true);
        assert!(line.ends_with(b"\n"));
        // JSON として valid。
        let _v: serde_json::Value =
            serde_json::from_slice(&line[..line.len() - 1]).expect("ndjson is valid json");
    }

    #[test]
    fn format_entry_json_array_prefixes_comma_after_first() {
        let e = make_entry("aid", 100, "T", "u");
        let first = format_entry(&e, ExportFormat::JsonArray, true);
        let second = format_entry(&e, ExportFormat::JsonArray, false);
        // 先頭要素はカンマ無し、2 件目以降は "," 始まり。
        assert_ne!(first[0], b',');
        assert_eq!(second[0], b',');
    }

    #[tokio::test]
    async fn send_export_chunks_emits_last_marker_once() {
        let (tx, mut rx) = mpsc::channel::<Result<ExportAuditChunk, Status>>(8);
        let entries = vec![
            make_entry("a", 1, "T", "u"),
            make_entry("b", 2, "T", "u"),
            make_entry("c", 3, "T", "u"),
        ];
        // 全部受け取れる十分大きい chunk_bytes（1 chunk に収まる）。
        send_export_chunks(tx, entries, ExportFormat::Ndjson, 1_048_576).await;
        let mut chunks: Vec<ExportAuditChunk> = Vec::new();
        while let Some(Ok(c)) = rx.recv().await {
            chunks.push(c);
        }
        // 1 件以上届く。
        assert!(!chunks.is_empty());
        // 末尾に is_last=true が 1 件だけ存在する。
        let last_count = chunks.iter().filter(|c| c.is_last).count();
        assert_eq!(last_count, 1);
        assert!(chunks.last().unwrap().is_last);
    }

    #[tokio::test]
    async fn send_export_chunks_splits_by_chunk_bytes() {
        let (tx, mut rx) = mpsc::channel::<Result<ExportAuditChunk, Status>>(16);
        // 各 entry が ~200 bytes になるよう適度な内容。
        let entries: Vec<AuditEntry> = (0..10)
            .map(|i| make_entry(&format!("audit-id-{}", i), i, "T", "u"))
            .collect();
        // chunk 上限を 256 bytes に絞ると複数 chunk になるはず。
        send_export_chunks(tx, entries, ExportFormat::Ndjson, 256).await;
        let mut chunks: Vec<ExportAuditChunk> = Vec::new();
        while let Some(Ok(c)) = rx.recv().await {
            chunks.push(c);
        }
        // 2 chunk 以上に分割されている。
        assert!(
            chunks.len() >= 2,
            "expected multiple chunks under 256 byte limit, got {}",
            chunks.len()
        );
        // 末尾だけ is_last=true。
        for c in &chunks[..chunks.len() - 1] {
            assert!(!c.is_last);
        }
        assert!(chunks.last().unwrap().is_last);
    }
}
