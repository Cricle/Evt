import { describe, expect, it } from 'vitest';
import { buildComposeRoute, buildCreateSpaceRoute, buildFollowingRoute, buildHomeRouteWithSpace, buildPostRoute, buildSettingRoute, buildTagSearchRoute, } from '@/utils/tagRoute';
describe('tag route utils', () => {
    it('preserves the current space when navigating to tag search', () => {
        expect(buildTagSearchRoute('rust', 'team-alpha')).toEqual({
            name: 'space',
            query: {
                q: 'rust',
                t: 'tag',
                space: 'team-alpha',
            },
        });
    });
    it('normalizes legacy and empty spaces to public for tag search', () => {
        expect(buildTagSearchRoute('evt', 'square')).toEqual({
            name: 'space',
            query: {
                q: 'evt',
                t: 'tag',
                space: 'public',
            },
        });
        expect(buildTagSearchRoute('evt', '')).toEqual({
            name: 'space',
            query: {
                q: 'evt',
                t: 'tag',
                space: 'public',
            },
        });
    });
    it('builds generic home routes that retain the current space', () => {
        expect(buildHomeRouteWithSpace({ t: 123 }, 'team-alpha')).toEqual({
            name: 'space',
            query: {
                t: 123,
                space: 'team-alpha',
            },
        });
    });
    it('builds post routes that retain the current space', () => {
        expect(buildPostRoute(42, 'team-alpha')).toEqual({
            name: 'post',
            query: {
                id: 42,
                space: 'team-alpha',
            },
        });
    });
    it('normalizes legacy and empty spaces for post routes', () => {
        expect(buildPostRoute(7, 'square')).toEqual({
            name: 'post',
            query: {
                id: 7,
                space: 'public',
            },
        });
        expect(buildPostRoute(7, '')).toEqual({
            name: 'post',
            query: {
                id: 7,
                space: 'public',
            },
        });
    });
    it('builds compose and create-space routes that retain the current space', () => {
        expect(buildComposeRoute('team-alpha')).toEqual({
            name: 'compose',
            query: {
                space: 'team-alpha',
            },
        });
        expect(buildCreateSpaceRoute('team-alpha')).toEqual({
            name: 'create-space',
            query: {
                space: 'team-alpha',
            },
        });
    });
    it('builds following routes that retain the current space', () => {
        expect(buildFollowingRoute('alice', 'Alice', 'follows', 'team-alpha')).toEqual({
            name: 'following',
            query: {
                s: 'alice',
                n: 'Alice',
                t: 'follows',
                space: 'team-alpha',
            },
        });
    });
    it('builds setting routes that retain the current space', () => {
        const route = buildSettingRoute('team-alpha');
        expect(route.name).toBe('setting');
        expect(route.query.space).toBe('team-alpha');
        expect(typeof route.query.t).toBe('number');
    });
});
//# sourceMappingURL=tagRoute.test.js.map