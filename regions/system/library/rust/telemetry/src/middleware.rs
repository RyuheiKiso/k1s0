/// HTTP/gRPC ミドルウェアのトレーシングサポート。
///
/// axum および tonic で使用するトレーシングミドルウェアの実装例を提供する。
/// 実際のミドルウェアは tower レイヤーとして構成される。
///
/// # axum でのミドルウェア使用例
///
/// ```ignore
/// use axum::{Router, middleware};
/// use tower_http::trace::TraceLayer;
///
/// let app = Router::new()
///     .route("/api/v1/orders", post(create_order))
///     .layer(TraceLayer::new_for_http());
/// ```
///
/// # tonic でのミドルウェア使用例
///
/// ```ignore
/// use tonic::transport::Server;
/// use tower_http::trace::TraceLayer;
///
/// Server::builder()
///     .layer(TraceLayer::new_for_grpc())
///     .add_service(order_service)
///     .serve(addr)
///     .await?;
/// ```

/// trace_request マクロは axum/tonic ハンドラにトレーシング情報を付与する。
///
/// # 使用例
///
/// ```ignore
/// #[tracing::instrument(
///     skip(state),
///     fields(
///         service = "order-server",
///         tier = "service",
///         http.method = "POST",
///         http.path = "/api/v1/orders",
///     )
/// )]
/// async fn create_order(
///     State(state): State<AppState>,
///     Json(input): Json<CreateOrderInput>,
/// ) -> Result<Json<Order>, AppError> {
///     tracing::info!("Processing create order request");
///     // ...
/// }
/// ```
#[macro_export]
macro_rules! trace_request {
    ($method:expr, $path:expr, $body:block) => {{
        let span = tracing::info_span!(
            "http_request",
            http.method = $method,
            http.path = $path,
        );
        let _enter = span.enter();
        let start = std::time::Instant::now();
        let result = $body;
        let duration = start.elapsed();
        tracing::info!(
            duration_ms = duration.as_millis() as u64,
            "Request completed"
        );
        result
    }};
}

/// trace_grpc_call マクロは gRPC メソッド呼び出しにトレーシング情報を付与する。
#[macro_export]
macro_rules! trace_grpc_call {
    ($method:expr, $body:block) => {{
        let span = tracing::info_span!(
            "grpc_call",
            rpc.method = $method,
        );
        let _enter = span.enter();
        let start = std::time::Instant::now();
        let result = $body;
        let duration = start.elapsed();
        tracing::info!(
            duration_ms = duration.as_millis() as u64,
            "gRPC call completed"
        );
        result
    }};
}
