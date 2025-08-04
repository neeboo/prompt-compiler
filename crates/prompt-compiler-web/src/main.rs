use anyhow::Result;
use prompt_compiler_web::create_app;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let app = create_app().await;
    
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("🌐 Prompt Compiler Web Server running on http://localhost:3000");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
