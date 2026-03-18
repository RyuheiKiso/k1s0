//! Payment サービス API 統合テスト。
//!
//! in-memory リポジトリを使用して handler レイヤーの統合テストを行う。

use axum::body::Body;
use axum::http::{Request, StatusCode};
use k1s0_payment_server::adapter::handler::{self, AppState};
use k1s0_payment_server::domain::entity::outbox::OutboxEvent;
use k1s0_payment_server::domain::entity::payment::{
    InitiatePayment, Payment, PaymentFilter, PaymentStatus,
};
use k1s0_payment_server::domain::repository::payment_repository::PaymentRepository;
use k1s0_payment_server::usecase;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;
use uuid::Uuid;

/// In-memory リポジトリ実装（統合テスト用）。
/// Arc+RwLock で状態を共有し、各テストで独立したインスタンスを使用する。
#[derive(Default)]
struct InMemoryPaymentRepository {
    payments: RwLock<Vec<Payment>>,
}

#[async_trait::async_trait]
impl PaymentRepository for InMemoryPaymentRepository {
    /// IDで決済を検索する。
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Payment>> {
        let payments = self.payments.read().await;
        Ok(payments.iter().find(|p| p.id == id).cloned())
    }

    /// 注文IDで決済を検索する。
    async fn find_by_order_id(&self, order_id: &str) -> anyhow::Result<Option<Payment>> {
        let payments = self.payments.read().await;
        Ok(payments.iter().find(|p| p.order_id == order_id).cloned())
    }

    /// フィルター条件で決済一覧を取得する。
    async fn find_all(&self, _filter: &PaymentFilter) -> anyhow::Result<Vec<Payment>> {
        let payments = self.payments.read().await;
        Ok(payments.clone())
    }

    /// フィルター条件に一致する決済件数を取得する。
    async fn count(&self, _filter: &PaymentFilter) -> anyhow::Result<i64> {
        let payments = self.payments.read().await;
        Ok(payments.len() as i64)
    }

    /// 決済を開始（作成）する。
    async fn create(&self, input: &InitiatePayment) -> anyhow::Result<Payment> {
        let now = chrono::Utc::now();
        let payment = Payment {
            id: Uuid::new_v4(),
            order_id: input.order_id.clone(),
            customer_id: input.customer_id.clone(),
            amount: input.amount,
            currency: input.currency.clone(),
            status: PaymentStatus::Initiated,
            payment_method: input.payment_method.clone(),
            transaction_id: None,
            error_code: None,
            error_message: None,
            version: 1,
            created_at: now,
            updated_at: now,
        };

        self.payments.write().await.push(payment.clone());
        Ok(payment)
    }

    /// 決済を完了する（楽観ロック付き）。
    async fn complete(
        &self,
        id: Uuid,
        transaction_id: &str,
        expected_version: i32,
    ) -> anyhow::Result<Payment> {
        let mut payments = self.payments.write().await;
        let payment = payments
            .iter_mut()
            .find(|p| p.id == id && p.version == expected_version)
            .ok_or_else(|| anyhow::anyhow!("Payment '{}' not found or version conflict", id))?;
        payment.status = PaymentStatus::Completed;
        payment.transaction_id = Some(transaction_id.to_string());
        payment.version += 1;
        payment.updated_at = chrono::Utc::now();
        Ok(payment.clone())
    }

    /// 決済を失敗にする（楽観ロック付き）。
    async fn fail(
        &self,
        id: Uuid,
        error_code: &str,
        error_message: &str,
        expected_version: i32,
    ) -> anyhow::Result<Payment> {
        let mut payments = self.payments.write().await;
        let payment = payments
            .iter_mut()
            .find(|p| p.id == id && p.version == expected_version)
            .ok_or_else(|| anyhow::anyhow!("Payment '{}' not found or version conflict", id))?;
        payment.status = PaymentStatus::Failed;
        payment.error_code = Some(error_code.to_string());
        payment.error_message = Some(error_message.to_string());
        payment.version += 1;
        payment.updated_at = chrono::Utc::now();
        Ok(payment.clone())
    }

    /// 決済を返金する（楽観ロック付き）。返金理由をOutboxイベントに記録する。
    async fn refund(&self, id: Uuid, expected_version: i32, _reason: Option<String>) -> anyhow::Result<Payment> {
        let mut payments = self.payments.write().await;
        let payment = payments
            .iter_mut()
            .find(|p| p.id == id && p.version == expected_version)
            .ok_or_else(|| anyhow::anyhow!("Payment '{}' not found or version conflict", id))?;
        payment.status = PaymentStatus::Refunded;
        payment.version += 1;
        payment.updated_at = chrono::Utc::now();
        Ok(payment.clone())
    }

