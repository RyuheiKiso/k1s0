DELETE FROM app_registry.app_versions WHERE app_id IN ('order-client', 'inventory-client', 'admin-tool');
DELETE FROM app_registry.apps WHERE id IN ('order-client', 'inventory-client', 'admin-tool');
