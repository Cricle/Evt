export const mergeCommentPage = <T>(existing: T[], incoming: T[], page: number): T[] => {
  if (page <= 1) {
    return [...incoming];
  }

  if (incoming.length === 0) {
    return existing;
  }

  return [...existing, ...incoming];
};

export const nextCommentPage = (currentPage: number, incomingCount: number, pageSize: number) => {
  if (incomingCount < pageSize) {
    return currentPage;
  }

  return currentPage + 1;
};
