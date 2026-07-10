-- Seed data for local development
-- Password for all users is 'password123' (bcrypt hash)

-- Users
INSERT INTO users (id, username, email, password_hash, display_name, roles) VALUES
    ('a0000000-0000-0000-0000-000000000001', 'admin', 'admin@dash.dev', '$2b$12$LJ3m4ys3Lk0TSwHnbfOMiOXPm1Qm0Il/7BQ0S0V8k5s0Y1k9qZS', 'Admin User', '{admin,publisher}'),
    ('a0000000-0000-0000-0000-000000000002', 'developer', 'dev@dash.dev', '$2b$12$LJ3m4ys3Lk0TSwHnbfOMiOXPm1Qm0Il/7BQ0S0V8k5s0Y1k9qZS', 'Developer User', '{publisher,user}'),
    ('a0000000-0000-0000-0000-000000000003', 'alice', 'alice@dash.dev', '$2b$12$LJ3m4ys3Lk0TSwHnbfOMiOXPm1Qm0Il/7BQ0S0V8k5s0Y1k9qZS', 'Alice Contributor', '{publisher}'),
    ('a0000000-0000-0000-0000-000000000004', 'bob', 'bob@dash.dev', '$2b$12$LJ3m4ys3Lk0TSwHnbfOMiOXPm1Qm0Il/7BQ0S0V8k5s0Y1k9qZS', 'Bob Maintainer', '{maintainer}')
ON CONFLICT (username) DO NOTHING;

-- Packages
INSERT INTO packages (id, name, description, license, author) VALUES
    ('b0000000-0000-0000-0000-000000000001', 'lodash', 'The Lodash library exported as Node.js modules.', 'MIT', '{"name": "John-David Dalton", "email": "john.david.dalton@gmail.com"}'),
    ('b0000000-0000-0000-0000-000000000002', 'express', 'Fast, unopinionated, minimalist web framework for Node.js', 'MIT', '{"name": "TJ Holowaychuk", "email": "tj@vision-media.ca"}'),
    ('b0000000-0000-0000-0000-000000000003', 'react', 'React is a JavaScript library for building user interfaces.', 'MIT', '{"name": "Facebook", "url": "https://github.com/facebook"}'),
    ('b0000000-0000-0000-0000-000000000004', 'typescript', 'TypeScript is a superset of JavaScript that compiles to clean JavaScript output.', 'Apache-2.0', '{"name": "Microsoft", "url": "https://github.com/microsoft"}'),
    ('b0000000-0000-0000-0000-000000000005', 'vite', 'Native-ESM powered web dev build tool.', 'MIT', '{"name": "Evan You", "email": "evan@vuejs.org"}'),
    ('b0000000-0000-0000-0000-000000000006', 'next', 'The React Framework for Production.', 'MIT', '{"name": "Vercel", "url": "https://vercel.com"}'),
    ('b0000000-0000-0000-0000-000000000007', 'bun', 'Bun is a fast all-in-one JavaScript runtime.', 'MIT', '{"name": "Oven.sh", "url": "https://oven.sh"}')
ON CONFLICT (name) DO NOTHING;

-- Versions for lodash
INSERT INTO package_versions (id, package_id, version, description, main, dist) SELECT
    'c0000000-0000-0000-0000-000000000001', id, '4.17.21', 'Lodash modular utilities.', 'lodash.js',
    '{"tarball": "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz", "integrity": "sha512-v2kDEe57lecTulaDIuNTPy3Ry4gLGJ6Z1O3vE1krgXZNrsQ+LFTGHVxVjcXPs17LhbZVGedAJv8XZ1tvj5FvSg==", "fileCount": 1049, "unpackedSize": 1426567}'
FROM packages WHERE name = 'lodash'
ON CONFLICT (package_id, version) DO NOTHING;

INSERT INTO package_versions (id, package_id, version, description, main, dist) SELECT
    'c0000000-0000-0000-0000-000000000002', id, '4.17.20', 'Lodash modular utilities.', 'lodash.js',
    '{"tarball": "https://registry.npmjs.org/lodash/-/lodash-4.17.20.tgz", "integrity": "sha512-p0J9NZ9MBeYdIgNZqR8hGDam5Mbz6fEN3eM8nT8fjOJ6ZR/F4BcS0Aa7VYOMcuvVlkGBy8MHZscN5P7eI8UpSQ==", "fileCount": 1049, "unpackedSize": 1425534}'
