import { defineStore } from 'pinia';
import { ref } from 'vue';
import { safeLocalStorageGet, safeLocalStorageSet } from '@/utils/storage';
const MOBILE_BREAKPOINT = 821;
const RIGHTBAR_BREAKPOINT = 1100;
export const EVT_THEME_KEY = 'EVT_THEME';
export const EVT_THEME_MODE_KEY = 'EVT_THEME_MODE';
export const EVT_THEME_PRESET_KEY = 'EVT_THEME_PRESET';
export const EVT_LOCALE_KEY = 'EVT_LOCALE';
function detectInitialLocale() {
    const persisted = safeLocalStorageGet(EVT_LOCALE_KEY);
    if (persisted === 'en-US' || persisted === 'zh-CN') {
        return persisted;
    }
    return navigator.language.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en-US';
}
function detectPreferredTheme() {
    if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
        return 'light';
    }
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}
function detectInitialThemeMode() {
    const persisted = safeLocalStorageGet(EVT_THEME_MODE_KEY);
    if (persisted === 'system' || persisted === 'light' || persisted === 'dark') {
        return persisted;
    }
    const legacyTheme = safeLocalStorageGet(EVT_THEME_KEY);
    if (legacyTheme === 'light' || legacyTheme === 'dark') {
        return legacyTheme;
    }
    return 'system';
}
function detectInitialThemePreset() {
    const persisted = safeLocalStorageGet(EVT_THEME_PRESET_KEY);
    if (persisted === 'emerald' ||
        persisted === 'ocean' ||
        persisted === 'amber' ||
        persisted === 'rose') {
        return persisted;
    }
    return 'emerald';
}
function resolveThemeMode(mode, osTheme = detectPreferredTheme()) {
    return mode === 'system' ? osTheme : mode;
}
function viewportWidth() {
    return (window.innerWidth ||
        document.documentElement.clientWidth ||
        document.body.clientWidth);
}
export const useStoreMain = defineStore('main', () => {
    const initialWidth = viewportWidth();
    const refresh = ref(Date.now());
    const refreshTopicFollow = ref(Date.now());
    const themeMode = ref(detectInitialThemeMode());
    const themePreset = ref(detectInitialThemePreset());
    const theme = ref(resolveThemeMode(themeMode.value));
    const locale = ref(detectInitialLocale());
    const collapsedLeft = ref(initialWidth <= MOBILE_BREAKPOINT);
    const collapsedRight = ref(initialWidth <= RIGHTBAR_BREAKPOINT);
    const drawerModelShow = ref(initialWidth <= MOBILE_BREAKPOINT);
    const desktopModelShow = ref(initialWidth > MOBILE_BREAKPOINT);
    const unreadMsgCount = ref(0);
    function doRefresh(val) {
        refresh.value = val || Date.now();
    }
    function doRefreshTopicFollow() {
        refreshTopicFollow.value = Date.now();
    }
    function updateUnreadMsgCount(count) {
        unreadMsgCount.value = count;
    }
    function triggerTheme(t) {
        safeLocalStorageSet(EVT_THEME_MODE_KEY, t);
        safeLocalStorageSet(EVT_THEME_KEY, t);
        themeMode.value = t;
        theme.value = t;
    }
    function triggerThemeMode(mode, osTheme) {
        safeLocalStorageSet(EVT_THEME_MODE_KEY, mode);
        themeMode.value = mode;
        theme.value = resolveThemeMode(mode, osTheme);
        safeLocalStorageSet(EVT_THEME_KEY, theme.value);
    }
    function triggerThemePreset(preset) {
        safeLocalStorageSet(EVT_THEME_PRESET_KEY, preset);
        themePreset.value = preset;
    }
    function syncResolvedTheme(osTheme) {
        theme.value = resolveThemeMode(themeMode.value, osTheme);
        safeLocalStorageSet(EVT_THEME_KEY, theme.value);
    }
    function triggerLocale(nextLocale) {
        locale.value = nextLocale;
    }
    function triggerCollapsedLeft(status) {
        collapsedLeft.value = status;
        drawerModelShow.value = status;
        desktopModelShow.value = !status;
    }
    function triggerCollapsedRight(status) {
        collapsedRight.value = status;
    }
    function syncViewportLayout(width = viewportWidth()) {
        const isMobile = width <= MOBILE_BREAKPOINT;
        collapsedLeft.value = isMobile;
        drawerModelShow.value = isMobile;
        desktopModelShow.value = !isMobile;
        collapsedRight.value = width <= RIGHTBAR_BREAKPOINT;
    }
    return {
        refresh,
        refreshTopicFollow,
        theme,
        themeMode,
        themePreset,
        locale,
        collapsedLeft,
        collapsedRight,
        drawerModelShow,
        desktopModelShow,
        unreadMsgCount,
        doRefresh,
        doRefreshTopicFollow,
        updateUnreadMsgCount,
        triggerTheme,
        triggerThemeMode,
        triggerThemePreset,
        syncResolvedTheme,
        triggerLocale,
        triggerCollapsedLeft,
        triggerCollapsedRight,
        syncViewportLayout,
    };
});
//# sourceMappingURL=main.js.map