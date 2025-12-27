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
        Ok(())
    }
}

