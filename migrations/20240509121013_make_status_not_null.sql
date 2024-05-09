BEGIN;
    UPDATE subscriptions SET status = 'active' WHERE status IS NULL;
    ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
END;