-- Create salary tracking table
CREATE TABLE IF NOT EXISTS salaries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    effective_date DATE NOT NULL,
    amount_cents INTEGER NOT NULL, -- Store salary in cents to avoid floating point issues
    currency TEXT NOT NULL DEFAULT 'GBP',
    frequency TEXT NOT NULL DEFAULT 'ANNUAL', -- ANNUAL, MONTHLY, WEEKLY, etc.
    notes TEXT, -- Optional notes about salary change (promotion, raise, etc.)
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Index for effective_date for querying salary at specific dates
CREATE INDEX idx_salaries_effective_date ON salaries(effective_date DESC);

-- Trigger to update updated_at timestamp
CREATE TRIGGER update_salaries_timestamp
AFTER UPDATE ON salaries
FOR EACH ROW
BEGIN
    UPDATE salaries SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;