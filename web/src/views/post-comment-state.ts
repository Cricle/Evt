import { mergeCommentPage, nextCommentPage } from '@/utils/comment-feed';

export interface CommentPageState<T> {
  page: number;
  noMore: boolean;
  items: T[];
}

export const createCommentPageState = <T>(): CommentPageState<T> => ({
  page: 1,
  noMore: false,
  items: [],
});

export const applyCommentPageState = <T>(
  current: CommentPageState<T>,
  incoming: T[],
  requestedPage: number,
  pageSize: number,
): CommentPageState<T> => ({
  page: nextCommentPage(requestedPage, incoming.length, pageSize),
  noMore: incoming.length < pageSize,
  items: mergeCommentPage(current.items, incoming, requestedPage),
});
