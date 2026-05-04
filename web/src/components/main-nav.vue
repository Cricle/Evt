<template>
    <div v-if="drawerModelShow">
        <n-drawer
            v-model:show="activeDrawerRef"
            :width="drawerWidth"
            :placement="placementRef"
            class="app-drawer"
        >
            <n-drawer-content :native-scrollbar="false" header-style="padding: 10px 12px;" body-content-style="padding: 0;">
                <template #header>
                    <div v-if="userInfo.id > 0" class="drawer-user-header">
                        <n-avatar
                            round
                            :size="30"
                            :src="userInfo.avatar || DEFAULT_USER_AVATAR"
                        />
                        <div class="drawer-user-copy">
                            <div class="drawer-user-name">{{ userInfo.nickname }}</div>
                            <div class="drawer-user-handle">@{{ userInfo.username }}</div>
                        </div>
                        <n-button class="drawer-logout-btn" quaternary circle size="small" @click="handleLogout">
                            <template #icon>
                                <n-icon><log-out-outline /></n-icon>
                            </template>
                        </n-button>
                    </div>
                </template>
                <sidebar />
            </n-drawer-content>
        </n-drawer>
    </div>
    <n-card
        size="small"
        :bordered="true"
        class="nav-title-card"
        header-style="padding: 10px 14px;"
        content-style="padding: 0;"
    >
        <template #header>
            <div class="navbar">
                <n-button
                    class="drawer-btn"
                    v-if="drawerModelShow && !back"
                    @click="activeDrawer"
                    quaternary
                    circle
                    size="medium"
                >
                    <template #icon>
                        <n-icon><dehaze-round /></n-icon>
                    </template>
                </n-button>
                <n-button
                    class="back-btn"
                    v-if="back"
                    @click="goBack"
                    quaternary
                    circle
                    size="small"
                >
                    <template #icon>
                        <n-icon><chevron-left-round /></n-icon>
                    </template>
                </n-button>

                <div class="title-shell">
                    <n-select
                        v-if="props.spaceOptions.length > 0"
                        :value="props.spaceValue"
                        :options="props.spaceOptions"
                        size="small"
                        class="title-space-select title-space-trigger"
                        :theme-overrides="spaceSelectThemeOverrides"
                        @update:value="emit('update:spaceValue', $event)"
                    />
                    <span v-else class="title-text">{{ props.title }}</span>
                </div>

                <n-button
                    v-if="props.actionLabel"
                    class="nav-action-btn"
                    type="primary"
                    secondary
                    :circle="props.actionIconOnly"
                    :round="!props.actionIconOnly"
                    size="small"
                    :title="props.actionLabel"
                    :aria-label="props.actionLabel"
                    @click="emit('action')"
                >
                    <template v-if="props.actionIcon === 'add'" #icon>
                        <n-icon><add-round /></n-icon>
                    </template>
                    <span v-if="!props.actionIconOnly">{{ props.actionLabel }}</span>
                </n-button>

                <n-switch
                    v-if="props.theme"
                    :value="theme === 'dark'"
                    @update:value="switchTheme"
                    size="small"
                    class="theme-switch-wrap"
                >
                    <template #checked-icon>
                        <n-icon :component="DarkModeOutlined" />
                    </template>
                    <template #unchecked-icon>
                        <n-icon :component="LightModeOutlined" />
                    </template>
                </n-switch>
            </div>
        </template>
    </n-card>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useStoreMain } from '@/store/main';
import { useStoreUser } from '@/store/user';
import { useRoute, useRouter } from 'vue-router';
import { NAvatar, NSelect, useMessage, DrawerPlacement } from 'naive-ui';
import {
  AddRound,
  LightModeOutlined,
  DarkModeOutlined,
  ChevronLeftRound,
  DehazeRound,
} from '@vicons/material';
import { LogOutOutline } from '@vicons/ionicons5';
import { storeToRefs } from 'pinia';
import { useStoreProfile } from '@/store/profile';
import { buildHomeRouteWithSpace } from '@/utils/tagRoute';
import { backWithFallback } from '@/utils/navigation';
import { DEFAULT_USER_AVATAR } from '@/utils/defaults';

const storeMain = useStoreMain();
const storeUser = useStoreUser();
const storeProfile = useStoreProfile();
const { desktopModelShow, drawerModelShow, theme } = storeToRefs(storeMain);
const { userInfo } = storeToRefs(storeUser);
const { currentSpaceSlug } = storeToRefs(storeProfile);

const route = useRoute();
const router = useRouter();
const activeDrawerRef = ref(false);
const placementRef = ref<DrawerPlacement>('left');
const drawerWidth = computed(() => {
  if (typeof window === 'undefined') {
    return 212;
  }
  return Math.min(Math.floor(window.innerWidth - 24), 212);
});

