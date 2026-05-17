// k1s0 tier2 manufacturing pack ストレステスト フィクスチャ
// Playwright spec / Testcontainers ストレステストが import するドメイン固有データを提供する
// 9 ストレステストシナリオのフィクスチャデータを定義する

// UUID ライブラリのインポート
use uuid::Uuid;

// テスト用テナント ID の定数定義（固定 UUID でテストの再現性を確保する）
pub const TENANT_A_ID: &str = "00000000-0000-0000-0000-000000000001";
// テスト用テナント B の ID（クロステナント分離テストに使用する）
pub const TENANT_B_ID: &str = "00000000-0000-0000-0000-000000000002";

// テスト用 ResourceUnit フィクスチャを生成するヘルパー
pub fn resource_unit_fixture(tenant_id: Uuid, resource_code: &str) -> serde_json::Value {
    // テスト用の ResourceUnit JSON を返す
    serde_json::json!({
        "id": Uuid::new_v4().to_string(),
        "tenant_id": tenant_id.to_string(),
        "resource_code": resource_code,
        "status": "Active",
        "version": 1,
    })
}

// テスト用 ProductionBatch フィクスチャを生成するヘルパー
pub fn production_batch_fixture(tenant_id: Uuid, batch_code: &str) -> serde_json::Value {
    // テスト用の ProductionBatch JSON を返す
    serde_json::json!({
        "id": Uuid::new_v4().to_string(),
        "tenant_id": tenant_id.to_string(),
        "batch_code": batch_code,
        "item_spec_id": Uuid::new_v4().to_string(),
        "quantity": 100,
        "status": "Planned",
        "version": 1,
    })
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[test]
    // フィクスチャが tenant_id を含むことを確認する
    fn test_fixture_contains_tenant_id() {
        // テナント A の UUID を生成する
        let tenant_id = Uuid::parse_str(TENANT_A_ID).unwrap();
        // ResourceUnit フィクスチャを生成する
        let fixture = resource_unit_fixture(tenant_id, "RC-001");
        // フィクスチャに tenant_id が含まれることを確認する
        assert_eq!(fixture["tenant_id"], TENANT_A_ID);
        assert_eq!(fixture["resource_code"], "RC-001");
    }

    #[test]
    // テナント A とテナント B の ID が異なることを確認する（クロステナントテストの前提）
    fn test_tenant_ids_are_distinct() {
        // テナント A と B の UUID が異なることを確認する
        assert_ne!(TENANT_A_ID, TENANT_B_ID);
        // UUID として parse できることを確認する
        assert!(Uuid::parse_str(TENANT_A_ID).is_ok());
        assert!(Uuid::parse_str(TENANT_B_ID).is_ok());
    }
}
