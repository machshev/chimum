mod config;
mod controller;
mod house;
mod schedule;
mod server;

use std::error::Error;

use crate::server::Server;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let mut server = Server::new();

    server.start().await?;

    Ok(())
}
