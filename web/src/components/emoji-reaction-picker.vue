<template>
  <div class="emoji-reaction-picker">
    <div v-for="group in normalizedGroups" :key="group.label" class="emoji-group">
      <div class="emoji-group-label">{{ group.label }}</div>
      <n-flex :size="[6, 6]">
        <n-button
          v-for="emoji in group.emojis"
          :key="emoji"
          class="emoji-btn"
          quaternary
          circle
          size="small"
          @click.stop="emit('select', emoji)"
        >
          {{ emoji }}
        </n-button>
      </n-flex>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { DEFAULT_REACTION_EMOJIS, REACTION_EMOJI_GROUPS, type ReactionEmojiGroup } from '@/utils/reactions';

const props = withDefaults(
  defineProps<{
    emojis?: string[];
    groups?: ReactionEmojiGroup[];
  }>(),
  {
    emojis: () => DEFAULT_REACTION_EMOJIS,
    groups: () => REACTION_EMOJI_GROUPS,
  },
);

const emit = defineEmits<{
  (e: 'select', emoji: string): void;
}>();

const normalizedGroups = computed(() => {
  if (props.groups.length > 0) {
    return props.groups;
  }
  return [
    {
      label: '全部',
      emojis: props.emojis,
    },
  ];
});
</script>

<style scoped lang="less">
.emoji-reaction-picker {
  padding: 8px;
  width: min(320px, 72vw);
  max-height: min(320px, 56vh);
  overflow: auto;
  border: 1px solid var(--accent-soft-ring);
  border-radius: 16px;
  background: var(--surface-elevated);
  box-shadow: 0 18px 40px rgba(23, 51, 39, 0.12);
  backdrop-filter: blur(12px);
}

.emoji-group + .emoji-group {
  margin-top: 10px;
}

.emoji-group-label {
  margin-bottom: 6px;
  padding-left: 2px;
  font-size: 11px;
  line-height: 1;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  opacity: 0.54;
}

.emoji-btn {
  width: 32px;
  height: 32px;
  font-size: 17px;
  line-height: 1;
  font-family: var(--emoji-font-stack);
  --n-color: color-mix(in srgb, var(--accent-soft-muted) 100%, transparent);
  --n-color-hover: color-mix(in srgb, var(--accent-soft-hover) 100%, transparent);
  --n-color-pressed: color-mix(in srgb, var(--accent-soft-hover) 100%, transparent);
  --n-color-focus: color-mix(in srgb, var(--accent-soft-hover) 100%, transparent);
  --n-ripple-color: transparent;
}

:global(.dark) .emoji-btn {
  --n-color: color-mix(in srgb, var(--accent-soft-dark) 100%, transparent);
  --n-color-hover: color-mix(in srgb, var(--accent-soft-hover-dark) 100%, transparent);
  --n-color-pressed: color-mix(in srgb, var(--accent-soft-hover-dark) 100%, transparent);
  --n-color-focus: color-mix(in srgb, var(--accent-soft-hover-dark) 100%, transparent);
}
</style>
