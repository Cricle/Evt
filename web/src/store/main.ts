import { defineStore } from 'pinia';
import { reactive, ref } from 'vue';
import { safeLocalStorageGet, safeLocalStorageSet } from '@/utils/storage';

const MOBILE_BREAKPOINT = 821;
const LEFTBAR_BREAKPOINT = 900;
const RIGHTBAR_BREAKPOINT = 1120;
export const EVT_THEME_KEY = 'EVT_THEME';
export const EVT_THEME_MODE_KEY = 'EVT_THEME_MODE';
export const EVT_THEME_PRESET_KEY = 'EVT_THEME_PRESET';
export const EVT_LOCALE_KEY = 'EVT_LOCALE';
export type AppLocale = 'zh-CN' | 'en-US';
export type AppThemeMode = 'system' | 'light' | 'dark';
export type AppThemePreset = 'emerald' | 'ocean' | 'amber' | 'rose';

function detectInitialLocale(): AppLocale {
  const persisted = safeLocalStorageGet(EVT_LOCALE_KEY);
  if (persisted === 'en-US' || persisted === 'zh-CN') {
    return persisted;
  }

  return navigator.language.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en-US';
}

function detectPreferredTheme(): 'light' | 'dark' {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return 'light';
  }

  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function detectInitialThemeMode(): AppThemeMode {
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

function detectInitialThemePreset(): AppThemePreset {
  const persisted = safeLocalStorageGet(EVT_THEME_PRESET_KEY);
  if (
    persisted === 'emerald' ||
    persisted === 'ocean' ||
    persisted === 'amber' ||
    persisted === 'rose'
  ) {
    return persisted;
  }

  return 'emerald';
}

function resolveThemeMode(
  mode: AppThemeMode,
  osTheme: 'light' | 'dark' = detectPreferredTheme(),
): 'light' | 'dark' {
  return mode === 'system' ? osTheme : mode;
}

function viewportWidth() {
  return (
    window.innerWidth ||
    document.documentElement.clientWidth ||
    document.body.clientWidth
  );
}

export const useStoreMain = defineStore('main', () => {
  const initialWidth = viewportWidth();
  const refresh = ref(Date.now());
  const refreshTopicFollow = ref(Date.now());
  const themeMode = ref<AppThemeMode>(detectInitialThemeMode());
  const themePreset = ref<AppThemePreset>(detectInitialThemePreset());
  const theme = ref<'light' | 'dark'>(resolveThemeMode(themeMode.value));
  const locale = ref<AppLocale>(detectInitialLocale());
  const collapsedLeft = ref(initialWidth <= LEFTBAR_BREAKPOINT);
  const collapsedRight = ref(initialWidth <= RIGHTBAR_BREAKPOINT);
  const drawerModelShow = ref(initialWidth <= LEFTBAR_BREAKPOINT);
  const desktopModelShow = ref(initialWidth > LEFTBAR_BREAKPOINT);
  const unreadMsgCount = ref(0);

  function doRefresh(val?: number) {
    refresh.value = val || Date.now();
  }

  function doRefreshTopicFollow() {
    refreshTopicFollow.value = Date.now();
  }

  function updateUnreadMsgCount(count: number) {
    unreadMsgCount.value = count;
  }

  function triggerTheme(t: 'light' | 'dark') {
    safeLocalStorageSet(EVT_THEME_MODE_KEY, t);
    safeLocalStorageSet(EVT_THEME_KEY, t);
    themeMode.value = t;
    theme.value = t;
  }

  function triggerThemeMode(mode: AppThemeMode, osTheme?: 'light' | 'dark') {
    safeLocalStorageSet(EVT_THEME_MODE_KEY, mode);
    themeMode.value = mode;
    theme.value = resolveThemeMode(mode, osTheme);
    safeLocalStorageSet(EVT_THEME_KEY, theme.value);
  }

  function triggerThemePreset(preset: AppThemePreset) {
    safeLocalStorageSet(EVT_THEME_PRESET_KEY, preset);
    themePreset.value = preset;
  }

  function syncResolvedTheme(osTheme?: 'light' | 'dark') {
    theme.value = resolveThemeMode(themeMode.value, osTheme);
    safeLocalStorageSet(EVT_THEME_KEY, theme.value);
  }

  function triggerLocale(nextLocale: AppLocale) {
    locale.value = nextLocale;
  }

  function triggerCollapsedLeft(status: boolean) {
    collapsedLeft.value = status;
    drawerModelShow.value = status;
    desktopModelShow.value = !status;
  }

  function triggerCollapsedRight(status: boolean) {
    collapsedRight.value = status;
  }

  function syncViewportLayout(width = viewportWidth()) {
    const isMobile = width <= LEFTBAR_BREAKPOINT;
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
