export function isTauriRuntime() {
    if (typeof window === 'undefined') {
        return false;
    }
    return '__TAURI__' in window || '__TAURI_INTERNALS__' in window;
}
//# sourceMappingURL=platform.js.map