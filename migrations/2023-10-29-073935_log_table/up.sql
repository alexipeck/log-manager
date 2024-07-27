
CREATE TABLE IF NOT EXISTS log (
    id INTEGER NOT NULL,
    source TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    level TEXT NOT NULL,
    location TEXT NOT NULL,
    content TEXT NOT NULL,
    PRIMARY KEY(id)
);