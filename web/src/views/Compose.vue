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
import { buildPostRoute } from '@/utils/tagRoute';

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
</style>
