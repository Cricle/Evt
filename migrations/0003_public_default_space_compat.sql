UPDATE spaces
SET
  name = '公共广场',
  description = '所有成员默认加入的公共广场'
WHERE slug IN ('public', 'square');

INSERT INTO spaces (slug, name, description, owner_user_id, visibility)
SELECT 'public', '公共广场', '所有成员默认加入的公共广场', u.id, 0
FROM users u
WHERE NOT EXISTS (
  SELECT 1
  FROM spaces s
  WHERE s.slug IN ('public', 'square')
)
ORDER BY u.id ASC
LIMIT 1
ON DUPLICATE KEY UPDATE
  name = VALUES(name),
  description = VALUES(description),
  visibility = VALUES(visibility);

UPDATE site_settings
SET payload = JSON_SET(
  COALESCE(payload, JSON_OBJECT()),
  '$.default_space_slug',
  'public'
)
WHERE JSON_UNQUOTE(JSON_EXTRACT(COALESCE(payload, JSON_OBJECT()), '$.default_space_slug')) = 'square';
