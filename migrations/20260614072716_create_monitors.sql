-- Add migration script here
CREATE FUNCTION update_updated_at_column () RETURNS trigger AS $$
BEGIN NEW .updated_at = now();
RETURN NEW;
END;
$$ language plpgsql;

CREATE TABLE monitors (
    id uuid PRIMARY KEY DEFAULT uuidv7 (),
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    interval_seconds int NOT NULL DEFAULT 60,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE TRIGGER update_monitors_updated_at BEFORE
    UPDATE ON monitors FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column ();
