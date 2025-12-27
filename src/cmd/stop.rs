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
            ctx.reply_restricted("Not currently playing, nothing to stop.".to_owned()).await?;
            return Ok(());
        };

        let handler_lock = handler.lock().await;
        handler_lock.queue().stop();

        ctx.reply_restricted("Stopped playback.".to_owned()).await?;
        Ok(())
    }
}

