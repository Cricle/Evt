import { describe, expect, it } from 'vitest';

import { goToAuth } from '@/utils/authRoute';

describe('auth route utils', () => {
  it('pushes the auth route with mode and redirect query', async () => {
    let pushed: unknown;
    const router = {
      push(payload: unknown) {
        pushed = payload;
        return Promise.resolve(payload);
      },
    };

    await goToAuth(router as never, 'signup', '/#/compose?space=public');

    expect(pushed).toEqual({
      name: 'auth',
      query: {
        mode: 'signup',
        redirect: '/#/compose?space=public',
      },
    });
  });

  it('omits the redirect query when it is not provided', async () => {
    let pushed: unknown;
    const router = {
      push(payload: unknown) {
        pushed = payload;
        return Promise.resolve(payload);
      },
    };

    await goToAuth(router as never, 'signin');

    expect(pushed).toEqual({
      name: 'auth',
      query: {
        mode: 'signin',
      },
    });
  });
});
