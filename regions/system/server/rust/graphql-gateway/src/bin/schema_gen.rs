// ARCH-CRIT-001: GraphQL スキーマ自動生成バイナリ。
// 実装コードから SDL を生成し stdout に出力する。
// reqwest クライアント（ServiceCatalogHttpClient）が Tokio ランタイムを要求するため
// #[tokio::main] でランタイムを提供する。
// CI での差分検出に使用する:
//   cargo run --bin schema-gen > /tmp/generated.graphql
//   diff api/graphql/schema.graphql /tmp/generated.graphql
// 差分があれば schema.graphql が実装と乖離していることを示す。
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sdl = k1s0_graphql_gateway_server::adapter::graphql_handler::build_sdl()?;
    print!("{}", sdl);
    Ok(())
}
