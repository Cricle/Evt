CREATE TABLE IF NOT EXISTS users (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  username VARCHAR(64) NOT NULL,
  phone_number VARCHAR(32) NULL,
  password_hash VARCHAR(255) NULL,
  status VARCHAR(16) NOT NULL DEFAULT 'active',
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  UNIQUE KEY uq_users_username (username),
  UNIQUE KEY uq_users_phone_number (phone_number)
);

CREATE TABLE IF NOT EXISTS site_settings (
  id BIGINT NOT NULL PRIMARY KEY,
  payload JSON NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

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

CREATE TABLE IF NOT EXISTS posts (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  user_id BIGINT NOT NULL,
  space_id BIGINT NOT NULL DEFAULT 0,
  content TEXT NOT NULL,
  tags VARCHAR(255) NOT NULL DEFAULT '',
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  CONSTRAINT fk_posts_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  INDEX idx_posts_user_id (user_id),
  INDEX idx_posts_created_at (created_at),
  INDEX idx_posts_space_id_id (space_id, id),
  INDEX idx_posts_space_user_id_id (space_id, user_id, id)
);

CREATE TABLE IF NOT EXISTS comments (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  post_id BIGINT NOT NULL,
  user_id BIGINT NOT NULL,
  content TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  CONSTRAINT fk_comments_post_id FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE,
  CONSTRAINT fk_comments_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  INDEX idx_comments_post_id (post_id),
  INDEX idx_comments_user_id (user_id),
  INDEX idx_comments_created_at (created_at),
  INDEX idx_comments_post_user_created_at (post_id, user_id, created_at, id)
);

CREATE TABLE IF NOT EXISTS follows (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  follower_id BIGINT NOT NULL,
  followee_id BIGINT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_follows_follower_id FOREIGN KEY (follower_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT fk_follows_followee_id FOREIGN KEY (followee_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT uq_follows_pair UNIQUE KEY (follower_id, followee_id),
  INDEX idx_follows_follower_id (follower_id),
  INDEX idx_follows_followee_id (followee_id),
  INDEX idx_follows_followee_id_id (followee_id, id)
);

CREATE TABLE IF NOT EXISTS attachments (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  user_id BIGINT NOT NULL,
  file_name VARCHAR(255) NOT NULL,
  content_type VARCHAR(255) NOT NULL,
  size_bytes BIGINT NOT NULL,
  storage_key VARCHAR(255) NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_attachments_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT uq_attachments_storage_key UNIQUE KEY (storage_key),
  INDEX idx_attachments_user_id (user_id)
);

CREATE TABLE IF NOT EXISTS messages (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  sender_user_id BIGINT NOT NULL,
  receiver_user_id BIGINT NOT NULL,
  type TINYINT NOT NULL DEFAULT 4,
  brief VARCHAR(255) NOT NULL DEFAULT '',
  content TEXT NOT NULL,
  post_id BIGINT NOT NULL DEFAULT 0,
  comment_id BIGINT NOT NULL DEFAULT 0,
  reply_id BIGINT NOT NULL DEFAULT 0,
  is_read BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_messages_sender_user_id FOREIGN KEY (sender_user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT fk_messages_receiver_user_id FOREIGN KEY (receiver_user_id) REFERENCES users(id) ON DELETE CASCADE,
  INDEX idx_messages_receiver_user_id (receiver_user_id),
  INDEX idx_messages_sender_user_id (sender_user_id),
  INDEX idx_messages_created_at (created_at),
  INDEX idx_messages_receiver_user_id_id (receiver_user_id, id),
  INDEX idx_messages_receiver_user_id_is_read (receiver_user_id, is_read)
);

CREATE TABLE IF NOT EXISTS post_contents (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  post_id BIGINT NOT NULL,
  user_id BIGINT NOT NULL,
  content_type INT NOT NULL,
  content TEXT NOT NULL,
  sort_order BIGINT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  CONSTRAINT fk_post_contents_post_id FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE,
  CONSTRAINT fk_post_contents_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  INDEX idx_post_contents_post_id_sort_order (post_id, sort_order, id),
  INDEX idx_post_contents_user_id (user_id)
);

CREATE TABLE IF NOT EXISTS comment_contents (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  comment_id BIGINT NOT NULL,
  user_id BIGINT NOT NULL,
  content_type INT NOT NULL,
  content TEXT NOT NULL,
  sort_order BIGINT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  CONSTRAINT fk_comment_contents_comment_id FOREIGN KEY (comment_id) REFERENCES comments(id) ON DELETE CASCADE,
  CONSTRAINT fk_comment_contents_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  INDEX idx_comment_contents_comment_id_sort_order (comment_id, sort_order, id),
  INDEX idx_comment_contents_user_id (user_id)
);

CREATE TABLE IF NOT EXISTS post_stars (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  post_id BIGINT NOT NULL,
  user_id BIGINT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_post_stars_post_id FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE,
  CONSTRAINT fk_post_stars_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT uq_post_stars_user_post UNIQUE KEY (user_id, post_id),
  INDEX idx_post_stars_post_id (post_id, id),
  INDEX idx_post_stars_user_id (user_id, id)
);

CREATE TABLE IF NOT EXISTS tags (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  user_id BIGINT NOT NULL,
  space_id BIGINT NOT NULL DEFAULT 0,
  tag VARCHAR(255) NOT NULL,
  quote_num BIGINT NOT NULL DEFAULT 0,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  CONSTRAINT fk_tags_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT uq_tags_space_tag UNIQUE KEY (space_id, tag),
  INDEX idx_tags_space_quote_num (space_id, quote_num, id),
  INDEX idx_tags_space_created_at (space_id, created_at, id),
  INDEX idx_tags_user_id (user_id)
);

CREATE TABLE IF NOT EXISTS topic_users (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  user_id BIGINT NOT NULL,
  space_id BIGINT NOT NULL DEFAULT 0,
  tag_id BIGINT NOT NULL,
  is_top BOOLEAN NOT NULL DEFAULT FALSE,
  is_pin BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  CONSTRAINT fk_topic_users_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT fk_topic_users_tag_id FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
  CONSTRAINT uq_topic_users_space_user_tag UNIQUE KEY (space_id, user_id, tag_id),
  INDEX idx_topic_users_space_user_id (space_id, user_id, is_top, is_pin, id),
  INDEX idx_topic_users_tag_id (tag_id)
);

CREATE TABLE IF NOT EXISTS user_profiles (
  user_id BIGINT NOT NULL PRIMARY KEY,
  nickname VARCHAR(64) NOT NULL,
  avatar VARCHAR(255) NOT NULL DEFAULT '',
  activation_code VARCHAR(64) NOT NULL DEFAULT '',
  is_admin BOOLEAN NOT NULL DEFAULT FALSE,
  balance BIGINT NOT NULL DEFAULT 0,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  CONSTRAINT fk_user_profiles_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS legacy_post_states (
  post_id BIGINT NOT NULL PRIMARY KEY,
  attachment_price BIGINT NOT NULL DEFAULT 0,
  visibility INT NOT NULL DEFAULT 0,
  is_lock BOOLEAN NOT NULL DEFAULT FALSE,
  is_top BOOLEAN NOT NULL DEFAULT FALSE,
  is_essence BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  CONSTRAINT fk_legacy_post_states_post_id FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS legacy_comment_states (
  comment_id BIGINT NOT NULL PRIMARY KEY,
  is_essence BOOLEAN NOT NULL DEFAULT FALSE,
  is_reaction BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  CONSTRAINT fk_legacy_comment_states_comment_id FOREIGN KEY (comment_id) REFERENCES comments(id) ON DELETE CASCADE,
  INDEX idx_legacy_comment_states_is_reaction (is_reaction, comment_id)
);

CREATE TABLE IF NOT EXISTS comment_reactions (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  user_id BIGINT NOT NULL,
  post_id BIGINT NOT NULL,
  comment_id BIGINT NOT NULL,
  is_thumbs_up BOOLEAN NOT NULL DEFAULT FALSE,
  is_thumbs_down BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  CONSTRAINT fk_comment_reactions_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT uq_comment_reactions_target UNIQUE KEY (user_id, comment_id),
  INDEX idx_comment_reactions_comment_id (comment_id)
);

CREATE TABLE IF NOT EXISTS friendships (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  user_id BIGINT NOT NULL,
  friend_id BIGINT NOT NULL,
  status TINYINT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  CONSTRAINT fk_friendships_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT fk_friendships_friend_id FOREIGN KEY (friend_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT uq_friendships_pair UNIQUE KEY (user_id, friend_id),
  INDEX idx_friendships_friend_id (friend_id)
);

CREATE TABLE IF NOT EXISTS wallet_recharges (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  user_id BIGINT NOT NULL,
  amount BIGINT NOT NULL,
  trade_no VARCHAR(128) NOT NULL DEFAULT '',
  trade_status VARCHAR(64) NOT NULL DEFAULT 'PENDING',
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  CONSTRAINT fk_wallet_recharges_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  INDEX idx_wallet_recharges_user_id (user_id)
);

CREATE TABLE IF NOT EXISTS wallet_statements (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  user_id BIGINT NOT NULL,
  change_amount BIGINT NOT NULL,
  balance_snapshot BIGINT NOT NULL,
  reason VARCHAR(255) NOT NULL,
  post_id BIGINT NOT NULL DEFAULT 0,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_wallet_statements_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  INDEX idx_wallet_statements_user_id (user_id)
);

CREATE TABLE IF NOT EXISTS phone_captchas (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  phone VARCHAR(32) NOT NULL,
  captcha VARCHAR(16) NOT NULL,
  use_times INT NOT NULL DEFAULT 0,
  expired_at TIMESTAMP NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  INDEX idx_phone_captchas_phone_created_at (phone, created_at)
);

CREATE TABLE IF NOT EXISTS attachment_purchase_records (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  user_id BIGINT NOT NULL,
  post_id BIGINT NOT NULL,
  paid_amount BIGINT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_attachment_purchase_records_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT fk_attachment_purchase_records_post_id FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE,
  CONSTRAINT uq_attachment_purchase_records_user_post UNIQUE KEY (user_id, post_id),
  INDEX idx_attachment_purchase_records_post_id (post_id)
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
  description = VALUES(description),
  visibility = VALUES(visibility);

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
