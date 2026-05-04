import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { EVT_LOCALE_KEY, EVT_THEME_MODE_KEY, EVT_THEME_PRESET_KEY } from '@/store/main';

describe('main store theme', () => {
  beforeEach(() => {
    vi.resetModules();
    localStorage.clear();
    setActivePinia(createPinia());
  });

  it('defaults to system theme mode and light resolved theme when there is no persisted preference', async () => {
    const { useStoreMain } = await import('@/store/main');
    const store = useStoreMain();

    expect(store.themeMode).toBe('system');
    expect(store.theme).toBe('light');
  });

  it('restores explicit dark theme mode from persisted preference', async () => {
    localStorage.setItem(EVT_THEME_MODE_KEY, 'dark');
    const { useStoreMain } = await import('@/store/main');
    const store = useStoreMain();

    expect(store.themeMode).toBe('dark');
    expect(store.theme).toBe('dark');
  });

  it('restores theme preset from persisted preference', async () => {
    localStorage.setItem(EVT_THEME_PRESET_KEY, 'ocean');
    const { useStoreMain } = await import('@/store/main');
    const store = useStoreMain();

    expect(store.themePreset).toBe('ocean');
  });

  it('defaults locale from browser language and restores persisted locale', async () => {
    vi.stubGlobal('navigator', {
      language: 'en-US',
    });

    const { useStoreMain } = await import('@/store/main');
    const store = useStoreMain();
    expect(store.locale).toBe('en-US');

    localStorage.setItem(EVT_LOCALE_KEY, 'zh-CN');
    vi.resetModules();
    setActivePinia(createPinia());
    const refreshedModule = await import('@/store/main');
    const refreshedStore = refreshedModule.useStoreMain();
    expect(refreshedStore.locale).toBe('zh-CN');
  });

  it('persists explicit theme mode and preset updates through store actions', async () => {
    const { useStoreMain, EVT_THEME_KEY } = await import('@/store/main');
    const store = useStoreMain();

    store.triggerThemeMode('dark', 'light');
    expect(store.themeMode).toBe('dark');
    expect(store.theme).toBe('dark');
    expect(localStorage.getItem(EVT_THEME_MODE_KEY)).toBe('dark');
    expect(localStorage.getItem(EVT_THEME_KEY)).toBe('dark');

    store.triggerThemePreset('ocean');
    expect(store.themePreset).toBe('ocean');
    expect(localStorage.getItem(EVT_THEME_PRESET_KEY)).toBe('ocean');
  });

  it('resolves system theme updates against the current os mode', async () => {
    const { useStoreMain, EVT_THEME_KEY } = await import('@/store/main');
    const store = useStoreMain();

    store.triggerThemeMode('system', 'dark');
    expect(store.themeMode).toBe('system');
    expect(store.theme).toBe('dark');
    expect(localStorage.getItem(EVT_THEME_MODE_KEY)).toBe('system');
    expect(localStorage.getItem(EVT_THEME_KEY)).toBe('dark');

    store.syncResolvedTheme('light');
    expect(store.theme).toBe('light');
    expect(localStorage.getItem(EVT_THEME_KEY)).toBe('light');
  });
});
