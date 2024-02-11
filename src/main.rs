
use axum::Router;
use chatiplex::chatiplex;
use tracing::debug;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init()
        .ok();

    let app = chatiplex("");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await?;

    Ok(())    
}
