fn main() -> Result<(), Box<dyn std::error::Error>> {
    // サービス定義用 proto ファイルパス
    let proto_file = "../../../../../../api/proto/k1s0/service/order/v1/order.proto";
    // イベント定義用 proto ファイルパス
    let event_proto_file =
        "../../../../../../api/proto/k1s0/event/service/order/v1/order_events.proto";
    // proto ルートディレクトリ（import 解決用）
    let proto_include = "../../../../../../api/proto";

    // サービス proto のコンパイル
    if !std::path::Path::new(proto_file).exists() {
        println!(
            "cargo:warning=Proto file not found, skipping tonic codegen: {}",
            proto_file
        );
        return Ok(());
    }

    match tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(&[proto_file], &[proto_include])
    {
        Ok(()) => {
            println!("cargo:warning=tonic-build succeeded for order proto");
        }
        Err(e) => {
            println!(
                "cargo:warning=tonic-build failed (protoc may not be installed): {}",
                e
            );
        }
    }

    // イベント proto のコンパイル（Kafka publish 用 Protobuf メッセージ）
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
        .compile_protos(&[event_proto_file], &[proto_include])
    {
        Ok(()) => {
            println!("cargo:warning=tonic-build succeeded for order event proto");
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
