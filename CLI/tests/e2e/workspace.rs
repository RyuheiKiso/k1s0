use k1s0::application::e2e_runner;

#[test]
fn workspace_roundtrip_scenario_passes() {
    let results = e2e_runner::run_all();
    let filtered: Vec<_> = results
        .iter()
        .filter(|r| r.name.contains("ラウンドトリップ"))
        .collect();
    assert_eq!(filtered.len(), 1);
    assert!(
        filtered[0].passed,
        "FAILED: {} - {:?}",
        filtered[0].name, filtered[0].detail
    );
}

#[test]
fn workspace_not_configured_scenario_passes() {
    let results = e2e_runner::run_all();
    let filtered: Vec<_> = results
        .iter()
        .filter(|r| r.name.contains("未設定時"))
        .collect();
    assert_eq!(filtered.len(), 1);
    assert!(
        filtered[0].passed,
        "FAILED: {} - {:?}",
        filtered[0].name, filtered[0].detail
    );
}
