// @generated
/// Generated client implementations.
pub mod project_master_service_client {
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
    pub struct ProjectMasterServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ProjectMasterServiceClient<tonic::transport::Channel> {
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
    impl<T> ProjectMasterServiceClient<T>
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
        ) -> ProjectMasterServiceClient<InterceptedService<T, F>>
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
            ProjectMasterServiceClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn list_project_types(
            &mut self,
            request: impl tonic::IntoRequest<super::ListProjectTypesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListProjectTypesResponse>,
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/ListProjectTypes",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService",
                        "ListProjectTypes",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_project_type(
            &mut self,
            request: impl tonic::IntoRequest<super::GetProjectTypeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetProjectTypeResponse>,
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/GetProjectType",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService",
                        "GetProjectType",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_project_type(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateProjectTypeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateProjectTypeResponse>,
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/CreateProjectType",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService",
                        "CreateProjectType",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_project_type(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateProjectTypeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateProjectTypeResponse>,
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/UpdateProjectType",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService",
                        "UpdateProjectType",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_project_type(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteProjectTypeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteProjectTypeResponse>,
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/DeleteProjectType",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService",
                        "DeleteProjectType",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_status_definitions(
            &mut self,
            request: impl tonic::IntoRequest<super::ListStatusDefinitionsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListStatusDefinitionsResponse>,
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/ListStatusDefinitions",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService",
                        "ListStatusDefinitions",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_status_definition(
            &mut self,
            request: impl tonic::IntoRequest<super::GetStatusDefinitionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetStatusDefinitionResponse>,
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/GetStatusDefinition",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService",
                        "GetStatusDefinition",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_status_definition(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateStatusDefinitionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateStatusDefinitionResponse>,
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/CreateStatusDefinition",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService",
                        "CreateStatusDefinition",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_status_definition(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateStatusDefinitionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateStatusDefinitionResponse>,
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/UpdateStatusDefinition",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService",
                        "UpdateStatusDefinition",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_status_definition(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteStatusDefinitionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteStatusDefinitionResponse>,
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/DeleteStatusDefinition",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService",
                        "DeleteStatusDefinition",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_status_definition_versions(
            &mut self,
            request: impl tonic::IntoRequest<super::ListStatusDefinitionVersionsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListStatusDefinitionVersionsResponse>,
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/ListStatusDefinitionVersions",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService",
                        "ListStatusDefinitionVersions",
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/GetTenantExtension",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService",
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/UpsertTenantExtension",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService",
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/DeleteTenantExtension",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService",
                        "DeleteTenantExtension",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_tenant_statuses(
            &mut self,
            request: impl tonic::IntoRequest<super::ListTenantStatusesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListTenantStatusesResponse>,
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/ListTenantStatuses",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService",
                        "ListTenantStatuses",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod project_master_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ProjectMasterServiceServer.
    #[async_trait]
    pub trait ProjectMasterService: std::marker::Send + std::marker::Sync + 'static {
        async fn list_project_types(
            &self,
            request: tonic::Request<super::ListProjectTypesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListProjectTypesResponse>,
            tonic::Status,
        >;
        async fn get_project_type(
            &self,
            request: tonic::Request<super::GetProjectTypeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetProjectTypeResponse>,
            tonic::Status,
        >;
        async fn create_project_type(
            &self,
            request: tonic::Request<super::CreateProjectTypeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateProjectTypeResponse>,
            tonic::Status,
        >;
        async fn update_project_type(
            &self,
            request: tonic::Request<super::UpdateProjectTypeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateProjectTypeResponse>,
            tonic::Status,
        >;
        async fn delete_project_type(
            &self,
            request: tonic::Request<super::DeleteProjectTypeRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteProjectTypeResponse>,
            tonic::Status,
        >;
        async fn list_status_definitions(
            &self,
            request: tonic::Request<super::ListStatusDefinitionsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListStatusDefinitionsResponse>,
            tonic::Status,
        >;
        async fn get_status_definition(
            &self,
            request: tonic::Request<super::GetStatusDefinitionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetStatusDefinitionResponse>,
            tonic::Status,
        >;
        async fn create_status_definition(
            &self,
            request: tonic::Request<super::CreateStatusDefinitionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateStatusDefinitionResponse>,
            tonic::Status,
        >;
        async fn update_status_definition(
            &self,
            request: tonic::Request<super::UpdateStatusDefinitionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateStatusDefinitionResponse>,
            tonic::Status,
        >;
        async fn delete_status_definition(
            &self,
            request: tonic::Request<super::DeleteStatusDefinitionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteStatusDefinitionResponse>,
            tonic::Status,
        >;
        async fn list_status_definition_versions(
            &self,
            request: tonic::Request<super::ListStatusDefinitionVersionsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListStatusDefinitionVersionsResponse>,
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
        async fn list_tenant_statuses(
            &self,
            request: tonic::Request<super::ListTenantStatusesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListTenantStatusesResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct ProjectMasterServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> ProjectMasterServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>>
    for ProjectMasterServiceServer<T>
    where
        T: ProjectMasterService,
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/ListProjectTypes" => {
                    #[allow(non_camel_case_types)]
                    struct ListProjectTypesSvc<T: ProjectMasterService>(pub Arc<T>);
                    impl<
                        T: ProjectMasterService,
                    > tonic::server::UnaryService<super::ListProjectTypesRequest>
                    for ListProjectTypesSvc<T> {
                        type Response = super::ListProjectTypesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListProjectTypesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProjectMasterService>::list_project_types(
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
                        let method = ListProjectTypesSvc(inner);
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/GetProjectType" => {
                    #[allow(non_camel_case_types)]
                    struct GetProjectTypeSvc<T: ProjectMasterService>(pub Arc<T>);
                    impl<
                        T: ProjectMasterService,
                    > tonic::server::UnaryService<super::GetProjectTypeRequest>
                    for GetProjectTypeSvc<T> {
                        type Response = super::GetProjectTypeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetProjectTypeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProjectMasterService>::get_project_type(
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
                        let method = GetProjectTypeSvc(inner);
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/CreateProjectType" => {
                    #[allow(non_camel_case_types)]
                    struct CreateProjectTypeSvc<T: ProjectMasterService>(pub Arc<T>);
                    impl<
                        T: ProjectMasterService,
                    > tonic::server::UnaryService<super::CreateProjectTypeRequest>
                    for CreateProjectTypeSvc<T> {
                        type Response = super::CreateProjectTypeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateProjectTypeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProjectMasterService>::create_project_type(
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
                        let method = CreateProjectTypeSvc(inner);
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/UpdateProjectType" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateProjectTypeSvc<T: ProjectMasterService>(pub Arc<T>);
                    impl<
                        T: ProjectMasterService,
                    > tonic::server::UnaryService<super::UpdateProjectTypeRequest>
                    for UpdateProjectTypeSvc<T> {
                        type Response = super::UpdateProjectTypeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateProjectTypeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProjectMasterService>::update_project_type(
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
                        let method = UpdateProjectTypeSvc(inner);
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/DeleteProjectType" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteProjectTypeSvc<T: ProjectMasterService>(pub Arc<T>);
                    impl<
                        T: ProjectMasterService,
                    > tonic::server::UnaryService<super::DeleteProjectTypeRequest>
                    for DeleteProjectTypeSvc<T> {
                        type Response = super::DeleteProjectTypeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteProjectTypeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProjectMasterService>::delete_project_type(
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
                        let method = DeleteProjectTypeSvc(inner);
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/ListStatusDefinitions" => {
                    #[allow(non_camel_case_types)]
                    struct ListStatusDefinitionsSvc<T: ProjectMasterService>(pub Arc<T>);
                    impl<
                        T: ProjectMasterService,
                    > tonic::server::UnaryService<super::ListStatusDefinitionsRequest>
                    for ListStatusDefinitionsSvc<T> {
                        type Response = super::ListStatusDefinitionsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListStatusDefinitionsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProjectMasterService>::list_status_definitions(
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
                        let method = ListStatusDefinitionsSvc(inner);
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/GetStatusDefinition" => {
                    #[allow(non_camel_case_types)]
                    struct GetStatusDefinitionSvc<T: ProjectMasterService>(pub Arc<T>);
                    impl<
                        T: ProjectMasterService,
                    > tonic::server::UnaryService<super::GetStatusDefinitionRequest>
                    for GetStatusDefinitionSvc<T> {
                        type Response = super::GetStatusDefinitionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetStatusDefinitionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProjectMasterService>::get_status_definition(
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
                        let method = GetStatusDefinitionSvc(inner);
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/CreateStatusDefinition" => {
                    #[allow(non_camel_case_types)]
                    struct CreateStatusDefinitionSvc<T: ProjectMasterService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: ProjectMasterService,
                    > tonic::server::UnaryService<super::CreateStatusDefinitionRequest>
                    for CreateStatusDefinitionSvc<T> {
                        type Response = super::CreateStatusDefinitionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateStatusDefinitionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProjectMasterService>::create_status_definition(
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
                        let method = CreateStatusDefinitionSvc(inner);
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/UpdateStatusDefinition" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateStatusDefinitionSvc<T: ProjectMasterService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: ProjectMasterService,
                    > tonic::server::UnaryService<super::UpdateStatusDefinitionRequest>
                    for UpdateStatusDefinitionSvc<T> {
                        type Response = super::UpdateStatusDefinitionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateStatusDefinitionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProjectMasterService>::update_status_definition(
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
                        let method = UpdateStatusDefinitionSvc(inner);
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/DeleteStatusDefinition" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteStatusDefinitionSvc<T: ProjectMasterService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: ProjectMasterService,
                    > tonic::server::UnaryService<super::DeleteStatusDefinitionRequest>
                    for DeleteStatusDefinitionSvc<T> {
                        type Response = super::DeleteStatusDefinitionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteStatusDefinitionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProjectMasterService>::delete_status_definition(
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
                        let method = DeleteStatusDefinitionSvc(inner);
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/ListStatusDefinitionVersions" => {
                    #[allow(non_camel_case_types)]
                    struct ListStatusDefinitionVersionsSvc<T: ProjectMasterService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: ProjectMasterService,
                    > tonic::server::UnaryService<
                        super::ListStatusDefinitionVersionsRequest,
                    > for ListStatusDefinitionVersionsSvc<T> {
                        type Response = super::ListStatusDefinitionVersionsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::ListStatusDefinitionVersionsRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProjectMasterService>::list_status_definition_versions(
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
                        let method = ListStatusDefinitionVersionsSvc(inner);
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/GetTenantExtension" => {
                    #[allow(non_camel_case_types)]
                    struct GetTenantExtensionSvc<T: ProjectMasterService>(pub Arc<T>);
                    impl<
                        T: ProjectMasterService,
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
                                <T as ProjectMasterService>::get_tenant_extension(
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/UpsertTenantExtension" => {
                    #[allow(non_camel_case_types)]
                    struct UpsertTenantExtensionSvc<T: ProjectMasterService>(pub Arc<T>);
                    impl<
                        T: ProjectMasterService,
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
                                <T as ProjectMasterService>::upsert_tenant_extension(
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/DeleteTenantExtension" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteTenantExtensionSvc<T: ProjectMasterService>(pub Arc<T>);
                    impl<
                        T: ProjectMasterService,
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
                                <T as ProjectMasterService>::delete_tenant_extension(
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
                "/k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService/ListTenantStatuses" => {
                    #[allow(non_camel_case_types)]
                    struct ListTenantStatusesSvc<T: ProjectMasterService>(pub Arc<T>);
                    impl<
                        T: ProjectMasterService,
                    > tonic::server::UnaryService<super::ListTenantStatusesRequest>
                    for ListTenantStatusesSvc<T> {
                        type Response = super::ListTenantStatusesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListTenantStatusesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as ProjectMasterService>::list_tenant_statuses(
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
                        let method = ListTenantStatusesSvc(inner);
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
    impl<T> Clone for ProjectMasterServiceServer<T> {
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
    pub const SERVICE_NAME: &str = "k1s0.business.taskmanagement.projectmaster.v1.ProjectMasterService";
    impl<T> tonic::server::NamedService for ProjectMasterServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
