// @generated
/// Generated client implementations.
pub mod feature_flag_service_client {
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
    pub struct FeatureFlagServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl FeatureFlagServiceClient<tonic::transport::Channel> {
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
    impl<T> FeatureFlagServiceClient<T>
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
        ) -> FeatureFlagServiceClient<InterceptedService<T, F>>
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
            FeatureFlagServiceClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn evaluate_flag(
            &mut self,
            request: impl tonic::IntoRequest<super::EvaluateFlagRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EvaluateFlagResponse>,
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
                "/k1s0.system.featureflag.v1.FeatureFlagService/EvaluateFlag",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.featureflag.v1.FeatureFlagService",
                        "EvaluateFlag",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_flag(
            &mut self,
            request: impl tonic::IntoRequest<super::GetFlagRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetFlagResponse>,
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
                "/k1s0.system.featureflag.v1.FeatureFlagService/GetFlag",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.featureflag.v1.FeatureFlagService",
                        "GetFlag",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_flags(
            &mut self,
            request: impl tonic::IntoRequest<super::ListFlagsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListFlagsResponse>,
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
                "/k1s0.system.featureflag.v1.FeatureFlagService/ListFlags",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.featureflag.v1.FeatureFlagService",
                        "ListFlags",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_flag(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateFlagRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateFlagResponse>,
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
                "/k1s0.system.featureflag.v1.FeatureFlagService/CreateFlag",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.featureflag.v1.FeatureFlagService",
                        "CreateFlag",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_flag(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateFlagRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateFlagResponse>,
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
                "/k1s0.system.featureflag.v1.FeatureFlagService/UpdateFlag",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.featureflag.v1.FeatureFlagService",
                        "UpdateFlag",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_flag(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteFlagRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteFlagResponse>,
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
                "/k1s0.system.featureflag.v1.FeatureFlagService/DeleteFlag",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.featureflag.v1.FeatureFlagService",
                        "DeleteFlag",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod feature_flag_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with FeatureFlagServiceServer.
    #[async_trait]
    pub trait FeatureFlagService: std::marker::Send + std::marker::Sync + 'static {
        async fn evaluate_flag(
            &self,
            request: tonic::Request<super::EvaluateFlagRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EvaluateFlagResponse>,
            tonic::Status,
        >;
        async fn get_flag(
            &self,
            request: tonic::Request<super::GetFlagRequest>,
        ) -> std::result::Result<tonic::Response<super::GetFlagResponse>, tonic::Status>;
        async fn list_flags(
            &self,
            request: tonic::Request<super::ListFlagsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListFlagsResponse>,
            tonic::Status,
        >;
        async fn create_flag(
            &self,
            request: tonic::Request<super::CreateFlagRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateFlagResponse>,
            tonic::Status,
        >;
        async fn update_flag(
            &self,
            request: tonic::Request<super::UpdateFlagRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateFlagResponse>,
            tonic::Status,
        >;
        async fn delete_flag(
            &self,
            request: tonic::Request<super::DeleteFlagRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteFlagResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct FeatureFlagServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> FeatureFlagServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for FeatureFlagServiceServer<T>
    where
        T: FeatureFlagService,
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
                "/k1s0.system.featureflag.v1.FeatureFlagService/EvaluateFlag" => {
                    #[allow(non_camel_case_types)]
                    struct EvaluateFlagSvc<T: FeatureFlagService>(pub Arc<T>);
                    impl<
                        T: FeatureFlagService,
                    > tonic::server::UnaryService<super::EvaluateFlagRequest>
                    for EvaluateFlagSvc<T> {
                        type Response = super::EvaluateFlagResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::EvaluateFlagRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as FeatureFlagService>::evaluate_flag(&inner, request)
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
                        let method = EvaluateFlagSvc(inner);
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
                "/k1s0.system.featureflag.v1.FeatureFlagService/GetFlag" => {
                    #[allow(non_camel_case_types)]
                    struct GetFlagSvc<T: FeatureFlagService>(pub Arc<T>);
                    impl<
                        T: FeatureFlagService,
                    > tonic::server::UnaryService<super::GetFlagRequest>
                    for GetFlagSvc<T> {
                        type Response = super::GetFlagResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetFlagRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as FeatureFlagService>::get_flag(&inner, request).await
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
                        let method = GetFlagSvc(inner);
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
                "/k1s0.system.featureflag.v1.FeatureFlagService/ListFlags" => {
                    #[allow(non_camel_case_types)]
                    struct ListFlagsSvc<T: FeatureFlagService>(pub Arc<T>);
                    impl<
                        T: FeatureFlagService,
                    > tonic::server::UnaryService<super::ListFlagsRequest>
                    for ListFlagsSvc<T> {
                        type Response = super::ListFlagsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListFlagsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as FeatureFlagService>::list_flags(&inner, request).await
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
                        let method = ListFlagsSvc(inner);
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
                "/k1s0.system.featureflag.v1.FeatureFlagService/CreateFlag" => {
                    #[allow(non_camel_case_types)]
                    struct CreateFlagSvc<T: FeatureFlagService>(pub Arc<T>);
                    impl<
                        T: FeatureFlagService,
                    > tonic::server::UnaryService<super::CreateFlagRequest>
                    for CreateFlagSvc<T> {
                        type Response = super::CreateFlagResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateFlagRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as FeatureFlagService>::create_flag(&inner, request)
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
                        let method = CreateFlagSvc(inner);
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
                "/k1s0.system.featureflag.v1.FeatureFlagService/UpdateFlag" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateFlagSvc<T: FeatureFlagService>(pub Arc<T>);
                    impl<
                        T: FeatureFlagService,
                    > tonic::server::UnaryService<super::UpdateFlagRequest>
                    for UpdateFlagSvc<T> {
                        type Response = super::UpdateFlagResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateFlagRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as FeatureFlagService>::update_flag(&inner, request)
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
                        let method = UpdateFlagSvc(inner);
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
                "/k1s0.system.featureflag.v1.FeatureFlagService/DeleteFlag" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteFlagSvc<T: FeatureFlagService>(pub Arc<T>);
                    impl<
                        T: FeatureFlagService,
                    > tonic::server::UnaryService<super::DeleteFlagRequest>
                    for DeleteFlagSvc<T> {
                        type Response = super::DeleteFlagResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteFlagRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as FeatureFlagService>::delete_flag(&inner, request)
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
                        let method = DeleteFlagSvc(inner);
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
    impl<T> Clone for FeatureFlagServiceServer<T> {
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
    pub const SERVICE_NAME: &str = "k1s0.system.featureflag.v1.FeatureFlagService";
    impl<T> tonic::server::NamedService for FeatureFlagServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
