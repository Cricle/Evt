import { describe, expect, it } from 'vitest';

import { prettyQuoteNum } from '@/utils/count';

describe('count utils', () => {
  it('formats quote counts using the expected thresholds', () => {
    expect(prettyQuoteNum(999)).toBe(999);
    expect(prettyQuoteNum(1_000)).toBe('1.0千');
    expect(prettyQuoteNum(10_000)).toBe('1.0万');
  });
});
