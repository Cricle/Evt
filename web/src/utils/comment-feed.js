export const mergeCommentPage = (existing, incoming, page) => {
    if (page <= 1) {
        return [...incoming];
    }
    if (incoming.length === 0) {
        return existing;
    }
    return [...existing, ...incoming];
};
export const nextCommentPage = (currentPage, incomingCount, pageSize) => {
    if (incomingCount < pageSize) {
        return currentPage;
    }
    return currentPage + 1;
};
//# sourceMappingURL=comment-feed.js.map