import { describe, expect, it } from 'vitest';

import { usePagination } from '@/composables/usePagination';

describe('usePagination', () => {
  it('resets and advances pages consistently', () => {
    const pagination = usePagination(30);
    let calls = 0;

    expect(pagination.page.value).toBe(1);
    expect(pagination.pageSize.value).toBe(30);

    pagination.nextPage(() => {
      calls += 1;
    });
    expect(pagination.page.value).toBe(2);
    expect(calls).toBe(1);

    pagination.totalPage.value = 2;
    pagination.nextPage(() => {
      calls += 1;
    });
    expect(pagination.noMore.value).toBe(true);
    expect(calls).toBe(1);

    pagination.reset();
    expect(pagination.page.value).toBe(1);
    expect(pagination.totalPage.value).toBe(0);
    expect(pagination.noMore.value).toBe(false);
  });
});