FROM packages WHERE name = 'lodash'
ON CONFLICT (package_id, version) DO NOTHING;

-- Versions for express
INSERT INTO package_versions (id, package_id, version, description, main, dependencies, dist) SELECT
    'c0000000-0000-0000-0000-000000000003', id, '4.21.0', 'Fast, unopinionated, minimalist web framework.', 'index.js',
    '{"accepts": "1.3.8", "array-flatten": "1.1.1", "body-parser": "1.20.3", "content-disposition": "0.5.4", "cookie-signature": "1.0.6"}',
    '{"tarball": "https://registry.npmjs.org/express/-/express-4.21.0.tgz", "integrity": "sha512-VQG3E2S6Iq3Pt++LkDOxin+HOpV6M9M7V6Yk28VhUHMGXRU3J4JGb9wj3Ud5aHzR+5G0UWOxlBQFD4uCLQBLVQ==", "fileCount": 56, "unpackedSize": 246784}'
FROM packages WHERE name = 'express'
ON CONFLICT (package_id, version) DO NOTHING;

INSERT INTO package_versions (id, package_id, version, description, main, dependencies, dist) SELECT
    'c0000000-0000-0000-0000-000000000004', id, '4.20.0', 'Fast, unopinionated, minimalist web framework.', 'index.js',
    '{"accepts": "1.3.8", "array-flatten": "1.1.1", "body-parser": "1.20.2"}',
    '{"tarball": "https://registry.npmjs.org/express/-/express-4.20.0.tgz", "integrity": "sha512-0hTvE3e1N4Bfbj3S22mNhLxPq0Y4sQ0SMH5w1OBK9I4Nl3FG4oCN2pGOIeMrKSpRFQIDeBy6j9sChPK5fyvVow==", "fileCount": 56, "unpackedSize": 245760}'
FROM packages WHERE name = 'express'
ON CONFLICT (package_id, version) DO NOTHING;

-- Versions for react
INSERT INTO package_versions (id, package_id, version, description, main, dist) SELECT
    'c0000000-0000-0000-0000-000000000005', id, '19.0.0', 'React is a JavaScript library for building user interfaces.', 'index.js',
    '{"tarball": "https://registry.npmjs.org/react/-/react-19.0.0.tgz", "integrity": "sha512-V8AVnmE0LJSQkY5e1tPa0E+HHhU7Wmwn8+L65GyRBQ1s1E/4lQ+TnI9hBL0ckPBYCoLPC8r1lF8M0q1xWx3c6A==", "fileCount": 19, "unpackedSize": 352256}'
FROM packages WHERE name = 'react'
ON CONFLICT (package_id, version) DO NOTHING;

INSERT INTO package_versions (id, package_id, version, description, main, dist) SELECT
    'c0000000-0000-0000-0000-000000000006', id, '18.3.1', 'React is a JavaScript library for building user interfaces.', 'index.js',
    '{"tarball": "https://registry.npmjs.org/react/-/react-18.3.1.tgz", "integrity": "sha512-wS+hAgJShR0KhEvPJArfuPVN1+Hz1t0Y6n5jLrGQbkb4urgPE/0Rve+1kMB1v/oWgHgm4WIcV+7FhNT5E3f9zA==", "fileCount": 18, "unpackedSize": 344064}'
FROM packages WHERE name = 'react'
ON CONFLICT (package_id, version) DO NOTHING;

-- Versions for typescript
INSERT INTO package_versions (id, package_id, version, description, main, dist) SELECT
    'c0000000-0000-0000-0000-000000000007', id, '5.6.0', 'TypeScript is a superset of JavaScript that compiles to clean JavaScript output.', 'lib/typescript.js',
    '{"tarball": "https://registry.npmjs.org/typescript/-/typescript-5.6.0.tgz", "integrity": "sha512-0WkBNfPV8Wgrz1C3G6UHqdV6LtfL6O3o0vR85sHh6fFb7LoPJIJaqDh6E9HQNwSg2hMB7DXw0n3rBr7L8Uc3oQ==", "fileCount": 584, "unpackedSize": 4280320}'
