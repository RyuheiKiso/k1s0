fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_monitor_proto =
        "../../../../../api/proto/k1s0/system/eventmonitor/v1/event_monitor.proto";
    // DLQ Manager の gRPC クライアントコードを生成するための proto ファイルパス
    let dlq_proto = "../../../../../api/proto/k1s0/system/dlq/v1/dlq.proto";
    let proto_include = "../../../../../api/proto";

    if !std::path::Path::new(event_monitor_proto).exists() {
        println!(
            "cargo:warning=Proto file not found, skipping tonic codegen: {}",
            event_monitor_proto
        );
        return Ok(());
    }

    // event_monitor サービス: サーバー側コードを生成する（このサービス自身がサーバー）
    match tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .out_dir("src/proto")
        .compile_protos(&[event_monitor_proto], &[proto_include])
    {
        Ok(()) => {
            println!("cargo:warning=tonic-build succeeded for event_monitor proto");
        }
        Err(e) => {
            println!(
                "cargo:warning=tonic-build failed (protoc may not be installed): {}",
                e
            );
        }
    }

    // DLQ Manager サービス: クライアント側コードを生成する（このサービスが gRPC クライアントとして呼び出す）
    if std::path::Path::new(dlq_proto).exists() {
        match tonic_build::configure()
            .build_server(false)
            .build_client(true)
            .out_dir("src/proto")
            .compile_protos(&[dlq_proto], &[proto_include])
        {
            Ok(()) => {
                println!("cargo:warning=tonic-build succeeded for dlq proto (client)");
            }
            Err(e) => {
                println!("cargo:warning=tonic-build failed for dlq proto: {}", e);
            }
        }
    } else {
        println!(
            "cargo:warning=DLQ proto file not found, skipping dlq client codegen: {}",
            dlq_proto
        );
    }

    Ok(())
}
