//! proto コード生成ビルドスクリプト
//! 共通ライブラリを使用してサービス proto とイベント proto を一括コンパイル
//!
//! クロスサービスイベント型（k1s0.event.service.order.v1.rs）は
//! src/proto/ にコミット済みのため、別途コンパイル不要。
//! order_events.proto は types.proto をインポートしないため、別途コンパイルすると
//! k1s0.system.common.v1.rs が Timestamp 等のない不完全版に上書きされる問題がある。
//! これを防ぐため、単一の compile_service_protos 呼び出しに統一する。

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "../../../../../../api/proto";

    // 共通ライブラリを使用して payment サービス proto をコンパイル
    // payment.proto が types.proto をインポートするため、k1s0.system.common.v1.rs は完全版で生成される
    k1s0_proto_build::compile_service_protos("payment", proto_root, "src/proto")?;

    Ok(())
}
