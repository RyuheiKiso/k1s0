use std::sync::Arc;

use super::SagaGrpcService;

/// SagaServiceTonic はtonic gRPCサービスのラッパー。
/// proto生成コード完成後にtonicのServiceトレイトを実装する。
pub struct SagaServiceTonic {
    inner: Arc<SagaGrpcService>,
}

impl SagaServiceTonic {
    pub fn new(inner: Arc<SagaGrpcService>) -> Self {
        Self { inner }
    }

    /// 内部サービスへの参照を返す。
    pub fn inner(&self) -> &SagaGrpcService {
        &self.inner
    }
}
