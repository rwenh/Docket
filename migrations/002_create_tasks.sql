CREATE TABLE IF NOT EXISTS tasks (
       id SERIAL PRIMARY KEY,
       title VARCHAR(255) NOT NULL,
       description TEXT,
       status TEXT NOT NULL DEFAULT 'todo' CHECK (status IN('todo', 'in_progress', 'done')),
       priority TEXT NOT NULL DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high')),
       due_date TIMESTAMPTZ,
       created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
       updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
       owner_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS ix_tasks_id ON tasks (id);
CREATE INDEX IF NOT EXISTS ix_tasks_owner_id ON tasks (owner_id);
