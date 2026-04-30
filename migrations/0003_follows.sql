CREATE TABLE IF NOT EXISTS follows (
  id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  follower_id BIGINT NOT NULL,
  followee_id BIGINT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT fk_follows_follower_id FOREIGN KEY (follower_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT fk_follows_followee_id FOREIGN KEY (followee_id) REFERENCES users(id) ON DELETE CASCADE,
  CONSTRAINT uq_follows_pair UNIQUE KEY (follower_id, followee_id),
  INDEX idx_follows_follower_id (follower_id),
  INDEX idx_follows_followee_id (followee_id)
);
