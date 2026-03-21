// AIモデルリポジトリのトレイト定義。
// モデル情報の永続化と取得を抽象化する。

use async_trait::async_trait;

use crate::domain::entity::model::AiModel;

/// AIモデルリポジトリのインターフェース。
/// モデル一覧の取得やID指定での検索を提供する。
// テスト時にmockallによるモック自動生成を有効にする
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ModelRepository: Send + Sync {
    /// 全モデルの一覧を取得する。
    async fn find_all(&self) -> Vec<AiModel>;

    /// 指定IDのモデルを取得する。存在しない場合はNone。
    #[allow(dead_code)]
    async fn find_by_id(&self, id: &str) -> Option<AiModel>;
}
