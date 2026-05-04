export const safeLocalStorageGet = (key: string) => {
  try {
    return localStorage.getItem(key);
  } catch (_err) {
    return null;
  }
};

export const safeLocalStorageSet = (key: string, value: string) => {
  try {
    localStorage.setItem(key, value);
    return true;
  } catch (_err) {
    return false;
  }
};
