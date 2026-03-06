// @generated
/// Generated client implementations.
pub mod quota_service_client {
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
    pub struct QuotaServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl QuotaServiceClient<tonic::transport::Channel> {
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
    impl<T> QuotaServiceClient<T>
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
        ) -> QuotaServiceClient<InterceptedService<T, F>>
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
            QuotaServiceClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn create_quota_policy(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateQuotaPolicyRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateQuotaPolicyResponse>,
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
                "/k1s0.system.quota.v1.QuotaService/CreateQuotaPolicy",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.quota.v1.QuotaService",
                        "CreateQuotaPolicy",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_quota_policy(
            &mut self,
            request: impl tonic::IntoRequest<super::GetQuotaPolicyRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetQuotaPolicyResponse>,
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
                "/k1s0.system.quota.v1.QuotaService/GetQuotaPolicy",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.quota.v1.QuotaService",
                        "GetQuotaPolicy",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_quota_policies(
            &mut self,
            request: impl tonic::IntoRequest<super::ListQuotaPoliciesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListQuotaPoliciesResponse>,
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
                "/k1s0.system.quota.v1.QuotaService/ListQuotaPolicies",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.quota.v1.QuotaService",
                        "ListQuotaPolicies",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_quota_policy(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateQuotaPolicyRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateQuotaPolicyResponse>,
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
                "/k1s0.system.quota.v1.QuotaService/UpdateQuotaPolicy",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.quota.v1.QuotaService",
                        "UpdateQuotaPolicy",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_quota_policy(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteQuotaPolicyRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteQuotaPolicyResponse>,
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
                "/k1s0.system.quota.v1.QuotaService/DeleteQuotaPolicy",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.quota.v1.QuotaService",
                        "DeleteQuotaPolicy",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_quota_usage(
            &mut self,
            request: impl tonic::IntoRequest<super::GetQuotaUsageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetQuotaUsageResponse>,
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
                "/k1s0.system.quota.v1.QuotaService/GetQuotaUsage",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("k1s0.system.quota.v1.QuotaService", "GetQuotaUsage"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn check_quota(
            &mut self,
            request: impl tonic::IntoRequest<super::CheckQuotaRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CheckQuotaResponse>,
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
                "/k1s0.system.quota.v1.QuotaService/CheckQuota",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("k1s0.system.quota.v1.QuotaService", "CheckQuota"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn increment_quota_usage(
            &mut self,
            request: impl tonic::IntoRequest<super::IncrementQuotaUsageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::IncrementQuotaUsageResponse>,
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
                "/k1s0.system.quota.v1.QuotaService/IncrementQuotaUsage",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.quota.v1.QuotaService",
                        "IncrementQuotaUsage",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn reset_quota_usage(
            &mut self,
            request: impl tonic::IntoRequest<super::ResetQuotaUsageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ResetQuotaUsageResponse>,
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
                "/k1s0.system.quota.v1.QuotaService/ResetQuotaUsage",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.quota.v1.QuotaService",
                        "ResetQuotaUsage",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod quota_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with QuotaServiceServer.
    #[async_trait]
    pub trait QuotaService: std::marker::Send + std::marker::Sync + 'static {
        async fn create_quota_policy(
            &self,
            request: tonic::Request<super::CreateQuotaPolicyRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateQuotaPolicyResponse>,
            tonic::Status,
        >;
        async fn get_quota_policy(
            &self,
            request: tonic::Request<super::GetQuotaPolicyRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetQuotaPolicyResponse>,
            tonic::Status,
        >;
        async fn list_quota_policies(
            &self,
            request: tonic::Request<super::ListQuotaPoliciesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListQuotaPoliciesResponse>,
            tonic::Status,
        >;
        async fn update_quota_policy(
            &self,
            request: tonic::Request<super::UpdateQuotaPolicyRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateQuotaPolicyResponse>,
            tonic::Status,
        >;
        async fn delete_quota_policy(
            &self,
            request: tonic::Request<super::DeleteQuotaPolicyRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteQuotaPolicyResponse>,
            tonic::Status,
        >;
        async fn get_quota_usage(
            &self,
            request: tonic::Request<super::GetQuotaUsageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetQuotaUsageResponse>,
            tonic::Status,
        >;
        async fn check_quota(
            &self,
            request: tonic::Request<super::CheckQuotaRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CheckQuotaResponse>,
            tonic::Status,
        >;
        async fn increment_quota_usage(
            &self,
            request: tonic::Request<super::IncrementQuotaUsageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::IncrementQuotaUsageResponse>,
            tonic::Status,
        >;
        async fn reset_quota_usage(
            &self,
            request: tonic::Request<super::ResetQuotaUsageRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ResetQuotaUsageResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct QuotaServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> QuotaServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for QuotaServiceServer<T>
    where
        T: QuotaService,
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
                "/k1s0.system.quota.v1.QuotaService/CreateQuotaPolicy" => {
                    #[allow(non_camel_case_types)]
                    struct CreateQuotaPolicySvc<T: QuotaService>(pub Arc<T>);
                    impl<
                        T: QuotaService,
                    > tonic::server::UnaryService<super::CreateQuotaPolicyRequest>
                    for CreateQuotaPolicySvc<T> {
                        type Response = super::CreateQuotaPolicyResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateQuotaPolicyRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QuotaService>::create_quota_policy(&inner, request)
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
                        let method = CreateQuotaPolicySvc(inner);
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
                "/k1s0.system.quota.v1.QuotaService/GetQuotaPolicy" => {
                    #[allow(non_camel_case_types)]
                    struct GetQuotaPolicySvc<T: QuotaService>(pub Arc<T>);
                    impl<
                        T: QuotaService,
                    > tonic::server::UnaryService<super::GetQuotaPolicyRequest>
                    for GetQuotaPolicySvc<T> {
                        type Response = super::GetQuotaPolicyResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetQuotaPolicyRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QuotaService>::get_quota_policy(&inner, request).await
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
                        let method = GetQuotaPolicySvc(inner);
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
                "/k1s0.system.quota.v1.QuotaService/ListQuotaPolicies" => {
                    #[allow(non_camel_case_types)]
                    struct ListQuotaPoliciesSvc<T: QuotaService>(pub Arc<T>);
                    impl<
                        T: QuotaService,
                    > tonic::server::UnaryService<super::ListQuotaPoliciesRequest>
                    for ListQuotaPoliciesSvc<T> {
                        type Response = super::ListQuotaPoliciesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListQuotaPoliciesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QuotaService>::list_quota_policies(&inner, request)
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
                        let method = ListQuotaPoliciesSvc(inner);
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
                "/k1s0.system.quota.v1.QuotaService/UpdateQuotaPolicy" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateQuotaPolicySvc<T: QuotaService>(pub Arc<T>);
                    impl<
                        T: QuotaService,
                    > tonic::server::UnaryService<super::UpdateQuotaPolicyRequest>
                    for UpdateQuotaPolicySvc<T> {
                        type Response = super::UpdateQuotaPolicyResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateQuotaPolicyRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QuotaService>::update_quota_policy(&inner, request)
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
                        let method = UpdateQuotaPolicySvc(inner);
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
                "/k1s0.system.quota.v1.QuotaService/DeleteQuotaPolicy" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteQuotaPolicySvc<T: QuotaService>(pub Arc<T>);
                    impl<
                        T: QuotaService,
                    > tonic::server::UnaryService<super::DeleteQuotaPolicyRequest>
                    for DeleteQuotaPolicySvc<T> {
                        type Response = super::DeleteQuotaPolicyResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteQuotaPolicyRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QuotaService>::delete_quota_policy(&inner, request)
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
                        let method = DeleteQuotaPolicySvc(inner);
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
                "/k1s0.system.quota.v1.QuotaService/GetQuotaUsage" => {
                    #[allow(non_camel_case_types)]
                    struct GetQuotaUsageSvc<T: QuotaService>(pub Arc<T>);
                    impl<
                        T: QuotaService,
                    > tonic::server::UnaryService<super::GetQuotaUsageRequest>
                    for GetQuotaUsageSvc<T> {
                        type Response = super::GetQuotaUsageResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetQuotaUsageRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QuotaService>::get_quota_usage(&inner, request).await
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
                        let method = GetQuotaUsageSvc(inner);
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
                "/k1s0.system.quota.v1.QuotaService/CheckQuota" => {
                    #[allow(non_camel_case_types)]
                    struct CheckQuotaSvc<T: QuotaService>(pub Arc<T>);
                    impl<
                        T: QuotaService,
                    > tonic::server::UnaryService<super::CheckQuotaRequest>
                    for CheckQuotaSvc<T> {
                        type Response = super::CheckQuotaResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CheckQuotaRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QuotaService>::check_quota(&inner, request).await
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
                        let method = CheckQuotaSvc(inner);
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
                "/k1s0.system.quota.v1.QuotaService/IncrementQuotaUsage" => {
                    #[allow(non_camel_case_types)]
                    struct IncrementQuotaUsageSvc<T: QuotaService>(pub Arc<T>);
                    impl<
                        T: QuotaService,
                    > tonic::server::UnaryService<super::IncrementQuotaUsageRequest>
                    for IncrementQuotaUsageSvc<T> {
                        type Response = super::IncrementQuotaUsageResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::IncrementQuotaUsageRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QuotaService>::increment_quota_usage(&inner, request)
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
                        let method = IncrementQuotaUsageSvc(inner);
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
                "/k1s0.system.quota.v1.QuotaService/ResetQuotaUsage" => {
                    #[allow(non_camel_case_types)]
                    struct ResetQuotaUsageSvc<T: QuotaService>(pub Arc<T>);
                    impl<
                        T: QuotaService,
                    > tonic::server::UnaryService<super::ResetQuotaUsageRequest>
                    for ResetQuotaUsageSvc<T> {
                        type Response = super::ResetQuotaUsageResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ResetQuotaUsageRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as QuotaService>::reset_quota_usage(&inner, request)
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
                        let method = ResetQuotaUsageSvc(inner);
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
    impl<T> Clone for QuotaServiceServer<T> {
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
    pub const SERVICE_NAME: &str = "k1s0.system.quota.v1.QuotaService";
    impl<T> tonic::server::NamedService for QuotaServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
