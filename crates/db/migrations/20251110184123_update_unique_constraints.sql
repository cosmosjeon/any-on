-- Update UNIQUE constraints to include user_id
-- This allows multiple users to have projects with the same git_repo_path
-- Each user's projects are isolated by user_id

BEGIN IMMEDIATE;
PRAGMA foreign_keys = OFF;

-- Recreate projects table with user_id in UNIQUE constraint
CREATE TABLE projects_new (
    id BLOB PRIMARY KEY,
    user_id TEXT,
    name TEXT NOT NULL,
    git_repo_path TEXT NOT NULL,
    setup_script TEXT DEFAULT '',
    dev_script TEXT DEFAULT '',
    cleanup_script TEXT DEFAULT '',
    copy_files TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    -- UNIQUE constraint now includes user_id
    UNIQUE(user_id, git_repo_path)
);

-- Copy data from old table
INSERT INTO projects_new (
    id, user_id, name, git_repo_path, setup_script, dev_script,
    cleanup_script, copy_files, created_at, updated_at
)
SELECT
    id, user_id, name, git_repo_path, setup_script, dev_script,
    cleanup_script, copy_files, created_at, updated_at
FROM projects;

-- Drop old table and rename new one
DROP TABLE projects;
ALTER TABLE projects_new RENAME TO projects;

-- Recreate indexes
CREATE INDEX idx_projects_user_id ON projects(user_id);

PRAGMA foreign_keys = ON;
COMMIT;