FROM packages WHERE name = 'typescript'
ON CONFLICT (package_id, version) DO NOTHING;

INSERT INTO package_versions (id, package_id, version, description, main, dist) SELECT
    'c0000000-0000-0000-0000-000000000008', id, '5.5.4', 'TypeScript is a superset of JavaScript that compiles to clean JavaScript output.', 'lib/typescript.js',
    '{"tarball": "https://registry.npmjs.org/typescript/-/typescript-5.5.4.tgz", "integrity": "sha512-0kIIkO6s1Vx0WNE7w59fD5D0A8HbqXOm3Q7NCxHDw4H6+QjE6D7rMRoZjs+9P0R9sB5UyTHjXAK0VgPeHegUbQ==", "fileCount": 580, "unpackedSize": 4259840}'
FROM packages WHERE name = 'typescript'
ON CONFLICT (package_id, version) DO NOTHING;

-- Versions for vite
INSERT INTO package_versions (id, package_id, version, description, main, dependencies, dist) SELECT
    'c0000000-0000-0000-0000-000000000009', id, '6.0.0', 'Native-ESM powered web dev build tool.', 'dist/node/index.js',
    '{"esbuild": "^0.24.0", "postcss": "^8.4.0", "rollup": "^4.0.0"}',
    '{"tarball": "https://registry.npmjs.org/vite/-/vite-6.0.0.tgz", "integrity": "sha512-Q2Z6H7kYQaO+eB23W5r/KWlx6zBEDpPnwNpN6xZ38/nVQNUSx4tMFHRYAw14xQxW73xrThGEQ4IY9QrVV3SxNw==", "fileCount": 212, "unpackedSize": 5898240}'
FROM packages WHERE name = 'vite'
ON CONFLICT (package_id, version) DO NOTHING;

INSERT INTO package_versions (id, package_id, version, description, main, dependencies, dist) SELECT
    'c0000000-0000-0000-0000-000000000010', id, '5.4.0', 'Native-ESM powered web dev build tool.', 'dist/node/index.js',
    '{"esbuild": "^0.21.0", "postcss": "^8.4.0", "rollup": "^4.0.0"}',
    '{"tarball": "https://registry.npmjs.org/vite/-/vite-5.4.0.tgz", "integrity": "sha512-4xGJtNt/K3lB2JX7NqS0bA7lQYovGXCaRAu+kHnL0mPnQb38C9oF68Qq5+R3bwA1bp6xyDrQqUKhjPEqU1Y9cg==", "fileCount": 208, "unpackedSize": 5767168}'
FROM packages WHERE name = 'vite'
ON CONFLICT (package_id, version) DO NOTHING;

-- Versions for next
INSERT INTO package_versions (id, package_id, version, description, main, dependencies, dist) SELECT
    'c0000000-0000-0000-0000-000000000011', id, '15.0.0', 'The React Framework for Production.', 'dist/server/next.js',
    '{"react": "^19.0.0", "react-dom": "^19.0.0", "styled-jsx": "5.1.6", "sharp": "^0.33.0"}',
    '{"tarball": "https://registry.npmjs.org/next/-/next-15.0.0.tgz", "integrity": "sha512-/uq7aLHyGZQ0wRKBXwWfxNmHY1y67gjPMACRFK12Pr0l8iAg5x4rsYFuM5fA+YmtJMfYV03L0JY0cMSy4Fi3Ew==", "fileCount": 1248, "unpackedSize": 16777216}'
FROM packages WHERE name = 'next'
ON CONFLICT (package_id, version) DO NOTHING;

-- Versions for bun
INSERT INTO package_versions (id, package_id, version, description, main, dist) SELECT
    'c0000000-0000-0000-0000-000000000012', id, '1.2.0', 'Bun is a fast all-in-one JavaScript runtime.', 'dist/bun.js',
    '{"tarball": "https://registry.npmjs.org/bun/-/bun-1.2.0.tgz", "integrity": "sha512-vWJ4p7GOPfHk6jnxRmEBJgFSOK7vxMNYWSN2+QfCbJ3bFn3o1yP83zIp8HURoMGlrKqISyJJw/RDniTyYcnxbA==", "fileCount": 36, "unpackedSize": 31457280}'
