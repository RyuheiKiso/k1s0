//! proto コード生成ビルドスクリプト
//! 共通ライブラリを使用してサービス proto とイベント proto を一括コンパイル

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 共通ライブラリを使用して proto をコンパイル
    k1s0_proto_build::compile_service_protos(
        "order",
        "../../../../../../api/proto",
        "src/proto",
    )
}
