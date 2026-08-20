CREATE EXTENSION IF NOT EXISTS citext;

CREATE TABLE users (
    id uuid PRIMARY KEY,
    pid text NOT NULL UNIQUE,
    full_name text NOT NULL,
    email citext NOT NULL UNIQUE,
    password_hash text NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
