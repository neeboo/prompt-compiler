use anyhow::Result;
use dotenvy::dotenv;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use prompt_compiler_node::run_server;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "prompt_compiler_node=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Get port from environment or use default
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a valid number");

    println!("🧠 Starting Prompt Compiler Node...");
    println!("📋 Make sure you have set the following environment variables:");
    println!("   - OPENAI_API_KEY (required)");
    println!("   - PORT (optional, default: 3000)");
    println!("   - OPENAI_BASE_URL (optional, default: https://api.openai.com/v1)");
    println!();

    // Verify required environment variables
    if std::env::var("OPENAI_API_KEY").is_err() && std::env::var("LLM_API_KEY").is_err() {
        eprintln!("❌ Error: OPENAI_API_KEY or LLM_API_KEY environment variable is required");
        std::process::exit(1);
    }

    // Start the server
    if let Err(e) = run_server(port).await {
        eprintln!("❌ Server error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
