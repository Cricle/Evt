import { resolveSpaceSlug } from '@/utils/spaces';

const withResolvedSpace = (
  query: Record<string, string | number>,
  currentSpaceSlug?: string | null,
) => ({
  ...query,
  space: resolveSpaceSlug(currentSpaceSlug, currentSpaceSlug),
});

export const buildHomeRouteWithSpace = (
  query: Record<string, string | number>,
  currentSpaceSlug?: string | null,
) => ({
  name: 'home',
  query: withResolvedSpace(query, currentSpaceSlug),
});

export const buildTagSearchRoute = (tag: string, currentSpaceSlug?: string | null) => ({
  ...buildHomeRouteWithSpace(
    {
      q: tag,
      t: 'tag',
    },
    currentSpaceSlug,
  ),
});

export const buildPostRoute = (id: number, currentSpaceSlug?: string | null) => ({
  name: 'post',
  query: withResolvedSpace({ id }, currentSpaceSlug),
});

export const buildComposeRoute = (currentSpaceSlug?: string | null) => ({
  name: 'compose',
  query: withResolvedSpace({}, currentSpaceSlug),
});

export const buildCreateSpaceRoute = (currentSpaceSlug?: string | null) => ({
  name: 'create-space',
  query: withResolvedSpace({}, currentSpaceSlug),
});

export const buildFollowingRoute = (
  username: string,
  nickname: string,
  tab: 'follows' | 'followings',
  currentSpaceSlug?: string | null,
) => ({
  name: 'following',
  query: withResolvedSpace(
    {
      s: username,
      n: nickname,
      t: tab,
    },
    currentSpaceSlug,
  ),
});

export const buildSettingRoute = (currentSpaceSlug?: string | null) => ({
  name: 'setting',
  query: withResolvedSpace(
    {
      t: Date.now(),
    },
    currentSpaceSlug,
  ),
});
