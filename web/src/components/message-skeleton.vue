<template>
    <div class="skeleton-item" v-for="i in new Array(num)" :key="i">
        <div class="content">
            <n-skeleton text :repeat="2" />
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
    --message-skeleton-bg: transparent;
    padding: 12px;
    display: flex;
    background-color: var(--message-skeleton-bg);

    .content {
        width: 100%;
    }
}

:global(.dark) .skeleton-item {
    --message-skeleton-bg: rgba(16, 16, 20, 0.75);
}
</style>
