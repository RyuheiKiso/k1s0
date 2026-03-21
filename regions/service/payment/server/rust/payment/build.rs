//! proto コード生成ビルドスクリプト
//! 共通ライブラリを使用してサービス proto とイベント proto を一括コンパイル

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "../../../../../../api/proto";

    // 共通ライブラリを使用して payment サービス proto をコンパイル
    k1s0_proto_build::compile_service_protos("payment", proto_root, "src/proto")?;

    // Saga: Payment Consumer が order.created イベントを受信するために order_events.proto を追加コンパイル（C-001）
    let order_events_proto = format!(
        "{}/k1s0/event/service/order/v1/order_events.proto",
        proto_root
    );
    if std::path::Path::new(&order_events_proto).exists() {
        match tonic_build::configure()
            .build_server(false)
            .build_client(false)
            .out_dir("src/proto")
            .compile_protos(&[&order_events_proto], &[proto_root])
        {
            Ok(()) => println!("cargo:warning=order_events.proto compiled for payment consumer"),
            Err(e) => println!(
                "cargo:warning=order_events.proto compile failed (protoc may not be installed): {}",
                e
            ),
        }
    }

    Ok(())
}
