// 本ファイルは k1s0 ハッシュチェーンの standalone fuzz harness。
//
// docs 正典:
//   docs/03_要件定義/30_非機能要件/H_完全性.md（NFR-H-INT-001 / 002）
//
// 役割:
//   fuzz_targets/crypto_fuzz.rs と等価。libfuzzer-sys 不在の環境向け代替で、
//   ChaCha8 PRNG で 3 連鎖の chain_step を回す。panic / OOM はプロセス
//   終了で検出（exit != 0 で CI fail）。
//
// 使い方:
//   cargo run --release --bin crypto_fuzz_standalone -- <iterations> <seed>

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use sha2::{Digest, Sha256};
use std::env;
use std::time::Instant;

// fuzz_targets/crypto_fuzz.rs と同一の chain_step。
fn chain_step(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    if data.len() >= 32 {
        hasher.update(&data[..32]);
        hasher.update(&data[32..]);
    } else {
        // GENESIS 慣用。
        hasher.update(b"GENESIS");
        hasher.update(data);
    }
    hasher.finalize().into()
}

// 1 fuzz iteration。fuzz_targets と同じく 3 連鎖まで回す。
fn one_iter(data: &[u8]) {
    let h1 = chain_step(data);
    let mut buf = Vec::with_capacity(32 + data.len());
    buf.extend_from_slice(&h1);
    buf.extend_from_slice(data);
    let h2 = chain_step(&buf);
    let mut buf2 = Vec::with_capacity(32 + data.len());
    buf2.extend_from_slice(&h2);
    buf2.extend_from_slice(data);
    let _h3 = chain_step(&buf2);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let iters: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(200_000);
    let seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

    // boundary edge cases。
    let edge: Vec<Vec<u8>> = vec![
        vec![],
        vec![0u8; 31], // GENESIS 経路（< 32）
        vec![0xff; 32],
        vec![0u8; 1024 * 8], // 大 payload
    ];
    for case in &edge {
        one_iter(case);
    }

    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut buf = Vec::with_capacity(4096);
    let start = Instant::now();
    for i in 0..iters {
        buf.clear();
        let len = rng.gen_range(0..2048);
        buf.resize(len, 0);
        rng.fill(&mut buf[..]);
        one_iter(&buf);
        if i % 20_000 == 0 {
            eprintln!(
                "[crypto_fuzz] iter={} elapsed={:.1}s",
                i,
                start.elapsed().as_secs_f64()
            );
        }
    }
    eprintln!(
        "[crypto_fuzz] PASS: {} iters, edge {} cases, no panic",
        iters,
        edge.len()
    );
}
