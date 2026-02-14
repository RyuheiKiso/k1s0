use k1s0::application::e2e_runner;

#[test]
fn all_system_region_scenarios_pass() {
    let results = e2e_runner::run_all();
    let filtered: Vec<_> = results
        .iter()
        .filter(|r| r.name.starts_with("System"))
        .collect();
    assert_eq!(filtered.len(), 4);
    for result in &filtered {
        assert!(
            result.passed,
            "FAILED: {} - {:?}",
            result.name, result.detail
        );
    }
}

#[test]
fn all_business_region_scenarios_pass() {
    let results = e2e_runner::run_all();
    let filtered: Vec<_> = results
        .iter()
        .filter(|r| r.name.starts_with("Business"))
        .collect();
    assert_eq!(filtered.len(), 12);
    for result in &filtered {
        assert!(
            result.passed,
            "FAILED: {} - {:?}",
            result.name, result.detail
        );
    }
}

#[test]
fn all_service_region_scenarios_pass() {
    let results = e2e_runner::run_all();
    let filtered: Vec<_> = results
        .iter()
        .filter(|r| r.name.starts_with("Service"))
        .collect();
    assert_eq!(filtered.len(), 4);
    for result in &filtered {
        assert!(
            result.passed,
            "FAILED: {} - {:?}",
            result.name, result.detail
        );
    }
}

#[test]
fn all_error_scenarios_pass() {
    let results = e2e_runner::run_all();
    let filtered: Vec<_> = results
        .iter()
        .filter(|r| {
            r.name.contains("未設定でプロジェクト")
                || r.name.contains("不正")
                || r.name.contains("失敗")
                || r.name.contains("空で")
        })
        .collect();
    assert_eq!(filtered.len(), 4);
    for result in &filtered {
        assert!(
            result.passed,
            "FAILED: {} - {:?}",
            result.name, result.detail
        );
    }
}
