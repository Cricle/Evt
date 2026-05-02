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
  --emoji-picker-border: var(--accent-soft-ring);
  --emoji-picker-bg: var(--surface-elevated);
  --emoji-picker-shadow: 0 18px 40px rgba(23, 51, 39, 0.12);
  display: grid;
  grid-template-columns: repeat(6, minmax(0, 1fr));
  gap: 5px;
  padding: 7px;
  border: 1px solid var(--emoji-picker-border);
  border-radius: 16px;
  background: var(--emoji-picker-bg);
  box-shadow: var(--emoji-picker-shadow);
  backdrop-filter: blur(12px);
}

.emoji-btn {
  --emoji-btn-bg: var(--accent-soft-muted);
  --emoji-btn-hover-bg: var(--accent-soft-hover);
  --emoji-btn-hover-shadow: var(--shadow-accent-mini);
  width: 32px;
  height: 32px;
  border: 0;
  border-radius: 10px;
  background: var(--emoji-btn-bg);
  font-size: 17px;
  line-height: 1;
  font-family: var(--emoji-font-stack);
  cursor: pointer;
  transition:
    transform 0.16s ease,
    background-color 0.16s ease,
    box-shadow 0.16s ease;

  &:hover {
    transform: translateY(-1px) scale(1.04);
    background: var(--emoji-btn-hover-bg);
    box-shadow: var(--emoji-btn-hover-shadow);
  }
}

@media screen and (max-width: 821px) {
  .emoji-reaction-picker {
    grid-template-columns: repeat(5, minmax(0, 1fr));
  }
}
</style>
