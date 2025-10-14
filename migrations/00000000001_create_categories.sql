-- Migration: create categories table and supporting constraints

CREATE TABLE IF NOT EXISTS categories (
    id UUID PRIMARY KEY,
    code TEXT UNIQUE NOT NULL,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    url_slug TEXT UNIQUE,
    category_type TEXT NOT NULL CHECK (category_type IN ('asset', 'equity', 'expense', 'income', 'liability')),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_on TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    updated_on TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);


-- Trigger to modify the the updated_on row after upate to categoryies row
CREATE TRIGGER IF NOT EXISTS trg_categories_set_updated_on
AFTER UPDATE ON categories
FOR EACH ROW
WHEN NEW.updated_on = OLD.updated_on
BEGIN
    UPDATE categories
    SET updated_on = (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
    WHERE rowid = NEW.rowid;
END;
