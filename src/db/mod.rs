use azel::{cmd::RequestError, DatabaseConfiguration};
use bigdecimal::{BigDecimal};
use diesel::{prelude::{Identifiable, Insertable, QueryDsl, Queryable}, BoolExpressionMethods, Expression, ExpressionMethods, OptionalExtension, Selectable};
use diesel_async::{AsyncPgConnection, AsyncConnection, RunQueryDsl};

use crate::schema::audio_ledger;

#[derive(Debug)]
#[derive(Queryable, Identifiable, Selectable)]
#[diesel(table_name = audio_ledger)]
pub struct AudioLedgerEntry {
    pub id: BigDecimal,
    pub link_or_name: String,
    pub downloaded: bool,
    pub file_path: String,
    pub uploader: BigDecimal,
}


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

/*
pub async fn load_maybe_known_audio_in_ledger(cfg: &DatabaseConfiguration, user_id: BigDecimal, name: &str) -> Result<Option<AudioLedgerEntry>, RequestError> {
    let Ok(mut conn) = AsyncPgConnection::establish(&cfg.url).await else {
        return Err(RequestError::User("Database connection failed".into()));
    };

    let Ok(val) = ({
        audio_ledger::table
            .filter(
                audio_ledger::uploader.eq(user_id)
                .and(audio_ledger::link_or_name.eq(name))
            )
            .first(&mut conn)
            .await
            .optional()
    }) else {
        return Err(RequestError::User("Database insert failed".into()));
    };

    Ok(val)
}
*/
