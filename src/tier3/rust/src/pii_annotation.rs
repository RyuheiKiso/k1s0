// pii_annotation.rs — field_pii annotation の Rust 実装（trait + attribute stub）
// PiiAnnotated trait と pii_fields!() マクロ概念宣言を含む
// 実際の proc_macro は将来の pii_derive crate で実装する（ここでは conceptual skeleton）
// Y-tier3-4: field_pii annotation compile-time 強制（4 言語等価強度の Rust 実装）

// ============================================================
// PiiAnnotated trait — PII フィールドを持つ型が実装すべきトレイト
// ============================================================

// PiiAnnotated は PII フィールドを持つ struct が実装する trait
// get_pii_fields: 実装型が PII フィールド名の静的スライスを返す
// 将来は #[derive(PiiAnnotation)] proc_macro が自動実装を生成する
pub trait PiiAnnotated {
    // PII フィールド名の静的スライスを返す（compile-time で確定する）
    fn get_pii_fields() -> &'static [&'static str]
    where
        // Self: Sized を要求して object-safe でない場合のコンパイルエラーを防ぐ
        Self: Sized;

    // has_pii_fields: PII フィールドが 1 件以上存在するかどうかを返す（convenience メソッド）
    fn has_pii_fields() -> bool
    where
        // Self: Sized を要求する
        Self: Sized,
    {
        // get_pii_fields() の返値が空でないかで判定する
        !Self::get_pii_fields().is_empty()
    }
}

// ============================================================
// #[pii] attribute stub 概念宣言
// ============================================================
// 将来の proc_macro crate (pii_derive) で実装する予定の attribute:
//
//   #[derive(PiiAnnotation)]
//   struct PersonalInfo {
//       #[pii]
//       email: String,
//       #[pii]
//       name: String,
//       tenant_id: String,  // PII でないフィールド
//   }
//
// 上記の derive により以下が自動生成される:
//   impl PiiAnnotated for PersonalInfo {
//       fn get_pii_fields() -> &'static [&'static str] {
//           &["email", "name"]
//       }
//   }
//
// 現状では proc_macro が未実装のため、手動実装 skeleton を用意する。

// ============================================================
// PiiAnnotationError — PII annotation 関連のエラー型
// ============================================================

// PiiAnnotationError は PII annotation 検証に関するエラー型
#[derive(Debug, PartialEq, Eq)]
pub enum PiiAnnotationError {
    // PII フィールドが strip されずに payload に残存している場合のエラー
    PiiFieldsNotStripped {
        // strip されていない PII フィールド名のリスト
        remaining_fields: Vec<String>,
    },
    // 対象型が PiiAnnotated を実装していないエラー（runtime 検出）
    NotAnnotated {
        // 型名（エラーメッセージに使用する）
        type_name: &'static str,
    },
}

impl std::fmt::Display for PiiAnnotationError {
    // Display トレイトの実装（エラーメッセージを人間が読める形式で返す）
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // エラー種別に応じたメッセージを生成する
        match self {
            // PiiFieldsNotStripped: strip 漏れフィールドを含むエラーメッセージを返す
            PiiAnnotationError::PiiFieldsNotStripped { remaining_fields } => {
                // strip 漏れフィールド名をカンマ区切りで連結する
                write!(
                    f,
                    "PII fields not stripped from payload: [{}]",
                    remaining_fields.join(", ")
                )
            }
            // NotAnnotated: 型名を含むエラーメッセージを返す
            PiiAnnotationError::NotAnnotated { type_name } => {
                // 型名を含むエラーメッセージを書き込む
                write!(f, "type '{}' does not implement PiiAnnotated", type_name)
            }
        }
    }
}

// ============================================================
// validate_pii_stripped — payload から PII フィールドが除去済みかを検証する汎用関数
// ============================================================

