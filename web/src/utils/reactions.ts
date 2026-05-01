import { CommentItemTypeEnum } from '@/utils/IEnum';
import { toPlainText } from '@/utils/content';

export const DEFAULT_REACTION_EMOJIS = [
  '👍',
  '❤️',
  '🫶',
  '👏',
  '🙌',
  '🎉',
  '😂',
  '🤔',
  '😮',
  '😍',
  '🥳',
  '😭',
  '😅',
  '😢',
  '😡',
  '🔥',
  '💯',
  '👀',
  '✅',
  '🙏',
  '🤝',
  '🚀',
];

const EMOJI_ONLY_PATTERN =
  /^(?:\p{Emoji_Presentation}|\p{Extended_Pictographic}|\uFE0F|\u200D)+$/u;

const normalizeReaction = (value: string) => toPlainText(value).replace(/\s+/g, '');

export const isEmojiReaction = (value: string) => {
  const normalized = normalizeReaction(value);
  return normalized.length > 0 && normalized.length <= 10 && EMOJI_ONLY_PATTERN.test(normalized);
};

export interface ReactionGroup {
  emoji: string;
  count: number;
  users: Item.UserInfo[];
}

export interface CommentReactionView {
  visibleComments: Item.CommentProps[];
  reactions: ReactionGroup[];
}

export interface ReplyReactionView {
  visibleReplies: Item.ReplyProps[];
  reactions: ReactionGroup[];
}

export interface ReactionDisplaySummary {
  visible: ReactionGroup[];
  hiddenCount: number;
}

const buildReactionMap = () => new Map<string, ReactionGroup>();

export const sortReactionGroups = (reactions: ReactionGroup[]) =>
  reactions.sort((left, right) => {
    if (right.count !== left.count) {
      return right.count - left.count;
    }
    return left.emoji.localeCompare(right.emoji);
  });

const appendReaction = (map: Map<string, ReactionGroup>, emoji: string, user: Item.UserInfo) => {
  const existing = map.get(emoji);
  if (existing) {
    existing.count += 1;
    existing.users.push(user);
    return;
  }
  map.set(emoji, {
    emoji,
    count: 1,
    users: [user],
  });
};

export const splitCommentReactions = (comments: Item.CommentProps[]): CommentReactionView => {
  const visibleComments: Item.CommentProps[] = [];
  const reactionMap = buildReactionMap();

  comments.forEach((comment) => {
    const textContents = comment.contents.filter(
      (content) => +content.type === CommentItemTypeEnum.TEXT,
    );
    const textOnly =
      textContents.length === comment.contents.length &&
      textContents.length === 1 &&
      isEmojiReaction(textContents[0].content);

    if (textOnly) {
      appendReaction(reactionMap, normalizeReaction(textContents[0].content), comment.user);
      return;
    }

    visibleComments.push(comment);
  });

  return {
    visibleComments,
    reactions: sortReactionGroups(Array.from(reactionMap.values())),
  };
};

export const splitReplyReactions = (replies: Item.ReplyProps[]): ReplyReactionView => {
  const visibleReplies: Item.ReplyProps[] = [];
  const reactionMap = buildReactionMap();

  replies.forEach((reply) => {
    if (isEmojiReaction(reply.content)) {
      appendReaction(reactionMap, normalizeReaction(reply.content), reply.user);
      return;
    }
    visibleReplies.push(reply);
  });

  return {
    visibleReplies,
    reactions: sortReactionGroups(Array.from(reactionMap.values())),
  };
};

export const summarizeReactionGroups = (
  reactions: ReactionGroup[],
  maxVisible = 6,
): ReactionDisplaySummary => {
  const safeMaxVisible = Math.max(0, Math.floor(maxVisible));
  const visible = safeMaxVisible === 0 ? [] : reactions.slice(0, safeMaxVisible);

  return {
    visible,
    hiddenCount: Math.max(reactions.length - visible.length, 0),
  };
};

export const upsertReactionGroup = (
  reactions: ReactionGroup[],
  emoji: string,
  user?: Item.UserInfo,
): ReactionGroup[] => {
  const nextReactions = reactions.map((reaction) => ({
    ...reaction,
    users: [...reaction.users],
  }));
  const existing = nextReactions.find((reaction) => reaction.emoji === emoji);

  if (existing) {
    existing.count += 1;
    if (user) {
      existing.users.unshift(user);
    }
    return sortReactionGroups(nextReactions);
  }

  return sortReactionGroups([
    ...nextReactions,
    {
      emoji,
      count: 1,
      users: user ? [user] : [],
    },
  ]);
};
