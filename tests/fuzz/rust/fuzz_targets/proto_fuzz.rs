// k1s0 internal Protobuf decode の fuzz target 雛形（cargo-fuzz）。
// 採用初期 で k1s0_tier1_proto_gen の AppendHashRequest / EvaluateDecisionRequest /
// MaskPiiRequest 等を decode する fuzz_target!() を実装する。
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // TODO(release-initial): k1s0::internal::audit::v1::AppendHashRequest::decode(data) を呼ぶ
    let _ = data;
});
