export const safeLocalStorageGet = (key) => {
    try {
        return localStorage.getItem(key);
    }
    catch (_err) {
        return null;
    }
};
export const safeLocalStorageSet = (key, value) => {
    try {
        localStorage.setItem(key, value);
        return true;
    }
    catch (_err) {
        return false;
    }
};
//# sourceMappingURL=storage.js.map