import { userInfo } from '@/api/auth';
import { TOKEN_KEY, useStoreUser } from '@/store/user';

export async function restoreUserSession() {
  const storeUser = useStoreUser();
  const token = localStorage.getItem(TOKEN_KEY) || '';

  if (!token) {
    storeUser.userLogout();
    return;
  }

  try {
    const profile = await userInfo(token);
    storeUser.updateUserinfo(profile);
  } catch (_error) {
    storeUser.userLogout();
  }
}
