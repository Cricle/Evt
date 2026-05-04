import { describe, expect, it } from 'vitest';
import { buildPostReactionsPath } from '@/api/post';
describe('post reaction api helpers', () => {
    it('builds the REST reaction path with the post id in the URL', () => {
        expect(buildPostReactionsPath(42)).toBe('/v1/posts/42/reactions');
    });
});
//# sourceMappingURL=post_reaction_api.test.js.map