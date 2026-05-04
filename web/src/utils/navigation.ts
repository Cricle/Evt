import type { RouteLocationRaw, Router } from 'vue-router';

type MinimalRouter = Pick<Router, 'back' | 'push' | 'resolve' | 'currentRoute'>;
type MinimalLocation = Pick<Location, 'assign'>;
type MinimalHistoryState = {
  back?: string | null;
} | null;

const HASH_ROUTE_PATHS = new Set([
  '/space',
  '/auth',
  '/compose',
  '/spaces/create',
  '/post',
  '/topic',
  '/announcement',
  '/anouncement',
  '/profile',
  '/u',
  '/messages',
  '/contacts',
  '/following',
  '/wallet',
  '/setting',
  '/admin/settings',
  '/404',
]);

export const normalizeResolvedHref = (href: string, fullPath: string) => {
  if (href.startsWith('#') || href.startsWith('/#') || href.startsWith('http')) {
    return href;
  }
  return `#${fullPath}`;
};

export const normalizeInitialHashRoute = (locationRef?: Location | null) => {
  if (!locationRef) {
    return false;
  }

  const { pathname, hash, search } = locationRef;
  if (hash || !HASH_ROUTE_PATHS.has(pathname)) {
    return false;
  }

  locationRef.replace(`${locationRef.origin}/#${pathname}${search}`);
  return true;
};

export const canUseHistoryBack = (state?: MinimalHistoryState) =>
  typeof state?.back === 'string' && state.back.length > 0;

export const pushWithFallback = async (
  router: MinimalRouter,
  target: RouteLocationRaw,
  locationRef?: MinimalLocation | null,
) => {
  const resolved = router.resolve(target);

  try {
    await router.push(target);
  } catch {
    // Navigation guards or duplicate navigations should not block the fallback.
  }

  if (router.currentRoute.value.fullPath !== resolved.fullPath && locationRef) {
    locationRef.assign(normalizeResolvedHref(resolved.href, resolved.fullPath));
  }
};

export const backWithFallback = async (
  router: MinimalRouter,
  fallbackTarget: RouteLocationRaw,
  locationRef?: MinimalLocation | null,
  state?: MinimalHistoryState,
) => {
  if (canUseHistoryBack(state)) {
    router.back();
    return;
  }

  await pushWithFallback(router, fallbackTarget, locationRef);
};
