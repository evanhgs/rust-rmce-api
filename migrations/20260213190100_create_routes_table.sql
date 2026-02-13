-- Create routes/parcours table for tracking running routes
CREATE TABLE routes (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    is_public BOOLEAN DEFAULT false,
    -- Store the route as GeoJSON or as an array of coordinates
    path_data JSONB NOT NULL,
    distance_meters REAL,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Indexes for faster queries
CREATE INDEX idx_routes_user_id ON routes(user_id);
CREATE INDEX idx_routes_public ON routes(is_public);
CREATE INDEX idx_routes_created_at ON routes(created_at);

