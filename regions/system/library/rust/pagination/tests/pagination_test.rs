use k1s0_pagination::{
    decode_cursor, default_page_request, encode_cursor, validate_per_page, CursorMeta,
    CursorRequest, PageRequest, PageResponse, PaginationError, PaginationMeta,
    PerPageValidationError,
};

// ============================================================
// Cursor encode/decode
// ============================================================

// カーソルのエンコードとデコードが正しくラウンドトリップすることを確認する。
#[test]
fn cursor_roundtrip_basic() {
    let sort_key = "2024-01-15T10:30:00Z";
    let id = "record-42";
    let encoded = encode_cursor(sort_key, id);
    let (dk, di) = decode_cursor(&encoded).unwrap();
    assert_eq!(dk, sort_key);
    assert_eq!(di, id);
}

// 空文字列でもカーソルのラウンドトリップが正常に機能することを確認する。
#[test]
fn cursor_roundtrip_empty_strings() {
    let encoded = encode_cursor("", "");
    let (dk, di) = decode_cursor(&encoded).unwrap();
    assert_eq!(dk, "");
    assert_eq!(di, "");
}

// Unicode文字を含むカーソルが正しくエンコード・デコードされることを確認する。
#[test]
fn cursor_roundtrip_unicode() {
    let sort_key = "日本語ソートキー";
    let id = "uuid-日本語";
    let encoded = encode_cursor(sort_key, id);
    let (dk, di) = decode_cursor(&encoded).unwrap();
    assert_eq!(dk, sort_key);
    assert_eq!(di, id);
}

// 特殊文字を含むカーソルが正しくエンコード・デコードされることを確認する。
#[test]
fn cursor_roundtrip_special_characters() {
    let sort_key = "key with spaces & symbols!@#$%";
    let id = "id/with/slashes";
    let encoded = encode_cursor(sort_key, id);
    let (dk, di) = decode_cursor(&encoded).unwrap();
    assert_eq!(dk, sort_key);
    assert_eq!(di, id);
}

// IDにパイプ文字が含まれる場合もセパレーターが正しく扱われることを確認する。
#[test]
fn cursor_roundtrip_pipe_in_id() {
    // The separator is '|'. If the id contains '|', split_once should only split on the first.
    let sort_key = "key";
    let id = "id|with|pipes";
    let encoded = encode_cursor(sort_key, id);
    let (dk, di) = decode_cursor(&encoded).unwrap();
    assert_eq!(dk, sort_key);
    assert_eq!(di, id);
}

// 無効なBase64文字列のデコードがエラーを返すことを確認する。
#[test]
fn cursor_decode_invalid_base64() {
    let result = decode_cursor("!!!not-base64!!!");
    assert!(result.is_err());
    match result.unwrap_err() {
        PerPageValidationError::InvalidCursor(_) => {}
        other => panic!("expected InvalidCursor, got: {:?}", other),
    }
}

// セパレーターを含まないBase64文字列のデコードがエラーを返すことを確認する。
#[test]
fn cursor_decode_valid_base64_but_no_separator() {
    // "noseparator" base64-encoded does not contain '|'
    use base64::{engine::general_purpose::STANDARD, Engine};
    let encoded = STANDARD.encode(b"noseparator");
    let result = decode_cursor(&encoded);
    assert!(result.is_err());
}

// 空文字列のカーソルデコードがエラーを返すことを確認する。
#[test]
fn cursor_decode_empty_string() {
    // Empty string is valid base64 that decodes to empty bytes -> no separator
    let result = decode_cursor("");
    assert!(result.is_err());
}

// ============================================================
// CursorRequest / CursorMeta construction
// ============================================================

// カーソルなしのCursorRequestが正しく構築されることを確認する。
#[test]
fn cursor_request_with_no_cursor() {
    let req = CursorRequest {
        cursor: None,
        limit: 10,
    };
    assert!(req.cursor.is_none());
    assert_eq!(req.limit, 10);
}

// カーソルありのCursorRequestが正しく構築されることを確認する。
#[test]
fn cursor_request_with_cursor() {
    let req = CursorRequest {
        cursor: Some("abc123".to_string()),
        limit: 50,
    };
    assert_eq!(req.cursor.as_deref(), Some("abc123"));
    assert_eq!(req.limit, 50);
}

// リミットが0のCursorRequestが正しく構築されることを確認する。
#[test]
fn cursor_request_zero_limit() {
    let req = CursorRequest {
        cursor: None,
        limit: 0,
    };
    assert_eq!(req.limit, 0);
}

