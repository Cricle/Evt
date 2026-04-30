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

CREATE TABLE IF NOT EXISTS post_collections (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  post_id BIGINT NOT NULL,
  user_id BIGINT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_post_collections_post_id FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE,
  CONSTRAINT fk_post_collections_user_id FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT uq_post_collections_user_post UNIQUE KEY (user_id, post_id),
  INDEX idx_post_collections_post_id (post_id, id),
  INDEX idx_post_collections_user_id (user_id, id)
);
