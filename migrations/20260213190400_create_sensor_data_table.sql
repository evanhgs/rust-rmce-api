-- Create sensor_data table for storing detailed sensor readings during runs
CREATE TABLE sensor_data (
    id SERIAL PRIMARY KEY,
    score_id INTEGER NOT NULL REFERENCES scores(id) ON DELETE CASCADE,
    timestamp_offset_ms INTEGER NOT NULL, -- Milliseconds from the start of the run
    -- Accelerometer data (m/s²)
    accel_x REAL,
    accel_y REAL,
    accel_z REAL,
    -- Gyroscope data (rad/s)
    gyro_x REAL,
    gyro_y REAL,
    gyro_z REAL,
    -- Orientation (degrees)
    orientation_azimuth REAL,
    orientation_pitch REAL,
    orientation_roll REAL,
    -- Additional metrics
    speed_kmh REAL,
    g_force REAL,
    inclination_degrees REAL,
    sound_db REAL,
    nearby_devices INTEGER,
    -- GPS coordinates (optional)
    latitude REAL,
    longitude REAL,
    altitude REAL
);

-- Index for querying sensor data by score
CREATE INDEX idx_sensor_data_score_id ON sensor_data(score_id);
CREATE INDEX idx_sensor_data_timestamp ON sensor_data(timestamp_offset_ms);

