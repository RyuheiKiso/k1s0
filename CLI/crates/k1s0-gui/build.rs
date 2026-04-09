fn main() {
    // ビルドスクリプトでは CARGO_MANIFEST_DIR が常に設定されているため expect を使用する（L-1）
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR はビルド時に Cargo が設定する");
    let ui_dir = std::path::Path::new(&manifest_dir).join("ui");
    let ui_dist = ui_dir.join("dist");

    // Always re-run this build script so we can check if ui/dist exists
    println!("cargo:rerun-if-changed=ui/src");
    println!("cargo:rerun-if-changed=ui/index.html");
    println!("cargo:rerun-if-changed=ui/package.json");

    if !ui_dist.exists() {
        // LOW-004 監査対応: pnpm-lock.yaml と一致するよう npm → pnpm に統一する
        println!("cargo:warning=Frontend not built. Running 'pnpm run build' in ui/...");
        let pnpm = if cfg!(windows) { "pnpm.cmd" } else { "pnpm" };
        let status = std::process::Command::new(pnpm)
            .args(["run", "build"])
            .current_dir(&ui_dir)
            .status()
            .expect("Failed to run 'pnpm run build'. Is pnpm installed?");
        assert!(status.success(), "Frontend build failed");
    }

    tauri_build::build();
}
