pub mod join;
pub mod leave;
pub mod play;

pub mod upload;

use azel::{cmd::{CommandTreeTop, DiscordCommandArgs, DiscordCommandDescriptor, RawCommandOptionEntry, RequestError}, discord::ExecutionContext};
use tracing as trc;

use strum::{EnumCount, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount, EnumIter)]
pub enum RequestKind {
    Ping,
    Join,
    Leave,
    Play,
    Upload,
}

impl DiscordCommandDescriptor for RequestKind {
    type Args<'a> = RequestArgs<'a>;

    fn name(&self) -> &'static str {
        match self {
            RequestKind::Ping => {
                "ping"
            },
            RequestKind::Join => {
                "join"
            },
            RequestKind::Leave => {
                "leave"
            },
            RequestKind::Play => {
                "play"
            },
            RequestKind::Upload => {
                "upload"
            },
        }
    }

    fn description(&self) -> &'static str {
        match self {
            RequestKind::Ping => {
                "Ping!"
            },
            RequestKind::Join => {
                "Ask Yamble to join a voice channel. (Or the one you're in!)"
            },
            RequestKind::Leave => {
                "Make Yamble leave voice."
            },
            RequestKind::Play => {
                "Ask Yamble to play something."
            },
            RequestKind::Upload => {
                "Upload an audio snippet that Yamble can play"
            },
        }
    }

    fn options(&self) -> Vec<RawCommandOptionEntry> {
        match self {
            RequestKind::Ping => {
                vec![]
            },
            RequestKind::Join => {
                vec![RawCommandOptionEntry::Channel {
                    name: "target",
                    description: "Channel to join",
                    required: false,
                }]
            },
            RequestKind::Leave => {
                vec![]
            },
            RequestKind::Play => {
                vec![RawCommandOptionEntry::String {
                    name: "music",
                    description: "Music to play",
                    required: true,
                }, RawCommandOptionEntry::Channel {
                    name: "target",
                    description: "Channel to join",
                    required: false,
                }]
            },
            RequestKind::Upload => {
                vec![RawCommandOptionEntry::String {
                    name: "name",
                    description: "Name to save file as",
                    required: false,
                }, RawCommandOptionEntry::Attachment {
                    name: "sound",
                    description: "sound to upload",
                    required: false,
                }]
            },
        }
    }

    fn parse<'a>(cmd: &'a serenity::all::CommandInteraction) -> Result<Self::Args<'a>, RequestError> {
        match cmd.data.name.as_str() {
            "ping" => {
                Ok(RequestArgs::Ping)
            },
            "join" => {
                Ok(RequestArgs::Join(join::Request::parse(cmd)?))
            },
            "leave" => {
                Ok(RequestArgs::Leave(leave::Request::parse(cmd)?))
            },
            "play" => {
                Ok(RequestArgs::Play(play::Request::parse(cmd)?))
            },
            "upload" => {
                Ok(RequestArgs::Upload(upload::Request::parse(cmd)?))
            },
            _ => {
                trc::error!("Unknown command {:?} received", cmd);
                Err(RequestError::Internal("Unknown command.".into()))
            },
        }
    }
}

#[tracing::instrument(name = "hello")]
pub fn generate_command_descriptions() -> Vec<CommandTreeTop<RequestKind>> {
    vec![
        CommandTreeTop::NakedChatInput(RequestKind::Ping, None),
        CommandTreeTop::NakedChatInput(RequestKind::Join, None),
        CommandTreeTop::NakedChatInput(RequestKind::Leave, None),
        CommandTreeTop::NakedChatInput(RequestKind::Play, None),
        CommandTreeTop::NakedChatInput(RequestKind::Upload, None),
    ]
}

#[derive(Debug)]
pub enum RequestArgs<'a> {
    Ping,
    Join(join::Request<'a>),
    Leave(leave::Request<'a>),
    Play(play::Request<'a>),
    Upload(upload::Request<'a>),
}

impl <'a> DiscordCommandArgs for RequestArgs<'a> {
    async fn execute(self, ctx: &ExecutionContext<'_>) -> Result<(), RequestError> {
        match self {
            RequestArgs::Ping => {
                // Just try pong.
                ctx.reply("Pong!".to_owned()).await
            },
            RequestArgs::Join(req) => {
                req.execute(ctx).await
            },
            RequestArgs::Leave(req) => {
                req.execute(ctx).await
            },
            RequestArgs::Play(req) => {
                req.execute(ctx).await
            },
            RequestArgs::Upload(req) => {
                req.execute(ctx).await
            },
        }
    }
}
