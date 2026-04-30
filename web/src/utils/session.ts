import { userInfo } from '@/api/auth';
import { useStoreMain } from '@/store/main';
import { TOKEN_KEY, useStoreUser } from '@/store/user';

export async function restoreUserSession() {
  const storeMain = useStoreMain();
  const storeUser = useStoreUser();
  const token = localStorage.getItem(TOKEN_KEY) || '';

  if (!token) {
    storeUser.userLogout();
    return;
  }

  try {
    const profile = await userInfo(token);
    storeUser.updateUserinfo(profile);
    storeMain.triggerAuth(false);
  } catch (_error) {
    storeUser.userLogout();
  }
}
