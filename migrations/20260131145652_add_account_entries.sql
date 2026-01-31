-- Add account_entries table for tracking account transactions/logs
CREATE TABLE IF NOT EXISTS account_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    account_id INTEGER NOT NULL,
    amount_cents INTEGER NOT NULL,
    entry_date DATE NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

-- Index for efficient queries by account and date
CREATE INDEX idx_account_entries_account_date ON account_entries(account_id, entry_date);

-- Trigger to update updated_at on row changes
CREATE TRIGGER update_account_entries_updated_at
AFTER UPDATE ON account_entries
BEGIN
    UPDATE account_entries SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;