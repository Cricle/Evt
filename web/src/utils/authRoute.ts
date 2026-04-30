import type { Router } from 'vue-router';

export type AuthMode = 'signin' | 'signup';

export function goToAuth(router: Router, mode: AuthMode = 'signin', redirect?: string) {
  return router.push({
    name: 'auth',
    query: {
      mode,
      ...(redirect ? { redirect } : {}),
    },
  });
}
