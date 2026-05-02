import { describe, expect, it } from 'vitest';

import { usePostContent } from '@/composables/usePostContent';

const buildPost = (): Item.PostProps => ({
  id: 1,
  user_id: 1,
  user: {
    id: 1,
    username: 'alice',
    nickname: 'Alice',
    avatar: '',
    phone: '',
    activation: '',
    is_admin: false,
    is_friend: false,
    is_following: false,
    created_on: 0,
    follows: 0,
    followings: 0,
    tweets_count: 0,
    balance: 0,
    status: 1,
  },
  attachment_price: 0,
  ip_loc: '',
  latest_replied_on: 0,
  created_on: 0,
  upvote_count: 0,
  comment_count: 0,
  collection_count: 0,
  share_count: 0,
  contents: [],
  tags: '',
  visibility: 0,
  is_lock: 0,
  is_top: 0,
  is_essence: 0,
  reactions: [],
});

describe('usePostContent', () => {
  it('syncs reaction groups back to the source post when the computed value is updated', () => {
    const source = buildPost();
    const model = usePostContent(source, true);
    const reactions = [{ emoji: '🔥', count: 2, users: [source.user] }];

    model.value = {
      ...model.value,
      reactions,
      upvote_count: 2,
      comment_count: 5,
    };

    expect(source.reactions).toEqual(reactions);
    expect(source.upvote_count).toBe(2);
    expect(source.comment_count).toBe(5);
  });
});
