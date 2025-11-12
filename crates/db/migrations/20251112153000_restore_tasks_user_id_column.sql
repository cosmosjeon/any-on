-- Re-add the user_id column and indexes to tasks after the plan status migration
-- The earlier migration that introduced the 'plan' status recreated the tasks table
-- without carrying over the user_id column added in 20251110184122. This migration
-- restores the column and related indexes for existing databases.

PRAGMA foreign_keys = OFF;

ALTER TABLE tasks ADD COLUMN user_id TEXT;

-- Recreate indexes that depended on the user_id column
CREATE INDEX IF NOT EXISTS idx_tasks_user_id ON tasks(user_id);
CREATE INDEX IF NOT EXISTS idx_tasks_project_user ON tasks(project_id, user_id);

PRAGMA foreign_keys = ON;
