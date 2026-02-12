-- Add password column to users table
-- Note: Pour les utilisateurs existants, vous devrez mettre à jour manuellement leurs mots de passe
ALTER TABLE users ADD COLUMN password TEXT;

-- Optionnel: rendre le champ obligatoire après avoir mis à jour les données existantes
-- ALTER TABLE users ALTER COLUMN password SET NOT NULL;