// 最大値のリミットを持つCursorRequestが正しく構築されることを確認する。
#[test]
fn cursor_request_large_limit() {
    let req = CursorRequest {
        cursor: None,
        limit: u32::MAX,
    };
    assert_eq!(req.limit, u32::MAX);
}

// has_moreがtrueのCursorMetaが正しく構築されることを確認する。
#[test]
fn cursor_meta_has_more_true() {
    let meta = CursorMeta {
        next_cursor: Some("next".to_string()),
        has_more: true,
    };
    assert!(meta.has_more);
    assert!(meta.next_cursor.is_some());
}

// has_moreがfalseのCursorMetaが正しく構築されることを確認する。
#[test]
fn cursor_meta_has_more_false() {
    let meta = CursorMeta {
        next_cursor: None,
        has_more: false,
    };
    assert!(!meta.has_more);
    assert!(meta.next_cursor.is_none());
}

// ============================================================
// PageRequest defaults and offset
// ============================================================

// PageRequestのデフォルト値がpage=1、per_page=20であることを確認する。
#[test]
fn page_request_default_values() {
    let req = PageRequest::default();
    assert_eq!(req.page, 1);
    assert_eq!(req.per_page, 20);
}

// default_page_request関数がDefaultトレイトと同じ値を返すことを確認する。
#[test]
fn default_page_request_function_matches_default_trait() {
    let a = default_page_request();
    let b = PageRequest::default();
    assert_eq!(a.page, b.page);
    assert_eq!(a.per_page, b.per_page);
}

// 1ページ目のオフセットが0であることを確認する。
#[test]
fn page_request_offset_page_one() {
    let req = PageRequest {
        page: 1,
        per_page: 20,
    };
    assert_eq!(req.offset(), 0);
}

// 2ページ目のオフセットがper_pageと等しいことを確認する。
#[test]
fn page_request_offset_page_two() {
    let req = PageRequest {
        page: 2,
        per_page: 20,
    };
    assert_eq!(req.offset(), 20);
}

// ページ番号が大きい場合もオフセットが正しく計算されることを確認する。
#[test]
fn page_request_offset_large_page() {
    let req = PageRequest {
        page: 1000,
        per_page: 50,
    };
    assert_eq!(req.offset(), 999 * 50);
}

// 1ページ目で総件数が多い場合has_nextがtrueを返すことを確認する。
#[test]
fn page_request_has_next_true_first_page() {
    let req = PageRequest {
        page: 1,
        per_page: 10,
    };
    assert!(req.has_next(25));
}

// ページ数と総件数が一致する境界でhas_nextがfalseを返すことを確認する。
#[test]
fn page_request_has_next_false_exact_boundary() {
    let req = PageRequest {
        page: 2,
        per_page: 10,
    };
    // page*per_page = 20, total = 20 -> no next
    assert!(!req.has_next(20));
}

// 現在ページが総件数を超える場合has_nextがfalseを返すことを確認する。
#[test]
fn page_request_has_next_false_beyond_total() {
    let req = PageRequest {
        page: 3,
        per_page: 10,
    };
    // page*per_page = 30 > total 25 -> no next
    assert!(!req.has_next(25));
}

// 総件数が0の場合has_nextがfalseを返すことを確認する。
#[test]
fn page_request_has_next_total_zero() {
    let req = PageRequest {
        page: 1,
        per_page: 10,
    };
    assert!(!req.has_next(0));
}

// ============================================================
// validate_per_page
// ============================================================

// per_pageの最小値（1）が有効であることを確認する。
#[test]
fn validate_per_page_min_boundary() {
    assert_eq!(validate_per_page(1).unwrap(), 1);
}

// per_pageの最大値（100）が有効であることを確認する。
#[test]
fn validate_per_page_max_boundary() {
    assert_eq!(validate_per_page(100).unwrap(), 100);
}

// per_pageの中間値（50）が有効であることを確認する。
#[test]
fn validate_per_page_mid_value() {
    assert_eq!(validate_per_page(50).unwrap(), 50);
}

// per_pageが0の場合にエラーが返されることを確認する。
#[test]
fn validate_per_page_zero_is_error() {
    let err = validate_per_page(0).unwrap_err();
    match err {
        PerPageValidationError::InvalidPerPage { value, min, max } => {
            assert_eq!(value, 0);
            assert_eq!(min, 1);
            assert_eq!(max, 100);
        }
        other => panic!("expected InvalidPerPage, got: {:?}", other),
    }
}

// per_pageが最大値を超える場合にエラーが返されることを確認する。
#[test]
fn validate_per_page_exceeds_max() {
    assert!(validate_per_page(101).is_err());
}

