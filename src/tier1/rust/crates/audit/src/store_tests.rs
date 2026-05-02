// テスト分離: src/CLAUDE.md「1 ファイル 500 行以内」遵守のため、store.rs から tests module を抽出。
// 設計正典との対応は store.rs 冒頭コメントを参照。
// 親 store.rs で `#[cfg(test)] mod store_tests;` として include される（test profile のみ build）。

use super::*;

    fn make_input(ts: i64, actor: &str, tenant: &str) -> AppendInput {
        AppendInput {
            timestamp_ms: ts,
            actor: actor.to_string(),
            action: "READ".to_string(),
            resource: "k1s0:tenant:T:resource:secret/db".to_string(),
            outcome: "SUCCESS".to_string(),
            attributes: Default::default(),
            tenant_id: tenant.to_string(),
        }
    }

    #[test]
    fn append_returns_audit_id_and_chains() {
        let s = InMemoryAuditStore::new();
        let id1 = s.append(make_input(1, "u1", "T")).unwrap();
        let id2 = s.append(make_input(2, "u2", "T")).unwrap();
        // 異なる id が出る。
        assert_ne!(id1, id2);
        // 第 2 entry の prev_id は第 1 の audit_id。
        let entries = s.entries.read().unwrap();
        assert_eq!(entries[1].prev_id, id1);
    }

    #[test]
    fn append_deterministic_hash() {
        // 同一入力で同一 hash が出る（ただし prev_id が "GENESIS" の最初のみ）。
        let a = InMemoryAuditStore::new();
        let b = InMemoryAuditStore::new();
        let id_a = a.append(make_input(100, "u", "T")).unwrap();
        let id_b = b.append(make_input(100, "u", "T")).unwrap();
        assert_eq!(id_a, id_b);
    }

    #[test]
    fn query_filters_by_tenant() {
        let s = InMemoryAuditStore::new();
        s.append(make_input(1, "u1", "T1")).unwrap();
        s.append(make_input(2, "u2", "T2")).unwrap();
        s.append(make_input(3, "u3", "T1")).unwrap();
        let r = s
            .query(QueryInput {
                tenant_id: "T1".to_string(),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(r.len(), 2);
        assert!(r.iter().all(|e| e.tenant_id == "T1"));
    }

    #[test]
    fn query_filters_by_range_and_limit() {
        let s = InMemoryAuditStore::new();
        for i in 1..=10 {
            s.append(make_input(i, "u", "T")).unwrap();
        }
        let r = s
            .query(QueryInput {
                tenant_id: "T".to_string(),
                from_ms: Some(3),
                to_ms: Some(7),
                limit: 0, // → 100 default
                ..Default::default()
            })
            .unwrap();
        // 範囲 3..=7 の 5 件。
        assert_eq!(r.len(), 5);
        assert_eq!(r[0].timestamp_ms, 3);
        assert_eq!(r[4].timestamp_ms, 7);

        // limit を効かせる。
        let r2 = s
            .query(QueryInput {
                tenant_id: "T".to_string(),
                limit: 3,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(r2.len(), 3);
    }

    #[test]
    fn query_filters_by_attributes() {
        let s = InMemoryAuditStore::new();
        let mut a1 = make_input(1, "u1", "T");
        a1.attributes.insert("ip".into(), "10.0.0.1".into());
        let mut a2 = make_input(2, "u2", "T");
        a2.attributes.insert("ip".into(), "10.0.0.2".into());
        s.append(a1).unwrap();
        s.append(a2).unwrap();
        let r = s
            .query(QueryInput {
                tenant_id: "T".to_string(),
                filters: [("ip".to_string(), "10.0.0.2".to_string())]
                    .into_iter()
                    .collect(),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].timestamp_ms, 2);
    }

    #[test]
    fn verify_chain_passes_after_appends() {
        let s = InMemoryAuditStore::new();
        for i in 1..=5 {
            s.append(make_input(i, "u", "T")).unwrap();
        }
        s.verify_chain().expect("chain valid");
    }

    #[test]
    fn verify_chain_detects_tamper() {
        let s = InMemoryAuditStore::new();
        s.append(make_input(1, "u", "T")).unwrap();
        s.append(make_input(2, "u", "T")).unwrap();
        // 1 件目の actor を直接書き換える（WORM 違反、検知されるべき）。
        {
            let mut entries = s.entries.write().unwrap();
            entries[0].actor = "u-evil".to_string();
        }
        let r = s.verify_chain();
        assert!(r.is_err(), "tamper should be detected");
    }

    /// VerifyChain RPC の詳細応答（proto VerifyChainResponse 互換）が、
    /// 改ざんされた entry の **正確な sequence 番号 + 検知理由** を
    /// 返すことを検証する。NFR-H-INT-001 / 002 の核心要件:
    /// "完全性違反は検知可能 + 違反箇所が特定可能"。
    #[test]
    fn verify_chain_detail_returns_first_bad_sequence_with_reason() {
        let s = InMemoryAuditStore::new();
        s.append(make_input(1, "u1", "T")).unwrap();
        s.append(make_input(2, "u2", "T")).unwrap();
        s.append(make_input(3, "u3", "T")).unwrap();
        // 改ざん前 detail: valid=true / checked=3 / first_bad=0
        let ok = s.verify_chain_detail("T", None, None).unwrap();
        assert!(ok.valid, "before tamper should be valid");
        assert_eq!(ok.checked_count, 3);
        assert_eq!(ok.first_bad_sequence, 0);
        assert_eq!(ok.reason, "");
        // 中央の 2 件目を改ざん。
        {
            let mut entries = s.entries.write().unwrap();
            entries[1].action = "tampered".to_string();
        }
        let bad = s.verify_chain_detail("T", None, None).unwrap();
        assert!(!bad.valid, "tamper must be reported invalid");
        // checked_count は valid だった entry 数（1 件目）まで。
        assert_eq!(
            bad.checked_count, 1,
            "checked_count should stop at the entry preceding the tamper"
        );
        // 2 件目で audit_id 不整合を検知するはず。
        assert_eq!(
            bad.first_bad_sequence, 2,
            "first_bad_sequence must point to the tampered entry"
        );
        assert!(
            bad.reason.contains("audit_id mismatch at sequence 2"),
            "reason should describe the actual mismatch (got: {:?})",
            bad.reason
        );
    }

    /// 中央の entry を **削除** した場合、それ以降の prev_id chain が壊れて
    /// 検知される。InMemory store は entries Vec を露出する WORM 違反テスト用 API
    /// を持たないため、unsafe な write 直接編集を使う。
    #[test]
    fn verify_chain_detail_detects_deletion_via_prev_id_break() {
        let s = InMemoryAuditStore::new();
        s.append(make_input(1, "u1", "T")).unwrap();
        s.append(make_input(2, "u2", "T")).unwrap();
        s.append(make_input(3, "u3", "T")).unwrap();
        // 2 件目を削除（→ 3 件目の prev_id が 1 件目の audit_id を指さない状態）。
        {
            let mut entries = s.entries.write().unwrap();
            entries.remove(1);
        }
        let bad = s.verify_chain_detail("T", None, None).unwrap();
        assert!(!bad.valid, "deletion must be reported invalid");
        // 1 件目はそのまま valid、2 件目の位置（旧 3 件目）で prev_id mismatch。
        assert_eq!(bad.first_bad_sequence, 2);
        assert!(
            bad.reason.contains("prev_id mismatch at sequence 2"),
            "reason should say prev_id mismatch (got: {:?})",
            bad.reason
        );
    }
