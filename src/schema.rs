// @generated automatically by Diesel CLI.

diesel::table! {
    audio_ledger (id) {
        id -> Int8,
        #[max_length = 1024]
        link_or_name -> Varchar,
        downloaded -> Bool,
        #[max_length = 1024]
        file_path -> Varchar,
        uploader -> Numeric,
    }
}

diesel::table! {
    playlist_entries (id) {
        id -> Int8,
        playlist -> Nullable<Int8>,
        audio -> Nullable<Int8>,
    }
}

diesel::table! {
    playlists (id) {
        id -> Int8,
        discord_user -> Numeric,
        #[max_length = 100]
        name -> Varchar,
    }
}

diesel::joinable!(playlist_entries -> audio_ledger (audio));
diesel::joinable!(playlist_entries -> playlists (playlist));

diesel::allow_tables_to_appear_in_same_query!(
    audio_ledger,
    playlist_entries,
    playlists,
);
