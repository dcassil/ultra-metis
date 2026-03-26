use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    cadre_mcp_server::run().await
}
