//! proto コード生成ビルドスクリプト
//! 共通ライブラリを使用してサービス proto とイベント proto を一括コンパイル
//!
//! クロスサービスイベント型（k1s0.event.service.payment.v1.rs）は
//! src/proto/ にコミット済みのため、別途コンパイル不要。
//! protoc が利用可能な場合でも単一の compile_service_protos 呼び出しに統一し、
//! k1s0.system.common.v1.rs の上書き競合を防止する。

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "../../../../../../api/proto";

    // 共通ライブラリを使用して order サービス proto をコンパイル
    // order.proto が types.proto をインポートするため、k1s0.system.common.v1.rs は完全版で生成される
    k1s0_proto_build::compile_service_protos("order", proto_root, "src/proto")?;

    Ok(())
}
