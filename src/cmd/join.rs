use std::marker::PhantomData;

use azel::discord::ExecutionContext;
use serenity::all::{ChannelId, CommandInteraction, Mention, ResolvedValue};

use super::RequestError;

#[derive(Debug)]
pub struct Request<'a> {
    channel: Option<ChannelId>,
    _phantom: &'a PhantomData<()>,
}

impl <'a> Request<'a> {
    pub fn parse(cmd: &'a CommandInteraction) -> Result<Self, RequestError> {
        let explicit_channel = cmd.data.options().iter().filter_map(|opt| {
            if opt.name.is_empty() {
                None
            } else if let ResolvedValue::Channel(target_channel) = opt.value {
                Some(target_channel.id)
            } else {
                None
            }
        }).next();

        Ok(Self {
            channel: explicit_channel,
            _phantom: &PhantomData,
        })
    }

    pub async fn execute(self, ctx: &ExecutionContext<'_>) -> Result<(), RequestError> {
        let guild_id = ctx.cmd.guild_id.ok_or_else(|| RequestError::User("command only available in a server".into()))?;

        let channel = if let Some(ch) = self.channel {
            let mut channels = guild_id.channels(ctx.ctx).await.map_err(|_e| RequestError::Internal("channel failed to load".into()))?;
            channels.remove(&ch).ok_or_else(|| RequestError::User("channel not in guild".into()))?
        } else {
            ctx.find_interactor_voice_channel(guild_id).await?
        };

        let manager = songbird::get(ctx.ctx).await.expect("songbird initialized").clone();
        let mut current_channel = None;
        if let Some(handler) = manager.get(guild_id) {
            let a = handler.lock().await;
            let maybe_channel_to_switch_from = a.current_channel();
            if a.current_channel() == Some(channel.id.into()) {
                ctx.reply(format!("Already in {}!", Mention::Channel(channel.id))).await?;
                return Ok(());
            }
            current_channel = maybe_channel_to_switch_from;
        }

        if let Err(_e) = manager.join(guild_id, channel.id).await {
            return Err(RequestError::Internal("Voice channel join failed.".into()));
        }

        match current_channel {
            Some(c) => {
                ctx.reply(format!("Switched to {} from {}!", Mention::Channel(channel.id), Mention::Channel(c.0.into()))).await?;
            },
            None => {
                ctx.reply(format!("Joined channel {}!", Mention::Channel(channel.id))).await?;
            },
        }

        Ok(())
    }
}
