use std::{borrow::Cow, marker::PhantomData, path::{Path, PathBuf}};
use azel::discord::ExecutionContext;
use songbird::input::cached::Memory;
use tracing as trc;

use serenity::all::{ChannelId, CommandInteraction, Mention, ResolvedValue};
use youtube_dl::YoutubeDl;

use super::RequestError;

#[derive(Debug)]
pub struct Request<'a> {
    music: &'a str,
    target: Option<ChannelId>,
    _phantom: &'a PhantomData<()>,
}

impl <'a> Request<'a> {
    pub fn parse(cmd: &'a CommandInteraction) -> Result<Self, RequestError> {
        let mut explicit_channel = None;
        let mut music = None;

        for option in cmd.data.options().iter() {
            if option.name == "target" {
                if let ResolvedValue::Channel(target_channel) = option.value {
                    explicit_channel = Some(target_channel.id);
                }
            }
            if option.name == "music" {
                if let ResolvedValue::String(provided_music) = option.value {
                    music = Some(provided_music);
                }
            }
        }

        Ok(Self {
            music: music.ok_or_else(|| RequestError::User("missing `music` required parameter".into()))?,
            target: explicit_channel,
            _phantom: &PhantomData,
        })
    }

    pub async fn execute(self, ctx: &ExecutionContext<'_>) -> Result<(), RequestError> {
        let guild_id = ctx.cmd.guild_id.ok_or_else(|| RequestError::User("command only available in a server".into()))?;

        let target = if let Some(ch) = self.target {
            let mut channels = guild_id.channels(ctx.ctx).await.map_err(|_e| RequestError::Internal("channel failed to load".into()))?;
            channels.remove(&ch).ok_or_else(|| RequestError::User("channel not in guild".into()))?
        } else {
            ctx.find_interactor_voice_channel(guild_id).await?
        };

        let manager = songbird::get(ctx.ctx).await.expect("songbird initialized").clone();
        let mut join_required = false;
        let handler = if let Some(handler) = manager.get(guild_id) {
            handler
        } else {
            join_required = true;
            match manager.join(guild_id, target.id).await {
                Ok(handler) => handler,
                Err(_e) => {
                    return Err(RequestError::Internal("Voice channel join failed.".into()));
                },
            }
        };

        let mut handler_lock = handler.lock().await;

        let current_joined_channel = handler_lock.current_channel();
        let mut channel_changed_from = None;
        if current_joined_channel != Some(target.id.into()) {
            channel_changed_from = current_joined_channel;
            match manager.join(guild_id, target.id).await {
                Ok(_handler) => {},
                Err(_e) => {
                    return Err(RequestError::Internal("Voice channel join failed.".into()));
                },
            }
        }

        let audio = load_else_download(ctx, self.music).await?;

        let mem = Memory::new(audio.into()).await.unwrap();
        handler_lock.play_input(mem.into());

        if let Some(ch) = channel_changed_from {
            ctx.reply(format!("Switched to {} from {}!\nPlaying {}", Mention::Channel(target.id), Mention::Channel(ch.0.into()), self.music)).await?;
        } else if join_required {
            ctx.reply(format!("Joined channel {}!\nPlaying {}", Mention::Channel(target.id), self.music)).await?;
        } else {
            ctx.reply(format!("Playing {} in {}!", self.music, Mention::Channel(target.id))).await?;
        }


        Ok(())
    }
}

// TODO lift ytdlp download to top later
const YTDLP_DOWNLOAD_PATH: &str = "resources/bin/ytdlp";

// TODO impl streaming properly instead of fully downloading first. Just don't play anything big
async fn load_else_download(ctx: &ExecutionContext<'_>, music: &str) -> Result<Vec<u8>, RequestError> {
    let load_path = if music.starts_with("https://www.youtube.com/watch") {
        // We expect this will take a while
        // TODO make this run out of band
        ctx.defer().await?;

        // pretend this always succeeds so that we can assume path exists later
        std::fs::create_dir_all(YTDLP_DOWNLOAD_PATH).ok();
        let mut contents = std::fs::read_dir(YTDLP_DOWNLOAD_PATH).map_err(|e| RequestError::Internal(format!("ytdlp check dir failed {e:?}").into()))?;
        // assume non-empty dir has only a single file that is ytdlp
        let ytdlp_path = match contents.next() {
            Some(p) => {
                trc::info!("YTDLP-LOAD-SKIP");
                p.map_err(|e| RequestError::Internal(format!("ytdlp check failed {e:?}").into()))?.path()
            },
            None => {
                trc::info!("YTDLP-LOAD-START");
                let p = youtube_dl::download_yt_dlp(YTDLP_DOWNLOAD_PATH).await.map_err(|_e| RequestError::Internal("file present".into()))?;
                trc::info!("YTDLP-LOAD-END");
                p
            },
        };

        let mut yt_client = YoutubeDl::new(music);
        yt_client.youtube_dl_path(ytdlp_path.as_path());
        yt_client.socket_timeout("15");
        yt_client.output_directory("downloads");

        trc::info!("METADATA-LOAD-START");
        let output = yt_client.clone()
            .extra_arg("--skip-download")
            .run_async().await
            .map_err(|e| RequestError::Internal(format!("ytdlp failed {e:?}").into()))?
            .into_single_video()
            .ok_or_else(|| RequestError::User("bad input -- could not find video".into()))?;
        trc::info!("METADATA-LOAD-END");
        // TODO Set dl size limits
        let video_download_dir = Path::new("downloads").join(output.id);

        // Best audio format, only
        if !std::fs::exists(video_download_dir.as_path()).map_err(|_e| RequestError::Internal("vid dl check failure".into()))? {
            trc::info!("VIDEO-DOWNLOAD-START");
            // yt_client.format("a[acodec^=mp3]/ba/b");
            // yt_client.extract_audio(true);
            // yt_client.extra_arg("--audio-format").extra_arg("mp3");
            yt_client.format("ba");
            yt_client.download_to_async(video_download_dir.clone()).await.map_err(|e| RequestError::Internal(format!("ytdlp failed {e:?}").into()))?;
            trc::info!("VIDEO-DOWNLOAD-END");
        } else {
            trc::info!("VIDEO-DOWNLOAD-SKIP");
        }

        let video_dl_contents = std::fs::read_dir(video_download_dir.as_path()).map_err(|_e| RequestError::Internal("video dl failure".into()))?;
        let mut downloaded_vid_path = None;
        // assume non-empty dir has only a single file that is video
        for value in video_dl_contents {
            downloaded_vid_path = Some(value.map_err(|_e| RequestError::Internal("dl files check failed".into()))?.path());
        }

        downloaded_vid_path.ok_or_else(|| RequestError::Internal("dl failed".into()))?
    } else {
        let path = if music.is_ascii() {
            Cow::Borrowed(music)
        } else {
            // Remove any path shenanigans so writes are isolated to this directory. We should do this
            // in the OS too but I'm lazy.
            Cow::Owned(music.chars().map(|c| if !c.is_ascii() || c == '/' || c == '\\' {
                '_'
            } else {
                c
            }).collect())
        };
        let path = &Path::new(path.as_ref());

        // Cobbled together to hopefully make it work.
        PathBuf::from("downloads").join(path).join("data.mp3")
    };

    trc::info!("PLAY-FILE-LOAD {:?} {:?}", load_path.canonicalize(), load_path);
    let audio_file = std::fs::read(load_path).expect("file readable");

    Ok(audio_file)
}
