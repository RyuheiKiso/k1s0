// @generated
/// Generated client implementations.
pub mod master_maintenance_service_client {
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
    pub struct MasterMaintenanceServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl MasterMaintenanceServiceClient<tonic::transport::Channel> {
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
    impl<T> MasterMaintenanceServiceClient<T>
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
        ) -> MasterMaintenanceServiceClient<InterceptedService<T, F>>
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
            MasterMaintenanceServiceClient::new(
                InterceptedService::new(inner, interceptor),
            )
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
        pub async fn create_table_definition(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateTableDefinitionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateTableDefinitionResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CreateTableDefinition",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "CreateTableDefinition",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_table_definition(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateTableDefinitionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateTableDefinitionResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/UpdateTableDefinition",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "UpdateTableDefinition",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_table_definition(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteTableDefinitionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteTableDefinitionResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/DeleteTableDefinition",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "DeleteTableDefinition",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_table_definition(
            &mut self,
            request: impl tonic::IntoRequest<super::GetTableDefinitionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetTableDefinitionResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/GetTableDefinition",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "GetTableDefinition",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_table_definitions(
            &mut self,
            request: impl tonic::IntoRequest<super::ListTableDefinitionsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListTableDefinitionsResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListTableDefinitions",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "ListTableDefinitions",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_columns(
            &mut self,
            request: impl tonic::IntoRequest<super::ListColumnsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListColumnsResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListColumns",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "ListColumns",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_columns(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateColumnsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateColumnsResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CreateColumns",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "CreateColumns",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_column(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateColumnRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateColumnResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/UpdateColumn",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "UpdateColumn",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_column(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteColumnRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteColumnResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/DeleteColumn",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "DeleteColumn",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_record(
            &mut self,
            request: impl tonic::IntoRequest<super::GetRecordRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetRecordResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/GetRecord",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "GetRecord",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_records(
            &mut self,
            request: impl tonic::IntoRequest<super::ListRecordsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListRecordsResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListRecords",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "ListRecords",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_record(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateRecordRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateRecordResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CreateRecord",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "CreateRecord",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_record(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateRecordRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateRecordResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/UpdateRecord",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "UpdateRecord",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_record(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteRecordRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteRecordResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/DeleteRecord",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "DeleteRecord",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn check_consistency(
            &mut self,
            request: impl tonic::IntoRequest<super::CheckConsistencyRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CheckConsistencyResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CheckConsistency",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "CheckConsistency",
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CreateRule",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "CreateRule",
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/GetRule",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "GetRule",
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/UpdateRule",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/DeleteRule",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "DeleteRule",
                    ),
                );
            self.inner.unary(req, path, codec).await
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListRules",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "ListRules",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn execute_rule(
            &mut self,
            request: impl tonic::IntoRequest<super::ExecuteRuleRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ExecuteRuleResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ExecuteRule",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "ExecuteRule",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_table_schema(
            &mut self,
            request: impl tonic::IntoRequest<super::GetTableSchemaRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetTableSchemaResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/GetTableSchema",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "GetTableSchema",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_relationships(
            &mut self,
            request: impl tonic::IntoRequest<super::ListRelationshipsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListRelationshipsResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListRelationships",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "ListRelationships",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_relationship(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateRelationshipRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateRelationshipResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CreateRelationship",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "CreateRelationship",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_relationship(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateRelationshipRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateRelationshipResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/UpdateRelationship",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "UpdateRelationship",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_relationship(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteRelationshipRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteRelationshipResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/DeleteRelationship",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "DeleteRelationship",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn import_records(
            &mut self,
            request: impl tonic::IntoRequest<super::ImportRecordsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ImportRecordsResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ImportRecords",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "ImportRecords",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn export_records(
            &mut self,
            request: impl tonic::IntoRequest<super::ExportRecordsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ExportRecordsResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ExportRecords",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "ExportRecords",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_import_job(
            &mut self,
            request: impl tonic::IntoRequest<super::GetImportJobRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetImportJobResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/GetImportJob",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "GetImportJob",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_display_configs(
            &mut self,
            request: impl tonic::IntoRequest<super::ListDisplayConfigsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListDisplayConfigsResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListDisplayConfigs",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "ListDisplayConfigs",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_display_config(
            &mut self,
            request: impl tonic::IntoRequest<super::GetDisplayConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetDisplayConfigResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/GetDisplayConfig",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "GetDisplayConfig",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_display_config(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateDisplayConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateDisplayConfigResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CreateDisplayConfig",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "CreateDisplayConfig",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn update_display_config(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateDisplayConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateDisplayConfigResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/UpdateDisplayConfig",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "UpdateDisplayConfig",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_display_config(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteDisplayConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteDisplayConfigResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/DeleteDisplayConfig",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "DeleteDisplayConfig",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_table_audit_logs(
            &mut self,
            request: impl tonic::IntoRequest<super::ListTableAuditLogsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListTableAuditLogsResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListTableAuditLogs",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "ListTableAuditLogs",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_record_audit_logs(
            &mut self,
            request: impl tonic::IntoRequest<super::ListRecordAuditLogsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListRecordAuditLogsResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListRecordAuditLogs",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "ListRecordAuditLogs",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_domains(
            &mut self,
            request: impl tonic::IntoRequest<super::ListDomainsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListDomainsResponse>,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListDomains",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "k1s0.system.mastermaintenance.v1.MasterMaintenanceService",
                        "ListDomains",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod master_maintenance_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with MasterMaintenanceServiceServer.
    #[async_trait]
    pub trait MasterMaintenanceService: std::marker::Send + std::marker::Sync + 'static {
        async fn create_table_definition(
            &self,
            request: tonic::Request<super::CreateTableDefinitionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateTableDefinitionResponse>,
            tonic::Status,
        >;
        async fn update_table_definition(
            &self,
            request: tonic::Request<super::UpdateTableDefinitionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateTableDefinitionResponse>,
            tonic::Status,
        >;
        async fn delete_table_definition(
            &self,
            request: tonic::Request<super::DeleteTableDefinitionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteTableDefinitionResponse>,
            tonic::Status,
        >;
        async fn get_table_definition(
            &self,
            request: tonic::Request<super::GetTableDefinitionRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetTableDefinitionResponse>,
            tonic::Status,
        >;
        async fn list_table_definitions(
            &self,
            request: tonic::Request<super::ListTableDefinitionsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListTableDefinitionsResponse>,
            tonic::Status,
        >;
        async fn list_columns(
            &self,
            request: tonic::Request<super::ListColumnsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListColumnsResponse>,
            tonic::Status,
        >;
        async fn create_columns(
            &self,
            request: tonic::Request<super::CreateColumnsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateColumnsResponse>,
            tonic::Status,
        >;
        async fn update_column(
            &self,
            request: tonic::Request<super::UpdateColumnRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateColumnResponse>,
            tonic::Status,
        >;
        async fn delete_column(
            &self,
            request: tonic::Request<super::DeleteColumnRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteColumnResponse>,
            tonic::Status,
        >;
        async fn get_record(
            &self,
            request: tonic::Request<super::GetRecordRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetRecordResponse>,
            tonic::Status,
        >;
        async fn list_records(
            &self,
            request: tonic::Request<super::ListRecordsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListRecordsResponse>,
            tonic::Status,
        >;
        async fn create_record(
            &self,
            request: tonic::Request<super::CreateRecordRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateRecordResponse>,
            tonic::Status,
        >;
        async fn update_record(
            &self,
            request: tonic::Request<super::UpdateRecordRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateRecordResponse>,
            tonic::Status,
        >;
        async fn delete_record(
            &self,
            request: tonic::Request<super::DeleteRecordRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteRecordResponse>,
            tonic::Status,
        >;
        async fn check_consistency(
            &self,
            request: tonic::Request<super::CheckConsistencyRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CheckConsistencyResponse>,
            tonic::Status,
        >;
        async fn create_rule(
            &self,
            request: tonic::Request<super::CreateRuleRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateRuleResponse>,
            tonic::Status,
        >;
        async fn get_rule(
            &self,
            request: tonic::Request<super::GetRuleRequest>,
        ) -> std::result::Result<tonic::Response<super::GetRuleResponse>, tonic::Status>;
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
        async fn list_rules(
            &self,
            request: tonic::Request<super::ListRulesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListRulesResponse>,
            tonic::Status,
        >;
        async fn execute_rule(
            &self,
            request: tonic::Request<super::ExecuteRuleRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ExecuteRuleResponse>,
            tonic::Status,
        >;
        async fn get_table_schema(
            &self,
            request: tonic::Request<super::GetTableSchemaRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetTableSchemaResponse>,
            tonic::Status,
        >;
        async fn list_relationships(
            &self,
            request: tonic::Request<super::ListRelationshipsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListRelationshipsResponse>,
            tonic::Status,
        >;
        async fn create_relationship(
            &self,
            request: tonic::Request<super::CreateRelationshipRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateRelationshipResponse>,
            tonic::Status,
        >;
        async fn update_relationship(
            &self,
            request: tonic::Request<super::UpdateRelationshipRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateRelationshipResponse>,
            tonic::Status,
        >;
        async fn delete_relationship(
            &self,
            request: tonic::Request<super::DeleteRelationshipRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteRelationshipResponse>,
            tonic::Status,
        >;
        async fn import_records(
            &self,
            request: tonic::Request<super::ImportRecordsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ImportRecordsResponse>,
            tonic::Status,
        >;
        async fn export_records(
            &self,
            request: tonic::Request<super::ExportRecordsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ExportRecordsResponse>,
            tonic::Status,
        >;
        async fn get_import_job(
            &self,
            request: tonic::Request<super::GetImportJobRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetImportJobResponse>,
            tonic::Status,
        >;
        async fn list_display_configs(
            &self,
            request: tonic::Request<super::ListDisplayConfigsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListDisplayConfigsResponse>,
            tonic::Status,
        >;
        async fn get_display_config(
            &self,
            request: tonic::Request<super::GetDisplayConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetDisplayConfigResponse>,
            tonic::Status,
        >;
        async fn create_display_config(
            &self,
            request: tonic::Request<super::CreateDisplayConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateDisplayConfigResponse>,
            tonic::Status,
        >;
        async fn update_display_config(
            &self,
            request: tonic::Request<super::UpdateDisplayConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<super::UpdateDisplayConfigResponse>,
            tonic::Status,
        >;
        async fn delete_display_config(
            &self,
            request: tonic::Request<super::DeleteDisplayConfigRequest>,
        ) -> std::result::Result<
            tonic::Response<super::DeleteDisplayConfigResponse>,
            tonic::Status,
        >;
        async fn list_table_audit_logs(
            &self,
            request: tonic::Request<super::ListTableAuditLogsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListTableAuditLogsResponse>,
            tonic::Status,
        >;
        async fn list_record_audit_logs(
            &self,
            request: tonic::Request<super::ListRecordAuditLogsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListRecordAuditLogsResponse>,
            tonic::Status,
        >;
        async fn list_domains(
            &self,
            request: tonic::Request<super::ListDomainsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListDomainsResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct MasterMaintenanceServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> MasterMaintenanceServiceServer<T> {
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
    for MasterMaintenanceServiceServer<T>
    where
        T: MasterMaintenanceService,
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CreateTableDefinition" => {
                    #[allow(non_camel_case_types)]
                    struct CreateTableDefinitionSvc<T: MasterMaintenanceService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::CreateTableDefinitionRequest>
                    for CreateTableDefinitionSvc<T> {
                        type Response = super::CreateTableDefinitionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateTableDefinitionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::create_table_definition(
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
                        let method = CreateTableDefinitionSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/UpdateTableDefinition" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateTableDefinitionSvc<T: MasterMaintenanceService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::UpdateTableDefinitionRequest>
                    for UpdateTableDefinitionSvc<T> {
                        type Response = super::UpdateTableDefinitionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateTableDefinitionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::update_table_definition(
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
                        let method = UpdateTableDefinitionSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/DeleteTableDefinition" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteTableDefinitionSvc<T: MasterMaintenanceService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::DeleteTableDefinitionRequest>
                    for DeleteTableDefinitionSvc<T> {
                        type Response = super::DeleteTableDefinitionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteTableDefinitionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::delete_table_definition(
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
                        let method = DeleteTableDefinitionSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/GetTableDefinition" => {
                    #[allow(non_camel_case_types)]
                    struct GetTableDefinitionSvc<T: MasterMaintenanceService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::GetTableDefinitionRequest>
                    for GetTableDefinitionSvc<T> {
                        type Response = super::GetTableDefinitionResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetTableDefinitionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::get_table_definition(
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
                        let method = GetTableDefinitionSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListTableDefinitions" => {
                    #[allow(non_camel_case_types)]
                    struct ListTableDefinitionsSvc<T: MasterMaintenanceService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::ListTableDefinitionsRequest>
                    for ListTableDefinitionsSvc<T> {
                        type Response = super::ListTableDefinitionsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListTableDefinitionsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::list_table_definitions(
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
                        let method = ListTableDefinitionsSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListColumns" => {
                    #[allow(non_camel_case_types)]
                    struct ListColumnsSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::ListColumnsRequest>
                    for ListColumnsSvc<T> {
                        type Response = super::ListColumnsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListColumnsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::list_columns(
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
                        let method = ListColumnsSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CreateColumns" => {
                    #[allow(non_camel_case_types)]
                    struct CreateColumnsSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::CreateColumnsRequest>
                    for CreateColumnsSvc<T> {
                        type Response = super::CreateColumnsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateColumnsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::create_columns(
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
                        let method = CreateColumnsSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/UpdateColumn" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateColumnSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::UpdateColumnRequest>
                    for UpdateColumnSvc<T> {
                        type Response = super::UpdateColumnResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateColumnRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::update_column(
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
                        let method = UpdateColumnSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/DeleteColumn" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteColumnSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::DeleteColumnRequest>
                    for DeleteColumnSvc<T> {
                        type Response = super::DeleteColumnResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteColumnRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::delete_column(
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
                        let method = DeleteColumnSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/GetRecord" => {
                    #[allow(non_camel_case_types)]
                    struct GetRecordSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::GetRecordRequest>
                    for GetRecordSvc<T> {
                        type Response = super::GetRecordResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetRecordRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::get_record(&inner, request)
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
                        let method = GetRecordSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListRecords" => {
                    #[allow(non_camel_case_types)]
                    struct ListRecordsSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::ListRecordsRequest>
                    for ListRecordsSvc<T> {
                        type Response = super::ListRecordsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListRecordsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::list_records(
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
                        let method = ListRecordsSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CreateRecord" => {
                    #[allow(non_camel_case_types)]
                    struct CreateRecordSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::CreateRecordRequest>
                    for CreateRecordSvc<T> {
                        type Response = super::CreateRecordResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateRecordRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::create_record(
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
                        let method = CreateRecordSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/UpdateRecord" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateRecordSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::UpdateRecordRequest>
                    for UpdateRecordSvc<T> {
                        type Response = super::UpdateRecordResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateRecordRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::update_record(
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
                        let method = UpdateRecordSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/DeleteRecord" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteRecordSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::DeleteRecordRequest>
                    for DeleteRecordSvc<T> {
                        type Response = super::DeleteRecordResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteRecordRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::delete_record(
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
                        let method = DeleteRecordSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CheckConsistency" => {
                    #[allow(non_camel_case_types)]
                    struct CheckConsistencySvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::CheckConsistencyRequest>
                    for CheckConsistencySvc<T> {
                        type Response = super::CheckConsistencyResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CheckConsistencyRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::check_consistency(
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
                        let method = CheckConsistencySvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CreateRule" => {
                    #[allow(non_camel_case_types)]
                    struct CreateRuleSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
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
                                <T as MasterMaintenanceService>::create_rule(
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/GetRule" => {
                    #[allow(non_camel_case_types)]
                    struct GetRuleSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
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
                                <T as MasterMaintenanceService>::get_rule(&inner, request)
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/UpdateRule" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateRuleSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
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
                                <T as MasterMaintenanceService>::update_rule(
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/DeleteRule" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteRuleSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
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
                                <T as MasterMaintenanceService>::delete_rule(
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListRules" => {
                    #[allow(non_camel_case_types)]
                    struct ListRulesSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
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
                                <T as MasterMaintenanceService>::list_rules(&inner, request)
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ExecuteRule" => {
                    #[allow(non_camel_case_types)]
                    struct ExecuteRuleSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::ExecuteRuleRequest>
                    for ExecuteRuleSvc<T> {
                        type Response = super::ExecuteRuleResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ExecuteRuleRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::execute_rule(
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
                        let method = ExecuteRuleSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/GetTableSchema" => {
                    #[allow(non_camel_case_types)]
                    struct GetTableSchemaSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::GetTableSchemaRequest>
                    for GetTableSchemaSvc<T> {
                        type Response = super::GetTableSchemaResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetTableSchemaRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::get_table_schema(
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
                        let method = GetTableSchemaSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListRelationships" => {
                    #[allow(non_camel_case_types)]
                    struct ListRelationshipsSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::ListRelationshipsRequest>
                    for ListRelationshipsSvc<T> {
                        type Response = super::ListRelationshipsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListRelationshipsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::list_relationships(
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
                        let method = ListRelationshipsSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CreateRelationship" => {
                    #[allow(non_camel_case_types)]
                    struct CreateRelationshipSvc<T: MasterMaintenanceService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::CreateRelationshipRequest>
                    for CreateRelationshipSvc<T> {
                        type Response = super::CreateRelationshipResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateRelationshipRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::create_relationship(
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
                        let method = CreateRelationshipSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/UpdateRelationship" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateRelationshipSvc<T: MasterMaintenanceService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::UpdateRelationshipRequest>
                    for UpdateRelationshipSvc<T> {
                        type Response = super::UpdateRelationshipResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateRelationshipRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::update_relationship(
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
                        let method = UpdateRelationshipSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/DeleteRelationship" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteRelationshipSvc<T: MasterMaintenanceService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::DeleteRelationshipRequest>
                    for DeleteRelationshipSvc<T> {
                        type Response = super::DeleteRelationshipResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteRelationshipRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::delete_relationship(
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
                        let method = DeleteRelationshipSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ImportRecords" => {
                    #[allow(non_camel_case_types)]
                    struct ImportRecordsSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::ImportRecordsRequest>
                    for ImportRecordsSvc<T> {
                        type Response = super::ImportRecordsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ImportRecordsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::import_records(
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
                        let method = ImportRecordsSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ExportRecords" => {
                    #[allow(non_camel_case_types)]
                    struct ExportRecordsSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::ExportRecordsRequest>
                    for ExportRecordsSvc<T> {
                        type Response = super::ExportRecordsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ExportRecordsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::export_records(
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
                        let method = ExportRecordsSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/GetImportJob" => {
                    #[allow(non_camel_case_types)]
                    struct GetImportJobSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::GetImportJobRequest>
                    for GetImportJobSvc<T> {
                        type Response = super::GetImportJobResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetImportJobRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::get_import_job(
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
                        let method = GetImportJobSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListDisplayConfigs" => {
                    #[allow(non_camel_case_types)]
                    struct ListDisplayConfigsSvc<T: MasterMaintenanceService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::ListDisplayConfigsRequest>
                    for ListDisplayConfigsSvc<T> {
                        type Response = super::ListDisplayConfigsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListDisplayConfigsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::list_display_configs(
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
                        let method = ListDisplayConfigsSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/GetDisplayConfig" => {
                    #[allow(non_camel_case_types)]
                    struct GetDisplayConfigSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::GetDisplayConfigRequest>
                    for GetDisplayConfigSvc<T> {
                        type Response = super::GetDisplayConfigResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetDisplayConfigRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::get_display_config(
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
                        let method = GetDisplayConfigSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/CreateDisplayConfig" => {
                    #[allow(non_camel_case_types)]
                    struct CreateDisplayConfigSvc<T: MasterMaintenanceService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::CreateDisplayConfigRequest>
                    for CreateDisplayConfigSvc<T> {
                        type Response = super::CreateDisplayConfigResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateDisplayConfigRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::create_display_config(
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
                        let method = CreateDisplayConfigSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/UpdateDisplayConfig" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateDisplayConfigSvc<T: MasterMaintenanceService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::UpdateDisplayConfigRequest>
                    for UpdateDisplayConfigSvc<T> {
                        type Response = super::UpdateDisplayConfigResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateDisplayConfigRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::update_display_config(
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
                        let method = UpdateDisplayConfigSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/DeleteDisplayConfig" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteDisplayConfigSvc<T: MasterMaintenanceService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::DeleteDisplayConfigRequest>
                    for DeleteDisplayConfigSvc<T> {
                        type Response = super::DeleteDisplayConfigResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteDisplayConfigRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::delete_display_config(
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
                        let method = DeleteDisplayConfigSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListTableAuditLogs" => {
                    #[allow(non_camel_case_types)]
                    struct ListTableAuditLogsSvc<T: MasterMaintenanceService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::ListTableAuditLogsRequest>
                    for ListTableAuditLogsSvc<T> {
                        type Response = super::ListTableAuditLogsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListTableAuditLogsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::list_table_audit_logs(
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
                        let method = ListTableAuditLogsSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListRecordAuditLogs" => {
                    #[allow(non_camel_case_types)]
                    struct ListRecordAuditLogsSvc<T: MasterMaintenanceService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::ListRecordAuditLogsRequest>
                    for ListRecordAuditLogsSvc<T> {
                        type Response = super::ListRecordAuditLogsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListRecordAuditLogsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::list_record_audit_logs(
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
                        let method = ListRecordAuditLogsSvc(inner);
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
                "/k1s0.system.mastermaintenance.v1.MasterMaintenanceService/ListDomains" => {
                    #[allow(non_camel_case_types)]
                    struct ListDomainsSvc<T: MasterMaintenanceService>(pub Arc<T>);
                    impl<
                        T: MasterMaintenanceService,
                    > tonic::server::UnaryService<super::ListDomainsRequest>
                    for ListDomainsSvc<T> {
                        type Response = super::ListDomainsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListDomainsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as MasterMaintenanceService>::list_domains(
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
                        let method = ListDomainsSvc(inner);
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
    impl<T> Clone for MasterMaintenanceServiceServer<T> {
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
    pub const SERVICE_NAME: &str = "k1s0.system.mastermaintenance.v1.MasterMaintenanceService";
    impl<T> tonic::server::NamedService for MasterMaintenanceServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
