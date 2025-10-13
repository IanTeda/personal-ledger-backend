-- Migration: create categories table and supporting constraints

CREATE TABLE IF NOT EXISTS categories (
    id UUID PRIMARY KEY,
    code TEXT UNIQUE NOT NULL,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
