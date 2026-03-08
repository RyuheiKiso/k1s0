use crate::domain::repository::item_repository::ItemRepository;
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

/// アイテムのドメインルールを検証するサービス。
pub struct ItemDomainService;

impl ItemDomainService {
    /// 指定されたアイテムに parent_item_id を設定した場合に循環参照が発生しないか検証する。
    ///
    /// parent_item_id から親チェーンをたどり、item_id に到達した場合はエラーを返す。
    pub async fn check_circular_parent(
        item_repo: &Arc<dyn ItemRepository>,
        item_id: Uuid,
        parent_item_id: Uuid,
    ) -> anyhow::Result<()> {
        if item_id == parent_item_id {
            anyhow::bail!("Circular parent detected: item cannot be its own parent");
        }

        let mut visited = HashSet::new();
        visited.insert(item_id);

        let mut current_id = parent_item_id;
        loop {
            if !visited.insert(current_id) {
                anyhow::bail!("Circular parent detected: setting this parent would create a cycle");
            }

            let Some(parent) = item_repo.find_by_id(current_id).await? else {
                // 親が見つからない場合は循環ではないので終了
                break;
            };

            match parent.parent_item_id {
                Some(next_parent_id) => {
                    current_id = next_parent_id;
                }
                None => {
                    // ルートに到達 — 循環なし
                    break;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::master_item::MasterItem;
    use crate::domain::repository::item_repository::MockItemRepository;
    use chrono::Utc;

    /// テスト用の MasterItem を生成するヘルパー。
    fn make_item(id: Uuid, parent_item_id: Option<Uuid>) -> MasterItem {
        MasterItem {
            id,
            category_id: Uuid::new_v4(),
            code: "TEST".to_string(),
            display_name: "Test Item".to_string(),
            description: None,
            attributes: None,
            parent_item_id,
            effective_from: None,
            effective_until: None,
            is_active: true,
            sort_order: 0,
            created_by: "test".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// 自分自身を親に設定 → 即座にエラー。
    #[tokio::test]
    async fn test_same_item_as_parent_returns_error() {
        let mock = MockItemRepository::new();
        let repo: Arc<dyn ItemRepository> = Arc::new(mock);
        let id = Uuid::new_v4();

        let result = ItemDomainService::check_circular_parent(&repo, id, id).await;

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cannot be its own parent")
        );
    }

    /// 有効な親チェーン: A → B (B の親は None) → 循環なし。
    #[tokio::test]
    async fn test_valid_parent_no_cycle() {
        let item_a = Uuid::new_v4();
        let item_b = Uuid::new_v4();

        let mut mock = MockItemRepository::new();
        let b_entity = make_item(item_b, None);
        mock.expect_find_by_id()
            .withf(move |id| *id == item_b)
            .times(1)
            .returning(move |_| Ok(Some(b_entity.clone())));

        let repo: Arc<dyn ItemRepository> = Arc::new(mock);
        let result = ItemDomainService::check_circular_parent(&repo, item_a, item_b).await;

        assert!(result.is_ok());
    }

    /// 3 要素の循環: A → B → C → A を設定しようとするとエラー。
    /// 既存チェーン: C.parent = B, B.parent = A。
    /// item_id = A, parent_item_id = C とすると A → C → B → A で循環検出。
    #[tokio::test]
    async fn test_chain_of_three_creates_cycle() {
        let item_a = Uuid::new_v4();
        let item_b = Uuid::new_v4();
        let item_c = Uuid::new_v4();

        // C の親は B、B の親は A
        let c_entity = make_item(item_c, Some(item_b));
        let b_entity = make_item(item_b, Some(item_a));

        let mut mock = MockItemRepository::new();
        mock.expect_find_by_id()
            .withf(move |id| *id == item_c)
            .times(1)
            .returning(move |_| Ok(Some(c_entity.clone())));
        mock.expect_find_by_id()
            .withf(move |id| *id == item_b)
            .times(1)
            .returning(move |_| Ok(Some(b_entity.clone())));

        let repo: Arc<dyn ItemRepository> = Arc::new(mock);

        // A の parent を C に設定 → C → B → A (visited に A があるので循環)
        let result = ItemDomainService::check_circular_parent(&repo, item_a, item_c).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cycle"));
    }

    /// 有効な長いチェーン: D → C → B → A (A は root) — 循環なし。
    #[tokio::test]
    async fn test_valid_long_chain_no_cycle() {
        let item_a = Uuid::new_v4();
        let item_b = Uuid::new_v4();
        let item_c = Uuid::new_v4();
        let item_d = Uuid::new_v4();

        let c_entity = make_item(item_c, Some(item_b));
        let b_entity = make_item(item_b, Some(item_a));
        let a_entity = make_item(item_a, None);

        let mut mock = MockItemRepository::new();
        mock.expect_find_by_id()
            .withf(move |id| *id == item_c)
            .times(1)
            .returning(move |_| Ok(Some(c_entity.clone())));
        mock.expect_find_by_id()
            .withf(move |id| *id == item_b)
            .times(1)
            .returning(move |_| Ok(Some(b_entity.clone())));
        mock.expect_find_by_id()
            .withf(move |id| *id == item_a)
            .times(1)
            .returning(move |_| Ok(Some(a_entity.clone())));

        let repo: Arc<dyn ItemRepository> = Arc::new(mock);

        // D の parent を C に設定 → C → B → A (root) → 循環なし
        let result = ItemDomainService::check_circular_parent(&repo, item_d, item_c).await;

        assert!(result.is_ok());
    }

    /// 親がリポジトリに見つからない場合 → 循環なしとして正常終了。
    #[tokio::test]
    async fn test_parent_not_found_returns_ok() {
        let item_a = Uuid::new_v4();
        let item_b = Uuid::new_v4();

        let mut mock = MockItemRepository::new();
        mock.expect_find_by_id()
            .withf(move |id| *id == item_b)
            .times(1)
            .returning(|_| Ok(None));

        let repo: Arc<dyn ItemRepository> = Arc::new(mock);
        let result = ItemDomainService::check_circular_parent(&repo, item_a, item_b).await;

        assert!(result.is_ok());
    }
}
