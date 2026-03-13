pub mod config_query;
pub mod feature_flag_query;
pub mod navigation_query;
pub mod service_catalog_mutation;
pub mod service_catalog_query;
pub mod subscription;
pub mod tenant_mutation;
pub mod tenant_query;

pub use config_query::ConfigQueryResolver;
pub use feature_flag_query::FeatureFlagQueryResolver;
pub use navigation_query::NavigationQueryResolver;
pub use service_catalog_mutation::ServiceCatalogMutationResolver;
pub use service_catalog_query::ServiceCatalogQueryResolver;
pub use subscription::SubscriptionResolver;
pub use tenant_mutation::TenantMutationResolver;
pub use tenant_query::TenantQueryResolver;
