use std::collections::HashSet;
use uuid::Uuid;

/// 親チェーン解決の抽象化トレイト（M-004）。
/// ItemDomainService が ItemRepository に直接依存しないよう、
/// 親アイテム ID の取得を抽象化する。
/// - アイテムが存在しない場合は Ok(None)（循環なし）
/// - アイテムに親がない（root）場合も Ok(None)（循環なし）
/// - 親が存在する場合は Ok(Some(parent_id))
#[async_trait::async_trait]
pub trait ParentChainResolver: Send + Sync {
    async fn find_parent_id(&self, id: Uuid) -> anyhow::Result<Option<Uuid>>;
}

/// アイテムのドメインルールを検証するサービス。
pub struct ItemDomainService;

impl ItemDomainService {
    /// 指定されたアイテムに parent_item_id を設定した場合に循環参照が発生しないか検証する。
    ///
    /// parent_item_id から親チェーンをたどり、item_id に到達した場合はエラーを返す。
    /// Repository の代わりに ParentChainResolver トレイトを受け取り、
    /// ドメインサービスの Repository 直接依存を除去する（M-004）。
    pub async fn check_circular_parent(
        resolver: &dyn ParentChainResolver,
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

            match resolver.find_parent_id(current_id).await? {
                None => {
                    // アイテムが見つからないかルートに到達 — 循環なし
                    break;
                }
                Some(next_parent_id) => {
                    current_id = next_parent_id;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// テスト用 ParentChainResolver のインメモリ実装。
    /// id → parent_id のマッピングを保持し、存在しない id は None を返す。
    struct InMemoryParentResolver {
        parent_map: Mutex<HashMap<Uuid, Option<Uuid>>>,
    }

    impl InMemoryParentResolver {
        fn new(entries: Vec<(Uuid, Option<Uuid>)>) -> Self {
            let mut map = HashMap::new();
            for (id, parent) in entries {
                map.insert(id, parent);
            }
            Self {
                parent_map: Mutex::new(map),
            }
        }
    }

    #[async_trait::async_trait]
    impl ParentChainResolver for InMemoryParentResolver {
        async fn find_parent_id(&self, id: Uuid) -> anyhow::Result<Option<Uuid>> {
            let map = self.parent_map.lock().unwrap();
            match map.get(&id) {
                None => Ok(None),          // アイテムが存在しない
                Some(parent) => Ok(*parent), // アイテムの親（None = root）
            }
        }
    }

    /// 自分自身を親に設定 → 即座にエラー。
    #[tokio::test]
    async fn test_same_item_as_parent_returns_error() {
        let resolver = InMemoryParentResolver::new(vec![]);
        let id = Uuid::new_v4();

        let result = ItemDomainService::check_circular_parent(&resolver, id, id).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot be its own parent"));
    }

    /// 有効な親チェーン: A → B (B の親は None) → 循環なし。
    #[tokio::test]
    async fn test_valid_parent_no_cycle() {
        let item_a = Uuid::new_v4();
        let item_b = Uuid::new_v4();

        let resolver = InMemoryParentResolver::new(vec![(item_b, None)]);
        let result = ItemDomainService::check_circular_parent(&resolver, item_a, item_b).await;

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
        let resolver = InMemoryParentResolver::new(vec![
            (item_c, Some(item_b)),
            (item_b, Some(item_a)),
        ]);

        // A の parent を C に設定 → C → B → A (visited に A があるので循環)
        let result = ItemDomainService::check_circular_parent(&resolver, item_a, item_c).await;

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

        let resolver = InMemoryParentResolver::new(vec![
            (item_c, Some(item_b)),
            (item_b, Some(item_a)),
            (item_a, None), // root
        ]);

        // D の parent を C に設定 → C → B → A (root) → 循環なし
        let result = ItemDomainService::check_circular_parent(&resolver, item_d, item_c).await;

        assert!(result.is_ok());
    }

    /// 親がリポジトリに見つからない場合 → 循環なしとして正常終了。
    #[tokio::test]
    async fn test_parent_not_found_returns_ok() {
        let item_a = Uuid::new_v4();
        let item_b = Uuid::new_v4();

        // item_b は登録されていないので find_parent_id は None を返す
        let resolver = InMemoryParentResolver::new(vec![]);
        let result = ItemDomainService::check_circular_parent(&resolver, item_a, item_b).await;

        assert!(result.is_ok());
    }
}
