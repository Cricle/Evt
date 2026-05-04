import { resolveSpaceSlug } from '@/utils/spaces';
const withResolvedSpace = (query, currentSpaceSlug) => ({
    ...query,
    space: resolveSpaceSlug(currentSpaceSlug, currentSpaceSlug),
});
export const buildHomeRouteWithSpace = (query, currentSpaceSlug) => ({
    name: 'space',
    query: withResolvedSpace(query, currentSpaceSlug),
});
export const buildTagSearchRoute = (tag, currentSpaceSlug) => ({
    ...buildHomeRouteWithSpace({
        q: tag,
        t: 'tag',
    }, currentSpaceSlug),
});
export const buildPostRoute = (id, currentSpaceSlug) => ({
    name: 'post',
    query: withResolvedSpace({ id }, currentSpaceSlug),
});
export const buildComposeRoute = (currentSpaceSlug, mode = 'post', quick) => ({
    name: 'compose',
    query: withResolvedSpace({
        ...(mode === 'event' ? { mode } : {}),
        ...(quick ? { quick } : {}),
    }, currentSpaceSlug),
});
export const buildCreateSpaceRoute = (currentSpaceSlug) => ({
    name: 'create-space',
    query: withResolvedSpace({}, currentSpaceSlug),
});
export const buildFollowingRoute = (username, nickname, tab, currentSpaceSlug) => ({
    name: 'following',
    query: withResolvedSpace({
        s: username,
        n: nickname,
        t: tab,
    }, currentSpaceSlug),
});
export const buildSettingRoute = (currentSpaceSlug) => ({
    name: 'setting',
    query: withResolvedSpace({
        t: Date.now(),
    }, currentSpaceSlug),
});
//# sourceMappingURL=tagRoute.js.map
