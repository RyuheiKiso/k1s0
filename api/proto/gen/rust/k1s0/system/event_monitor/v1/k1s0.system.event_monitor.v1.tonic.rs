// @generated
/// Generated client implementations.
pub mod event_monitor_service_client {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct EventMonitorServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl EventMonitorServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> EventMonitorServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::Body>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> EventMonitorServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::Body>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::Body>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::Body>,
            >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
        {
            EventMonitorServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        pub async fn list_events(
            &mut self,
            request: impl tonic::IntoRequest<super::ListEventsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListEventsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.event_monitor.v1.EventMonitorService/ListEvents",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.event_monitor.v1.EventMonitorService",
                        "ListEvents",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn trace_by_correlation(
            &mut self,
            request: impl tonic::IntoRequest<super::TraceByCorrelationRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TraceByCorrelationResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.event_monitor.v1.EventMonitorService/TraceByCorrelation",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.event_monitor.v1.EventMonitorService",
                        "TraceByCorrelation",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_flows(
            &mut self,
            request: impl tonic::IntoRequest<super::ListFlowsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListFlowsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.event_monitor.v1.EventMonitorService/ListFlows",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.event_monitor.v1.EventMonitorService",
                        "ListFlows",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_flow(
            &mut self,
            request: impl tonic::IntoRequest<super::GetFlowRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetFlowResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.event_monitor.v1.EventMonitorService/GetFlow",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.event_monitor.v1.EventMonitorService",
                        "GetFlow",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_flow(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateFlowRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateFlowResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.event_monitor.v1.EventMonitorService/CreateFlow",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.event_monitor.v1.EventMonitorService",
                        "CreateFlow",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_flow(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateFlowRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateFlowResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.event_monitor.v1.EventMonitorService/UpdateFlow",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.event_monitor.v1.EventMonitorService",
                        "UpdateFlow",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_flow(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteFlowRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteFlowResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.event_monitor.v1.EventMonitorService/DeleteFlow",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.event_monitor.v1.EventMonitorService",
                        "DeleteFlow",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_flow_kpi(
            &mut self,
            request: impl tonic::IntoRequest<super::GetFlowKpiRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetFlowKpiResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.event_monitor.v1.EventMonitorService/GetFlowKpi",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.event_monitor.v1.EventMonitorService",
                        "GetFlowKpi",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_kpi_summary(
            &mut self,
            request: impl tonic::IntoRequest<super::GetKpiSummaryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetKpiSummaryResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.event_monitor.v1.EventMonitorService/GetKpiSummary",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.event_monitor.v1.EventMonitorService",
                        "GetKpiSummary",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_slo_status(
            &mut self,
            request: impl tonic::IntoRequest<super::GetSloStatusRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetSloStatusResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.event_monitor.v1.EventMonitorService/GetSloStatus",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.event_monitor.v1.EventMonitorService",
                        "GetSloStatus",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_slo_burn_rate(
            &mut self,
            request: impl tonic::IntoRequest<super::GetSloBurnRateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetSloBurnRateResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.event_monitor.v1.EventMonitorService/GetSloBurnRate",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.event_monitor.v1.EventMonitorService",
                        "GetSloBurnRate",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn preview_replay(
            &mut self,
            request: impl tonic::IntoRequest<super::PreviewReplayRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PreviewReplayResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.event_monitor.v1.EventMonitorService/PreviewReplay",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.event_monitor.v1.EventMonitorService",
                        "PreviewReplay",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn execute_replay(
            &mut self,
            request: impl tonic::IntoRequest<super::ExecuteReplayRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ExecuteReplayResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.event_monitor.v1.EventMonitorService/ExecuteReplay",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.event_monitor.v1.EventMonitorService",
                        "ExecuteReplay",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod event_monitor_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with EventMonitorServiceServer.
    #[async_trait]
    pub trait EventMonitorService: std::marker::Send + std::marker::Sync + 'static {
        async fn list_events(
            &self,
            request: tonic::Request<super::ListEventsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListEventsResponse>,
            tonic::Status,
        >;
        async fn trace_by_correlation(
            &self,
            request: tonic::Request<super::TraceByCorrelationRequest>,
        ) -> std::result::Result<
            tonic::Response<super::TraceByCorrelationResponse>,
            tonic::Status,
        >;
        async fn list_flows(
            &self,
            request: tonic::Request<super::ListFlowsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListFlowsResponse>,
            tonic::Status,
        >;
        async fn get_flow(
            &self,
            request: tonic::Request<super::GetFlowRequest>,
        ) -> std::result::Result<tonic::Response<super::GetFlowResponse>, tonic::Status>;
        async fn create_flow(
            &self,
            request: tonic::Request<super::CreateFlowRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateFlowResponse>,
            tonic::Status,
        >;
        async fn update_flow(
            &self,
            request: tonic::Request<super::UpdateFlowRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateFlowResponse>,
            tonic::Status,
        >;
        async fn delete_flow(
            &self,
            request: tonic::Request<super::DeleteFlowRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteFlowResponse>,
            tonic::Status,
        >;
        async fn get_flow_kpi(
            &self,
            request: tonic::Request<super::GetFlowKpiRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetFlowKpiResponse>,
            tonic::Status,
        >;
        async fn get_kpi_summary(
            &self,
            request: tonic::Request<super::GetKpiSummaryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetKpiSummaryResponse>,
            tonic::Status,
        >;
        async fn get_slo_status(
            &self,
            request: tonic::Request<super::GetSloStatusRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetSloStatusResponse>,
            tonic::Status,
        >;
        async fn get_slo_burn_rate(
            &self,
            request: tonic::Request<super::GetSloBurnRateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetSloBurnRateResponse>,
            tonic::Status,
        >;
        async fn preview_replay(
            &self,
            request: tonic::Request<super::PreviewReplayRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PreviewReplayResponse>,
            tonic::Status,
        >;
        async fn execute_replay(
            &self,
            request: tonic::Request<super::ExecuteReplayRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ExecuteReplayResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct EventMonitorServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> EventMonitorServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for EventMonitorServiceServer<T>
    where
        T: EventMonitorService,
        B: Body + std::marker::Send + 'static,
        B::Error: Into<StdError> + std::marker::Send + 'static,
    {
        type Response = http::Response<tonic::body::Body>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            match req.uri().path() {
                "/k1s0.system.event_monitor.v1.EventMonitorService/ListEvents" => {
                    #[allow(non_camel_case_types)]
                    struct ListEventsSvc<T: EventMonitorService>(pub Arc<T>);
                    impl<
                        T: EventMonitorService,
                    > tonic::server::UnaryService<super::ListEventsRequest>
                    for ListEventsSvc<T> {
                        type Response = super::ListEventsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListEventsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EventMonitorService>::list_events(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ListEventsSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/k1s0.system.event_monitor.v1.EventMonitorService/TraceByCorrelation" => {
                    #[allow(non_camel_case_types)]
                    struct TraceByCorrelationSvc<T: EventMonitorService>(pub Arc<T>);
                    impl<
                        T: EventMonitorService,
                    > tonic::server::UnaryService<super::TraceByCorrelationRequest>
                    for TraceByCorrelationSvc<T> {
                        type Response = super::TraceByCorrelationResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TraceByCorrelationRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EventMonitorService>::trace_by_correlation(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = TraceByCorrelationSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/k1s0.system.event_monitor.v1.EventMonitorService/ListFlows" => {
                    #[allow(non_camel_case_types)]
                    struct ListFlowsSvc<T: EventMonitorService>(pub Arc<T>);
                    impl<
                        T: EventMonitorService,
                    > tonic::server::UnaryService<super::ListFlowsRequest>
                    for ListFlowsSvc<T> {
                        type Response = super::ListFlowsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListFlowsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EventMonitorService>::list_flows(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ListFlowsSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/k1s0.system.event_monitor.v1.EventMonitorService/GetFlow" => {
                    #[allow(non_camel_case_types)]
                    struct GetFlowSvc<T: EventMonitorService>(pub Arc<T>);
                    impl<
                        T: EventMonitorService,
                    > tonic::server::UnaryService<super::GetFlowRequest>
                    for GetFlowSvc<T> {
                        type Response = super::GetFlowResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetFlowRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EventMonitorService>::get_flow(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetFlowSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/k1s0.system.event_monitor.v1.EventMonitorService/CreateFlow" => {
                    #[allow(non_camel_case_types)]
                    struct CreateFlowSvc<T: EventMonitorService>(pub Arc<T>);
                    impl<
                        T: EventMonitorService,
                    > tonic::server::UnaryService<super::CreateFlowRequest>
                    for CreateFlowSvc<T> {
                        type Response = super::CreateFlowResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateFlowRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EventMonitorService>::create_flow(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CreateFlowSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/k1s0.system.event_monitor.v1.EventMonitorService/UpdateFlow" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateFlowSvc<T: EventMonitorService>(pub Arc<T>);
                    impl<
                        T: EventMonitorService,
                    > tonic::server::UnaryService<super::UpdateFlowRequest>
                    for UpdateFlowSvc<T> {
                        type Response = super::UpdateFlowResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateFlowRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EventMonitorService>::update_flow(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = UpdateFlowSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/k1s0.system.event_monitor.v1.EventMonitorService/DeleteFlow" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteFlowSvc<T: EventMonitorService>(pub Arc<T>);
                    impl<
                        T: EventMonitorService,
                    > tonic::server::UnaryService<super::DeleteFlowRequest>
                    for DeleteFlowSvc<T> {
                        type Response = super::DeleteFlowResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteFlowRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EventMonitorService>::delete_flow(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = DeleteFlowSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/k1s0.system.event_monitor.v1.EventMonitorService/GetFlowKpi" => {
                    #[allow(non_camel_case_types)]
                    struct GetFlowKpiSvc<T: EventMonitorService>(pub Arc<T>);
                    impl<
                        T: EventMonitorService,
                    > tonic::server::UnaryService<super::GetFlowKpiRequest>
                    for GetFlowKpiSvc<T> {
                        type Response = super::GetFlowKpiResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetFlowKpiRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EventMonitorService>::get_flow_kpi(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetFlowKpiSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/k1s0.system.event_monitor.v1.EventMonitorService/GetKpiSummary" => {
                    #[allow(non_camel_case_types)]
                    struct GetKpiSummarySvc<T: EventMonitorService>(pub Arc<T>);
                    impl<
                        T: EventMonitorService,
                    > tonic::server::UnaryService<super::GetKpiSummaryRequest>
                    for GetKpiSummarySvc<T> {
                        type Response = super::GetKpiSummaryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetKpiSummaryRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EventMonitorService>::get_kpi_summary(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetKpiSummarySvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/k1s0.system.event_monitor.v1.EventMonitorService/GetSloStatus" => {
                    #[allow(non_camel_case_types)]
                    struct GetSloStatusSvc<T: EventMonitorService>(pub Arc<T>);
                    impl<
                        T: EventMonitorService,
                    > tonic::server::UnaryService<super::GetSloStatusRequest>
                    for GetSloStatusSvc<T> {
                        type Response = super::GetSloStatusResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetSloStatusRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EventMonitorService>::get_slo_status(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetSloStatusSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/k1s0.system.event_monitor.v1.EventMonitorService/GetSloBurnRate" => {
                    #[allow(non_camel_case_types)]
                    struct GetSloBurnRateSvc<T: EventMonitorService>(pub Arc<T>);
                    impl<
                        T: EventMonitorService,
                    > tonic::server::UnaryService<super::GetSloBurnRateRequest>
                    for GetSloBurnRateSvc<T> {
                        type Response = super::GetSloBurnRateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetSloBurnRateRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EventMonitorService>::get_slo_burn_rate(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetSloBurnRateSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/k1s0.system.event_monitor.v1.EventMonitorService/PreviewReplay" => {
                    #[allow(non_camel_case_types)]
                    struct PreviewReplaySvc<T: EventMonitorService>(pub Arc<T>);
                    impl<
                        T: EventMonitorService,
                    > tonic::server::UnaryService<super::PreviewReplayRequest>
                    for PreviewReplaySvc<T> {
                        type Response = super::PreviewReplayResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PreviewReplayRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EventMonitorService>::preview_replay(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = PreviewReplaySvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/k1s0.system.event_monitor.v1.EventMonitorService/ExecuteReplay" => {
                    #[allow(non_camel_case_types)]
                    struct ExecuteReplaySvc<T: EventMonitorService>(pub Arc<T>);
                    impl<
                        T: EventMonitorService,
                    > tonic::server::UnaryService<super::ExecuteReplayRequest>
                    for ExecuteReplaySvc<T> {
                        type Response = super::ExecuteReplayResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ExecuteReplayRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EventMonitorService>::execute_replay(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ExecuteReplaySvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        let mut response = http::Response::new(
                            tonic::body::Body::default(),
                        );
                        let headers = response.headers_mut();
                        headers
                            .insert(
                                tonic::Status::GRPC_STATUS,
                                (tonic::Code::Unimplemented as i32).into(),
                            );
                        headers
                            .insert(
                                http::header::CONTENT_TYPE,
                                tonic::metadata::GRPC_CONTENT_TYPE,
                            );
                        Ok(response)
                    })
                }
            }
        }
    }
    impl<T> Clone for EventMonitorServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    /// Generated gRPC service name
    pub const SERVICE_NAME: &str = "k1s0.system.event_monitor.v1.EventMonitorService";
    impl<T> tonic::server::NamedService for EventMonitorServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
