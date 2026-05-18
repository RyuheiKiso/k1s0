// tier2 第二業界 stub service の実装 (Rust)
// このファイルは pack/manufacturing と同型の公開 API を提供する
// 業界固有語を一切含まないことを CI で検証する

// usecase モジュール: stub service のユースケース層を公開する
pub mod usecase;

// リソース管理の抽象インターフェース
pub trait ResourceManager {
    // リソースの作成: resource_type を受け取り、生成した ID を返す
    fn create_resource(&self, resource_type: &str) -> Result<String, String>;
    // リソースの取得: resource_id を受け取り、存在すれば値を返す
    fn get_resource(&self, resource_id: &str) -> Result<Option<String>, String>;
    // リソースの削除: resource_id を指定してリソースを削除する
    fn delete_resource(&self, resource_id: &str) -> Result<(), String>;
}

// スタブ実装構造体
pub struct StubResourceManager;

// ResourceManager の stub 実装
impl ResourceManager for StubResourceManager {
    // リソース作成: 常に成功するスタブ実装
    fn create_resource(&self, resource_type: &str) -> Result<String, String> {
        // スタブ用の固定 ID を返す
        Ok(format!("stub-{}-001", resource_type))
    }

    // リソース取得: 常に None を返すスタブ実装
    fn get_resource(&self, _resource_id: &str) -> Result<Option<String>, String> {
        // スタブなのでリソースは常に存在しないとして None を返す
        Ok(None)
    }

    // リソース削除: 常に成功するスタブ実装
    fn delete_resource(&self, _resource_id: &str) -> Result<(), String> {
        // スタブなので常に成功として Ok を返す
        Ok(())
    }
}

// コンパイルテスト: 業界固有語が含まれていないことを確認する
#[cfg(test)]
mod tests {
    // StubResourceManager と ResourceManager trait をインポートする
    use super::*;

    // create_resource が成功することを確認するテスト
    #[test]
    fn test_create_resource() {
        // スタブマネージャーを作成する
        let manager = StubResourceManager;
        // リソースを作成する
        let result = manager.create_resource("item");
        // 作成が成功することを確認する
        assert!(result.is_ok());
    }

    // get_resource が None を返すことを確認するテスト
    #[test]
    fn test_get_resource_returns_none() {
        // スタブマネージャーを作成する
        let manager = StubResourceManager;
        // 存在しないリソースを取得する
        let result = manager.get_resource("stub-item-001");
        // スタブは常に None を返すことを確認する
        assert!(result.unwrap().is_none());
    }

    // delete_resource が成功することを確認するテスト
    #[test]
    fn test_delete_resource() {
        // スタブマネージャーを作成する
        let manager = StubResourceManager;
        // リソースを削除する
        let result = manager.delete_resource("stub-item-001");
        // 削除が成功することを確認する
        assert!(result.is_ok());
    }
}
