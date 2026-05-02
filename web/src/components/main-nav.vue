<template>
    <div v-if="drawerModelShow">
        <n-drawer
            v-model:show="activeDrawerRef"
            :width="drawerWidth"
            :placement="placementRef"
            class="app-drawer"
        >
            <n-drawer-content>
                <sidebar />
            </n-drawer-content>
        </n-drawer>
    </div>
    <n-card size="small" :bordered="true" class="nav-title-card">
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
import { useRoute, useRouter } from 'vue-router';
import { NSelect, useMessage, useOsTheme, DrawerPlacement } from 'naive-ui';
import {
  AddRound,
  LightModeOutlined,
  DarkModeOutlined,
  ChevronLeftRound,
  DehazeRound,
} from '@vicons/material';
import { storeToRefs } from 'pinia';
import { useStoreProfile } from '@/store/profile';
import { buildHomeRouteWithSpace } from '@/utils/tagRoute';
import { backWithFallback } from '@/utils/navigation';

const storeMain = useStoreMain();
const storeProfile = useStoreProfile();
const { desktopModelShow, drawerModelShow, theme } = storeToRefs(storeMain);
const { currentSpaceSlug } = storeToRefs(storeProfile);

const route = useRoute();
const router = useRouter();
const activeDrawerRef = ref(false);
const placementRef = ref<DrawerPlacement>('left');
const drawerWidth = computed(() => {
  if (typeof window === 'undefined') {
    return 320;
  }
  return Math.min(Math.floor(window.innerWidth - 16), 320);
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
const switchTheme = (theme: boolean) => {
  if (theme) {
    localStorage.setItem('EVT_THEME', 'dark');
    storeMain.triggerTheme('dark');
  } else {
    localStorage.setItem('EVT_THEME', 'light');
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
  if (!localStorage.getItem('EVT_THEME')) {
    switchTheme((useOsTheme() as unknown as string) === 'dark');
  }
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
    border-radius: 0;
    border-bottom: 0;
    background-color: var(--nav-bg);
    backdrop-filter: blur(12px);

    .navbar {
        min-height: 30px;
        display: flex;
        align-items: center;
        gap: 8px;

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
    gap: 10px;
    min-width: 0;
    flex: 1 1 auto;
    padding-right: 8px;
}

.title-text {
    flex: 0 0 auto;
    font-weight: 600;
}

.title-space-select {
    width: 220px;
    max-width: 100%;
}

.nav-action-btn {
    flex: 0 0 auto;
}

.title-space-trigger .n-base-selection {
    background: transparent;
    border: 0;
    box-shadow: none;
    padding-left: 0;
}

.title-space-trigger .n-base-selection-label {
    font-weight: 600;
}

.app-drawer .n-drawer-content {
    padding: 0;
    overflow: hidden;
}

.app-drawer .n-drawer-body-content-wrapper {
    overflow: hidden;
}

@media screen and (max-width: 821px) {
    .title-shell {
        padding-right: 4px;
    }

    .title-space-select {
        width: min(46vw, 180px);
    }

    .nav-action-btn {
        min-width: 34px;
    }
}

.dark .nav-title-card {
    background-color: rgba(16, 16, 20, 0.75);
}
</style>
