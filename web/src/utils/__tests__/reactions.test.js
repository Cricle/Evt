import { describe, expect, it } from 'vitest';
import { CommentItemTypeEnum } from '@/utils/IEnum';
import { DEFAULT_REACTION_EMOJIS, isEmojiReaction, splitCommentReactions, summarizeReactionGroups, upsertReactionGroup, } from '@/utils/reactions';
const buildUser = (id, username) => ({
    id,
    username,
    nickname: username,
    avatar: '',
    is_admin: false,
    is_friend: false,
    is_following: false,
    created_on: 0,
    follows: 0,
    followings: 0,
    status: 1,
});
describe('reaction utils', () => {
    it('recognizes emoji-only reactions', () => {
        expect(DEFAULT_REACTION_EMOJIS.length).toBeGreaterThan(5);
        expect(isEmojiReaction('👍')).toBe(true);
        expect(isEmojiReaction('👍 👍')).toBe(true);
        expect(isEmojiReaction('hello')).toBe(false);
    });
    it('splits emoji-only comments into aggregated reaction groups', () => {
        const emojiComment = {
            id: 1,
            post_id: 1,
            user_id: 1,
            user: buildUser(1, 'alice'),
            contents: [
                {
                    id: 1,
                    comment_id: 1,
                    user_id: 1,
                    type: CommentItemTypeEnum.TEXT,
                    content: '👍',
                    sort: 100,
                    created_on: 0,
                },
            ],
            ip_loc: '',
            is_essence: 0,
            thumbs_up_count: 0,
            is_thumbs_up: 0,
            is_thumbs_down: 0,
            created_on: 0,
        };
        const textComment = {
            ...emojiComment,
            id: 2,
            user_id: 2,
            user: buildUser(2, 'bob'),
            contents: [{ ...emojiComment.contents[0], id: 2, comment_id: 2, user_id: 2, content: 'hello' }],
        };
        const secondEmojiComment = {
            ...emojiComment,
            id: 3,
            user_id: 3,
            user: buildUser(3, 'carol'),
            contents: [{ ...emojiComment.contents[0], id: 3, comment_id: 3, user_id: 3, content: '👍' }],
        };
        const view = splitCommentReactions([emojiComment, textComment, secondEmojiComment]);
        expect(view.visibleComments).toHaveLength(1);
        expect(view.reactions).toEqual([
            {
                emoji: '👍',
                count: 2,
                users: [emojiComment.user, secondEmojiComment.user],
            },
        ]);
    });
    it('sorts larger reaction groups first and summarizes overflow', () => {
        const reactions = [
            { emoji: '👏', count: 2, users: [buildUser(1, 'alice'), buildUser(2, 'bob')] },
            { emoji: '👍', count: 5, users: [buildUser(3, 'carol')] },
            { emoji: '🎉', count: 1, users: [buildUser(4, 'dave')] },
        ];
        const summary = summarizeReactionGroups(reactions.sort((left, right) => right.count - left.count), 2);
        expect(summary.visible).toEqual([
            { emoji: '👍', count: 5, users: [buildUser(3, 'carol')] },
            { emoji: '👏', count: 2, users: [buildUser(1, 'alice'), buildUser(2, 'bob')] },
        ]);
        expect(summary.hiddenCount).toBe(1);
    });
    it('upserts a reaction group and keeps reactions sorted by count', () => {
        const user = buildUser(5, 'ellen');
        const reactions = upsertReactionGroup([
            { emoji: '👏', count: 2, users: [buildUser(1, 'alice'), buildUser(2, 'bob')] },
            { emoji: '🔥', count: 1, users: [buildUser(4, 'dave')] },
        ], '🔥', user);
        expect(reactions).toEqual([
            { emoji: '👏', count: 2, users: [buildUser(1, 'alice'), buildUser(2, 'bob')] },
            { emoji: '🔥', count: 2, users: [user, buildUser(4, 'dave')] },
        ]);
    });
});
//# sourceMappingURL=reactions.test.js.map