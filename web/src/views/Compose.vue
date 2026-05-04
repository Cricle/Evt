<template>
  <div class="compose-page">
    <main-nav :title="pageTitle" :back="true" />
    <div class="compose-shell">
      <section v-if="isEventMode" class="compose-event-hero">
        <div class="compose-event-hero-copy">
          <span class="compose-event-hero-kicker">事件时间轴</span>
          <h1>先发布事件主题，再持续追加节点</h1>
          <p>适合记录项目推进、活动进展、事故处理和任何需要长期更新的公共过程。</p>
        </div>
        <div class="compose-event-hero-steps">
          <span>1. 创建主题</span>
          <span>2. 追加节点</span>
          <span>3. 持续更新</span>
        </div>
      </section>
      <n-space vertical :size="14">
        <div class="compose-panel">
          <compose-editor
            :mode="composeMode"
            :quick-mode="quickMode"
            page-mode
            @post-success="handlePostSuccess"
          />
        </div>
      </n-space>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useStoreProfile } from '@/store/profile';
import { storeToRefs } from 'pinia';
import { resolveSpaceSlug } from '@/utils/spaces';
import { buildPostRoute } from '@/utils/tagRoute';

const route = useRoute();
const router = useRouter();
const storeProfile = useStoreProfile();
const { currentSpaceSlug } = storeToRefs(storeProfile);

const composeMode = computed<'post' | 'event'>(() =>
  route.query.mode === 'event' ? 'event' : 'post',
);
const quickMode = computed<'' | 'media'>(() =>
  route.query.quick === 'media' ? 'media' : '',
);
const isEventMode = computed(() => composeMode.value === 'event');

const pageTitle = computed(() => (composeMode.value === 'event' ? '创建事件' : '发布话题'));

onMounted(() => {
  const routeSpace = typeof route.query.space === 'string' ? route.query.space : '';
  currentSpaceSlug.value = resolveSpaceSlug(
    routeSpace || currentSpaceSlug.value,
    storeProfile.profile.defaultSpaceSlug,
  );
});

const handlePostSuccess = (post: Item.PostProps) => {
  router.replace(buildPostRoute(post.id, currentSpaceSlug.value));
};
</script>

<style scoped lang="less">
.compose-page {
  min-height: 100vh;
  background:
    radial-gradient(circle at left top, var(--page-hero-bg-glow), transparent 34%),
    radial-gradient(circle at top right, var(--page-hero-bg-accent), transparent 26%),
    linear-gradient(180deg, var(--page-hero-bg-base) 0%, var(--page-hero-bg-bottom) 100%);
}

.compose-shell {
  max-width: 880px;
  margin: 0 auto;
  padding: 24px 18px 80px;
  display: grid;
  gap: 14px;
}

.compose-panel {
  width: 100%;
}

.compose-event-hero {
  display: grid;
  gap: 14px;
  padding: 20px 22px;
  border: 1px solid color-mix(in srgb, var(--panel-border) 84%, transparent);
  border-radius: 26px;
  background:
    radial-gradient(circle at top right, color-mix(in srgb, var(--accent-soft) 74%, transparent), transparent 36%),
    color-mix(in srgb, var(--panel-bg) 90%, transparent);
  box-shadow: var(--panel-shadow);
}

.compose-event-hero-copy {
  display: grid;
  gap: 8px;

  h1 {
    margin: 0;
    font-size: 30px;
    line-height: 1.15;
  }

  p {
    margin: 0;
    font-size: 14px;
    line-height: 1.75;
    color: var(--editor-text-subtle);
  }
}

.compose-event-hero-kicker {
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.08em;
  color: var(--accent-primary);
}

.compose-event-hero-steps {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;

  span {
    display: inline-flex;
    align-items: center;
    min-height: 32px;
    padding: 0 12px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--accent-soft-muted) 88%, transparent);
    color: var(--editor-text-main);
    font-size: 12px;
    font-weight: 600;
  }
}

@media screen and (max-width: 821px) {
  .compose-shell {
    padding: 12px 10px 80px;
  }

  .compose-event-hero {
    padding: 16px 16px 18px;
    border-radius: 22px;
  }

  .compose-event-hero-copy h1 {
    font-size: 24px;
  }
}
</style>
