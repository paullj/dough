-- Create accounts table with autoincrement id and public_id using sqids
CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    public_id TEXT NOT NULL UNIQUE, -- Short public ID like commit SHA using sqids
    name TEXT NOT NULL UNIQUE,
    account_type TEXT NOT NULL,
    currency TEXT NOT NULL DEFAULT 'GBP',
    initial_balance_cents INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for public_id for fast lookups
CREATE INDEX idx_accounts_public_id ON accounts(public_id);

-- Trigger to update updated_at timestamp
CREATE TRIGGER update_accounts_timestamp
AFTER UPDATE ON accounts
FOR EACH ROW
BEGIN
    UPDATE accounts SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;