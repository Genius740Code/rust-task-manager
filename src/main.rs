use anyhow::Result;
use clap::Parser;
use std::time::Duration;

mod app;
mod system;
mod ui;

use app::App;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Update interval in milliseconds
    #[arg(short, long, default_value = "1000")]
    interval: u64,
    
    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let mut app = App::new(Duration::from_millis(cli.interval), cli.debug)?;
    app.run().await?;
    
    Ok(())
}