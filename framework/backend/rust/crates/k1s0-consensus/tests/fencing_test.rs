//! フェンシングバリデータの統合テスト。

use k1s0_consensus::lock::fencing::FencingValidator;

#[test]
fn test_sequential_tokens() {
    let validator = FencingValidator::new();

    for i in 1..=100 {
        assert!(validator.validate(i).is_ok(), "token {i} should be valid");
    }
    assert_eq!(validator.current(), 100);
}

#[test]
fn test_stale_token_rejected() {
    let validator = FencingValidator::new();
    validator.validate(10).unwrap();
    validator.validate(20).unwrap();

    // 古いトークンは拒否される
    assert!(validator.validate(15).is_err());
    assert!(validator.validate(10).is_err());
    assert!(validator.validate(1).is_err());

    // 新しいトークンは受け入れられる
    assert!(validator.validate(21).is_ok());
}

#[test]
fn test_gap_tokens() {
    let validator = FencingValidator::new();
    assert!(validator.validate(5).is_ok());
    assert!(validator.validate(100).is_ok());
    assert!(validator.validate(1000).is_ok());
    assert_eq!(validator.current(), 1000);
}

#[test]
fn test_with_initial_value() {
    let validator = FencingValidator::with_initial(50);

    assert!(validator.validate(49).is_err());
    assert!(validator.validate(50).is_err());
    assert!(validator.validate(51).is_ok());
}

#[test]
fn test_concurrent_validation() {
    use std::sync::Arc;
    use std::thread;

    let validator = Arc::new(FencingValidator::new());
    let mut handles = Vec::new();

    // 各スレッドが異なる範囲のトークンを検証
    for i in 0..10 {
        let v = Arc::clone(&validator);
        handles.push(thread::spawn(move || {
            // 競合状態があるため、一部は失敗する可能性がある
            let token = (i + 1) * 1000;
            let _ = v.validate(token);
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    // 最終的に何らかの値が設定されている
    assert!(validator.current() > 0);
}
