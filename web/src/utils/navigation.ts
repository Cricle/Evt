import type { RouteLocationRaw, Router } from 'vue-router';

type MinimalRouter = Pick<Router, 'push' | 'resolve' | 'currentRoute'>;
type MinimalLocation = Pick<Location, 'assign'>;

export const normalizeResolvedHref = (href: string, fullPath: string) => {
  if (href.startsWith('#') || href.startsWith('/#') || href.startsWith('http')) {
    return href;
  }
  return `#${fullPath}`;
};

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
