-- Add user_id columns to all main tables for multi-user support
-- This migration adds user_id TEXT columns (nullable initially) to allow
-- gradual transition from single-user to multi-user architecture.

PRAGMA foreign_keys = OFF;

-- 1. projects table
ALTER TABLE projects ADD COLUMN user_id TEXT;
CREATE INDEX idx_projects_user_id ON projects(user_id);

-- 2. tasks table
ALTER TABLE tasks ADD COLUMN user_id TEXT;
CREATE INDEX idx_tasks_user_id ON tasks(user_id);
CREATE INDEX idx_tasks_project_user ON tasks(project_id, user_id);

-- 3. task_attempts table
ALTER TABLE task_attempts ADD COLUMN user_id TEXT;
CREATE INDEX idx_task_attempts_user_id ON task_attempts(user_id);

-- 4. execution_processes table
ALTER TABLE execution_processes ADD COLUMN user_id TEXT;
CREATE INDEX idx_execution_processes_user_id ON execution_processes(user_id);

-- 5. images table
ALTER TABLE images ADD COLUMN user_id TEXT;
CREATE INDEX idx_images_user_id ON images(user_id);

-- 6. tags table
ALTER TABLE tags ADD COLUMN user_id TEXT;
CREATE INDEX idx_tags_user_id ON tags(user_id);

-- 7. drafts table
ALTER TABLE drafts ADD COLUMN user_id TEXT;
CREATE INDEX idx_drafts_user_id ON drafts(user_id);

-- 8. executor_sessions table
ALTER TABLE executor_sessions ADD COLUMN user_id TEXT;
CREATE INDEX idx_executor_sessions_user_id ON executor_sessions(user_id);

-- 9. merges table
ALTER TABLE merges ADD COLUMN user_id TEXT;
CREATE INDEX idx_merges_user_id ON merges(user_id);

PRAGMA foreign_keys = ON;