FROM packages WHERE name = 'bun'
ON CONFLICT (package_id, version) DO NOTHING;

-- Downloads (sample download records)
INSERT INTO downloads (id, package_id, version_id, ip_address, user_agent, country_code, downloaded_at) SELECT
    gen_random_uuid(), p.id, pv.id, '192.168.1.1', 'npm/10.0.0 node/v20.0.0', 'US',
    NOW() - (random() * INTERVAL '30 days')
FROM packages p JOIN package_versions pv ON pv.package_id = p.id
WHERE p.name = 'lodash' AND pv.version = '4.17.21'
ON CONFLICT DO NOTHING;

INSERT INTO downloads (id, package_id, version_id, ip_address, user_agent, country_code, downloaded_at) SELECT
    gen_random_uuid(), p.id, pv.id, '10.0.0.1', 'yarn/1.22.0', 'GB',
    NOW() - (random() * INTERVAL '30 days')
FROM packages p JOIN package_versions pv ON pv.package_id = p.id
WHERE p.name = 'lodash' AND pv.version = '4.17.21'
ON CONFLICT DO NOTHING;

INSERT INTO downloads (id, package_id, version_id, ip_address, user_agent, country_code, downloaded_at) SELECT
    gen_random_uuid(), p.id, pv.id, '172.16.0.1', 'bun/1.0.0', 'DE',
    NOW() - (random() * INTERVAL '30 days')
FROM packages p JOIN package_versions pv ON pv.package_id = p.id
WHERE p.name = 'express' AND pv.version = '4.21.0'
ON CONFLICT DO NOTHING;

INSERT INTO downloads (id, package_id, version_id, ip_address, user_agent, country_code, downloaded_at) SELECT
    gen_random_uuid(), p.id, pv.id, '203.0.113.1', 'npm/11.0.0', 'JP',
    NOW() - (random() * INTERVAL '30 days')
FROM packages p JOIN package_versions pv ON pv.package_id = p.id
WHERE p.name = 'typescript' AND pv.version = '5.6.0'
ON CONFLICT DO NOTHING;

INSERT INTO downloads (id, package_id, version_id, ip_address, user_agent, country_code, downloaded_at) SELECT
    gen_random_uuid(), p.id, pv.id, '198.51.100.1', 'pnpm/9.0.0', 'FR',
    NOW() - (random() * INTERVAL '30 days')
FROM packages p JOIN package_versions pv ON pv.package_id = p.id
WHERE p.name = 'react' AND pv.version = '19.0.0'
ON CONFLICT DO NOTHING;

-- Audit log entries
INSERT INTO audit_log (id, actor_id, action, resource_type, resource_id, details, ip_address, created_at) VALUES
    (gen_random_uuid(), 'a0000000-0000-0000-0000-000000000001', 'package:publish', 'package',
     (SELECT id FROM packages WHERE name = 'lodash'), '{"version": "4.17.21"}', '192.168.1.100', NOW() - INTERVAL '25 days'),
    (gen_random_uuid(), 'a0000000-0000-0000-0000-000000000002', 'package:publish', 'package',
     (SELECT id FROM packages WHERE name = 'express'), '{"version": "4.21.0"}', '10.0.0.50', NOW() - INTERVAL '20 days'),
    (gen_random_uuid(), 'a0000000-0000-0000-0000-000000000003', 'package:publish', 'package',
     (SELECT id FROM packages WHERE name = 'react'), '{"version": "19.0.0"}', '172.16.0.10', NOW() - INTERVAL '15 days'),
    (gen_random_uuid(), 'a0000000-0000-0000-0000-000000000001', 'package:update', 'package',
     (SELECT id FROM packages WHERE name = 'typescript'), '{"version": "5.6.0"}', '192.168.1.100', NOW() - INTERVAL '10 days'),
    (gen_random_uuid(), 'a0000000-0000-0000-0000-000000000004', 'package:publish', 'package',
     (SELECT id FROM packages WHERE name = 'vite'), '{"version": "6.0.0"}', '203.0.113.50', NOW() - INTERVAL '5 days')
ON CONFLICT DO NOTHING;
