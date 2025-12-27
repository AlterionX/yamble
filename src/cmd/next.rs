use std::marker::PhantomData;

use azel::discord::ExecutionContext;
use serenity::all::CommandInteraction;

use super::RequestError;

#[derive(Debug)]
pub struct Request<'a> {
    _phantom: &'a PhantomData<()>,
}

impl <'a> Request<'a> {
    pub fn parse(_cmd: &'a CommandInteraction) -> Result<Self, RequestError> {
        Ok(Self {
            _phantom: &PhantomData,
        })
    }

    pub async fn execute(self, ctx: &ExecutionContext<'_>) -> Result<(), RequestError> {
        let guild_id = ctx.cmd.guild_id.ok_or_else(|| RequestError::User("command only available in a server".into()))?;

        let manager = songbird::get(ctx.ctx).await.expect("songbird initialized").clone();
        let Some(handler) = manager.get(guild_id) else {
            ctx.reply_restricted("Not currently playing, nothing to adjust.".to_owned()).await?;
            return Ok(());
        };

        let handler_lock = handler.lock().await;
        match handler_lock.queue().skip() {
            Ok(_) => {
                ctx.reply_restricted("Skipped to next track.".to_owned()).await?;
            },
            Err(_) => {
                // silently ignore if no next track
                handler_lock.queue().stop();
                ctx.reply_restricted("No next track. Playback stopped.".to_owned()).await?;
            },
        }

        Ok(())
    }
}


