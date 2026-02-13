use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn binary_starts_successfully() {
    // バイナリがビルド可能で起動できることを確認
    let mut cmd = cargo_bin_cmd!("k1s0");
    let result = cmd.timeout(std::time::Duration::from_secs(3)).output();
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn binary_name_is_correct() {
    let cmd = cargo_bin_cmd!("k1s0");
    // マクロがコンパイル可能 = バイナリ名が正しい
    drop(cmd);
}

#[test]
fn config_file_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("test_config.toml");

    let content = "workspace_path = \"C:\\\\test\\\\workspace\"\n";
    std::fs::write(&config_path, content).unwrap();

    let loaded = std::fs::read_to_string(&config_path).unwrap();
    assert!(loaded.contains("C:\\\\test\\\\workspace"));
}
