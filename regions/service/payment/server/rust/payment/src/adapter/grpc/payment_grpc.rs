use crate::domain::entity::payment::Payment as DomainPayment;
use crate::domain::entity::payment::PaymentStatus;
use crate::proto::k1s0::service::payment::v1::payment_service_server::PaymentService;
use crate::proto::k1s0::service::payment::v1::{
    CompletePaymentRequest, CompletePaymentResponse, FailPaymentRequest, FailPaymentResponse,
    GetPaymentRequest, GetPaymentResponse, InitiatePaymentRequest, InitiatePaymentResponse,
    ListPaymentsRequest, ListPaymentsResponse, Payment, RefundPaymentRequest,
    RefundPaymentResponse,
};
use crate::proto::k1s0::system::common::v1::PaginationResult;
use crate::usecase;
use chrono::{DateTime, Utc};
// カスタム Timestamp 型（k1s0.system.common.v1.Timestamp）を使用
use crate::proto::k1s0::system::common::v1::Timestamp;
use k1s0_auth::{actor_from_claims, Claims};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub struct PaymentGrpcService {
    pub initiate_payment_uc: Arc<usecase::initiate_payment::InitiatePaymentUseCase>,
    pub get_payment_uc: Arc<usecase::get_payment::GetPaymentUseCase>,
    pub list_payments_uc: Arc<usecase::list_payments::ListPaymentsUseCase>,
    pub complete_payment_uc: Arc<usecase::complete_payment::CompletePaymentUseCase>,
    pub fail_payment_uc: Arc<usecase::fail_payment::FailPaymentUseCase>,
    pub refund_payment_uc: Arc<usecase::refund_payment::RefundPaymentUseCase>,
}

impl PaymentGrpcService {
    pub fn new(
        initiate_payment_uc: Arc<usecase::initiate_payment::InitiatePaymentUseCase>,
        get_payment_uc: Arc<usecase::get_payment::GetPaymentUseCase>,
        list_payments_uc: Arc<usecase::list_payments::ListPaymentsUseCase>,
        complete_payment_uc: Arc<usecase::complete_payment::CompletePaymentUseCase>,
        fail_payment_uc: Arc<usecase::fail_payment::FailPaymentUseCase>,
        refund_payment_uc: Arc<usecase::refund_payment::RefundPaymentUseCase>,
    ) -> Self {
        Self {
            initiate_payment_uc,
            get_payment_uc,
            list_payments_uc,
            complete_payment_uc,
            fail_payment_uc,
            refund_payment_uc,
        }
    }
}

#[tonic::async_trait]
impl PaymentService for PaymentGrpcService {
    async fn initiate_payment(
        &self,
        request: Request<InitiatePaymentRequest>,
    ) -> Result<Response<InitiatePaymentResponse>, Status> {
        // 認証ミドルウェアが設定したClaimsがない場合は認証エラーを返す
        let claims: &Claims = request
            .extensions()
            .get()
            .ok_or_else(|| Status::unauthenticated("認証情報が見つかりません"))?;
        let _actor = actor_from_claims(Some(claims));
        let req = request.into_inner();
        let input = crate::domain::entity::payment::InitiatePayment {
            order_id: req.order_id,
            customer_id: req.customer_id,
            amount: req.amount,
            currency: req.currency,
            payment_method: req.payment_method,
        };
        let payment = self
            .initiate_payment_uc
            .execute(&input)
            .await
            .map_err(map_anyhow_to_status)?;

        Ok(Response::new(InitiatePaymentResponse {
            payment: Some(proto_payment(payment)),
        }))
    }

    async fn get_payment(
        &self,
        request: Request<GetPaymentRequest>,
    ) -> Result<Response<GetPaymentResponse>, Status> {
        // 認証チェック（readも認証必須）
        request
            .extensions()
            .get::<Claims>()
            .ok_or_else(|| Status::unauthenticated("認証情報が見つかりません"))?;
        let payment_id = parse_uuid(&request.get_ref().payment_id, "payment_id")?;
        let payment = self
            .get_payment_uc
            .execute(payment_id)
            .await
            .map_err(map_anyhow_to_status)?;

        Ok(Response::new(GetPaymentResponse {
            payment: Some(proto_payment(payment)),
        }))
    }

    async fn list_payments(
        &self,
        request: Request<ListPaymentsRequest>,
    ) -> Result<Response<ListPaymentsResponse>, Status> {
        // 認証チェック（readも認証必須）
        request
            .extensions()
            .get::<Claims>()
            .ok_or_else(|| Status::unauthenticated("認証情報が見つかりません"))?;
        let req = request.into_inner();
        let status = req
            .status
            .as_deref()
            .map(|s| s.parse::<PaymentStatus>())
            .transpose()
            .map_err(Status::invalid_argument)?;
        let page = req.pagination.as_ref().map(|p| p.page).unwrap_or(1).max(1);
        let page_size = req
            .pagination
            .as_ref()
            .map(|p| p.page_size)
            .unwrap_or(20)
            .clamp(1, 100);
        let offset = ((page - 1) as i64) * (page_size as i64);

        let filter = crate::domain::entity::payment::PaymentFilter {
            order_id: req.order_id,
            customer_id: req.customer_id,
            status,
            limit: Some(page_size as i64),
            offset: Some(offset),
        };
        let (payments, total_count) = self
            .list_payments_uc
            .execute(&filter)
            .await
            .map_err(map_anyhow_to_status)?;

        let has_next = (page as i64 * page_size as i64) < total_count;
        Ok(Response::new(ListPaymentsResponse {
            payments: payments.into_iter().map(proto_payment).collect(),
            pagination: Some(PaginationResult {
                total_count,
                page,
                page_size,
                has_next,
            }),
        }))
    }

