import { describe, expect, it } from 'vitest';

import { normalizeDefaultSpaceSlug, resolveSpaceSlug } from '@/utils/spaces';

describe('spaces utils', () => {
  it('normalizes the default space slug and legacy alias', () => {
    expect(normalizeDefaultSpaceSlug()).toBe('public');
    expect(normalizeDefaultSpaceSlug('square')).toBe('public');
    expect(normalizeDefaultSpaceSlug(' Team-Alpha ')).toBe('team-alpha');
  });

  it('resolves the current space slug with fallback semantics', () => {
    expect(resolveSpaceSlug('square', 'team-alpha')).toBe('team-alpha');
    expect(resolveSpaceSlug('', 'square')).toBe('public');
    expect(resolveSpaceSlug('private-lab', 'public')).toBe('private-lab');
  });
});
