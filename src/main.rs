mod config;
mod controller;
mod house;
mod schedule;
mod server;

use std::error::Error;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    server::start_server().await?;

    Ok(())
}
