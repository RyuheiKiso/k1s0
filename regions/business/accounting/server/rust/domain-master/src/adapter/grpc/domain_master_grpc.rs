use crate::usecase;
use std::sync::Arc;

pub struct DomainMasterGrpcService {
    pub manage_categories_uc: Arc<usecase::manage_categories::ManageCategoriesUseCase>,
    pub manage_items_uc: Arc<usecase::manage_items::ManageItemsUseCase>,
    pub get_item_versions_uc: Arc<usecase::get_item_versions::GetItemVersionsUseCase>,
    pub manage_tenant_extensions_uc:
        Arc<usecase::manage_tenant_extensions::ManageTenantExtensionsUseCase>,
}

impl DomainMasterGrpcService {
    pub fn new(
        manage_categories_uc: Arc<usecase::manage_categories::ManageCategoriesUseCase>,
        manage_items_uc: Arc<usecase::manage_items::ManageItemsUseCase>,
        get_item_versions_uc: Arc<usecase::get_item_versions::GetItemVersionsUseCase>,
        manage_tenant_extensions_uc: Arc<
            usecase::manage_tenant_extensions::ManageTenantExtensionsUseCase,
        >,
    ) -> Self {
        Self {
            manage_categories_uc,
            manage_items_uc,
            get_item_versions_uc,
            manage_tenant_extensions_uc,
        }
    }
}
