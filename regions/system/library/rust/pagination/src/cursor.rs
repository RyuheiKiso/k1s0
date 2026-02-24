use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};

use crate::error::PaginationError;

/// Cursor-based pagination request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorRequest {
    pub cursor: Option<String>,
    pub limit: u32,
}

/// Cursor-based pagination response metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorMeta {
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

const SEPARATOR: char = '|';

/// Encode a sort key and id into a base64 cursor string.
pub fn encode_cursor(sort_key: &str, id: &str) -> String {
    let combined = format!("{}{}{}", sort_key, SEPARATOR, id);
    STANDARD.encode(combined.as_bytes())
}

/// Decode a base64 cursor string into (sort_key, id).
pub fn decode_cursor(cursor: &str) -> Result<(String, String), PaginationError> {
    let bytes = STANDARD
        .decode(cursor)
        .map_err(|e| PaginationError::InvalidCursor(e.to_string()))?;
    let combined =
        String::from_utf8(bytes).map_err(|e| PaginationError::InvalidCursor(e.to_string()))?;
    let (sort_key, id) = combined
        .split_once(SEPARATOR)
        .ok_or_else(|| PaginationError::InvalidCursor("missing separator".to_string()))?;
    Ok((sort_key.to_string(), id.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let sort_key = "2024-01-15T10:30:00Z";
        let id = "some-record-id-123";
        let encoded = encode_cursor(sort_key, id);
        let (decoded_sort_key, decoded_id) = decode_cursor(&encoded).unwrap();
        assert_eq!(decoded_sort_key, sort_key);
        assert_eq!(decoded_id, id);
    }

    #[test]
    fn test_decode_invalid_cursor() {
        assert!(decode_cursor("!!!not-base64!!!").is_err());
    }

    #[test]
    fn test_decode_missing_separator() {
        let encoded = STANDARD.encode(b"noseparator");
        assert!(decode_cursor(&encoded).is_err());
    }

    #[test]
    fn test_cursor_request_default() {
        let req = CursorRequest {
            cursor: None,
            limit: 20,
        };
        assert!(req.cursor.is_none());
        assert_eq!(req.limit, 20);
    }

    #[test]
    fn test_cursor_meta() {
        let meta = CursorMeta {
            next_cursor: Some("abc".to_string()),
            has_more: true,
        };
        assert_eq!(meta.next_cursor, Some("abc".to_string()));
        assert!(meta.has_more);
    }
}
