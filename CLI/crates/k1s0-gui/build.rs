fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let ui_dir = std::path::Path::new(&manifest_dir).join("ui");
    let ui_dist = ui_dir.join("dist");

    // Always re-run this build script so we can check if ui/dist exists
    println!("cargo:rerun-if-changed=ui/src");
    println!("cargo:rerun-if-changed=ui/index.html");
    println!("cargo:rerun-if-changed=ui/package.json");

    if !ui_dist.exists() {
        println!("cargo:warning=Frontend not built. Running 'npm run build' in ui/...");
        let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
        let status = std::process::Command::new(npm)
            .args(["run", "build"])
            .current_dir(&ui_dir)
            .status()
            .expect("Failed to run 'npm run build'. Is Node.js installed?");
        assert!(status.success(), "Frontend build failed");
    }

    tauri_build::build()
}
