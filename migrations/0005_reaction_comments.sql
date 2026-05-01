ALTER TABLE legacy_comment_states
  ADD COLUMN is_reaction BOOLEAN NOT NULL DEFAULT FALSE AFTER is_essence;

CREATE INDEX idx_legacy_comment_states_is_reaction
  ON legacy_comment_states (is_reaction, comment_id);
