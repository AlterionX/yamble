mod cmd;
mod audio;

mod schema;
mod db;

use tracing as trc;
use serenity::async_trait;
use songbird::SerenityInit;

#[tokio::main]
async fn main() {
    let cfg = azel::setup_default_log_and_load_configuration().unwrap();

    let mut discord = azel::build_client(
        cfg,
        cmd::generate_command_descriptions(),
        |b| b.register_songbird()
    ).await.expect("client to be built");


    trc::info!("BOOT-CMPL");

    discord.0.start().await.expect("no error");
}
