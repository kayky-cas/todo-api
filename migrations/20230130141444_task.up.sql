-- Add up migration script here
CREATE TABLE
IF NOT EXISTS tasks (
    id UUID PRIMARY KEY NOT NULL DEFAULT (uuid_generate_v4()),
    name VARCHAR(255) NOT NULL,
    description VARCHAR NOT NULL,
    tag VARCHAR(20) NOT NULL,
    date DATE

    user_id UUID,

    CONSTRAINT fk_user_task FOREIGN KEY(user_id) REFERENCES users(id)
);
