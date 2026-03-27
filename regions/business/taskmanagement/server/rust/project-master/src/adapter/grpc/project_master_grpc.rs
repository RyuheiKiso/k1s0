// ProjectMasterService gRPC 実装。
// tonic サービストレイトを実装してクライアントからの RPC 呼び出しを処理する。
use std::sync::Arc;
// tonic-build が生成するトレイト impl の追加時に使用する（現時点ではトレイト実装待ち）
#[allow(unused_imports)]
use tonic::{Request, Response, Status};

use crate::usecase::manage_project_types::ManageProjectTypesUseCase;
use crate::usecase::manage_status_definitions::ManageStatusDefinitionsUseCase;
use crate::usecase::get_status_definition_versions::GetStatusDefinitionVersionsUseCase;
use crate::usecase::manage_tenant_extensions::ManageTenantExtensionsUseCase;

pub struct ProjectMasterGrpcService {
    pub manage_project_types_uc: Arc<ManageProjectTypesUseCase>,
    pub manage_status_definitions_uc: Arc<ManageStatusDefinitionsUseCase>,
    pub get_versions_uc: Arc<GetStatusDefinitionVersionsUseCase>,
    pub manage_tenant_extensions_uc: Arc<ManageTenantExtensionsUseCase>,
}

impl ProjectMasterGrpcService {
    pub fn new(
        manage_project_types_uc: Arc<ManageProjectTypesUseCase>,
        manage_status_definitions_uc: Arc<ManageStatusDefinitionsUseCase>,
        get_versions_uc: Arc<GetStatusDefinitionVersionsUseCase>,
        manage_tenant_extensions_uc: Arc<ManageTenantExtensionsUseCase>,
    ) -> Self {
        Self {
            manage_project_types_uc,
            manage_status_definitions_uc,
            get_versions_uc,
            manage_tenant_extensions_uc,
        }
    }
}
// 注: tonic-build が生成する ProjectMasterServiceServer トレイトの impl は
// build.rs のコード生成後にここに追加する。