// validate_pii_stripped は PiiAnnotated を実装する型 T の PII フィールドが
// payload（HashMap<&str, V>）から strip 済みかどうかを検証する
// T: PiiAnnotated を実装する型（compile-time で PII フィールド名が確定する）
// payload: PII strip 済みのはずの送信データ（&str → V の HashMap）
// 返値: strip 済みの場合は Ok(())、PII フィールドが残存する場合は Err(PiiAnnotationError)
pub fn validate_pii_stripped<T, V>(
    // PII strip 済みのはずの送信データ（&str → V の HashMap）
    payload: &std::collections::HashMap<&str, V>,
) -> Result<(), PiiAnnotationError>
where
    // T: PiiAnnotated + Sized を要求する
    T: PiiAnnotated + Sized,
{
    // T の PII フィールド名リストを取得する（compile-time 確定）
    let pii_fields = T::get_pii_fields();
    // payload に残存する PII フィールド名を収集する
    let remaining: Vec<String> = pii_fields
        .iter()
        // payload に存在するフィールドをフィルタする
        .filter(|&&field| payload.contains_key(field))
        // String に変換する
        .map(|&field| field.to_owned())
        .collect();
    // 残存する PII フィールドがある場合はエラーを返す
    if remaining.is_empty() {
        // 全 PII フィールドが strip 済み
        Ok(())
    } else {
        // 残存する PII フィールドを含むエラーを返す
        Err(PiiAnnotationError::PiiFieldsNotStripped {
            remaining_fields: remaining,
        })
    }
}

// ============================================================
// 手動実装 skeleton の例（proc_macro が実装されるまでの参照実装）
// ============================================================

// ExamplePiiRecord は @FieldPii 相当のアノテーション手動実装の例
// 将来は #[derive(PiiAnnotation)] で自動生成される
#[derive(Debug, Clone)]
pub struct ExamplePiiRecord {
    // PII フィールド: email（手動で get_pii_fields に列挙する必要がある）
    pub email: String,
    // PII フィールド: name（手動で get_pii_fields に列挙する必要がある）
    pub name: String,
    // PII でないフィールド: tenant_id（get_pii_fields に含めない）
    pub tenant_id: String,
}

impl PiiAnnotated for ExamplePiiRecord {
    // get_pii_fields: email と name を PII フィールドとして返す
    fn get_pii_fields() -> &'static [&'static str] {
        // 静的スライスとして PII フィールド名を返す（compile-time 確定）
        &["email", "name"]
    }
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;
    // HashMap をテスト内で使用する
    use std::collections::HashMap;

    #[test]
    // ExamplePiiRecord の get_pii_fields が正しいフィールド名を返すことを確認する
    fn test_get_pii_fields_returns_annotated_fields() {
        // get_pii_fields が ["email", "name"] を返すことを確認する
        let fields = ExamplePiiRecord::get_pii_fields();
        // フィールド数が 2 件であることを確認する
        assert_eq!(fields.len(), 2);
        // email が含まれることを確認する
        assert!(fields.contains(&"email"));
        // name が含まれることを確認する
        assert!(fields.contains(&"name"));
        // tenant_id が含まれないことを確認する
        assert!(!fields.contains(&"tenant_id"));
    }

    #[test]
    // validate_pii_stripped が PII フィールドを含む payload を検出することを確認する
    fn test_validate_pii_stripped_detects_remaining_pii() {
        // email を含む（strip されていない）payload を用意する
        let mut payload: HashMap<&str, &str> = HashMap::new();
        // PII フィールド（email）を payload に含めて strip 漏れを再現する
        payload.insert("email", "test@example.com");
        // tenant_id は PII でないので含んでも良い
        payload.insert("tenant_id", "t-001");
        // validate_pii_stripped が Err を返すことを確認する
        let result = validate_pii_stripped::<ExamplePiiRecord, &str>(&payload);
        // PII フィールドが残存しているため Err が返ることを確認する
        assert!(matches!(
            result,
            Err(PiiAnnotationError::PiiFieldsNotStripped { .. })
        ));
    }

    #[test]
    // validate_pii_stripped が PII フィールドが strip 済みの payload を受理することを確認する
    fn test_validate_pii_stripped_accepts_clean_payload() {
        // PII フィールドを含まない（strip 済みの）payload を用意する
        let mut payload: HashMap<&str, &str> = HashMap::new();
        // tenant_id のみ含む（PII でないフィールドのみ）
        payload.insert("tenant_id", "t-001");
        // validate_pii_stripped が Ok を返すことを確認する
        let result = validate_pii_stripped::<ExamplePiiRecord, &str>(&payload);
        // PII フィールドが strip 済みのため Ok が返ることを確認する
        assert!(result.is_ok());
    }
}
