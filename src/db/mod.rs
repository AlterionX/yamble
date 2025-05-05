use azel::{cmd::RequestError, DatabaseConfiguration};
use bigdecimal::BigDecimal;
use diesel::prelude::Insertable;
use diesel_async::{AsyncPgConnection, AsyncConnection, RunQueryDsl};

use crate::schema::audio_ledger;

#[derive(Debug)]
#[derive(Insertable)]
#[diesel(table_name = audio_ledger)]
pub struct NewAudioLedgerEntry<'a> {
    pub link_or_name: &'a str,
    pub downloaded: bool,
    pub file_path: String,
    pub uploader: BigDecimal,
}

pub async fn track_known_audio_in_ledger(cfg: &DatabaseConfiguration, data: &NewAudioLedgerEntry<'_>) -> Result<(), RequestError> {
    let Ok(mut conn) = AsyncPgConnection::establish(&cfg.url).await else {
        return Err(RequestError::User("Database connection failed".into()));
    };

    let Ok(_) = diesel::insert_into(audio_ledger::table).values(data).execute(&mut conn).await else {
        return Err(RequestError::User("Database insert failed".into()));
    };

    Ok(())
}
