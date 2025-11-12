-- Ensure a dedicated unique index exists for (user_id, git_repo_path)
-- Even if a prior UNIQUE constraint exists, this index guarantees enforcement
-- and avoids race conditions across future schema changes.

CREATE UNIQUE INDEX IF NOT EXISTS idx_projects_user_repo_path
ON projects(user_id, git_repo_path);

