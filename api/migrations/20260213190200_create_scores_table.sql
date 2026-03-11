-- Create scores/times table for tracking user performance on routes
CREATE TABLE scores (
    id SERIAL PRIMARY KEY,
    route_id INTEGER NOT NULL REFERENCES routes(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    time_seconds REAL NOT NULL,
    max_speed_kmh REAL,
    avg_speed_kmh REAL,
    max_g_force REAL,
    max_inclination_degrees REAL,
    max_sound_db REAL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Indexes for leaderboard queries
CREATE INDEX idx_scores_route_id ON scores(route_id);
CREATE INDEX idx_scores_user_id ON scores(user_id);
CREATE INDEX idx_scores_time ON scores(time_seconds);
CREATE INDEX idx_scores_max_speed ON scores(max_speed_kmh);
CREATE INDEX idx_scores_created_at ON scores(created_at);

