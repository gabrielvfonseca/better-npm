-- ============================================================================
-- VIEWS
-- ============================================================================

-- Package download counts by day
CREATE OR REPLACE VIEW daily_downloads AS
SELECT
    date_trunc('day', downloaded_at) AS day,
    package_id,
    p.name AS package_name,
    count(*) AS download_count
FROM downloads
JOIN packages p ON p.id = downloads.package_id
GROUP BY day, package_id, p.name
ORDER BY day DESC;

-- Package download totals
CREATE OR REPLACE VIEW total_downloads AS
SELECT
    p.id AS package_id,
    p.name AS package_name,
    COUNT(*) AS total_downloads
FROM packages p
JOIN downloads d ON d.package_id = p.id
GROUP BY p.id, p.name;

-- Package latest version view
CREATE OR REPLACE VIEW package_latest AS
SELECT DISTINCT ON (p.id)
    p.id,
    p.name,
    p.description,
    pv.version,
    pv.created_at AS version_created_at
FROM packages p
JOIN package_versions pv ON pv.package_id = p.id
ORDER BY p.id, string_to_array(pv.version, '.')::int[] DESC;

-- Active users (users who published in last 30 days)
CREATE OR REPLACE VIEW active_users AS
SELECT DISTINCT
    u.id,
    u.username,
    u.email,
    u.created_at
FROM users u
JOIN audit_log al ON al.actor_id = u.id
WHERE al.action IN ('package:publish', 'package:update')
  AND al.created_at > NOW() - INTERVAL '30 days';

-- User download stats
CREATE OR REPLACE VIEW user_download_stats AS
SELECT
    u.id,
    u.username,
    COUNT(d.id) AS total_downloads,
    COUNT(DISTINCT d.package_id) AS unique_packages,
    MAX(d.downloaded_at) AS last_download
FROM users u
LEFT JOIN packages p ON p.name = u.username
LEFT JOIN downloads d ON d.package_id = p.id
GROUP BY u.id, u.username;

-- Package summary with latest version and downloads
CREATE OR REPLACE VIEW package_summary AS
SELECT
    p.id,
    p.name,
    p.description,
    p.license,
    lv.version AS latest_version,
    lv.version_created_at,
    COALESCE(td.total_downloads, 0) AS total_downloads,
    COALESCE(dd30.download_30d, 0) AS downloads_last_30_days,
    p.created_at,
    p.updated_at
FROM packages p
LEFT JOIN package_latest lv ON lv.id = p.id
LEFT JOIN total_downloads td ON td.package_id = p.id
LEFT JOIN (
    SELECT package_id, SUM(download_count) AS download_30d
    FROM daily_downloads
    WHERE day >= CURRENT_DATE - INTERVAL '30 days'
    GROUP BY package_id
) dd30 ON dd30.package_id = p.id
WHERE p.is_deprecated = false;

-- Top packages by downloads (last 30 days)
CREATE OR REPLACE VIEW top_packages_30d AS
SELECT
    p.id,
    p.name,
    p.description,
    COALESCE(SUM(dd.download_count), 0) AS downloads
FROM packages p
LEFT JOIN daily_downloads dd ON dd.package_id = p.id
    AND dd.day >= CURRENT_DATE - INTERVAL '30 days'
WHERE p.is_deprecated = false
GROUP BY p.id, p.name, p.description
ORDER BY downloads DESC;

-- Recently updated packages
CREATE OR REPLACE VIEW recently_updated AS
SELECT DISTINCT ON (p.id)
    p.id,
    p.name,
    p.description,
    pv.version,
    pv.created_at AS version_published_at
FROM packages p
JOIN package_versions pv ON pv.package_id = p.id
WHERE p.is_deprecated = false AND pv.deprecated = false
ORDER BY p.id, pv.created_at DESC;

-- Package dependency graph (direct dependencies)
CREATE OR REPLACE VIEW package_dependencies AS
SELECT
    pv.id AS version_id,
    pv.package_id,
    p.name AS package_name,
    pv.version,
    dep.key AS dependency_name,
    dep.value AS dependency_version
FROM package_versions pv
JOIN packages p ON p.id = pv.package_id
CROSS JOIN LATERAL jsonb_each_text(pv.dependencies) AS dep;
