-- Add 'vite' to the allowed framework values for apps and workspaces.
ALTER TABLE apps DROP CONSTRAINT apps_framework_check;
ALTER TABLE apps ADD CONSTRAINT apps_framework_check CHECK (framework IN ('worker', 'nuxtjs', 'react-router', 'vite'));

ALTER TABLE workspaces DROP CONSTRAINT workspaces_framework_check;
ALTER TABLE workspaces ADD CONSTRAINT workspaces_framework_check CHECK (framework IN ('worker', 'nuxtjs', 'react-router', 'vite'));
