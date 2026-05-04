import { mergeCommentPage, nextCommentPage } from '@/utils/comment-feed';
export const createCommentPageState = () => ({
    page: 1,
    noMore: false,
    items: [],
});
export const applyCommentPageState = (current, incoming, requestedPage, pageSize) => ({
    page: nextCommentPage(requestedPage, incoming.length, pageSize),
    noMore: incoming.length < pageSize,
    items: mergeCommentPage(current.items, incoming, requestedPage),
});
//# sourceMappingURL=post-comment-state.js.map