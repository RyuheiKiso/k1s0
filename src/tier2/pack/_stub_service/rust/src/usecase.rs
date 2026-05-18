//! usecase.rs — _stub_service ユースケース層
//! 業界中立語のみを使用する stub service のユースケース実装
//! manufacturing pack と完全に語彙を分離する（pack 間の依存禁止）
//! テナント識別子は TenantContext 経由でのみ取得する（API 引数経由は禁止）

// UUID ライブラリのインポート（集約 ID / リソース ID に使用する）
use uuid::Uuid;
// シリアライズライブラリのインポート（ユースケース結果の JSON 化に使用する）
use serde::{Deserialize, Serialize};

// UsecaseError: ユースケース処理中に発生するエラーの列挙型
#[derive(Debug)]
pub enum UsecaseError {
    // リソースが見つからない場合に発生する
    NotFound(Uuid),
    // リソースの状態が操作に対して不正な場合に発生する
    InvalidState(String),
    // 永続化に失敗した場合に発生する
    PersistenceFailure(String),
}

// impl Display for UsecaseError: エラーメッセージを文字列化する
impl std::fmt::Display for UsecaseError {
    // fmt: エラーメッセージを文字列化して返す
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // エラー種別ごとにメッセージを返す
        match self {
            // リソース未発見エラーメッセージを返す
            UsecaseError::NotFound(id) => write!(f, "リソースが見つからない: {}", id),
            // 不正状態エラーメッセージを返す
            UsecaseError::InvalidState(msg) => write!(f, "不正な状態: {}", msg),
            // 永続化失敗エラーメッセージを返す
            UsecaseError::PersistenceFailure(msg) => write!(f, "永続化失敗: {}", msg),
        }
    }
}

// ResourceStatus: リソースの状態を表す業界中立型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceStatus {
    // アクティブ（利用可能）状態
    Active,
    // 処理中（操作実行中）状態
    Processing,
    // 完了（処理が終了した）状態
    Completed,
    // 無効（使用不可）状態
    Inactive,
}

// ResourceStatusStr: ResourceStatus を文字列に変換するヘルパー
impl ResourceStatus {
    // as_str: ResourceStatus を文字列スライスに変換する
    pub fn as_str(&self) -> &'static str {
        // 各 enum 値を業界中立語の文字列にマッピングする
        match self {
            // アクティブ状態文字列を返す
            ResourceStatus::Active => "Active",
            // 処理中状態文字列を返す
            ResourceStatus::Processing => "Processing",
            // 完了状態文字列を返す
            ResourceStatus::Completed => "Completed",
            // 無効状態文字列を返す
            ResourceStatus::Inactive => "Inactive",
        }
    }
}

// CreateResourceCommand: リソース作成コマンド
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResourceCommand {
    // テナント ID (AuthContext から取得する、API 引数での受け取りは禁止)
    pub tenant_id: Uuid,
    // リソース種別（業界中立語）
    pub resource_type: String,
    // リソースの表示名（業界中立語）
    pub display_name: String,
}

// ResourceCreated: リソース作成成功時のイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCreated {
    // 生成されたリソースの主キー UUID
    pub resource_id: Uuid,
    // テナント ID
    pub tenant_id: Uuid,
    // リソース種別
    pub resource_type: String,
    // 初期状態（常に Active）
    pub initial_status: ResourceStatus,
}

// ActivateResourceCommand: リソース有効化コマンド
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivateResourceCommand {
    // 有効化するリソースの ID
    pub resource_id: Uuid,
    // テナント ID (AuthContext から取得する)
    pub tenant_id: Uuid,
}

// DeactivateResourceCommand: リソース無効化コマンド
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeactivateResourceCommand {
    // 無効化するリソースの ID
    pub resource_id: Uuid,
    // テナント ID (AuthContext から取得する)
    pub tenant_id: Uuid,
    // 無効化の理由（業界中立語）
    pub reason: String,
}

// StubUsecaseHandler: _stub_service のユースケースハンドラ
// ResourceManager trait を使用してリソース操作を抽象化する
pub struct StubUsecaseHandler<R: crate::ResourceManager> {
    // ResourceManager の実装（依存性逆転で注入する）
    repository: R,
}

// impl StubUsecaseHandler: ユースケースハンドラの実装
impl<R: crate::ResourceManager> StubUsecaseHandler<R> {
    // StubUsecaseHandler を生成する
    // repository: ResourceManager の実装を注入する
    pub fn new(repository: R) -> Self {
        // repository を保持するインスタンスを生成する
        Self { repository }
    }

