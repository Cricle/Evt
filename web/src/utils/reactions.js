import { CommentItemTypeEnum } from '@/utils/IEnum';
import { toPlainText } from '@/utils/content';
export const REACTION_EMOJI_GROUPS = [
    {
        label: '常用',
        emojis: ['😀', '😂', '🥹', '😍', '😎', '🤔', '😮', '😭', '😡', '🥳', '🔥', '💯'],
    },
    {
        label: '手势',
        emojis: ['👍', '👎', '👏', '🙌', '👌', '✌️', '🤞', '🫶', '🙏', '🤝', '💪', '👀'],
    },
    {
        label: '情绪',
        emojis: ['🙂', '😊', '😄', '😅', '🤣', '😉', '😇', '🤩', '😘', '😋', '😴', '🤯'],
    },
    {
        label: '爱心',
        emojis: ['❤️', '🩷', '🧡', '💛', '💚', '🩵', '💙', '💜', '🤍', '🖤', '🤎', '💔'],
    },
    {
        label: '人物',
        emojis: ['🙋', '🙆', '💁', '🧑‍💻', '👨‍💻', '👩‍💻', '🧠', '🫡', '🤷', '🙍', '🙇', '🕺'],
    },
    {
        label: '自然',
        emojis: ['🌱', '🌳', '🌸', '🌻', '🍀', '🍁', '🌈', '☀️', '🌙', '⭐', '⚡', '☁️'],
    },
    {
        label: '动物',
        emojis: ['🐶', '🐱', '🐼', '🐨', '🦊', '🐯', '🐮', '🐸', '🐵', '🐧', '🦄', '🐝'],
    },
    {
        label: '食物',
        emojis: ['☕', '🍵', '🧋', '🍎', '🍉', '🍇', '🍕', '🍔', '🍟', '🍜', '🍰', '🍻'],
    },
    {
        label: '物件',
        emojis: ['📌', '📣', '📎', '📷', '🎧', '💡', '🧭', '⌛', '🪄', '🎁', '🏆', '🚀'],
    },
    {
        label: '符号',
        emojis: ['✅', '❌', '⭕', '❗', '❓', '💬', '🗯️', '♻️', '💤', '🎯', '🔒', '🔔'],
    },
];
export const DEFAULT_REACTION_EMOJIS = REACTION_EMOJI_GROUPS.flatMap((group) => group.emojis);
const EMOJI_ONLY_PATTERN = /^(?:\p{Emoji_Presentation}|\p{Extended_Pictographic}|\uFE0F|\u200D)+$/u;
const normalizeReaction = (value) => toPlainText(value).replace(/\s+/g, '');
export const isEmojiReaction = (value) => {
    const normalized = normalizeReaction(value);
    return normalized.length > 0 && normalized.length <= 10 && EMOJI_ONLY_PATTERN.test(normalized);
};
const buildReactionMap = () => new Map();
export const sortReactionGroups = (reactions) => reactions.sort((left, right) => {
    if (right.count !== left.count) {
        return right.count - left.count;
    }
    return left.emoji.localeCompare(right.emoji);
});
const appendReaction = (map, emoji, user) => {
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
export const splitCommentReactions = (comments) => {
    const visibleComments = [];
    const reactionMap = buildReactionMap();
    comments.forEach((comment) => {
        const textContents = comment.contents.filter((content) => +content.type === CommentItemTypeEnum.TEXT);
        const textOnly = textContents.length === comment.contents.length &&
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
export const summarizeReactionGroups = (reactions, maxVisible = 6) => {
    const safeMaxVisible = Math.max(0, Math.floor(maxVisible));
    const visible = safeMaxVisible === 0 ? [] : reactions.slice(0, safeMaxVisible);
    return {
        visible,
        hiddenCount: Math.max(reactions.length - visible.length, 0),
    };
};
export const upsertReactionGroup = (reactions, emoji, user) => {
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
//# sourceMappingURL=reactions.js.map
