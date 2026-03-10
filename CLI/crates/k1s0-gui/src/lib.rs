mod auth_session;
mod commands;

const DEV_SERVER_ADDR: &str = "127.0.0.1:1420";
const DEV_SERVER_IPV6_ADDR: &str = "[::1]:1420";

/// In dev mode, if the Vite dev server (port 1420) is not running,
/// start a minimal static file server from `ui/dist/` so the exe works standalone.
#[cfg(dev)]
fn ensure_dev_server() {
    use std::time::Duration;

    if is_dev_server_available(Duration::from_millis(300)) {
        return; // Vite dev server is already running
    }

    let dist_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/dist");
    if !dist_dir.exists() {
        eprintln!("Warning: ui/dist not found. Run 'npm run build' in the ui/ directory.");
        return;
    }

    std::thread::spawn(move || {
        let listener = match std::net::TcpListener::bind(DEV_SERVER_ADDR) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to bind fallback server on {DEV_SERVER_ADDR}: {e}");
                return;
            }
        };
        for stream in listener.incoming().flatten() {
            serve_request(stream, &dist_dir);
        }
    });

    // Wait for the server to be ready
    for _ in 0..20 {
        std::thread::sleep(Duration::from_millis(50));
        if is_dev_server_available(Duration::from_millis(100)) {
            return;
        }
    }
}

#[cfg(dev)]
fn is_dev_server_available(timeout: std::time::Duration) -> bool {
    use std::net::{SocketAddr, TcpStream};

    [DEV_SERVER_ADDR, DEV_SERVER_IPV6_ADDR]
        .into_iter()
        .filter_map(|address| address.parse::<SocketAddr>().ok())
        .any(|address| TcpStream::connect_timeout(&address, timeout).is_ok())
}

#[cfg(dev)]
fn serve_request(mut stream: std::net::TcpStream, dist_dir: &std::path::Path) {
    use std::io::{Read, Write};

    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf).unwrap_or(0);
    let request = String::from_utf8_lossy(&buf[..n]);
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");

    // Strip query string
    let path = path.split('?').next().unwrap_or("/");

    let file_path = if path == "/" {
        dist_dir.join("index.html")
    } else {
        let candidate = dist_dir.join(path.trim_start_matches('/'));
        if candidate.is_file() {
            candidate
        } else {
            // SPA fallback: serve index.html for unknown routes
            dist_dir.join("index.html")
        }
    };

    if let Ok(contents) = std::fs::read(&file_path) {
        let mime = match file_path.extension().and_then(|e| e.to_str()) {
            Some("html") => "text/html; charset=utf-8",
            Some("js") => "application/javascript",
            Some("css") => "text/css",
            Some("svg") => "image/svg+xml",
            Some("json") => "application/json",
            Some("wasm") => "application/wasm",
            Some("png") => "image/png",
            Some("ico") => "image/x-icon",
            _ => "application/octet-stream",
        };
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            contents.len()
        );
        let _ = stream.write_all(header.as_bytes());
        let _ = stream.write_all(&contents);
    } else {
        let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n");
    }
}

/// Tauri アプリケーションを起動する。
///
/// # Panics
///
/// Tauri アプリケーションの初期化または実行に失敗した場合にパニックする。
pub fn run() {
    #[cfg(dev)]
    ensure_dev_server();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::execute_init,
            commands::execute_generate,
            commands::execute_generate_at,
            commands::execute_build,
            commands::execute_test,
            commands::execute_test_at,
            commands::execute_deploy,
            commands::execute_deploy_rollback,
            commands::get_failed_prod_rollback_target,
            commands::scan_placements,
            commands::scan_buildable_targets,
            commands::scan_deployable_targets,
            commands::scan_testable_targets,
            commands::validate_name,
            commands::execute_test_with_progress,
            commands::execute_test_with_progress_at,
            commands::execute_build_with_progress,
            commands::execute_deploy_with_progress,
            commands::detect_workspace_root,
            commands::resolve_workspace_root,
            commands::start_device_authorization,
            commands::poll_device_authorization,
            commands::get_auth_session,
            commands::clear_auth_session,
            commands::execute_validate_config_schema,
            commands::execute_validate_navigation,
            commands::execute_generate_config_types,
            commands::execute_generate_navigation_types,
            commands::write_config_types,
            commands::write_navigation_types,
            // deps
            commands::execute_deps,
            commands::scan_services,
            // dev
            commands::execute_dev_up,
            commands::execute_dev_down,
            commands::execute_dev_status,
            commands::execute_dev_logs,
            commands::scan_dev_targets,
            // migrate
            commands::execute_migrate_create,
            commands::execute_migrate_up,
            commands::execute_migrate_down,
            commands::execute_migrate_status,
            commands::execute_migrate_repair,
            commands::scan_migrate_targets,
            // scaffold databases
            commands::scan_databases,
            // event codegen
            commands::preview_event_codegen,
            commands::execute_event_codegen,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
