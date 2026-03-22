fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "../../../../../../api/proto";
    k1s0_proto_build::compile_service_protos("board", proto_root, "src/proto")?;
    Ok(())
}
