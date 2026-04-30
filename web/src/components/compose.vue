<template>
  <compose-editor v-bind="$attrs" @post-success="emit('post-success', $event)" />
</template>

<script setup lang="ts">
import ComposeEditor from '@/components/compose-editor.vue';

const emit = defineEmits<{
  (e: 'post-success', post: Item.PostProps): void;
}>();
</script>
