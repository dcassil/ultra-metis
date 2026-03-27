use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let db = cadre_control_api::db::init_db("cadre-control.db")?;
    tracing::info!("Control API database initialized");

    // Keep the connection alive (will be used by the API server in later tasks).
    drop(db);

    Ok(())
}
