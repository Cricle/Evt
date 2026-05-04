import { describe, expect, it } from 'vitest';

import { EVENT_POST_TAG, isEventPost, normalizePostTags } from '@/utils/postKind';

describe('post kind utils', () => {
  it('normalizes string tags from comma separated payloads', () => {
    expect(normalizePostTags(`foo, ${EVENT_POST_TAG},Bar`)).toEqual(['foo', EVENT_POST_TAG, 'bar']);
  });

  it('normalizes object tags from post payloads', () => {
    expect(normalizePostTags({ Foo: 1, [EVENT_POST_TAG]: 2 })).toEqual(['foo', EVENT_POST_TAG]);
  });

  it('detects event posts by reserved tag', () => {
    expect(isEventPost({ tags: { [EVENT_POST_TAG]: 1 } })).toBe(true);
    expect(isEventPost({ tags: 'topic,news' })).toBe(false);
  });
});
