// AI Gateway gRPCサービスのビジネスロジック層。
// gRPCリクエストをドメイン層のユースケースに委譲する。

use std::sync::Arc;

use crate::usecase::complete::{CompleteError, CompleteInput, CompleteOutput, MessageInput};
use crate::usecase::embed::{EmbedError, EmbedInput, EmbedOutput};
use crate::usecase::get_usage::{GetUsageInput, GetUsageOutput};
use crate::usecase::list_models::ListModelsOutput;
use crate::usecase::{CompleteUseCase, EmbedUseCase, GetUsageUseCase, ListModelsUseCase};

/// gRPCエラー型。tonicのStatusに変換可能。
#[derive(Debug)]
pub enum GrpcError {
    InvalidArgument(String),
    NotFound(String),
    Internal(String),
}

/// gRPC補完リクエスト
pub struct GrpcCompleteRequest {
    pub model: String,
    pub messages: Vec<GrpcMessage>,
    pub max_tokens: i32,
    pub tenant_id: String,
    pub strategy: String,
}

/// gRPCメッセージ
pub struct GrpcMessage {
    pub role: String,
    pub content: String,
}

/// gRPCエンベディングリクエスト
pub struct GrpcEmbedRequest {
    pub model: String,
    pub inputs: Vec<String>,
}

/// gRPC使用量取得リクエスト
pub struct GrpcGetUsageRequest {
    pub tenant_id: String,
    pub start: String,
    pub end: String,
}

/// AI Gateway gRPCサービス。
/// ユースケースを呼び出してgRPC固有の変換を行う。
pub struct AiGatewayGrpcService {
    complete_uc: Arc<CompleteUseCase>,
    embed_uc: Arc<EmbedUseCase>,
    list_models_uc: Arc<ListModelsUseCase>,
    get_usage_uc: Arc<GetUsageUseCase>,
}

impl AiGatewayGrpcService {
    /// 新しいgRPCサービスを生成する。
    pub fn new(
        complete_uc: Arc<CompleteUseCase>,
        embed_uc: Arc<EmbedUseCase>,
        list_models_uc: Arc<ListModelsUseCase>,
        get_usage_uc: Arc<GetUsageUseCase>,
    ) -> Self {
        Self {
            complete_uc,
            embed_uc,
            list_models_uc,
            get_usage_uc,
        }
    }

    /// テキスト補完を実行する。
    pub async fn complete(
        &self,
        req: GrpcCompleteRequest,
    ) -> Result<CompleteOutput, GrpcError> {
        let input = CompleteInput {
            model: req.model,
            messages: req
                .messages
                .into_iter()
                .map(|m| MessageInput {
                    role: m.role,
                    content: m.content,
                })
                .collect(),
            max_tokens: req.max_tokens,
            tenant_id: req.tenant_id,
            strategy: req.strategy,
        };

        self.complete_uc.execute(input).await.map_err(|e| match e {
            CompleteError::GuardrailViolation(msg) => GrpcError::InvalidArgument(msg),
            CompleteError::ModelNotFound(msg) => GrpcError::NotFound(msg),
            CompleteError::LlmError(msg) => GrpcError::Internal(msg),
            CompleteError::Internal(msg) => GrpcError::Internal(msg),
        })
    }

    /// エンベディングを実行する。
    pub async fn embed(&self, req: GrpcEmbedRequest) -> Result<EmbedOutput, GrpcError> {
        let input = EmbedInput {
            model: req.model,
            inputs: req.inputs,
        };

        self.embed_uc
            .execute(input)
            .await
            .map_err(|EmbedError::LlmError(msg)| GrpcError::Internal(msg))
    }

    /// モデル一覧を取得する。
    pub async fn list_models(&self) -> ListModelsOutput {
        self.list_models_uc.execute().await
    }

    /// 使用量を取得する。
    pub async fn get_usage(&self, req: GrpcGetUsageRequest) -> GetUsageOutput {
        let input = GetUsageInput {
            tenant_id: req.tenant_id,
            start: req.start,
            end: req.end,
        };
        self.get_usage_uc.execute(input).await
    }
}
