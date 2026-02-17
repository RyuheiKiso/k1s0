mod commands;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::execute_init,
            commands::execute_generate,
            commands::execute_build,
            commands::execute_test,
            commands::execute_deploy,
            commands::scan_placements,
            commands::scan_buildable_targets,
            commands::scan_deployable_targets,
            commands::validate_name,
            commands::execute_test_with_progress,
            commands::execute_build_with_progress,
            commands::execute_deploy_with_progress,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
