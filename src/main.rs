mod cmd;
mod audio;

mod schema;
mod db;

use std::num::NonZeroUsize;

use tracing as trc;
use serenity::async_trait;
use songbird::{driver::SampleRate, SerenityInit};

#[tokio::main]
async fn main() {
    let cfg = azel::setup_default_log_and_load_configuration().unwrap();

    let mut discord = azel::build_client(
        cfg,
        cmd::generate_command_descriptions(),
        |b| b.register_songbird_from_config(songbird::Config::default()
            .playout_buffer_length(NonZeroUsize::new(50).unwrap())
            .playout_spike_length(10)
            .decode_sample_rate(SampleRate::Hz16000)
        )
    ).await.expect("client to be built");


    trc::info!("BOOT-CMPL");

    discord.0.start().await.expect("no error");
}
