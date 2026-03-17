// @generated
/// Generated client implementations.
pub mod rule_engine_service_client {
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
    pub struct RuleEngineServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl RuleEngineServiceClient<tonic::transport::Channel> {
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
    impl<T> RuleEngineServiceClient<T>
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
        ) -> RuleEngineServiceClient<InterceptedService<T, F>>
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
            RuleEngineServiceClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn list_rules(
            &mut self,
            request: impl tonic::IntoRequest<super::ListRulesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListRulesResponse>,
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/ListRules",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.rule_engine.v1.RuleEngineService",
                        "ListRules",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_rule(
            &mut self,
            request: impl tonic::IntoRequest<super::GetRuleRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetRuleResponse>,
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/GetRule",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.rule_engine.v1.RuleEngineService",
                        "GetRule",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_rule(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateRuleRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateRuleResponse>,
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/CreateRule",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.rule_engine.v1.RuleEngineService",
                        "CreateRule",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_rule(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateRuleRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateRuleResponse>,
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/UpdateRule",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.rule_engine.v1.RuleEngineService",
                        "UpdateRule",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_rule(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteRuleRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteRuleResponse>,
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/DeleteRule",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.rule_engine.v1.RuleEngineService",
                        "DeleteRule",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_rule_sets(
            &mut self,
            request: impl tonic::IntoRequest<super::ListRuleSetsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListRuleSetsResponse>,
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/ListRuleSets",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.rule_engine.v1.RuleEngineService",
                        "ListRuleSets",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_rule_set(
            &mut self,
            request: impl tonic::IntoRequest<super::GetRuleSetRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetRuleSetResponse>,
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/GetRuleSet",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.rule_engine.v1.RuleEngineService",
                        "GetRuleSet",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_rule_set(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateRuleSetRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateRuleSetResponse>,
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/CreateRuleSet",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.rule_engine.v1.RuleEngineService",
                        "CreateRuleSet",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_rule_set(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateRuleSetRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateRuleSetResponse>,
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/UpdateRuleSet",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.rule_engine.v1.RuleEngineService",
                        "UpdateRuleSet",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_rule_set(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteRuleSetRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteRuleSetResponse>,
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/DeleteRuleSet",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.rule_engine.v1.RuleEngineService",
                        "DeleteRuleSet",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn publish_rule_set(
            &mut self,
            request: impl tonic::IntoRequest<super::PublishRuleSetRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PublishRuleSetResponse>,
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/PublishRuleSet",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.rule_engine.v1.RuleEngineService",
                        "PublishRuleSet",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn rollback_rule_set(
            &mut self,
            request: impl tonic::IntoRequest<super::RollbackRuleSetRequest>,
        ) -> std::result::Result<
            tonic::Response<super::RollbackRuleSetResponse>,
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/RollbackRuleSet",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.rule_engine.v1.RuleEngineService",
                        "RollbackRuleSet",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn evaluate(
            &mut self,
            request: impl tonic::IntoRequest<super::EvaluateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EvaluateResponse>,
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/Evaluate",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.rule_engine.v1.RuleEngineService",
                        "Evaluate",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn evaluate_dry_run(
            &mut self,
            request: impl tonic::IntoRequest<super::EvaluateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EvaluateResponse>,
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/EvaluateDryRun",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.rule_engine.v1.RuleEngineService",
                        "EvaluateDryRun",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod rule_engine_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with RuleEngineServiceServer.
    #[async_trait]
    pub trait RuleEngineService: std::marker::Send + std::marker::Sync + 'static {
        async fn list_rules(
            &self,
            request: tonic::Request<super::ListRulesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListRulesResponse>,
            tonic::Status,
        >;
        async fn get_rule(
            &self,
            request: tonic::Request<super::GetRuleRequest>,
        ) -> std::result::Result<tonic::Response<super::GetRuleResponse>, tonic::Status>;
        async fn create_rule(
            &self,
            request: tonic::Request<super::CreateRuleRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateRuleResponse>,
            tonic::Status,
        >;
        async fn update_rule(
            &self,
            request: tonic::Request<super::UpdateRuleRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateRuleResponse>,
            tonic::Status,
        >;
        async fn delete_rule(
            &self,
            request: tonic::Request<super::DeleteRuleRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteRuleResponse>,
            tonic::Status,
        >;
        async fn list_rule_sets(
            &self,
            request: tonic::Request<super::ListRuleSetsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListRuleSetsResponse>,
            tonic::Status,
        >;
        async fn get_rule_set(
            &self,
            request: tonic::Request<super::GetRuleSetRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetRuleSetResponse>,
            tonic::Status,
        >;
        async fn create_rule_set(
            &self,
            request: tonic::Request<super::CreateRuleSetRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateRuleSetResponse>,
            tonic::Status,
        >;
        async fn update_rule_set(
            &self,
            request: tonic::Request<super::UpdateRuleSetRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateRuleSetResponse>,
            tonic::Status,
        >;
        async fn delete_rule_set(
            &self,
            request: tonic::Request<super::DeleteRuleSetRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteRuleSetResponse>,
            tonic::Status,
        >;
        async fn publish_rule_set(
            &self,
            request: tonic::Request<super::PublishRuleSetRequest>,
        ) -> std::result::Result<
            tonic::Response<super::PublishRuleSetResponse>,
            tonic::Status,
        >;
        async fn rollback_rule_set(
            &self,
            request: tonic::Request<super::RollbackRuleSetRequest>,
        ) -> std::result::Result<
            tonic::Response<super::RollbackRuleSetResponse>,
            tonic::Status,
        >;
        async fn evaluate(
            &self,
            request: tonic::Request<super::EvaluateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EvaluateResponse>,
            tonic::Status,
        >;
        async fn evaluate_dry_run(
            &self,
            request: tonic::Request<super::EvaluateRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EvaluateResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct RuleEngineServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> RuleEngineServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for RuleEngineServiceServer<T>
    where
        T: RuleEngineService,
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/ListRules" => {
                    #[allow(non_camel_case_types)]
                    struct ListRulesSvc<T: RuleEngineService>(pub Arc<T>);
                    impl<
                        T: RuleEngineService,
                    > tonic::server::UnaryService<super::ListRulesRequest>
                    for ListRulesSvc<T> {
                        type Response = super::ListRulesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListRulesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as RuleEngineService>::list_rules(&inner, request).await
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
                        let method = ListRulesSvc(inner);
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/GetRule" => {
                    #[allow(non_camel_case_types)]
                    struct GetRuleSvc<T: RuleEngineService>(pub Arc<T>);
                    impl<
                        T: RuleEngineService,
                    > tonic::server::UnaryService<super::GetRuleRequest>
                    for GetRuleSvc<T> {
                        type Response = super::GetRuleResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetRuleRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as RuleEngineService>::get_rule(&inner, request).await
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
                        let method = GetRuleSvc(inner);
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/CreateRule" => {
                    #[allow(non_camel_case_types)]
                    struct CreateRuleSvc<T: RuleEngineService>(pub Arc<T>);
                    impl<
                        T: RuleEngineService,
                    > tonic::server::UnaryService<super::CreateRuleRequest>
                    for CreateRuleSvc<T> {
                        type Response = super::CreateRuleResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateRuleRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as RuleEngineService>::create_rule(&inner, request).await
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
                        let method = CreateRuleSvc(inner);
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/UpdateRule" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateRuleSvc<T: RuleEngineService>(pub Arc<T>);
                    impl<
                        T: RuleEngineService,
                    > tonic::server::UnaryService<super::UpdateRuleRequest>
                    for UpdateRuleSvc<T> {
                        type Response = super::UpdateRuleResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateRuleRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as RuleEngineService>::update_rule(&inner, request).await
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
                        let method = UpdateRuleSvc(inner);
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/DeleteRule" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteRuleSvc<T: RuleEngineService>(pub Arc<T>);
                    impl<
                        T: RuleEngineService,
                    > tonic::server::UnaryService<super::DeleteRuleRequest>
                    for DeleteRuleSvc<T> {
                        type Response = super::DeleteRuleResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteRuleRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as RuleEngineService>::delete_rule(&inner, request).await
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
                        let method = DeleteRuleSvc(inner);
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/ListRuleSets" => {
                    #[allow(non_camel_case_types)]
                    struct ListRuleSetsSvc<T: RuleEngineService>(pub Arc<T>);
                    impl<
                        T: RuleEngineService,
                    > tonic::server::UnaryService<super::ListRuleSetsRequest>
                    for ListRuleSetsSvc<T> {
                        type Response = super::ListRuleSetsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListRuleSetsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as RuleEngineService>::list_rule_sets(&inner, request)
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
                        let method = ListRuleSetsSvc(inner);
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/GetRuleSet" => {
                    #[allow(non_camel_case_types)]
                    struct GetRuleSetSvc<T: RuleEngineService>(pub Arc<T>);
                    impl<
                        T: RuleEngineService,
                    > tonic::server::UnaryService<super::GetRuleSetRequest>
                    for GetRuleSetSvc<T> {
                        type Response = super::GetRuleSetResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetRuleSetRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as RuleEngineService>::get_rule_set(&inner, request)
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
                        let method = GetRuleSetSvc(inner);
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/CreateRuleSet" => {
                    #[allow(non_camel_case_types)]
                    struct CreateRuleSetSvc<T: RuleEngineService>(pub Arc<T>);
                    impl<
                        T: RuleEngineService,
                    > tonic::server::UnaryService<super::CreateRuleSetRequest>
                    for CreateRuleSetSvc<T> {
                        type Response = super::CreateRuleSetResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateRuleSetRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as RuleEngineService>::create_rule_set(&inner, request)
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
                        let method = CreateRuleSetSvc(inner);
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/UpdateRuleSet" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateRuleSetSvc<T: RuleEngineService>(pub Arc<T>);
                    impl<
                        T: RuleEngineService,
                    > tonic::server::UnaryService<super::UpdateRuleSetRequest>
                    for UpdateRuleSetSvc<T> {
                        type Response = super::UpdateRuleSetResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateRuleSetRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as RuleEngineService>::update_rule_set(&inner, request)
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
                        let method = UpdateRuleSetSvc(inner);
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/DeleteRuleSet" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteRuleSetSvc<T: RuleEngineService>(pub Arc<T>);
                    impl<
                        T: RuleEngineService,
                    > tonic::server::UnaryService<super::DeleteRuleSetRequest>
                    for DeleteRuleSetSvc<T> {
                        type Response = super::DeleteRuleSetResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteRuleSetRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as RuleEngineService>::delete_rule_set(&inner, request)
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
                        let method = DeleteRuleSetSvc(inner);
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/PublishRuleSet" => {
                    #[allow(non_camel_case_types)]
                    struct PublishRuleSetSvc<T: RuleEngineService>(pub Arc<T>);
                    impl<
                        T: RuleEngineService,
                    > tonic::server::UnaryService<super::PublishRuleSetRequest>
                    for PublishRuleSetSvc<T> {
                        type Response = super::PublishRuleSetResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PublishRuleSetRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as RuleEngineService>::publish_rule_set(&inner, request)
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
                        let method = PublishRuleSetSvc(inner);
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/RollbackRuleSet" => {
                    #[allow(non_camel_case_types)]
                    struct RollbackRuleSetSvc<T: RuleEngineService>(pub Arc<T>);
                    impl<
                        T: RuleEngineService,
                    > tonic::server::UnaryService<super::RollbackRuleSetRequest>
                    for RollbackRuleSetSvc<T> {
                        type Response = super::RollbackRuleSetResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RollbackRuleSetRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as RuleEngineService>::rollback_rule_set(&inner, request)
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
                        let method = RollbackRuleSetSvc(inner);
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/Evaluate" => {
                    #[allow(non_camel_case_types)]
                    struct EvaluateSvc<T: RuleEngineService>(pub Arc<T>);
                    impl<
                        T: RuleEngineService,
                    > tonic::server::UnaryService<super::EvaluateRequest>
                    for EvaluateSvc<T> {
                        type Response = super::EvaluateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::EvaluateRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as RuleEngineService>::evaluate(&inner, request).await
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
                        let method = EvaluateSvc(inner);
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
                "/k1s0.system.rule_engine.v1.RuleEngineService/EvaluateDryRun" => {
                    #[allow(non_camel_case_types)]
                    struct EvaluateDryRunSvc<T: RuleEngineService>(pub Arc<T>);
                    impl<
                        T: RuleEngineService,
                    > tonic::server::UnaryService<super::EvaluateRequest>
                    for EvaluateDryRunSvc<T> {
                        type Response = super::EvaluateResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::EvaluateRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as RuleEngineService>::evaluate_dry_run(&inner, request)
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
                        let method = EvaluateDryRunSvc(inner);
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
    impl<T> Clone for RuleEngineServiceServer<T> {
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
    pub const SERVICE_NAME: &str = "k1s0.system.rule_engine.v1.RuleEngineService";
    impl<T> tonic::server::NamedService for RuleEngineServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