    /// Outbox イベントを挿入する（統合テストでは何もしない）。
    async fn insert_outbox_event(
        &self,
        _aggregate_type: &str,
        _aggregate_id: &str,
        _event_type: &str,
        _payload: &serde_json::Value,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    /// 未パブリッシュの Outbox イベントを取得する（統合テストでは空を返す）。
    async fn fetch_unpublished_events(&self, _limit: i64) -> anyhow::Result<Vec<OutboxEvent>> {
        Ok(vec![])
    }

    /// Outbox イベントをパブリッシュ済みとしてマークする（統合テストでは何もしない）。
    async fn mark_event_published(&self, _event_id: Uuid) -> anyhow::Result<()> {
        Ok(())
    }
}

/// テスト用 Axum アプリを構築するヘルパー関数。
/// 全ユースケースを InMemoryPaymentRepository で初期化し、認証なしで動作させる。
fn build_app() -> axum::Router {
    let repo: Arc<dyn PaymentRepository> = Arc::new(InMemoryPaymentRepository::default());
    let metrics = Arc::new(k1s0_telemetry::metrics::Metrics::new("payment-test"));

    let state = AppState {
        initiate_payment_uc: Arc::new(usecase::initiate_payment::InitiatePaymentUseCase::new(
            repo.clone(),
        )),
        get_payment_uc: Arc::new(usecase::get_payment::GetPaymentUseCase::new(repo.clone())),
        list_payments_uc: Arc::new(usecase::list_payments::ListPaymentsUseCase::new(
            repo.clone(),
        )),
        complete_payment_uc: Arc::new(usecase::complete_payment::CompletePaymentUseCase::new(
            repo.clone(),
        )),
        fail_payment_uc: Arc::new(usecase::fail_payment::FailPaymentUseCase::new(repo.clone())),
        refund_payment_uc: Arc::new(usecase::refund_payment::RefundPaymentUseCase::new(repo)),
        metrics,
        auth_state: None,
        db_pool: None,
    };

    handler::router(state)
}

/// ヘルスチェックエンドポイントが正常に応答することを確認する。
#[tokio::test]
async fn test_healthz() {
    let app = build_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

/// 決済開始→取得の一連のフローが正常に動作することを確認する。
#[tokio::test]
async fn test_initiate_and_get_payment() {
    let app = build_app();

    // 決済を開始する
    let body = serde_json::json!({
        "order_id": "ORD-001",
        "customer_id": "CUST-001",
        "amount": 5000,
        "currency": "JPY",
        "payment_method": "credit_card"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/payments")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(created["order_id"], "ORD-001");
    assert_eq!(created["customer_id"], "CUST-001");
    assert_eq!(created["amount"], 5000);
    assert_eq!(created["status"], "initiated");
    assert_eq!(created["version"], 1);

    let payment_id = created["id"].as_str().unwrap();

    // 作成した決済を取得して内容を確認する
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/payments/{}", payment_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(fetched["id"], payment_id);
    assert_eq!(fetched["status"], "initiated");
}

/// 空の決済一覧が正しく返されることを確認する。
#[tokio::test]
async fn test_list_payments_empty() {
    let app = build_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/payments")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(list["total"], 0);
    assert!(list["payments"].as_array().unwrap().is_empty());
}

/// 存在しない決済IDで取得すると 404 が返ることを確認する。
#[tokio::test]
async fn test_get_payment_not_found() {
    let app = build_app();
    let fake_id = Uuid::new_v4();
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/payments/{}", fake_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let err: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(err["error"]["code"], "SVC_PAYMENT_NOT_FOUND");
}

/// 不正な UUID 形式で 400 が返ることを確認する。
#[tokio::test]
async fn test_invalid_uuid_returns_400() {
    let app = build_app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/payments/not-a-uuid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let err: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(err["error"]["code"], "SVC_PAYMENT_VALIDATION_FAILED");
    assert!(!err["error"]["details"].as_array().unwrap().is_empty());
}

/// 決済開始→完了の正常フローを確認する（critical）。
/// initiated → completed の遷移が正しく行われることを検証する。
#[tokio::test]
async fn test_initiate_then_complete() {
    let app = build_app();

    // 決済を開始する
    let body = serde_json::json!({
        "order_id": "ORD-002",
        "customer_id": "CUST-002",
        "amount": 10000,
        "currency": "JPY",
        "payment_method": "credit_card"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/payments")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let payment_id = created["id"].as_str().unwrap();
    assert_eq!(created["status"], "initiated");

    // 決済を完了する
    let complete_body = serde_json::json!({ "transaction_id": "TXN-001" });
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/payments/{}/complete", payment_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&complete_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(completed["status"], "completed");
    assert_eq!(completed["version"], 2);
}

/// 決済開始→失敗の正常フローを確認する（critical）。
/// initiated → failed の遷移が正しく行われることを検証する。
#[tokio::test]
async fn test_initiate_then_fail() {
    let app = build_app();

    // 決済を開始する
    let body = serde_json::json!({
        "order_id": "ORD-003",
        "customer_id": "CUST-003",
        "amount": 3000,
        "currency": "JPY"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/payments")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let payment_id = created["id"].as_str().unwrap();
    assert_eq!(created["status"], "initiated");

    // 決済を失敗にする
    let fail_body = serde_json::json!({
        "error_code": "INSUFFICIENT_FUNDS",
        "error_message": "残高不足"
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/payments/{}/fail", payment_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&fail_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let failed: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(failed["status"], "failed");
    assert_eq!(failed["version"], 2);
}

/// 決済開始→完了→返金の正常フローを確認する（critical）。
/// initiated → completed → refunded の遷移が正しく行われることを検証する。
#[tokio::test]
async fn test_initiate_complete_then_refund() {
    let app = build_app();

    // 決済を開始する
    let body = serde_json::json!({
        "order_id": "ORD-004",
        "customer_id": "CUST-004",
        "amount": 8000,
        "currency": "JPY",
        "payment_method": "credit_card"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/payments")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let payment_id = created["id"].as_str().unwrap();

    // 決済を完了する
    let complete_body = serde_json::json!({ "transaction_id": "TXN-002" });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/payments/{}/complete", payment_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&complete_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let completed: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(completed["status"], "completed");
    assert_eq!(completed["version"], 2);

    // 決済を返金する
    let refund_body = serde_json::json!({ "reason": "顧客都合による返金" });
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/payments/{}/refund", payment_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&refund_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let refunded: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(refunded["status"], "refunded");
    assert_eq!(refunded["version"], 3);
}

/// 不正遷移テスト: initiated → refund は許可されないことを確認する。
/// 返金は completed 状態からのみ可能。
#[tokio::test]
async fn test_refund_initiated_payment_fails() {
    let app = build_app();

    // 決済を開始する
    let body = serde_json::json!({
        "order_id": "ORD-005",
        "customer_id": "CUST-005",
        "amount": 2000,
        "currency": "JPY"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/payments")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let payment_id = created["id"].as_str().unwrap();

    // initiated 状態から直接返金を試みる（不正遷移）
    let refund_body = serde_json::json!({ "reason": "不正な返金試行" });
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/payments/{}/refund", payment_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&refund_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let err: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(
        err["error"]["code"],
        "SVC_PAYMENT_INVALID_STATUS_TRANSITION"
    );
}

/// 不正遷移テスト: failed → complete は許可されないことを確認する。
/// 失敗した決済を完了にすることはできない。
#[tokio::test]
async fn test_complete_failed_payment_fails() {
    let app = build_app();

    // 決済を開始する
    let body = serde_json::json!({
        "order_id": "ORD-006",
        "customer_id": "CUST-006",
        "amount": 4000,
        "currency": "JPY"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/payments")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let payment_id = created["id"].as_str().unwrap();

    // まず決済を失敗にする
    let fail_body = serde_json::json!({
        "error_code": "TIMEOUT",
        "error_message": "Gateway timeout"
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/payments/{}/fail", payment_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&fail_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // failed 状態から complete を試みる（不正遷移）
    let complete_body = serde_json::json!({ "transaction_id": "TXN-INVALID" });
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/v1/payments/{}/complete", payment_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&complete_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let err: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(
        err["error"]["code"],
        "SVC_PAYMENT_INVALID_STATUS_TRANSITION"
    );
}

/// バリデーションエラーテスト: 不正な入力で 400 が返ることを確認する。
/// order_id が空の場合はバリデーションエラーになる。
#[tokio::test]
async fn test_initiate_payment_validation_error() {
    let app = build_app();

    // order_id が空のリクエスト（バリデーションエラー）
    let body = serde_json::json!({
        "order_id": "",
        "customer_id": "CUST-007",
        "amount": 1000,
        "currency": "JPY"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/payments")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let err: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(err["error"]["code"], "SVC_PAYMENT_VALIDATION_FAILED");
}
