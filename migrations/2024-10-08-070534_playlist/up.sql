CREATE TABLE playlists (
    id BIGSERIAL PRIMARY KEY,
    discord_user NUMERIC(20, 0) NOT NULL,
    name VARCHAR(100) NOT NULL
);

CREATE UNIQUE INDEX unique_playlist_name_per_user ON playlists (name, discord_user);

CREATE TABLE playlist_entries (
    id BIGSERIAL PRIMARY KEY,
    playlist BIGINT REFERENCES playlists,
    audio BIGINT REFERENCES audio_ledger
);