    async fn complete_payment(
        &self,
        request: Request<CompletePaymentRequest>,
    ) -> Result<Response<CompletePaymentResponse>, Status> {
        // 認証ミドルウェアが設定したClaimsがない場合は認証エラーを返す
        let claims: &Claims = request
            .extensions()
            .get()
            .ok_or_else(|| Status::unauthenticated("認証情報が見つかりません"))?;
        let _actor = actor_from_claims(Some(claims));
        let req = request.into_inner();
        let payment_id = parse_uuid(&req.payment_id, "payment_id")?;

        let payment = self
            .complete_payment_uc
            .execute(payment_id, &req.transaction_id)
            .await
            .map_err(map_anyhow_to_status)?;

        Ok(Response::new(CompletePaymentResponse {
            payment: Some(proto_payment(payment)),
        }))
    }

    async fn fail_payment(
        &self,
        request: Request<FailPaymentRequest>,
    ) -> Result<Response<FailPaymentResponse>, Status> {
        // 認証ミドルウェアが設定したClaimsがない場合は認証エラーを返す
        let claims: &Claims = request
            .extensions()
            .get()
            .ok_or_else(|| Status::unauthenticated("認証情報が見つかりません"))?;
        let _actor = actor_from_claims(Some(claims));
        let req = request.into_inner();
        let payment_id = parse_uuid(&req.payment_id, "payment_id")?;

        let payment = self
            .fail_payment_uc
            .execute(payment_id, &req.error_code, &req.error_message)
            .await
            .map_err(map_anyhow_to_status)?;

        Ok(Response::new(FailPaymentResponse {
            payment: Some(proto_payment(payment)),
        }))
    }

    /// gRPC返金ハンドラー。リクエストの返金理由をユースケースに伝播する。
    async fn refund_payment(
        &self,
        request: Request<RefundPaymentRequest>,
    ) -> Result<Response<RefundPaymentResponse>, Status> {
        // 認証ミドルウェアが設定したClaimsがない場合は認証エラーを返す
        let claims: &Claims = request
            .extensions()
            .get()
            .ok_or_else(|| Status::unauthenticated("認証情報が見つかりません"))?;
        let _actor = actor_from_claims(Some(claims));
        let req = request.into_inner();
        let payment_id = parse_uuid(&req.payment_id, "payment_id")?;

        // 返金理由をユースケースに伝播する
        let payment = self
            .refund_payment_uc
            .execute(payment_id, req.reason)
            .await
            .map_err(map_anyhow_to_status)?;

        Ok(Response::new(RefundPaymentResponse {
            payment: Some(proto_payment(payment)),
        }))
    }
}

#[allow(clippy::result_large_err)]
fn parse_uuid(raw: &str, field_name: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(raw)
        .map_err(|_| Status::invalid_argument(format!("invalid {}: '{}'", field_name, raw)))
}

fn proto_payment(payment: DomainPayment) -> Payment {
    // ドメインステータスを proto enum 値にマッピングする
    use crate::proto::k1s0::service::payment::v1::PaymentStatus as ProtoStatus;
    let status_enum = match payment.status {
        PaymentStatus::Initiated => ProtoStatus::Pending as i32,
        PaymentStatus::Completed => ProtoStatus::Succeeded as i32,
        PaymentStatus::Failed => ProtoStatus::Failed as i32,
        PaymentStatus::Refunded => ProtoStatus::Refunded as i32,
    };
    Payment {
        id: payment.id.to_string(),
        order_id: payment.order_id,
        customer_id: payment.customer_id,
        amount: payment.amount,
        currency: payment.currency,
        status: payment.status.as_str().to_string(),
        status_enum,
        payment_method: payment.payment_method,
        transaction_id: payment.transaction_id,
        error_code: payment.error_code,
        error_message: payment.error_message,
        version: payment.version,
        created_at: Some(datetime_to_timestamp(payment.created_at)),
        updated_at: Some(datetime_to_timestamp(payment.updated_at)),
    }
}

fn datetime_to_timestamp(value: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: value.timestamp(),
        nanos: value.timestamp_subsec_nanos() as i32,
    }
}

/// anyhow::Error をドメインエラー型で型ベースに tonic::Status へ変換する。
/// ダウンキャストに失敗した場合は internal エラーとする。
fn map_anyhow_to_status(err: anyhow::Error) -> Status {
    use crate::domain::error::PaymentError;

    match err.downcast::<PaymentError>() {
        Ok(domain_err) => {
            let msg = domain_err.to_string();
            match domain_err {
                PaymentError::NotFound(_) => Status::not_found(msg),
                PaymentError::InvalidStatusTransition { .. } => Status::failed_precondition(msg),
                PaymentError::ValidationFailed(_) => Status::invalid_argument(msg),
                PaymentError::VersionConflict(_) => Status::aborted(msg),
                // 冪等性違反は HTTP 409 Conflict に相当する gRPC already_exists にマッピングする。
                PaymentError::IdempotencyViolation { .. } => Status::already_exists(msg),
                PaymentError::Internal(_) => Status::internal(msg),
            }
        }
        Err(err) => Status::internal(err.to_string()),
    }
}
