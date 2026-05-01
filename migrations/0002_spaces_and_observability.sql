CREATE TABLE IF NOT EXISTS spaces (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  slug VARCHAR(64) NOT NULL,
  name VARCHAR(128) NOT NULL,
  description VARCHAR(512) NOT NULL DEFAULT '',
  owner_user_id BIGINT NOT NULL,
  visibility TINYINT NOT NULL DEFAULT 0,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  CONSTRAINT fk_spaces_owner_user_id FOREIGN KEY (owner_user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT uq_spaces_slug UNIQUE KEY (slug),
  INDEX idx_spaces_owner_user_id (owner_user_id),
  INDEX idx_spaces_created_at (created_at, id)
);

CREATE TABLE IF NOT EXISTS space_members (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  space_id BIGINT NOT NULL,
  user_id BIGINT NOT NULL,
  role TINYINT NOT NULL,
  invited_by_user_id BIGINT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  CONSTRAINT fk_space_members_space_id FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE,
  CONSTRAINT fk_space_members_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT fk_space_members_invited_by_user_id FOREIGN KEY (invited_by_user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT uq_space_members_pair UNIQUE KEY (space_id, user_id),
  INDEX idx_space_members_user_id (user_id, role),
  INDEX idx_space_members_space_role (space_id, role, id)
);

ALTER TABLE posts
  ADD COLUMN space_id BIGINT NOT NULL DEFAULT 0 AFTER user_id,
  ADD INDEX idx_posts_space_id_id (space_id, id),
  ADD INDEX idx_posts_space_user_id_id (space_id, user_id, id);

ALTER TABLE comments
  ADD INDEX idx_comments_post_user_created_at (post_id, user_id, created_at, id);

ALTER TABLE comment_replies
  ADD INDEX idx_comment_replies_comment_created_at (comment_id, created_at, id);

ALTER TABLE tags
  ADD COLUMN space_id BIGINT NOT NULL DEFAULT 0 AFTER user_id,
  ADD CONSTRAINT uq_tags_space_tag UNIQUE KEY (space_id, tag),
  ADD INDEX idx_tags_space_quote_num (space_id, quote_num, id),
  ADD INDEX idx_tags_space_created_at (space_id, created_at, id);

ALTER TABLE topic_users
  ADD COLUMN space_id BIGINT NOT NULL DEFAULT 0 AFTER user_id,
  ADD CONSTRAINT uq_topic_users_space_user_tag UNIQUE KEY (space_id, user_id, tag_id),
  ADD INDEX idx_topic_users_space_user_id (space_id, user_id, is_top, is_pin, id);

CREATE TABLE IF NOT EXISTS search_posts (
  post_id BIGINT NOT NULL PRIMARY KEY,
  space_id BIGINT NOT NULL,
  user_id BIGINT NOT NULL,
  username VARCHAR(64) NOT NULL,
  nickname VARCHAR(64) NOT NULL DEFAULT '',
  content_text MEDIUMTEXT NOT NULL,
  tags_text VARCHAR(512) NOT NULL DEFAULT '',
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  FULLTEXT KEY ft_search_posts_content (content_text, tags_text, username, nickname),
  INDEX idx_search_posts_space_created_at (space_id, created_at, post_id),
  INDEX idx_search_posts_user_id (user_id)
);

INSERT INTO spaces (id, slug, name, description, owner_user_id, visibility)
SELECT 1, 'public', '公共广场', '所有成员默认加入的公共广场', u.id, 0
FROM users u
ORDER BY u.id ASC
LIMIT 1
ON DUPLICATE KEY UPDATE
  name = VALUES(name),
  description = VALUES(description);

UPDATE posts
SET space_id = 1
WHERE space_id = 0;

UPDATE tags
SET space_id = 1
WHERE space_id = 0;

UPDATE topic_users
SET space_id = 1
WHERE space_id = 0;

INSERT INTO space_members (space_id, user_id, role, invited_by_user_id)
SELECT 1, s.owner_user_id, 2, s.owner_user_id
FROM spaces s
WHERE s.id = 1
ON DUPLICATE KEY UPDATE
  role = VALUES(role),
  invited_by_user_id = VALUES(invited_by_user_id);

INSERT INTO search_posts (post_id, space_id, user_id, username, nickname, content_text, tags_text, created_at)
SELECT
  p.id,
  p.space_id,
  p.user_id,
  u.username,
  COALESCE(up.nickname, u.username) AS nickname,
  COALESCE(
    NULLIF(
      GROUP_CONCAT(
        CASE
          WHEN pc.content_type IN (1, 2, 6) THEN pc.content
          ELSE NULL
        END
        ORDER BY pc.sort_order ASC, pc.id ASC
        SEPARATOR ' '
      ),
      ''
    ),
    p.content
  ) AS content_text,
  COALESCE(p.tags, '') AS tags_text,
  p.created_at
FROM posts p
INNER JOIN users u ON u.id = p.user_id
LEFT JOIN user_profiles up ON up.user_id = u.id
LEFT JOIN post_contents pc ON pc.post_id = p.id
GROUP BY p.id, p.space_id, p.user_id, u.username, up.nickname, p.content, p.tags, p.created_at
ON DUPLICATE KEY UPDATE
  space_id = VALUES(space_id),
  user_id = VALUES(user_id),
  username = VALUES(username),
  nickname = VALUES(nickname),
  content_text = VALUES(content_text),
  tags_text = VALUES(tags_text),
  created_at = VALUES(created_at);
