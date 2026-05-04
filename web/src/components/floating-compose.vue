<template>
  <transition name="float-compose">
    <div v-if="showFloatingCompose" class="floating-compose-wrap">
      <n-dropdown
        placement="top-end"
        trigger="click"
        :options="composeOptions"
        @select="handleComposeSelect"
      >
        <button
          class="floating-compose"
          type="button"
          aria-label="创建内容"
          @click.stop.prevent
          @pointerdown="handlePointerDown"
          @pointerup="handlePointerUp"
          @pointerleave="clearLongPress"
          @pointercancel="clearLongPress"
        >
          <input
            ref="quickMediaInputRef"
            class="floating-compose-input"
            type="file"
            accept="image/*,video/*"
            multiple
            @change="handleQuickMediaPick"
          />
          <span class="floating-compose-plus">+</span>
        </button>
      </n-dropdown>
    </div>
  </transition>
</template>

<script setup lang="ts">
import { computed, h, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { NIcon, type DropdownOption } from 'naive-ui';
import { TOKEN_KEY, useStoreUser } from '@/store/user';
import { useStoreProfile } from '@/store/profile';
import { storeToRefs } from 'pinia';
import { buildComposeRoute, type ComposeMode } from '@/utils/tagRoute';
import { safeLocalStorageGet } from '@/utils/storage';
import { normalizeResolvedHref } from '@/utils/navigation';
import { CalendarOutline, CreateOutline, ImagesOutline } from '@vicons/ionicons5';
import { setPendingAttachmentSelection } from '@/utils/composeDraft';

const router = useRouter();
const route = useRoute();
const storeUser = useStoreUser();
const storeProfile = useStoreProfile();
const { userLogined } = storeToRefs(storeUser);
const { currentSpaceSlug } = storeToRefs(storeProfile);
const hasToken = typeof window !== 'undefined' && !!safeLocalStorageGet(TOKEN_KEY);
const quickMediaInputRef = ref<HTMLInputElement | null>(null);
const longPressTimer = ref<number | null>(null);
const longPressTriggered = ref(false);
const showFloatingCompose = computed(() => {
  const routeName = route.name;
  return routeName === 'space' && (userLogined.value || hasToken);
});

const renderIcon = (icon: typeof CalendarOutline) => () =>
  h(NIcon, null, {
    default: () => h(icon),
  });

const composeOptions = computed<DropdownOption[]>(() => [
  {
    label: '创建话题',
    key: 'post',
    icon: renderIcon(CreateOutline),
  },
  {
    label: '创建事件',
    key: 'event',
    icon: renderIcon(CalendarOutline),
  },
  {
    label: '快速图视频话题',
    key: 'quick-media',
    icon: renderIcon(ImagesOutline),
  },
]);

const goCompose = async (mode: ComposeMode, quick?: 'media') => {
  const target = buildComposeRoute(currentSpaceSlug.value, mode, quick);
  const resolved = router.resolve(target);
  if (route.fullPath === resolved.fullPath) {
    return;
  }

  try {
    await router.push(target);
  } catch {
    // fall through to location.assign below
  }

  if (router.currentRoute.value.fullPath !== resolved.fullPath && typeof window !== 'undefined') {
    window.location.assign(normalizeResolvedHref(resolved.href, resolved.fullPath));
  }
};

const clearLongPress = () => {
  if (longPressTimer.value !== null) {
    window.clearTimeout(longPressTimer.value);
    longPressTimer.value = null;
  }
};

const openQuickMediaPicker = () => {
  longPressTriggered.value = true;
  clearLongPress();
  quickMediaInputRef.value?.click();
};

const handlePointerDown = () => {
  longPressTriggered.value = false;
  clearLongPress();
  longPressTimer.value = window.setTimeout(() => {
    openQuickMediaPicker();
  }, 420);
};

const handlePointerUp = () => {
  window.setTimeout(() => {
    longPressTriggered.value = false;
  }, 0);
  clearLongPress();
};

const handleQuickMediaPick = async (event: Event) => {
  const input = event.target as HTMLInputElement;
  const files = Array.from(input.files || []);
  input.value = '';
  if (files.length === 0) {
    return;
  }

  setPendingAttachmentSelection({
    files,
    mode: 'post',
    source: 'quick-media',
  });
  await goCompose('post', 'media');
};

const handleComposeSelect = (key: string | number) => {
  if (key === 'quick-media') {
    openQuickMediaPicker();
    return;
  }
  if (key === 'event') {
    void goCompose('event');
    return;
  }
  if (longPressTriggered.value) {
    return;
  }
  void goCompose('post');
};
</script>

<style scoped lang="less">
.floating-compose {
  --floating-compose-bg: linear-gradient(135deg, #0f9f6e, #3dc788);
  --floating-compose-shadow:
    0 18px 40px rgba(16, 133, 90, 0.28),
    inset 0 1px 0 rgba(255, 255, 255, 0.22);
  --floating-compose-shadow-hover: 0 24px 50px rgba(16, 133, 90, 0.34);
  --floating-compose-text: #fff;
  position: fixed;
  right: 28px;
  bottom: 30px;
  z-index: 140;
  width: 64px;
  height: 64px;
  border: 0;
  border-radius: 999px;
  background: var(--floating-compose-bg);
  color: var(--floating-compose-text);
  box-shadow: var(--floating-compose-shadow);
  cursor: pointer;
  transition:
    transform 0.2s ease,
    box-shadow 0.2s ease,
    filter 0.2s ease;

  &:hover {
    transform: translateY(-3px) scale(1.03);
    box-shadow: var(--floating-compose-shadow-hover);
    filter: saturate(1.05);
  }

  &:active {
    transform: translateY(0) scale(0.98);
  }
}

.floating-compose-plus {
  display: inline-block;
  font-size: 36px;
  font-weight: 500;
  line-height: 1;
  transform: translateY(-1px);
}

.floating-compose-input {
  display: none;
}

.float-compose-enter-active,
.float-compose-leave-active {
  transition: all 0.24s ease;
}

.float-compose-enter-from,
.float-compose-leave-to {
  opacity: 0;
  transform: translateY(20px) scale(0.9);
}

:global(.dark) .floating-compose {
  --floating-compose-bg: linear-gradient(135deg, #17855d, #2fc08a);
  --floating-compose-shadow:
    0 18px 44px rgba(7, 15, 12, 0.52),
    inset 0 1px 0 rgba(255, 255, 255, 0.16);
  --floating-compose-shadow-hover: 0 24px 54px rgba(7, 15, 12, 0.62);
  --floating-compose-text: #fff;
}

@media screen and (max-width: 821px) {
  .floating-compose {
    right: 16px;
    bottom: 18px;
    width: 56px;
    height: 56px;
  }

  .floating-compose-plus {
    font-size: 32px;
  }
}
</style>
