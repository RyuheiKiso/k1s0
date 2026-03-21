//! proto コード生成ビルドスクリプト
//! 共通ライブラリを使用してサービス proto とイベント proto を一括コンパイル

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "../../../../../../api/proto";

    // 共通ライブラリを使用して order サービス proto をコンパイル
    k1s0_proto_build::compile_service_protos("order", proto_root, "src/proto")?;

    // Saga 補償: Order Consumer が payment イベントを受信するために payment_events.proto を追加コンパイル（C-001）
    let payment_events_proto = format!(
        "{}/k1s0/event/service/payment/v1/payment_events.proto",
        proto_root
    );
    if std::path::Path::new(&payment_events_proto).exists() {
        match tonic_build::configure()
            .build_server(false)
            .build_client(false)
            .out_dir("src/proto")
            .compile_protos(&[&payment_events_proto], &[proto_root])
        {
            Ok(()) => println!("cargo:warning=payment_events.proto compiled for order consumer"),
            Err(e) => println!(
                "cargo:warning=payment_events.proto compile failed (protoc may not be installed): {}",
                e
            ),
        }
    }

    Ok(())
}
