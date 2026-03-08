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
