<template>
  <div class="emoji-reaction-picker">
    <button
      v-for="emoji in emojis"
      :key="emoji"
      type="button"
      class="emoji-btn"
      @click.stop="emit('select', emoji)"
    >
      {{ emoji }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { DEFAULT_REACTION_EMOJIS } from '@/utils/reactions';

withDefaults(
  defineProps<{
    emojis?: string[];
  }>(),
  {
    emojis: () => DEFAULT_REACTION_EMOJIS,
  },
);

const emit = defineEmits<{
  (e: 'select', emoji: string): void;
}>();
</script>

<style scoped lang="less">
.emoji-reaction-picker {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.96);
  box-shadow: 0 16px 36px rgba(23, 51, 39, 0.12);
}

.emoji-btn {
  width: 34px;
  height: 34px;
  border: 0;
  border-radius: 999px;
  background: transparent;
  font-size: 18px;
  line-height: 1;
  cursor: pointer;
  transition:
    transform 0.16s ease,
    background-color 0.16s ease;

  &:hover {
    transform: translateY(-1px) scale(1.08);
    background: rgba(16, 136, 91, 0.08);
  }
}

.dark {
  .emoji-reaction-picker {
    background: rgba(29, 29, 35, 0.96);
    box-shadow: 0 16px 36px rgba(0, 0, 0, 0.3);
  }
}
</style>
