// @generated
/// Generated client implementations.
pub mod config_service_client {
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
    pub struct ConfigServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ConfigServiceClient<tonic::transport::Channel> {
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
    impl<T> ConfigServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
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
        ) -> ConfigServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
        {
            ConfigServiceClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn get_config(
            &mut self,
            request: impl tonic::IntoRequest<super::GetConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetConfigResponse>,
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
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.config.v1.ConfigService/GetConfig",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("k1s0.system.config.v1.ConfigService", "GetConfig"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_configs(
            &mut self,
            request: impl tonic::IntoRequest<super::ListConfigsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListConfigsResponse>,
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
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.config.v1.ConfigService/ListConfigs",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("k1s0.system.config.v1.ConfigService", "ListConfigs"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_config(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateConfigResponse>,
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
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.config.v1.ConfigService/UpdateConfig",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.config.v1.ConfigService",
                        "UpdateConfig",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_config(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteConfigResponse>,
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
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.config.v1.ConfigService/DeleteConfig",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.config.v1.ConfigService",
                        "DeleteConfig",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_service_config(
            &mut self,
            request: impl tonic::IntoRequest<super::GetServiceConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetServiceConfigResponse>,
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
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.config.v1.ConfigService/GetServiceConfig",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.config.v1.ConfigService",
                        "GetServiceConfig",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn watch_config(
            &mut self,
            request: impl tonic::IntoRequest<super::WatchConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::WatchConfigResponse>>,
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
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.config.v1.ConfigService/WatchConfig",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("k1s0.system.config.v1.ConfigService", "WatchConfig"),
                );
            self.inner.server_streaming(req, path, codec).await
        }
        pub async fn get_config_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::GetConfigSchemaRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetConfigSchemaResponse>,
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
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.config.v1.ConfigService/GetConfigSchema",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.config.v1.ConfigService",
                        "GetConfigSchema",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn upsert_config_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::UpsertConfigSchemaRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpsertConfigSchemaResponse>,
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
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.config.v1.ConfigService/UpsertConfigSchema",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.config.v1.ConfigService",
                        "UpsertConfigSchema",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_config_schemas(
            &mut self,
            request: impl tonic::IntoRequest<super::ListConfigSchemasRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListConfigSchemasResponse>,
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
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/k1s0.system.config.v1.ConfigService/ListConfigSchemas",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.config.v1.ConfigService",
                        "ListConfigSchemas",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod config_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ConfigServiceServer.
    #[async_trait]
    pub trait ConfigService: std::marker::Send + std::marker::Sync + 'static {
        async fn get_config(
            &self,
            request: tonic::Request<super::GetConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetConfigResponse>,
            tonic::Status,
        >;
        async fn list_configs(
            &self,
            request: tonic::Request<super::ListConfigsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListConfigsResponse>,
            tonic::Status,
        >;
        async fn update_config(
            &self,
            request: tonic::Request<super::UpdateConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateConfigResponse>,
            tonic::Status,
        >;
        async fn delete_config(
            &self,
            request: tonic::Request<super::DeleteConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteConfigResponse>,
            tonic::Status,
        >;
        async fn get_service_config(
            &self,
            request: tonic::Request<super::GetServiceConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetServiceConfigResponse>,
            tonic::Status,
        >;
        /// Server streaming response type for the WatchConfig method.
        type WatchConfigStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::WatchConfigResponse, tonic::Status>,
            >
            + std::marker::Send
            + 'static;
        async fn watch_config(
            &self,
            request: tonic::Request<super::WatchConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<Self::WatchConfigStream>,
            tonic::Status,
        >;
        async fn get_config_schema(
            &self,
            request: tonic::Request<super::GetConfigSchemaRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetConfigSchemaResponse>,
            tonic::Status,
        >;
        async fn upsert_config_schema(
            &self,
            request: tonic::Request<super::UpsertConfigSchemaRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpsertConfigSchemaResponse>,
            tonic::Status,
        >;
        async fn list_config_schemas(
            &self,
            request: tonic::Request<super::ListConfigSchemasRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListConfigSchemasResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct ConfigServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> ConfigServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ConfigServiceServer<T>
    where
        T: ConfigService,
        B: Body + std::marker::Send + 'static,
        B::Error: Into<StdError> + std::marker::Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
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
                "/k1s0.system.config.v1.ConfigService/GetConfig" => {
                    #[allow(non_camel_case_types)]
                    struct GetConfigSvc<T: ConfigService>(pub Arc<T>);
                    impl<
                        T: ConfigService,
                    > tonic::server::UnaryService<super::GetConfigRequest>
                    for GetConfigSvc<T> {
                        type Response = super::GetConfigResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetConfigRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ConfigService>::get_config(&inner, request).await
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
                        let method = GetConfigSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
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
                "/k1s0.system.config.v1.ConfigService/ListConfigs" => {
                    #[allow(non_camel_case_types)]
                    struct ListConfigsSvc<T: ConfigService>(pub Arc<T>);
                    impl<
                        T: ConfigService,
                    > tonic::server::UnaryService<super::ListConfigsRequest>
                    for ListConfigsSvc<T> {
                        type Response = super::ListConfigsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListConfigsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ConfigService>::list_configs(&inner, request).await
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
                        let method = ListConfigsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
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
                "/k1s0.system.config.v1.ConfigService/UpdateConfig" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateConfigSvc<T: ConfigService>(pub Arc<T>);
                    impl<
                        T: ConfigService,
                    > tonic::server::UnaryService<super::UpdateConfigRequest>
                    for UpdateConfigSvc<T> {
                        type Response = super::UpdateConfigResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateConfigRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ConfigService>::update_config(&inner, request).await
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
                        let method = UpdateConfigSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
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
                "/k1s0.system.config.v1.ConfigService/DeleteConfig" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteConfigSvc<T: ConfigService>(pub Arc<T>);
                    impl<
                        T: ConfigService,
                    > tonic::server::UnaryService<super::DeleteConfigRequest>
                    for DeleteConfigSvc<T> {
                        type Response = super::DeleteConfigResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteConfigRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ConfigService>::delete_config(&inner, request).await
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
                        let method = DeleteConfigSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
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
                "/k1s0.system.config.v1.ConfigService/GetServiceConfig" => {
                    #[allow(non_camel_case_types)]
                    struct GetServiceConfigSvc<T: ConfigService>(pub Arc<T>);
                    impl<
                        T: ConfigService,
                    > tonic::server::UnaryService<super::GetServiceConfigRequest>
                    for GetServiceConfigSvc<T> {
                        type Response = super::GetServiceConfigResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetServiceConfigRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ConfigService>::get_service_config(&inner, request)
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
                        let method = GetServiceConfigSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
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
                "/k1s0.system.config.v1.ConfigService/WatchConfig" => {
                    #[allow(non_camel_case_types)]
                    struct WatchConfigSvc<T: ConfigService>(pub Arc<T>);
                    impl<
                        T: ConfigService,
                    > tonic::server::ServerStreamingService<super::WatchConfigRequest>
                    for WatchConfigSvc<T> {
                        type Response = super::WatchConfigResponse;
                        type ResponseStream = T::WatchConfigStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::WatchConfigRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ConfigService>::watch_config(&inner, request).await
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
                        let method = WatchConfigSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/k1s0.system.config.v1.ConfigService/GetConfigSchema" => {
                    #[allow(non_camel_case_types)]
                    struct GetConfigSchemaSvc<T: ConfigService>(pub Arc<T>);
                    impl<
                        T: ConfigService,
                    > tonic::server::UnaryService<super::GetConfigSchemaRequest>
                    for GetConfigSchemaSvc<T> {
                        type Response = super::GetConfigSchemaResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetConfigSchemaRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ConfigService>::get_config_schema(&inner, request)
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
                        let method = GetConfigSchemaSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
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
                "/k1s0.system.config.v1.ConfigService/UpsertConfigSchema" => {
                    #[allow(non_camel_case_types)]
                    struct UpsertConfigSchemaSvc<T: ConfigService>(pub Arc<T>);
                    impl<
                        T: ConfigService,
                    > tonic::server::UnaryService<super::UpsertConfigSchemaRequest>
                    for UpsertConfigSchemaSvc<T> {
                        type Response = super::UpsertConfigSchemaResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpsertConfigSchemaRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ConfigService>::upsert_config_schema(&inner, request)
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
                        let method = UpsertConfigSchemaSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
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
                "/k1s0.system.config.v1.ConfigService/ListConfigSchemas" => {
                    #[allow(non_camel_case_types)]
                    struct ListConfigSchemasSvc<T: ConfigService>(pub Arc<T>);
                    impl<
                        T: ConfigService,
                    > tonic::server::UnaryService<super::ListConfigSchemasRequest>
                    for ListConfigSchemasSvc<T> {
                        type Response = super::ListConfigSchemasResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListConfigSchemasRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ConfigService>::list_config_schemas(&inner, request)
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
                        let method = ListConfigSchemasSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
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
                            tonic::body::BoxBody::default(),
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
    impl<T> Clone for ConfigServiceServer<T> {
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
    pub const SERVICE_NAME: &str = "k1s0.system.config.v1.ConfigService";
    impl<T> tonic::server::NamedService for ConfigServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
