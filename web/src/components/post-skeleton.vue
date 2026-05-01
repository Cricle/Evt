<template>
    <div class="skeleton-item" v-for="i in new Array(num)" :key="i">
        <div class="user">
            <n-skeleton circle size="small" />
        </div>
        <div class="content">
            <n-skeleton text :repeat="3" />
            <n-skeleton text style="width: 60%" />
        </div>
    </div>
</template>

<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    num: number;
  }>(),
  {
    num: 1,
  },
);
</script>

<style lang="less" scoped>
.skeleton-item {
    --post-skeleton-bg: transparent;
    padding: 12px;
    display: flex;
    background-color: var(--post-skeleton-bg);

    .user {
        width: 42px;
    }

    .content {
        width: calc(100% - 42px);
    }
}

:global(.dark) .skeleton-item {
    --post-skeleton-bg: rgba(16, 16, 20, 0.75);
}
</style>