const props = withDefaults(
  defineProps<{
    title: string;
    back?: boolean;
    theme?: boolean;
    spaceValue?: string;
    spaceOptions?: Array<{ label: string; value: string }>;
    actionLabel?: string;
    actionIcon?: 'add' | '';
    actionIconOnly?: boolean;
  }>(),
  {
    title: '',
    back: false,
    theme: true,
    spaceValue: '',
    spaceOptions: () => [],
    actionLabel: '',
    actionIcon: '',
    actionIconOnly: false,
  },
);
const emit = defineEmits<{
  (e: 'update:spaceValue', value: string): void;
  (e: 'action'): void;
}>();
const spaceSelectThemeOverrides = computed(() => ({
  peers: {
    InternalSelection: {
      color: 'transparent',
      colorActive: 'transparent',
      colorDisabled: 'transparent',
      colorFocus: 'transparent',
      boxShadowFocus: 'none',
      border: 'none',
      borderActive: 'none',
      borderFocus: 'none',
      borderHover: 'none',
      borderRadius: '0',
      textColor: 'var(--editor-text-main)',
      placeholderColor: 'var(--editor-text-main)',
      arrowColor: 'var(--editor-text-subtle)',
    },
  },
}));
const switchTheme = (theme: boolean) => {
  if (theme) {
    storeMain.triggerTheme('dark');
  } else {
    storeMain.triggerTheme('light');
  }
};
const goBack = async () => {
  await backWithFallback(
    router,
    buildHomeRouteWithSpace({}, currentSpaceSlug.value),
    typeof window !== 'undefined' ? window.location : null,
    typeof window !== 'undefined' ? window.history.state : null,
  );
};
const activeDrawer = () => {
  activeDrawerRef.value = true;
};
const handleLogout = () => {
  storeUser.userLogout();
  storeMain.doRefresh();
  activeDrawerRef.value = false;
  router.push({
    name: 'home',
  });
};

watch(
  () => route.fullPath,
  () => {
    activeDrawerRef.value = false;
  },
);

watch(drawerModelShow, (isDrawerMode) => {
  if (!isDrawerMode) {
    activeDrawerRef.value = false;
  }
});

onMounted(() => {
  // 移动端特殊处理
  if (!desktopModelShow.value) {
    window.$message = useMessage();
  }
});
</script>

<style lang="less">
.nav-title-card {
    z-index: 99;
    width: 100%;
    top: 0;
    position: sticky;
    margin-top: 0;
    border-radius: 0;
    border-top: 0;
    border-left: 0;
    border-right: 0;
    border-bottom: 1px solid color-mix(in srgb, var(--panel-border) 78%, transparent);
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel-bg) 78%, transparent), color-mix(in srgb, var(--panel-bg) 62%, transparent)),
      color-mix(in srgb, var(--nav-bg) 72%, transparent);
    box-shadow:
      inset 0 1px 0 color-mix(in srgb, #ffffff 34%, transparent),
      0 6px 18px rgba(15, 23, 42, 0.05);
    backdrop-filter: blur(18px) saturate(140%);
    -webkit-backdrop-filter: blur(18px) saturate(140%);

    .navbar {
        min-height: 40px;
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 0;

        .drawer-btn,
        .back-btn {
            margin-right: 0;
        }

        .theme-switch-wrap {
            margin-left: auto;
        }
    }
}


.title-shell {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex: 1 1 auto;
    padding-right: 6px;
}

.title-text {
    flex: 0 0 auto;
    font-weight: 600;
    letter-spacing: 0.01em;
    font-size: 15px;
}

.title-space-select {
    width: auto;
    max-width: min(100%, 240px);
}

.nav-action-btn {
    flex: 0 0 auto;
    --n-border: none;
}

.title-space-trigger .n-base-selection {
    background: transparent;
    border: 0;
    box-shadow: none;
    border-radius: 0;
    padding-left: 0;
    padding-right: 0;
    min-height: auto;
}

.title-space-trigger .n-base-selection-label {
    font-weight: 600;
    padding-left: 0;
    padding-right: 0;
}

.title-space-trigger .n-base-selection-input {
    min-height: auto;
}

.title-space-trigger .n-base-selection-placeholder,
.title-space-trigger .n-base-selection__border,
.title-space-trigger .n-base-selection__state-border {
    display: none;
}

.title-space-trigger .n-base-selection .n-base-suffix {
    margin-left: 4px;
    opacity: 0.5;
}

.title-space-trigger .n-base-selection:hover .n-base-suffix {
    opacity: 0.75;
}

.title-space-trigger .n-base-selection.n-base-selection--active,
.title-space-trigger .n-base-selection.n-base-selection--focus {
    background: color-mix(in srgb, var(--accent-soft-muted) 48%, transparent);
    border-radius: 10px;
    padding-left: 8px;
    padding-right: 8px;
}

.title-space-trigger .n-base-selection.n-base-selection--active .n-base-selection-label,
.title-space-trigger .n-base-selection.n-base-selection--focus .n-base-selection-label {
    padding-left: 0;
    padding-right: 0;
}

.theme-switch-wrap .n-switch {
    background: color-mix(in srgb, var(--accent-soft-muted) 56%, transparent);
}

.app-drawer .n-drawer-content {
    padding: 0;
    overflow: hidden;
    border-radius: 0;
}

.drawer-user-header {
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 0;
}

.drawer-user-copy {
    min-width: 0;
    flex: 1 1 auto;
}

.drawer-user-name {
    font-size: 14px;
    font-weight: 600;
    line-height: 1.2;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.drawer-user-handle {
    margin-top: 2px;
    font-size: 12px;
    line-height: 1.2;
    opacity: 0.62;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.drawer-logout-btn {
    flex: 0 0 auto;
}

.app-drawer .n-drawer-mask {
    backdrop-filter: blur(3px);
}

.app-drawer .n-drawer-content .n-drawer-header,
.app-drawer .n-drawer-content .n-drawer-body-content-wrapper,
.app-drawer .n-drawer-content .n-drawer-body {
    padding: 0;
}

.app-drawer .n-drawer-body-content-wrapper {
    overflow: hidden;
}

.app-drawer .n-drawer {
    border-radius: 0;
}

@media screen and (max-width: 821px) {
    .nav-title-card {
        border-radius: 0;
    }

    .title-shell {
        padding-right: 0;
    }

    .title-space-select {
        width: min(48vw, 176px);
    }

    .nav-action-btn {
        min-width: 34px;
    }

    .theme-switch-wrap {
        transform: scale(0.96);
    }
}

.dark .nav-title-card {
    background-color: rgba(16, 22, 22, 0.62);
}
</style>
