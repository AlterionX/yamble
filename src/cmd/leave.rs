use std::marker::PhantomData;

use azel::{cmd::RequestError, discord::ExecutionContext};
use serenity::all::CommandInteraction;

#[derive(Debug)]
pub struct Request<'a> {
    _phantom: &'a PhantomData<()>,
}

impl <'a> Request<'a> {
    pub fn parse(_cmd: &'a CommandInteraction) -> Result<Self, RequestError> {
        Ok(Self {_phantom: &PhantomData,})
    }

    pub async fn execute(self, ctx: &ExecutionContext<'_>) -> Result<(), RequestError> {
        let guild_id = ctx.cmd.guild_id.ok_or_else(|| RequestError::User("command only available in a server".into()))?;

        let manager = songbird::get(ctx.ctx).await.expect("songbird initialized").clone();
        if manager.get(guild_id).is_none() {
            ctx.reply("No channel to leave! (If I am in a channel, you can remove me by running `/play` again followed by `/leave`.)".to_owned()).await?;
            return Ok(());
        }

        manager.leave(guild_id).await.map_err(|_e| RequestError::Internal("Voice channel leave failed.".into()))?;

        ctx.reply("Left channel -- we just don't know which one (yet).".to_owned()).await?;

        Ok(())
    }
}
