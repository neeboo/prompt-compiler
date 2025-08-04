use anyhow::Result;
use clap::Parser;
use prompt_compiler_cli::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.execute().await?;
    
    Ok(())
}
