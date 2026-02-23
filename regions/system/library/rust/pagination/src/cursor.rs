use base64::{engine::general_purpose::STANDARD, Engine};

use crate::error::PaginationError;

pub fn encode_cursor(id: &str) -> String {
    STANDARD.encode(id.as_bytes())
}

pub fn decode_cursor(cursor: &str) -> Result<String, PaginationError> {
    let bytes = STANDARD
        .decode(cursor)
        .map_err(|e| PaginationError::InvalidCursor(e.to_string()))?;
    String::from_utf8(bytes).map_err(|e| PaginationError::InvalidCursor(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let id = "some-record-id-123";
        let encoded = encode_cursor(id);
        let decoded = decode_cursor(&encoded).unwrap();
        assert_eq!(decoded, id);
    }

    #[test]
    fn test_decode_invalid_cursor() {
        assert!(decode_cursor("!!!not-base64!!!").is_err());
    }

    #[test]
    fn test_encode_empty_string() {
        let encoded = encode_cursor("");
        let decoded = decode_cursor(&encoded).unwrap();
        assert_eq!(decoded, "");
    }
}
