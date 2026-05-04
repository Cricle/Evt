import { describe, expect, it } from 'vitest';
function resolveActionState(currentUserId, contactUserIds, item) {
    if (item.user_id === currentUserId) {
        return 'self';
    }
    if (item.is_friend || contactUserIds.has(item.user_id)) {
        return 'friend';
    }
    return 'add';
}
describe('Contacts search action state', () => {
    it('marks current user as self', () => {
        expect(resolveActionState(7, new Set(), {
            user_id: 7,
            username: 'alice',
            nickname: 'Alice',
            is_friend: false,
        })).toBe('self');
    });
    it('marks known friends as friend', () => {
        expect(resolveActionState(7, new Set([11]), {
            user_id: 11,
            username: 'bob',
            nickname: 'Bob',
            is_friend: false,
        })).toBe('friend');
        expect(resolveActionState(7, new Set(), {
            user_id: 12,
            username: 'carol',
            nickname: 'Carol',
            is_friend: true,
        })).toBe('friend');
    });
    it('allows sending a friend request to unrelated users', () => {
        expect(resolveActionState(7, new Set([11]), {
            user_id: 13,
            username: 'dave',
            nickname: 'Dave',
            is_friend: false,
        })).toBe('add');
    });
});
//# sourceMappingURL=Contacts.search.test.js.map