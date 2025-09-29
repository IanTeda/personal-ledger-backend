-- Migration: create categories table and supporting constraints
--
-- Keep the migration portable across Postgres and SQLite by avoiding
-- Postgres-only features (ENUM types and PL/pgSQL triggers). Store
-- `category_type` as TEXT and validate allowed values with a CHECK
-- constraint. Use CURRENT_TIMESTAMP for defaults which works on both DBs.

CREATE TABLE IF NOT EXISTS categories (
    id UUID PRIMARY KEY,
    code TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    slug TEXT UNIQUE,
    category_type TEXT NOT NULL CHECK (category_type IN ('asset','liability','income','expense','equity')),
    color TEXT CHECK (color IS NULL OR (length(color) = 7 AND substr(color,1,1) = '#')),
    icon TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_categories_code_lower ON categories (LOWER(code));

