import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@/utils/request', () => ({
  Api: {
    v1: {
      user: {
        post: {
          follow: vi.fn(),
          unfollow: vi.fn(),
        },
      },
    },
  },
}));

import UserAction from '@/composables/useUserAction';
import { Api } from '@/utils/request';

describe('UserAction.followAction', () => {
  const success = vi.fn();
  const error = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    (globalThis as any).window = {
      $message: {
        success,
        error,
      },
    };
  });

  it('follows a user after confirming', async () => {
    const follow = vi.mocked(Api.v1.user.post.follow);
    follow.mockResolvedValue({});

    let onPositiveClick: (() => void | Promise<void>) | undefined;
    const dialog = {
      success(options: { onPositiveClick: () => void | Promise<void> }) {
        onPositiveClick = options.onPositiveClick;
      },
    };

    const promise = UserAction.followAction(dialog as never, 7, 'alice', false);
    await onPositiveClick?.();

    await expect(promise).resolves.toBe(true);
    expect(follow).toHaveBeenCalledWith({ user_id: 7 });
    expect(success).toHaveBeenCalledWith('关注成功');
    expect(error).not.toHaveBeenCalled();
  });

  it('shows an error when following fails', async () => {
    const follow = vi.mocked(Api.v1.user.post.follow);
    follow.mockRejectedValue(new Error('boom'));

    let onPositiveClick: (() => void | Promise<void>) | undefined;
    const dialog = {
      success(options: { onPositiveClick: () => void | Promise<void> }) {
        onPositiveClick = options.onPositiveClick;
      },
    };

    const promise = UserAction.followAction(dialog as never, 7, 'alice', false);
    await onPositiveClick?.();

    await expect(promise).rejects.toThrow('boom');
    expect(error).toHaveBeenCalledWith('关注失败');
  });

  it('unfollows a user after confirming', async () => {
    const unfollow = vi.mocked(Api.v1.user.post.unfollow);
    unfollow.mockResolvedValue({});

    let onPositiveClick: (() => void | Promise<void>) | undefined;
    const dialog = {
      success(options: { onPositiveClick: () => void | Promise<void> }) {
        onPositiveClick = options.onPositiveClick;
      },
    };

    const promise = UserAction.followAction(dialog as never, 8, 'bob', true);
    await onPositiveClick?.();

    await expect(promise).resolves.toBe(false);
    expect(unfollow).toHaveBeenCalledWith({ user_id: 8 });
    expect(success).toHaveBeenCalledWith('操作成功');
    expect(error).not.toHaveBeenCalled();
  });
});
