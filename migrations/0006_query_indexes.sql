CREATE INDEX idx_posts_user_id_id ON posts (user_id, id);

CREATE INDEX idx_comments_post_id_id ON comments (post_id, id);

CREATE INDEX idx_follows_followee_id_id ON follows (followee_id, id);

CREATE INDEX idx_messages_receiver_user_id_id ON messages (receiver_user_id, id);

CREATE INDEX idx_messages_receiver_user_id_is_read ON messages (receiver_user_id, is_read);
