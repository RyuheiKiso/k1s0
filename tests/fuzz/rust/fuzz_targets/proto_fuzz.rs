// 本ファイルは k1s0 内部 Protobuf decoder の fuzz target。
//
// docs 正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/03_内部インタフェース方式設計/01_内部gRPC契約方式.md
//   docs/03_要件定義/30_非機能要件/H_完全性.md（NFR-H-INT-*）
//
// 目的:
//   prost::Message::decode が panic / OOM / 無限ループしないことを保証する。
//   tier1 内部 gRPC は信頼境界外（同 cluster の他 Pod）から bytes を受け取る経路があるため、
//   decoder の crash / DoS は SEV1 に直結する。代表 message 4 種を順番に decode して
//   いずれの長さ・値の組合せでも panic しないことを確認する。

#![no_main]

use libfuzzer_sys::fuzz_target;
use prost::Message;

// 代表 internal proto を import（lib.rs の module 階層を経由）。
use k1s0_tier1_proto_gen::k1s0::internal::audit::v1::{AppendHashRequest, VerifyChainRequest};
use k1s0_tier1_proto_gen::k1s0::internal::decision::v1::EvaluateDecisionRequest;
use k1s0_tier1_proto_gen::k1s0::internal::pii::v1::MaskPiiRequest;

// fuzz_target! は libfuzzer 入口。`cargo fuzz run proto_fuzz -- -max_total_time=300` で起動。
fuzz_target!(|data: &[u8]| {
    // 4 message 型を順次 decode し、Result を捨てる（成功 / 失敗いずれでも panic しないこと）。
    let _ = AppendHashRequest::decode(data);
    let _ = VerifyChainRequest::decode(data);
    let _ = EvaluateDecisionRequest::decode(data);
    let _ = MaskPiiRequest::decode(data);
});
