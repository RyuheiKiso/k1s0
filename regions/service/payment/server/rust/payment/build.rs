fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_file = "../../../../../../api/proto/k1s0/service/payment/v1/payment.proto";
    let proto_include = "../../../../../../api/proto";

    if !std::path::Path::new(proto_file).exists() {
        println!(
            "cargo:warning=Proto file not found, skipping tonic codegen: {}",
            proto_file
        );
        return Ok(());
    }

    // サービス定義用 proto のコンパイル
    match tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(&[proto_file], &[proto_include])
    {
        Ok(()) => {
            println!("cargo:warning=tonic-build succeeded for payment proto");
        }
        Err(e) => {
            println!(
                "cargo:warning=tonic-build failed (protoc may not be installed): {}",
                e
            );
        }
    }

    // イベント用 proto のコンパイル（Kafka publish で使用する Protobuf メッセージ）
    let event_proto_file =
        "../../../../../../api/proto/k1s0/event/service/payment/v1/payment_events.proto";
    let event_proto_include = "../../../../../../api/proto";

    if !std::path::Path::new(event_proto_file).exists() {
        println!(
            "cargo:warning=Event proto file not found, skipping tonic codegen: {}",
            event_proto_file
        );
        return Ok(());
    }

    match tonic_build::configure()
        .build_server(false)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(&[event_proto_file], &[event_proto_include])
    {
        Ok(()) => {
            println!("cargo:warning=tonic-build succeeded for payment event proto");
        }
        Err(e) => {
            println!(
                "cargo:warning=tonic-build failed for event proto (protoc may not be installed): {}",
                e
            );
        }
    }

    Ok(())
}
