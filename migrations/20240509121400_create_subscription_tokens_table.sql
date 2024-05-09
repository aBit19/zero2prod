-- Add migration script here
create table subscription_tokens (
    token TEXT PRIMARY KEY,
    subscription_id uuid NOT NULL REFERENCES subscriptions(id)
)