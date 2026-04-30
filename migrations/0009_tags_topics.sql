ALTER TABLE posts
  ADD COLUMN tags VARCHAR(255) NOT NULL DEFAULT '';

CREATE TABLE IF NOT EXISTS tags (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  user_id BIGINT NOT NULL,
  tag VARCHAR(255) NOT NULL,
  quote_num BIGINT NOT NULL DEFAULT 0,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  CONSTRAINT fk_tags_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT uq_tags_tag UNIQUE KEY (tag),
  INDEX idx_tags_quote_num (quote_num, id),
  INDEX idx_tags_created_at (created_at, id),
  INDEX idx_tags_user_id (user_id)
);

CREATE TABLE IF NOT EXISTS topic_users (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  user_id BIGINT NOT NULL,
  tag_id BIGINT NOT NULL,
  is_top BOOLEAN NOT NULL DEFAULT FALSE,
  is_pin BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  CONSTRAINT fk_topic_users_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT fk_topic_users_tag_id FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE,
  CONSTRAINT uq_topic_users_user_tag UNIQUE KEY (user_id, tag_id),
  INDEX idx_topic_users_user_id (user_id, is_top, is_pin, id),
  INDEX idx_topic_users_tag_id (tag_id)
);
