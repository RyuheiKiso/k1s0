// @generated
/// Generated client implementations.
pub mod domain_master_service_client {
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
    pub struct DomainMasterServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl DomainMasterServiceClient<tonic::transport::Channel> {
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
    impl<T> DomainMasterServiceClient<T>
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
        ) -> DomainMasterServiceClient<InterceptedService<T, F>>
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
            DomainMasterServiceClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn list_categories(
            &mut self,
            request: impl tonic::IntoRequest<super::ListCategoriesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListCategoriesResponse>,
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/ListCategories",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.accounting.domainmaster.v1.DomainMasterService",
                        "ListCategories",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_category(
            &mut self,
            request: impl tonic::IntoRequest<super::GetCategoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetCategoryResponse>,
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/GetCategory",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.accounting.domainmaster.v1.DomainMasterService",
                        "GetCategory",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_category(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateCategoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateCategoryResponse>,
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/CreateCategory",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.accounting.domainmaster.v1.DomainMasterService",
                        "CreateCategory",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_category(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateCategoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateCategoryResponse>,
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/UpdateCategory",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.accounting.domainmaster.v1.DomainMasterService",
                        "UpdateCategory",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_category(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteCategoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteCategoryResponse>,
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/DeleteCategory",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.accounting.domainmaster.v1.DomainMasterService",
                        "DeleteCategory",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_items(
            &mut self,
            request: impl tonic::IntoRequest<super::ListItemsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListItemsResponse>,
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/ListItems",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.accounting.domainmaster.v1.DomainMasterService",
                        "ListItems",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_item(
            &mut self,
            request: impl tonic::IntoRequest<super::GetItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetItemResponse>,
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/GetItem",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.accounting.domainmaster.v1.DomainMasterService",
                        "GetItem",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_item(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateItemResponse>,
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/CreateItem",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.accounting.domainmaster.v1.DomainMasterService",
                        "CreateItem",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_item(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateItemResponse>,
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/UpdateItem",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.accounting.domainmaster.v1.DomainMasterService",
                        "UpdateItem",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_item(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteItemResponse>,
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/DeleteItem",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.accounting.domainmaster.v1.DomainMasterService",
                        "DeleteItem",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_item_versions(
            &mut self,
            request: impl tonic::IntoRequest<super::ListItemVersionsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListItemVersionsResponse>,
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/ListItemVersions",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.accounting.domainmaster.v1.DomainMasterService",
                        "ListItemVersions",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_tenant_extension(
            &mut self,
            request: impl tonic::IntoRequest<super::GetTenantExtensionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetTenantExtensionResponse>,
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/GetTenantExtension",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.accounting.domainmaster.v1.DomainMasterService",
                        "GetTenantExtension",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn upsert_tenant_extension(
            &mut self,
            request: impl tonic::IntoRequest<super::UpsertTenantExtensionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpsertTenantExtensionResponse>,
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/UpsertTenantExtension",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.accounting.domainmaster.v1.DomainMasterService",
                        "UpsertTenantExtension",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_tenant_extension(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteTenantExtensionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteTenantExtensionResponse>,
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/DeleteTenantExtension",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.accounting.domainmaster.v1.DomainMasterService",
                        "DeleteTenantExtension",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_tenant_items(
            &mut self,
            request: impl tonic::IntoRequest<super::ListTenantItemsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListTenantItemsResponse>,
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/ListTenantItems",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.accounting.domainmaster.v1.DomainMasterService",
                        "ListTenantItems",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod domain_master_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with DomainMasterServiceServer.
    #[async_trait]
    pub trait DomainMasterService: std::marker::Send + std::marker::Sync + 'static {
        async fn list_categories(
            &self,
            request: tonic::Request<super::ListCategoriesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListCategoriesResponse>,
            tonic::Status,
        >;
        async fn get_category(
            &self,
            request: tonic::Request<super::GetCategoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetCategoryResponse>,
            tonic::Status,
        >;
        async fn create_category(
            &self,
            request: tonic::Request<super::CreateCategoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateCategoryResponse>,
            tonic::Status,
        >;
        async fn update_category(
            &self,
            request: tonic::Request<super::UpdateCategoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateCategoryResponse>,
            tonic::Status,
        >;
        async fn delete_category(
            &self,
            request: tonic::Request<super::DeleteCategoryRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteCategoryResponse>,
            tonic::Status,
        >;
        async fn list_items(
            &self,
            request: tonic::Request<super::ListItemsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListItemsResponse>,
            tonic::Status,
        >;
        async fn get_item(
            &self,
            request: tonic::Request<super::GetItemRequest>,
        ) -> std::result::Result<tonic::Response<super::GetItemResponse>, tonic::Status>;
        async fn create_item(
            &self,
            request: tonic::Request<super::CreateItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateItemResponse>,
            tonic::Status,
        >;
        async fn update_item(
            &self,
            request: tonic::Request<super::UpdateItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateItemResponse>,
            tonic::Status,
        >;
        async fn delete_item(
            &self,
            request: tonic::Request<super::DeleteItemRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteItemResponse>,
            tonic::Status,
        >;
        async fn list_item_versions(
            &self,
            request: tonic::Request<super::ListItemVersionsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListItemVersionsResponse>,
            tonic::Status,
        >;
        async fn get_tenant_extension(
            &self,
            request: tonic::Request<super::GetTenantExtensionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetTenantExtensionResponse>,
            tonic::Status,
        >;
        async fn upsert_tenant_extension(
            &self,
            request: tonic::Request<super::UpsertTenantExtensionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpsertTenantExtensionResponse>,
            tonic::Status,
        >;
        async fn delete_tenant_extension(
            &self,
            request: tonic::Request<super::DeleteTenantExtensionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteTenantExtensionResponse>,
            tonic::Status,
        >;
        async fn list_tenant_items(
            &self,
            request: tonic::Request<super::ListTenantItemsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListTenantItemsResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct DomainMasterServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> DomainMasterServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for DomainMasterServiceServer<T>
    where
        T: DomainMasterService,
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/ListCategories" => {
                    #[allow(non_camel_case_types)]
                    struct ListCategoriesSvc<T: DomainMasterService>(pub Arc<T>);
                    impl<
                        T: DomainMasterService,
                    > tonic::server::UnaryService<super::ListCategoriesRequest>
                    for ListCategoriesSvc<T> {
                        type Response = super::ListCategoriesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListCategoriesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DomainMasterService>::list_categories(&inner, request)
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
                        let method = ListCategoriesSvc(inner);
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/GetCategory" => {
                    #[allow(non_camel_case_types)]
                    struct GetCategorySvc<T: DomainMasterService>(pub Arc<T>);
                    impl<
                        T: DomainMasterService,
                    > tonic::server::UnaryService<super::GetCategoryRequest>
                    for GetCategorySvc<T> {
                        type Response = super::GetCategoryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetCategoryRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DomainMasterService>::get_category(&inner, request)
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
                        let method = GetCategorySvc(inner);
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/CreateCategory" => {
                    #[allow(non_camel_case_types)]
                    struct CreateCategorySvc<T: DomainMasterService>(pub Arc<T>);
                    impl<
                        T: DomainMasterService,
                    > tonic::server::UnaryService<super::CreateCategoryRequest>
                    for CreateCategorySvc<T> {
                        type Response = super::CreateCategoryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateCategoryRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DomainMasterService>::create_category(&inner, request)
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
                        let method = CreateCategorySvc(inner);
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/UpdateCategory" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateCategorySvc<T: DomainMasterService>(pub Arc<T>);
                    impl<
                        T: DomainMasterService,
                    > tonic::server::UnaryService<super::UpdateCategoryRequest>
                    for UpdateCategorySvc<T> {
                        type Response = super::UpdateCategoryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateCategoryRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DomainMasterService>::update_category(&inner, request)
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
                        let method = UpdateCategorySvc(inner);
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/DeleteCategory" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteCategorySvc<T: DomainMasterService>(pub Arc<T>);
                    impl<
                        T: DomainMasterService,
                    > tonic::server::UnaryService<super::DeleteCategoryRequest>
                    for DeleteCategorySvc<T> {
                        type Response = super::DeleteCategoryResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteCategoryRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DomainMasterService>::delete_category(&inner, request)
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
                        let method = DeleteCategorySvc(inner);
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/ListItems" => {
                    #[allow(non_camel_case_types)]
                    struct ListItemsSvc<T: DomainMasterService>(pub Arc<T>);
                    impl<
                        T: DomainMasterService,
                    > tonic::server::UnaryService<super::ListItemsRequest>
                    for ListItemsSvc<T> {
                        type Response = super::ListItemsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListItemsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DomainMasterService>::list_items(&inner, request)
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
                        let method = ListItemsSvc(inner);
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/GetItem" => {
                    #[allow(non_camel_case_types)]
                    struct GetItemSvc<T: DomainMasterService>(pub Arc<T>);
                    impl<
                        T: DomainMasterService,
                    > tonic::server::UnaryService<super::GetItemRequest>
                    for GetItemSvc<T> {
                        type Response = super::GetItemResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetItemRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DomainMasterService>::get_item(&inner, request).await
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
                        let method = GetItemSvc(inner);
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/CreateItem" => {
                    #[allow(non_camel_case_types)]
                    struct CreateItemSvc<T: DomainMasterService>(pub Arc<T>);
                    impl<
                        T: DomainMasterService,
                    > tonic::server::UnaryService<super::CreateItemRequest>
                    for CreateItemSvc<T> {
                        type Response = super::CreateItemResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateItemRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DomainMasterService>::create_item(&inner, request)
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
                        let method = CreateItemSvc(inner);
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/UpdateItem" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateItemSvc<T: DomainMasterService>(pub Arc<T>);
                    impl<
                        T: DomainMasterService,
                    > tonic::server::UnaryService<super::UpdateItemRequest>
                    for UpdateItemSvc<T> {
                        type Response = super::UpdateItemResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateItemRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DomainMasterService>::update_item(&inner, request)
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
                        let method = UpdateItemSvc(inner);
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/DeleteItem" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteItemSvc<T: DomainMasterService>(pub Arc<T>);
                    impl<
                        T: DomainMasterService,
                    > tonic::server::UnaryService<super::DeleteItemRequest>
                    for DeleteItemSvc<T> {
                        type Response = super::DeleteItemResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteItemRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DomainMasterService>::delete_item(&inner, request)
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
                        let method = DeleteItemSvc(inner);
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/ListItemVersions" => {
                    #[allow(non_camel_case_types)]
                    struct ListItemVersionsSvc<T: DomainMasterService>(pub Arc<T>);
                    impl<
                        T: DomainMasterService,
                    > tonic::server::UnaryService<super::ListItemVersionsRequest>
                    for ListItemVersionsSvc<T> {
                        type Response = super::ListItemVersionsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListItemVersionsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DomainMasterService>::list_item_versions(
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
                        let method = ListItemVersionsSvc(inner);
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/GetTenantExtension" => {
                    #[allow(non_camel_case_types)]
                    struct GetTenantExtensionSvc<T: DomainMasterService>(pub Arc<T>);
                    impl<
                        T: DomainMasterService,
                    > tonic::server::UnaryService<super::GetTenantExtensionRequest>
                    for GetTenantExtensionSvc<T> {
                        type Response = super::GetTenantExtensionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetTenantExtensionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DomainMasterService>::get_tenant_extension(
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
                        let method = GetTenantExtensionSvc(inner);
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/UpsertTenantExtension" => {
                    #[allow(non_camel_case_types)]
                    struct UpsertTenantExtensionSvc<T: DomainMasterService>(pub Arc<T>);
                    impl<
                        T: DomainMasterService,
                    > tonic::server::UnaryService<super::UpsertTenantExtensionRequest>
                    for UpsertTenantExtensionSvc<T> {
                        type Response = super::UpsertTenantExtensionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpsertTenantExtensionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DomainMasterService>::upsert_tenant_extension(
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
                        let method = UpsertTenantExtensionSvc(inner);
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/DeleteTenantExtension" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteTenantExtensionSvc<T: DomainMasterService>(pub Arc<T>);
                    impl<
                        T: DomainMasterService,
                    > tonic::server::UnaryService<super::DeleteTenantExtensionRequest>
                    for DeleteTenantExtensionSvc<T> {
                        type Response = super::DeleteTenantExtensionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteTenantExtensionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DomainMasterService>::delete_tenant_extension(
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
                        let method = DeleteTenantExtensionSvc(inner);
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
                "/k1s0.business.accounting.domainmaster.v1.DomainMasterService/ListTenantItems" => {
                    #[allow(non_camel_case_types)]
                    struct ListTenantItemsSvc<T: DomainMasterService>(pub Arc<T>);
                    impl<
                        T: DomainMasterService,
                    > tonic::server::UnaryService<super::ListTenantItemsRequest>
                    for ListTenantItemsSvc<T> {
                        type Response = super::ListTenantItemsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListTenantItemsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as DomainMasterService>::list_tenant_items(
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
                        let method = ListTenantItemsSvc(inner);
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
    impl<T> Clone for DomainMasterServiceServer<T> {
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
    pub const SERVICE_NAME: &str = "k1s0.business.accounting.domainmaster.v1.DomainMasterService";
    impl<T> tonic::server::NamedService for DomainMasterServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
