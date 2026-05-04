<template>
    <n-config-provider
        :theme="iTheme"
        :theme-overrides="themeOverrides"
        :locale="naiveLocale"
        :date-locale="naiveDateLocale"
    >
        <n-message-provider>
            <n-dialog-provider>
                <div
                    class="app-container"
                    :class="{
                      dark: iTheme?.name === 'dark',
                      mobile: !desktopModelShow,
                      'desktop-with-rightbar': desktopModelShow && !collapsedRight,
                      'desktop-no-rightbar': desktopModelShow && collapsedRight,
                    }"
                >
                    <div has-sider class="main-wrap" position="static" >
                        <!-- 侧边栏 -->
                        <div v-if="desktopModelShow">
                            <sidebar />
                        </div>

                        <div class="content-wrap">
                            <router-view
                                class="app-wrap"
                                v-slot="{ Component }"
                            >
                                <transition name="page-fade" mode="out-in">
                                    <keep-alive>
                                        <component
                                            v-if="$route.meta.keepAlive"
                                            :is="Component"
                                        />
                                    </keep-alive>
                                </transition>
                                <transition name="page-fade" mode="out-in">
                                    <component
                                        v-if="!$route.meta.keepAlive"
                                        :is="Component"
                                    />
                                </transition>
                            </router-view>
                        </div>

                        <!-- 右侧 -->
                        <rightbar />
                    </div>
                    <floating-compose />
                </div>
            </n-dialog-provider>
        </n-message-provider>
        <n-global-style />
    </n-config-provider>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, computed, watch } from 'vue';
import { useStoreMain } from '@/store/main';
import { darkTheme, useOsTheme } from 'naive-ui';
import { getSiteProfile } from '@/api/site';
import { useStoreProfile } from '@/store/profile';
import { storeToRefs } from 'pinia';
import { restoreUserSession } from '@/utils/session';
import { buildThemeCssVars, buildThemeOverrides } from '@/theme';
import { persistLocale, setMomentLocale, useI18n } from '@/i18n';
import { safeLocalStorageSet } from '@/utils/storage';

const storeMain = useStoreMain();
const storeProfile = useStoreProfile();
const { theme, themeMode, themePreset, desktopModelShow, collapsedRight } = storeToRefs(storeMain);
const { locale, naiveLocale, naiveDateLocale } = useI18n();

const iTheme = computed(() => (theme.value === 'dark' ? darkTheme : null));
const themeOverrides = computed(() =>
  buildThemeOverrides(themePreset.value, theme.value === 'dark'),
);
const syncViewportLayout = () => storeMain.syncViewportLayout();
const osThemeRef = useOsTheme();

const syncThemeClass = (nextTheme: 'light' | 'dark') => {
  if (typeof document === 'undefined') {
    return;
  }

  document.documentElement.classList.toggle('dark', nextTheme === 'dark');
  document.body.classList.toggle('dark', nextTheme === 'dark');
};

const syncThemeVars = () => {
  if (typeof document === 'undefined') {
    return;
  }

  const vars = buildThemeCssVars(themePreset.value, theme.value === 'dark');
  Object.entries(vars).forEach(([key, value]) => {
    document.documentElement.style.setProperty(key, value);
  });
};

function loadSiteProfile() {
    storeProfile.loadDefaultSiteProfile();
    if (import.meta.env.VITE_USE_WEB_PROFILE.toLowerCase() === 'true') {
        getSiteProfile()
            .then((res) => {
                storeProfile.updateSiteProfile(res);
            }).catch(() => {});
    }
}

onMounted(() => {
  storeMain.syncResolvedTheme(osThemeRef.value === 'dark' ? 'dark' : 'light');
  setMomentLocale(locale.value);
  syncThemeClass(theme.value);
  syncThemeVars();
  syncViewportLayout();
  window.addEventListener('resize', syncViewportLayout);
  loadSiteProfile();
  restoreUserSession();
});

watch([theme, themePreset], ([nextTheme]) => {
  syncThemeClass(nextTheme);
  syncThemeVars();
  safeLocalStorageSet('EVT_THEME', nextTheme);
});

watch([themeMode, osThemeRef], ([mode, osTheme]) => {
  if (mode !== 'system') {
    return;
  }

  storeMain.syncResolvedTheme(osTheme === 'dark' ? 'dark' : 'light');
});

watch(locale, (nextLocale) => {
  persistLocale(nextLocale);
  setMomentLocale(nextLocale);
});

onBeforeUnmount(() => {
  window.removeEventListener('resize', syncViewportLayout);
});
</script>

<style scoped lang="less">
.page-fade-enter-active,
.page-fade-leave-active {
  transition: opacity 0.24s ease, transform 0.24s ease;
}

.page-fade-enter-from,
.page-fade-leave-to {
  opacity: 0;
  transform: translateY(8px);
}
</style>
