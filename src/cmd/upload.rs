use std::{fs::File, io::Write, marker::PhantomData};

use azel::discord::ExecutionContext;
use serenity::all::{Attachment, CommandInteraction, ResolvedValue};

use crate::db::{self, NewAudioLedgerEntry};

use super::RequestError;

#[derive(Debug)]
pub struct Request<'a> {
    name: &'a str,
    sound: &'a Attachment,
    _phantom: &'a PhantomData<()>,
}

impl <'a> Request<'a> {
    pub fn parse(cmd: &'a CommandInteraction) -> Result<Self, RequestError> {
        let mut uploaded = None;
        let mut name = None;

        for option in cmd.data.options().iter() {
            if option.name == "sound" {
                if let ResolvedValue::Attachment(value) = option.value {
                    uploaded = Some(value);
                }
            }
            if option.name == "name" {
                if let ResolvedValue::String(n) = option.value {
                    name = Some(n);
                }
            }
        }

        Ok(Self {
            name: name.ok_or_else(|| RequestError::User("missing `name` required parameter".into()))?,
            sound: uploaded.ok_or_else(|| RequestError::User("missing `name` required parameter".into()))?,
            _phantom: &PhantomData,
        })
    }

    pub async fn execute(self, ctx: &ExecutionContext<'_>) -> Result<(), RequestError> {
        ctx.cmd.defer(ctx.ctx).await.map_err(|_| {
            RequestError::Internal("Could not connect to Discord!".into())
        })?;
        let (download_path, mut download_output) = generate_filepath(self.sound.filename.as_str())?;

        let new_data = NewAudioLedgerEntry {
            link_or_name: self.name,
            downloaded: true,
            file_path: download_path,
            uploader: u64::from(ctx.cmd.user.id).into(),
        };

        let Ok(download_data) = self.sound.download().await else {
            return Err(RequestError::Internal("Could not download file.".into()));
        };

        let Ok(_) = download_output.write_all(download_data.as_slice()) else {
            return Err(RequestError::Internal("Could not download file.".into()));
        };

        db::track_known_audio_in_ledger(ctx.db_cfg, &new_data).await?;

        ctx.reply(format!("Uploaded! {:?}", new_data)).await?;

        Ok(())
    }
}

fn generate_filepath(filename: &str) -> Result<(String, File), RequestError> {
    loop {
        let download_path = format!("uploaded/{}/{}", uuid::Uuid::new_v4(), filename);
        let Ok(f) = File::create(download_path.as_str()) else {
            continue;
        };

        return Ok((download_path, f));
    }
}
