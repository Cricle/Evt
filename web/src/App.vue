<template>
    <n-config-provider :theme="iTheme">
        <n-message-provider>
            <n-dialog-provider>
                <div
                    class="app-container"
                    :class="{ dark: iTheme?.name === 'dark', mobile: !desktopModelShow }"
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
import { onBeforeUnmount, onMounted, computed } from 'vue';
import { useStoreMain } from '@/store/main';
import { darkTheme } from 'naive-ui';
import { getSiteProfile } from '@/api/site';
import { useStoreProfile } from '@/store/profile';
import { storeToRefs } from 'pinia';
import { restoreUserSession } from '@/utils/session';

const storeMain = useStoreMain();
const storeProfile = useStoreProfile();
const { theme, desktopModelShow } = storeToRefs(storeMain);

const iTheme = computed(() => (theme.value === 'dark' ? darkTheme : null));
const syncViewportLayout = () => storeMain.syncViewportLayout();

function loadSiteProfile() {
    storeProfile.loadDefaultSiteProfile();
    if (import.meta.env.VITE_USE_WEB_PROFILE.toLowerCase() === 'true') {
        getSiteProfile()
            .then((res) => {
                storeProfile.updateSiteProfile(res);
            }).catch((err) => {
                console.log(err);
            });
    }
}

onMounted(() => {
  syncViewportLayout();
  window.addEventListener('resize', syncViewportLayout);
  loadSiteProfile();
  restoreUserSession();
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
