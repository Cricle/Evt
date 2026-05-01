import { describe, expect, it } from 'vitest';

import { mergeCommentPage, nextCommentPage } from '@/utils/comment-feed';

describe('comment feed utils', () => {
  it('replaces the list on first page and appends later pages', () => {
    expect(mergeCommentPage([1, 2], [3, 4], 1)).toEqual([3, 4]);
    expect(mergeCommentPage([1, 2], [3, 4], 2)).toEqual([1, 2, 3, 4]);
    expect(mergeCommentPage([1, 2], [], 2)).toEqual([1, 2]);
  });

  it('only advances comment page when a full page is returned', () => {
    expect(nextCommentPage(1, 20, 20)).toBe(2);
    expect(nextCommentPage(2, 20, 20)).toBe(3);
    expect(nextCommentPage(2, 5, 20)).toBe(2);
  });
});