    // リソースを作成する: CreateResourceCommand を受け取り ResourceCreated を返す
    pub fn handle_create(
        &self,
        cmd: CreateResourceCommand,
    ) -> Result<ResourceCreated, UsecaseError> {
        // リソースを永続化する
        let resource_id = Uuid::new_v4();
        // ResourceManager.create_resource を呼び出す
        self.repository
            .create_resource(&cmd.resource_type)
            .map_err(|e| UsecaseError::PersistenceFailure(e))?;
        // 作成イベントを返す
        Ok(ResourceCreated {
            resource_id,
            tenant_id: cmd.tenant_id,
            resource_type: cmd.resource_type,
            // 初期状態は常に Active とする
            initial_status: ResourceStatus::Active,
        })
    }

    // リソースを有効化する: ActivateResourceCommand を受け取り Result を返す
    pub fn handle_activate(
        &self,
        cmd: ActivateResourceCommand,
    ) -> Result<(), UsecaseError> {
        // リソースの存在確認を行う
        let existing = self.repository
            .get_resource(&cmd.resource_id.to_string())
            .map_err(|e| UsecaseError::PersistenceFailure(e))?;
        // リソースが存在しない場合はエラーを返す
        if existing.is_none() {
            // NotFound エラーを返す
            return Err(UsecaseError::NotFound(cmd.resource_id));
        }
        // 有効化処理を実行する (stub では何もしない)
        Ok(())
    }

    // リソースを無効化する: DeactivateResourceCommand を受け取り Result を返す
    pub fn handle_deactivate(
        &self,
        cmd: DeactivateResourceCommand,
    ) -> Result<(), UsecaseError> {
        // リソースの存在確認を行う
        let existing = self.repository
            .get_resource(&cmd.resource_id.to_string())
            .map_err(|e| UsecaseError::PersistenceFailure(e))?;
        // リソースが存在しない場合はエラーを返す
        if existing.is_none() {
            // NotFound エラーを返す
            return Err(UsecaseError::NotFound(cmd.resource_id));
        }
        // 無効化処理を実行する (stub では理由をログに記録するのみ)
        let _ = &cmd.reason;
        // 無効化成功を返す
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;
    // StubResourceManager をインポートする
    use crate::StubResourceManager;

    // handle_create が ResourceCreated を返すことを確認する
    #[test]
    fn test_handle_create_returns_resource_created() {
        // StubUsecaseHandler を生成する
        let handler = StubUsecaseHandler::new(StubResourceManager);
        // CreateResourceCommand を生成する
        let cmd = CreateResourceCommand {
            tenant_id: Uuid::new_v4(),
            resource_type: "generic_item".to_string(),
            display_name: "テスト汎用リソース".to_string(),
        };
        // コマンドを実行する
        let result = handler.handle_create(cmd);
        // 成功することを確認する
        assert!(result.is_ok());
        // 初期状態が Active であることを確認する
        assert_eq!(result.unwrap().initial_status, ResourceStatus::Active);
    }

    // handle_activate が存在しないリソースに対して NotFound を返すことを確認する
    #[test]
    fn test_handle_activate_not_found() {
        // StubUsecaseHandler を生成する
        let handler = StubUsecaseHandler::new(StubResourceManager);
        // 存在しないリソース ID を使用する
        let resource_id = Uuid::new_v4();
        // ActivateResourceCommand を生成する
        let cmd = ActivateResourceCommand {
            resource_id,
            tenant_id: Uuid::new_v4(),
        };
        // コマンドを実行する (StubResourceManager は常に None を返す)
        let result = handler.handle_activate(cmd);
        // NotFound エラーが返ることを確認する
        assert!(matches!(result, Err(UsecaseError::NotFound(_))));
    }

    // ResourceStatus::as_str が正しい文字列を返すことを確認する
    #[test]
    fn test_resource_status_as_str() {
        // Active 状態の文字列表現を確認する
        assert_eq!(ResourceStatus::Active.as_str(), "Active");
        // Processing 状態の文字列表現を確認する
        assert_eq!(ResourceStatus::Processing.as_str(), "Processing");
        // Completed 状態の文字列表現を確認する
        assert_eq!(ResourceStatus::Completed.as_str(), "Completed");
        // Inactive 状態の文字列表現を確認する
        assert_eq!(ResourceStatus::Inactive.as_str(), "Inactive");
    }
}
