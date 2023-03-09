-- Add up migration script here
CREATE TABLE
IF NOT EXISTS tasks (
    id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4()),
    name VARCHAR(255) NOT NULL,
    description VARCHAR,
    tag VARCHAR(20) NOT NULL,
    date TIMESTAMPTZ,
    user_id UUID NOT NULL,

    CONSTRAINT fk_user_task FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);
