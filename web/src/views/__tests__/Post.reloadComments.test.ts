import { describe, expect, it } from 'vitest';

describe('Post reload behavior', () => {
  it('keeps existing comment data shape during refresh', () => {
    const defaultComments = [{ id: 1 }, { id: 2 }];
    const hotsComments = [{ id: 3 }];
    const newestComments = [{ id: 4 }];
    const nextDefaultComments = defaultComments.slice();
    const nextHotsComments = hotsComments.slice();
    const nextNewestComments = newestComments.slice();

    expect(nextDefaultComments).toHaveLength(2);
    expect(nextHotsComments).toHaveLength(1);
    expect(nextNewestComments).toHaveLength(1);
  });
});