// per_pageに最大のu32値を渡した場合にエラーが返されることを確認する。
#[test]
fn validate_per_page_far_exceeds_max() {
    assert!(validate_per_page(u32::MAX).is_err());
}

// ============================================================
// PageResponse construction and metadata
// ============================================================

// PageResponseが基本的な情報を正しく保持することを確認する。
#[test]
fn page_response_new_basic() {
    let req = PageRequest {
        page: 1,
        per_page: 10,
    };
    let items = vec![1, 2, 3, 4, 5];
    let resp = PageResponse::new(items, 25, &req);
    assert_eq!(resp.items.len(), 5);
    assert_eq!(resp.total, 25);
    assert_eq!(resp.page, 1);
    assert_eq!(resp.per_page, 10);
    assert_eq!(resp.total_pages, 3); // ceil(25/10)
}

// 総件数がper_pageで割り切れる場合にtotal_pagesが正しく計算されることを確認する。
#[test]
fn page_response_exact_division() {
    let req = PageRequest {
        page: 1,
        per_page: 5,
    };
    let resp: PageResponse<i32> = PageResponse::new(vec![], 20, &req);
    assert_eq!(resp.total_pages, 4); // 20/5 = 4 exact
}

// 総件数が0の場合にPageResponseが正しく構築されることを確認する。
#[test]
fn page_response_zero_total() {
    let req = PageRequest {
        page: 1,
        per_page: 10,
    };
    let resp: PageResponse<i32> = PageResponse::new(vec![], 0, &req);
    assert_eq!(resp.total_pages, 0);
    assert_eq!(resp.total, 0);
}

// per_pageが0の場合でもPageResponseがパニックせず0ページを返すことを確認する。
#[test]
fn page_response_per_page_zero_no_panic() {
    let req = PageRequest {
        page: 1,
        per_page: 0,
    };
    let resp: PageResponse<i32> = PageResponse::new(vec![], 100, &req);
    assert_eq!(resp.total_pages, 0);
}

// 総件数が1件の場合にtotal_pagesが1であることを確認する。
#[test]
fn page_response_single_item_total() {
    let req = PageRequest {
        page: 1,
        per_page: 10,
    };
    let resp = PageResponse::new(vec!["one"], 1, &req);
    assert_eq!(resp.total_pages, 1);
    assert_eq!(resp.items.len(), 1);
}

// ============================================================
// PaginationMeta
// ============================================================

// PageResponseからPaginationMetaが正しく生成されることを確認する。
#[test]
fn pagination_meta_from_response() {
    let req = PageRequest {
        page: 3,
        per_page: 15,
    };
    let resp: PageResponse<i32> = PageResponse::new(vec![1, 2], 50, &req);
    let meta = resp.meta();
    assert_eq!(meta.total, 50);
    assert_eq!(meta.page, 3);
    assert_eq!(meta.per_page, 15);
    assert_eq!(meta.total_pages, 4); // ceil(50/15) = 4
}

// PaginationMetaが直接構築されたときに全フィールドが正しく保持されることを確認する。
#[test]
fn pagination_meta_direct_construction() {
    let meta = PaginationMeta {
        total: 200,
        page: 5,
        per_page: 25,
        total_pages: 8,
    };
    assert_eq!(meta.total, 200);
    assert_eq!(meta.page, 5);
    assert_eq!(meta.per_page, 25);
    assert_eq!(meta.total_pages, 8);
}

// ============================================================
// PaginationError type alias
// ============================================================

// PaginationErrorがPerPageValidationErrorの型エイリアスとして機能することを確認する。
#[test]
fn pagination_error_is_alias_for_per_page_validation_error() {
    // Verify the type alias compiles and works
    fn takes_pagination_error(_e: PaginationError) {}
    let err = PerPageValidationError::InvalidCursor("test".to_string());
    takes_pagination_error(err);
}

// InvalidCursorエラーの表示メッセージが正しい内容を含むことを確認する。
#[test]
fn pagination_error_display_invalid_cursor() {
    let err = PaginationError::InvalidCursor("bad data".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("invalid cursor"));
    assert!(msg.contains("bad data"));
}

// InvalidPerPageエラーの表示メッセージが値と範囲を含むことを確認する。
#[test]
fn pagination_error_display_invalid_per_page() {
    let err = PaginationError::InvalidPerPage {
        value: 200,
        min: 1,
        max: 100,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("200"));
    assert!(msg.contains("1"));
    assert!(msg.contains("100"));
}
