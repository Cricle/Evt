import { ref } from 'vue';
import { describe, expect, it, vi } from 'vitest';

import { normalizeResolvedHref, pushWithFallback } from '@/utils/navigation';

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
});
