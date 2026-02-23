/// Storage テンプレートのレンダリング統合テスト。
use std::fs;
use std::path::Path;

use k1s0_cli::template::context::TemplateContextBuilder;
use k1s0_cli::template::TemplateEngine;
use tempfile::TempDir;

fn template_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates")
}

fn render_storage(service_name: &str, tier: &str) -> Option<(TempDir, Vec<String>)> {
    let tpl_dir = template_dir();
    let cat_dir = tpl_dir.join("storage");
    if !cat_dir.exists() {
        return None;
    }

    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let ctx = TemplateContextBuilder::new(service_name, tier, "go", "storage").build();

    let mut engine = TemplateEngine::new(&tpl_dir).unwrap();
    let generated = engine.render_to_dir(&ctx, &output_dir).unwrap();

    let names: Vec<String> = generated
        .iter()
        .map(|p| {
            p.strip_prefix(&output_dir)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();

    Some((tmp, names))
}

fn read_output(tmp: &TempDir, path: &str) -> String {
    fs::read_to_string(tmp.path().join("output").join(path)).unwrap()
}

#[test]
fn test_storage_file_list() {
    let Some((_, names)) = render_storage("order-api", "service") else {
        eprintln!("SKIP: storage テンプレートディレクトリが未作成");
        return;
    };

    assert!(
        names.iter().any(|n| n.contains("storage-class.yaml")),
        "storage-class.yaml missing. Generated: {names:?}"
    );
    assert!(
        names.iter().any(|n| n.contains("pvc.yaml")),
        "pvc.yaml missing. Generated: {names:?}"
    );
}

#[test]
fn test_storage_class_has_service_name() {
    let Some((tmp, _)) = render_storage("order-api", "service") else {
        eprintln!("SKIP: storage テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "storage-class.yaml");
    assert!(
        content.contains("order-api"),
        "StorageClass should contain service name\n--- storage-class.yaml ---\n{content}"
    );
}

#[test]
fn test_storage_class_has_tier_pool() {
    let Some((tmp, _)) = render_storage("order-api", "service") else {
        eprintln!("SKIP: storage テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "storage-class.yaml");
    assert!(
        content.contains("k1s0-service-pool"),
        "StorageClass should contain tier-based pool\n--- storage-class.yaml ---\n{content}"
    );
}

#[test]
fn test_storage_class_has_ceph_provisioner() {
    let Some((tmp, _)) = render_storage("order-api", "service") else {
        eprintln!("SKIP: storage テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "storage-class.yaml");
    assert!(
        content.contains("rbd.csi.ceph.com"),
        "StorageClass should use Ceph RBD provisioner\n--- storage-class.yaml ---\n{content}"
    );
}

#[test]
fn test_pvc_has_service_name() {
    let Some((tmp, _)) = render_storage("order-api", "service") else {
        eprintln!("SKIP: storage テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "pvc.yaml");
    assert!(
        content.contains("order-api"),
        "PVC should contain service name\n--- pvc.yaml ---\n{content}"
    );
}

#[test]
fn test_pvc_has_namespace() {
    let Some((tmp, _)) = render_storage("order-api", "service") else {
        eprintln!("SKIP: storage テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "pvc.yaml");
    assert!(
        content.contains("k1s0-service"),
        "PVC should contain namespace\n--- pvc.yaml ---\n{content}"
    );
}

#[test]
fn test_pvc_system_storage_size() {
    let Some((tmp, _)) = render_storage("auth-service", "system") else {
        eprintln!("SKIP: storage テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "pvc.yaml");
    assert!(
        content.contains("storage: 50Gi"),
        "System tier should have 50Gi storage\n--- pvc.yaml ---\n{content}"
    );
}

#[test]
fn test_pvc_business_storage_size() {
    let Some((tmp, _)) = render_storage("order-api", "business") else {
        eprintln!("SKIP: storage テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "pvc.yaml");
    assert!(
        content.contains("storage: 20Gi"),
        "Business tier should have 20Gi storage\n--- pvc.yaml ---\n{content}"
    );
}

#[test]
fn test_pvc_service_storage_size() {
    let Some((tmp, _)) = render_storage("order-api", "service") else {
        eprintln!("SKIP: storage テンプレートディレクトリが未作成");
        return;
    };

    let content = read_output(&tmp, "pvc.yaml");
    assert!(
        content.contains("storage: 10Gi"),
        "Service tier should have 10Gi storage\n--- pvc.yaml ---\n{content}"
    );
}

#[test]
fn test_storage_no_tera_syntax() {
    let Some((tmp, names)) = render_storage("order-api", "service") else {
        eprintln!("SKIP: storage テンプレートディレクトリが未作成");
        return;
    };

    for name in &names {
        let content = read_output(&tmp, name);
        assert!(!content.contains("{%"), "Tera block syntax found in {name}");
        assert!(!content.contains("{#"), "Tera comment found in {name}");
    }
}
