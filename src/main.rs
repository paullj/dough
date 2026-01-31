mod cli;
mod config;
mod db;
mod id;
mod money;

use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    dotenv::dotenv().ok();

    color_eyre::install()?;
    cli::run().await
}
