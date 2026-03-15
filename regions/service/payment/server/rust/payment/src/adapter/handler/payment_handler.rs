use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use k1s0_server_common::error::ServiceError;
use serde::Deserialize;
use uuid::Uuid;

use super::AppState;
use crate::adapter::presenter::payment_presenter::{PaymentDetailResponse, PaymentListResponse};
use crate::domain::entity::payment::{InitiatePayment, PaymentFilter, PaymentStatus};
use crate::domain::error::PaymentError;

/// 決済開始リクエストボディ。
#[derive(Debug, Deserialize)]
pub struct InitiatePaymentRequest {
    pub order_id: String,
    pub customer_id: String,
    pub amount: i64,
    pub currency: String,
    pub payment_method: Option<String>,
}

/// 決済完了リクエストボディ。
#[derive(Debug, Deserialize)]
pub struct CompletePaymentRequest {
    pub transaction_id: String,
}

/// 決済失敗リクエストボディ。
#[derive(Debug, Deserialize)]
pub struct FailPaymentRequest {
    pub error_code: String,
    pub error_message: String,
}

/// 決済返金リクエストボディ。
#[derive(Debug, Deserialize)]
pub struct RefundPaymentRequest {
    pub reason: Option<String>,
}

/// 一覧取得用クエリパラメータ。
#[derive(Debug, Deserialize)]
pub struct ListPaymentsQuery {
    pub order_id: Option<String>,
    pub customer_id: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn initiate_payment(
    State(state): State<AppState>,
    Json(body): Json<InitiatePaymentRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    let input = InitiatePayment {
        order_id: body.order_id,
        customer_id: body.customer_id,
        amount: body.amount,
        currency: body.currency,
        payment_method: body.payment_method,
    };

    let payment = state
        .initiate_payment_uc
        .execute(&input)
        .await
        .map_err(map_payment_error)?;

    let response = PaymentDetailResponse::from_entity(&payment);
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_payment(
    State(state): State<AppState>,
    Path(payment_id): Path<String>,
) -> Result<impl IntoResponse, ServiceError> {
    let id = parse_uuid(&payment_id)?;

    let payment = state
        .get_payment_uc
        .execute(id)
        .await
        .map_err(map_payment_error)?;

    let response = PaymentDetailResponse::from_entity(&payment);
    Ok(Json(response))
}

pub async fn list_payments(
    State(state): State<AppState>,
    Query(query): Query<ListPaymentsQuery>,
) -> Result<impl IntoResponse, ServiceError> {
    let status = match &query.status {
        Some(s) => Some(
            s.parse::<PaymentStatus>()
                .map_err(|_| ServiceError::BadRequest {
                    code: k1s0_server_common::error::ErrorCode::new(
                        "SVC_PAYMENT_VALIDATION_FAILED",
                    ),
                    message: format!("invalid payment status: '{}'", s),
                    details: vec![],
                })?,
        ),
        None => None,
    };

    let filter = PaymentFilter {
        order_id: query.order_id,
        customer_id: query.customer_id,
        status,
        limit: query.limit.or(Some(50)),
        offset: query.offset.or(Some(0)),
    };

    let (payments, total) = state
        .list_payments_uc
        .execute(&filter)
        .await
        .map_err(map_payment_error)?;

    let response = PaymentListResponse::from_entities(&payments, total);
    Ok(Json(response))
}

pub async fn complete_payment(
    State(state): State<AppState>,
    Path(payment_id): Path<String>,
    Json(body): Json<CompletePaymentRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    let id = parse_uuid(&payment_id)?;

    let payment = state
        .complete_payment_uc
        .execute(id, &body.transaction_id)
        .await
        .map_err(map_payment_error)?;

    let response = PaymentDetailResponse::from_entity(&payment);
    Ok(Json(response))
}

pub async fn fail_payment(
    State(state): State<AppState>,
    Path(payment_id): Path<String>,
    Json(body): Json<FailPaymentRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    let id = parse_uuid(&payment_id)?;

    let payment = state
        .fail_payment_uc
        .execute(id, &body.error_code, &body.error_message)
        .await
        .map_err(map_payment_error)?;

    let response = PaymentDetailResponse::from_entity(&payment);
    Ok(Json(response))
}

pub async fn refund_payment(
    State(state): State<AppState>,
    Path(payment_id): Path<String>,
    Json(_body): Json<RefundPaymentRequest>,
) -> Result<impl IntoResponse, ServiceError> {
    let id = parse_uuid(&payment_id)?;

    let payment = state
        .refund_payment_uc
        .execute(id)
        .await
        .map_err(map_payment_error)?;

    let response = PaymentDetailResponse::from_entity(&payment);
    Ok(Json(response))
}

/// UUID パースヘルパー。
fn parse_uuid(s: &str) -> Result<Uuid, ServiceError> {
    Uuid::parse_str(s).map_err(|_| ServiceError::BadRequest {
        code: k1s0_server_common::error::ErrorCode::new("SVC_PAYMENT_VALIDATION_FAILED"),
        message: format!("invalid payment_id format: '{}'", s),
        details: vec![k1s0_server_common::error::ErrorDetail::new(
            "payment_id",
            "invalid_format",
            "must be a valid UUID",
        )],
    })
}

/// anyhow::Error を ServiceError に変換する。
///
/// PaymentError がダウンキャスト可能な場合は型安全に変換し、
/// それ以外は Internal エラーとして扱う。
fn map_payment_error(err: anyhow::Error) -> ServiceError {
    match err.downcast::<PaymentError>() {
        Ok(payment_err) => payment_err.into(),
        Err(other) => ServiceError::Internal {
            code: k1s0_server_common::error::ErrorCode::new("SVC_PAYMENT_INTERNAL_ERROR"),
            message: other.to_string(),
        },
    }
}
