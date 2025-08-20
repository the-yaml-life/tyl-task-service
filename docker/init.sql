-- Database initialization script for TYL Microservice

-- Create the main entities table
-- Replace with your actual domain model schema
CREATE TABLE IF NOT EXISTS entities (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'Active',
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_entities_name ON entities(name);
CREATE INDEX IF NOT EXISTS idx_entities_status ON entities(status);
CREATE INDEX IF NOT EXISTS idx_entities_created_at ON entities(created_at);

-- Create a function to automatically update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger to automatically update updated_at
CREATE TRIGGER update_entities_updated_at 
    BEFORE UPDATE ON entities 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Insert some sample data for development
INSERT INTO entities (id, name, status, description) VALUES
    ('550e8400-e29b-41d4-a716-446655440000', 'Sample Entity 1', 'Active', 'This is a sample entity for testing'),
    ('550e8400-e29b-41d4-a716-446655440001', 'Sample Entity 2', 'Inactive', 'Another sample entity'),
    ('550e8400-e29b-41d4-a716-446655440002', 'Sample Entity 3', 'Pending', 'A pending entity')
ON CONFLICT (id) DO NOTHING;

-- Create additional tables as needed for your domain
-- Example: audit table for tracking changes
CREATE TABLE IF NOT EXISTS entity_audit (
    id BIGSERIAL PRIMARY KEY,
    entity_id UUID NOT NULL,
    operation VARCHAR(20) NOT NULL, -- 'INSERT', 'UPDATE', 'DELETE'
    old_values JSONB,
    new_values JSONB,
    changed_by VARCHAR(255),
    changed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_entity_audit_entity_id ON entity_audit(entity_id);
CREATE INDEX IF NOT EXISTS idx_entity_audit_changed_at ON entity_audit(changed_at);