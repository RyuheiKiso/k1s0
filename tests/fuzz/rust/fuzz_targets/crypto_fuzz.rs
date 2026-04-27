// k1s0 crypto primitive（AES-GCM / SHA-256 / HMAC）の fuzz target 雛形。
// 採用初期 で seal / unseal の round-trip 不変条件を検証する。
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // TODO(release-initial): seal(data) → unseal(seal(data)) == data の検証を実装
    let _ = data;
});
