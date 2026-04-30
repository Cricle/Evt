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
import { useRouter } from 'vue-router';

const router = useRouter();

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
  min-height: 100vh;
  background:
    radial-gradient(circle at top right, rgba(61, 199, 136, 0.18), transparent 26%),
    linear-gradient(180deg, #f5faf7 0%, #edf1ef 100%);
}

.compose-shell {
  padding: 24px 18px 72px;
}

.compose-panel {
  max-width: 840px;
  margin: 0 auto;
}

@media screen and (max-width: 821px) {
  .compose-shell {
    padding: 16px 12px 64px;
  }
}
</style>
