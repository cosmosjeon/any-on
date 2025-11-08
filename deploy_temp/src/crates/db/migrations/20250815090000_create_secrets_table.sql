PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS secrets (
    id            BLOB PRIMARY KEY,
    user_id       TEXT NOT NULL,
    provider      TEXT NOT NULL,
    name          TEXT NOT NULL,
    key_version   INTEGER NOT NULL DEFAULT 1,
    secret_blob   BLOB NOT NULL,
    created_at    TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now', 'subsec'))
);

CREATE UNIQUE INDEX IF NOT EXISTS secrets_user_provider_name_idx
    ON secrets(user_id, provider, name);
