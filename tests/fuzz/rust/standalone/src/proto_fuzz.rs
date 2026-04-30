// 本ファイルは k1s0 内部 Protobuf decoder の standalone fuzz harness。
//
// docs 正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/03_内部インタフェース方式設計/01_内部gRPC契約方式.md
//   docs/03_要件定義/30_非機能要件/H_完全性.md（NFR-H-INT-*）
//
// 役割:
//   fuzz_targets/proto_fuzz.rs と等価。libfuzzer-sys を使えない環境向けに、
//   ChaCha8 PRNG + 既知 corpus + boundary edge case で大量 case を回し、
//   prost::Message::decode が panic / abort しないことを検証する。
//   panic はプロセス終了で検出し、exit code が 0 以外なら CI が fail する。
//
// 使い方:
//   cargo run --release --bin proto_fuzz_standalone -- <iterations> <seed>
//   例: cargo run --release --bin proto_fuzz_standalone -- 200000 42

use prost::Message;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::env;
use std::time::Instant;

// 4 message 型は fuzz_targets/proto_fuzz.rs と同じ。
use k1s0_tier1_proto_gen::k1s0::internal::audit::v1::{AppendHashRequest, VerifyChainRequest};
use k1s0_tier1_proto_gen::k1s0::internal::decision::v1::EvaluateDecisionRequest;
use k1s0_tier1_proto_gen::k1s0::internal::pii::v1::MaskPiiRequest;

// 1 case を 4 message 型すべてで decode する。panic は std::process::abort で起き、
// 通常 unwind で捕捉できないため、本関数自体に異常終了を観測する仕組みは置かない
// （runtime が abort で die する）。
fn try_decode(data: &[u8]) {
    let _ = AppendHashRequest::decode(data);
    let _ = VerifyChainRequest::decode(data);
    let _ = EvaluateDecisionRequest::decode(data);
    let _ = MaskPiiRequest::decode(data);
}

fn main() {
    // CLI: <iterations> <seed>。
    let args: Vec<String> = env::args().collect();
    let iters: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(100_000);
    let seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

    // boundary edge cases（空 / 1 byte / proto invalid varint 等）。
    let edge: Vec<Vec<u8>> = vec![
        vec![],
        vec![0u8],
        vec![0xff; 1],
        vec![0xff; 1024],
        // varint 過剰長（10 byte 超は decoder error）。
        vec![0x80; 11],
        // tag=1 wire_type=2 (length-delimited) length=u32::MAX 相当
        vec![0x0a, 0xff, 0xff, 0xff, 0xff, 0x0f],
        // valid empty message。
        vec![0x08, 0x00],
    ];
    for case in &edge {
        try_decode(case);
    }

    // PRNG ベース random fuzzing。
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut buf = Vec::with_capacity(8192);
    let start = Instant::now();
    for i in 0..iters {
        buf.clear();
        // 0..4096 byte の任意長 input を生成。
        let len = rng.gen_range(0..4096);
        buf.resize(len, 0);
        rng.fill(&mut buf[..]);
        try_decode(&buf);
        if i % 10_000 == 0 {
            let elapsed = start.elapsed().as_secs_f64();
            eprintln!("[proto_fuzz] iter={} elapsed={:.1}s", i, elapsed);
        }
    }
    eprintln!(
        "[proto_fuzz] PASS: {} iters, edge {} cases, no panic",
        iters,
        edge.len()
    );
}
