-- Create Susbcriptions Table
CREATE TABLE subscriptions (
    id uuid NOT NULL PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    subscibed_at timestamptz NOT NULL DEFAULT NOW()
); 