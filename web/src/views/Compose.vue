<template>
  <div class="compose-page">
    <main-nav title="发布动态" :back="true" />
    <div class="compose-shell">
      <section class="compose-panel">
        <compose-editor page-mode @post-success="handlePostSuccess" />
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useStoreProfile } from '@/store/profile';
import { storeToRefs } from 'pinia';
import { resolveSpaceSlug } from '@/utils/spaces';

const route = useRoute();
const router = useRouter();
const storeProfile = useStoreProfile();
const { currentSpaceSlug } = storeToRefs(storeProfile);

onMounted(() => {
  const routeSpace = typeof route.query.space === 'string' ? route.query.space : '';
  currentSpaceSlug.value = resolveSpaceSlug(
    routeSpace || currentSpaceSlug.value,
    storeProfile.profile.defaultSpaceSlug,
  );
});

const handlePostSuccess = (post: Item.PostProps) => {
  router.replace({
    name: 'post',
    query: {
      id: post.id,
    },
  });
};
</script>

<style scoped lang="less">
.compose-page {
  --compose-bg-base: #f4f8f4;
  --compose-bg-glow: rgba(255, 255, 255, 0.72);
  --compose-bg-top: rgba(24, 160, 88, 0.14);
  --compose-bg-bottom: #edf4ef;
  min-height: 100vh;
  background:
    radial-gradient(circle at left top, var(--compose-bg-glow), transparent 34%),
    radial-gradient(circle at top right, var(--compose-bg-top), transparent 26%),
    linear-gradient(180deg, var(--compose-bg-base) 0%, var(--compose-bg-bottom) 100%);
}

.compose-shell {
  padding: 24px 18px 80px;
}

.compose-panel {
  max-width: 840px;
  margin: 0 auto;
  width: 100%;
}

@media screen and (max-width: 821px) {
  .compose-shell {
    padding: 12px 10px 80px;
  }
}

:global(.dark) .compose-page {
  --compose-bg-base: #0d1212;
  --compose-bg-glow: rgba(25, 33, 33, 0.46);
  --compose-bg-top: rgba(99, 226, 183, 0.12);
  --compose-bg-bottom: #121818;
}
</style>
