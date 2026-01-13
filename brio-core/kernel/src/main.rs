use brio_kernel::host;

use anyhow::Result;
use host::BrioHostState;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting Brio Kernel...");

    // In a real scenario, this URL would come from config
    let db_url = "sqlite::memory:";

    // Initialize Host State
    let _state = BrioHostState::new(db_url).await.unwrap_or_else(|e| {
        // For now, if DB fails (e.g. no sqlite lib installed), just panic or handle gracefully
        // But since we use sqlite::memory:, it should work if sqlx is happy.
        // If we want to be robust we might need to create the file if it's file-based.
        panic!("Failed to initialize host state: {}", e);
    });

    info!("Brio Kernel initialized successfully.");

    // Keep alive
    // tokio::signal::ctrl_c().await?;

    Ok(())
}
