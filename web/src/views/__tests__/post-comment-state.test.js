import { describe, expect, it } from 'vitest';
import { applyCommentPageState, createCommentPageState } from '@/views/post-comment-state';
describe('post comment state', () => {
    it('replaces first page results and advances when page is full', () => {
        const initial = createCommentPageState();
        expect(applyCommentPageState(initial, [3, 4], 1, 2)).toEqual({
            page: 2,
            noMore: false,
            items: [3, 4],
        });
    });
    it('appends later pages and marks no-more when page is partial', () => {
        const current = {
            page: 2,
            noMore: false,
            items: [1, 2],
        };
        expect(applyCommentPageState(current, [5], 2, 2)).toEqual({
            page: 2,
            noMore: true,
            items: [1, 2, 5],
        });
    });
    it('starts clean for a reload instead of keeping stale cached items', () => {
        expect(createCommentPageState()).toEqual({
            page: 1,
            noMore: false,
            items: [],
        });
    });
});
//# sourceMappingURL=post-comment-state.test.js.map