import { ref } from 'vue';
import { describe, expect, it, vi } from 'vitest';

import {
  backWithFallback,
  canUseHistoryBack,
  normalizeResolvedHref,
  pushWithFallback,
} from '@/utils/navigation';

describe('navigation helpers', () => {
  it('normalizes plain router paths into hash navigation targets', () => {
    expect(normalizeResolvedHref('/#/compose?space=public', '/compose?space=public')).toBe(
      '/#/compose?space=public',
    );
    expect(normalizeResolvedHref('/compose?space=public', '/compose?space=public')).toBe(
      '#/compose?space=public',
    );
  });

  it('falls back to location.assign when router state did not advance', async () => {
    const assign = vi.fn();
    const router = {
      back: vi.fn(),
      currentRoute: ref({ fullPath: '/' }),
      resolve: () => ({
        href: '/#/compose?space=public',
        fullPath: '/compose?space=public',
      }),
      push: vi.fn().mockResolvedValue(undefined),
    };

    await pushWithFallback(router as never, { name: 'compose' }, { assign });

    expect(router.push).toHaveBeenCalled();
    expect(assign).toHaveBeenCalledWith('/#/compose?space=public');
  });

  it('does not use the fallback once the router reaches the target route', async () => {
    const assign = vi.fn();
    const router = {
      back: vi.fn(),
      currentRoute: ref({ fullPath: '/' }),
      resolve: () => ({
        href: '/#/compose?space=public',
        fullPath: '/compose?space=public',
      }),
      push: vi.fn().mockImplementation(async () => {
        router.currentRoute.value = { fullPath: '/compose?space=public' };
      }),
    };

    await pushWithFallback(router as never, { name: 'compose' }, { assign });

    expect(assign).not.toHaveBeenCalled();
  });

  it('detects when browser history can go back inside the app', () => {
    expect(canUseHistoryBack({ back: '/compose?space=public' })).toBe(true);
    expect(canUseHistoryBack({ back: '' })).toBe(false);
    expect(canUseHistoryBack(null)).toBe(false);
  });

  it('uses router.back when history state points to a previous route', async () => {
    const assign = vi.fn();
    const router = {
      back: vi.fn(),
      currentRoute: ref({ fullPath: '/post?id=1&space=public' }),
      resolve: () => ({
        href: '/#/home?space=public',
        fullPath: '/?space=public',
      }),
      push: vi.fn().mockResolvedValue(undefined),
    };

    await backWithFallback(
      router as never,
      { name: 'home', query: { space: 'public' } },
      { assign },
      { back: '/compose?space=public' },
    );

    expect(router.back).toHaveBeenCalledOnce();
    expect(router.push).not.toHaveBeenCalled();
    expect(assign).not.toHaveBeenCalled();
  });

  it('falls back to pushing the home route when there is no in-app history', async () => {
    const assign = vi.fn();
    const router = {
      back: vi.fn(),
      currentRoute: ref({ fullPath: '/post?id=1&space=public' }),
      resolve: () => ({
        href: '/#/home?space=public',
        fullPath: '/?space=public',
      }),
      push: vi.fn().mockResolvedValue(undefined),
    };

    await backWithFallback(
      router as never,
      { name: 'home', query: { space: 'public' } },
      { assign },
      null,
    );

    expect(router.back).not.toHaveBeenCalled();
    expect(router.push).toHaveBeenCalledOnce();
    expect(assign).toHaveBeenCalledWith('/#/home?space=public');
  });
});
