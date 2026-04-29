// 本ファイルは k1s0 ハッシュチェーン構築の fuzz target。
//
// docs 正典:
//   docs/03_要件定義/30_非機能要件/H_完全性.md（NFR-H-INT-001 / 002）
//
// 目的:
//   Audit WORM ストアが採用する SHA-256 ハッシュチェーン形式
//   `next_hash = SHA256(prev_hash || canonical_bytes)` を任意入力で繰り返し回し、
//   panic / OOM / 異常終了が起きないことを保証する。任意 fuzz 入力を 32-byte 区切りで
//   prev_hash + payload に分けて 1 step を計算する単純なループ。

#![no_main]

use libfuzzer_sys::fuzz_target;
use sha2::{Digest, Sha256};

// 1 chain step を計算する純関数（impl と等価ロジック、依存最小化のため自前 inline）。
// data の先頭 32 bytes を prev_hash 扱い、残りを canonical body 扱いとして hash する。
fn chain_step(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    if data.len() >= 32 {
        hasher.update(&data[..32]);
        hasher.update(&data[32..]);
    } else {
        // 32 bytes 未満は GENESIS 扱い（"GENESIS" + data）。実装と同じ慣用。
        hasher.update(b"GENESIS");
        hasher.update(data);
    }
    hasher.finalize().into()
}

fuzz_target!(|data: &[u8]| {
    // 1 step だけでなく 3 連鎖まで回して途中の panic も検出する。
    let h1 = chain_step(data);
    let mut buf = Vec::with_capacity(32 + data.len());
    buf.extend_from_slice(&h1);
    buf.extend_from_slice(data);
    let h2 = chain_step(&buf);
    let mut buf2 = Vec::with_capacity(32 + data.len());
    buf2.extend_from_slice(&h2);
    buf2.extend_from_slice(data);
    let _h3 = chain_step(&buf2);
});
