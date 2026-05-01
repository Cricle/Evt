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
  --emoji-picker-border: rgba(18, 75, 51, 0.08);
  --emoji-picker-bg: rgba(255, 255, 255, 0.98);
  --emoji-picker-shadow: 0 18px 40px rgba(23, 51, 39, 0.12);
  display: grid;
  grid-template-columns: repeat(6, minmax(0, 1fr));
  gap: 8px;
  padding: 10px;
  border: 1px solid var(--emoji-picker-border);
  border-radius: 18px;
  background: var(--emoji-picker-bg);
  box-shadow: var(--emoji-picker-shadow);
  backdrop-filter: blur(12px);
}

.emoji-btn {
  --emoji-btn-bg: rgba(16, 136, 91, 0.04);
  --emoji-btn-hover-bg: rgba(16, 136, 91, 0.12);
  --emoji-btn-hover-shadow: 0 8px 18px rgba(16, 136, 91, 0.14);
  width: 40px;
  height: 40px;
  border: 0;
  border-radius: 12px;
  background: var(--emoji-btn-bg);
  font-size: 21px;
  line-height: 1;
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

:global(.dark) .emoji-reaction-picker {
  --emoji-picker-border: rgba(148, 163, 184, 0.14);
  --emoji-picker-bg: rgba(24, 28, 32, 0.98);
  --emoji-picker-shadow: 0 18px 40px rgba(0, 0, 0, 0.28);
}

:global(.dark) .emoji-btn {
  --emoji-btn-bg: rgba(99, 226, 183, 0.08);
  --emoji-btn-hover-bg: rgba(99, 226, 183, 0.16);
  --emoji-btn-hover-shadow: 0 8px 18px rgba(0, 0, 0, 0.22);
}
</style>
