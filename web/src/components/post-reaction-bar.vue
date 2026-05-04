<template>
  <n-popover trigger="click" placement="top-start" :disabled="readonly">
    <template #trigger>
      <n-flex class="reaction-bar" :class="{ 'reaction-bar-readonly': readonly }" :size="[6, 6]" @click.stop>
        <template v-if="visibleReactions.length > 0">
          <n-button
            v-for="reaction in visibleReactions"
            :key="reaction.emoji"
            class="reaction-chip"
            :class="{ 'reaction-chip-active': reaction.active }"
            :title="reaction.users.map((user) => user.nickname || user.username).join('、')"
            quaternary
            round
            size="tiny"
            @click.stop="emit('select', reaction.emoji)"
          >
            <span class="reaction-chip-body">
              <span class="reaction-chip-emoji">{{ reaction.emoji }}</span>
              <span class="reaction-chip-count">{{ reaction.count }}</span>
            </span>
          </n-button>
          <n-button
            v-if="hiddenReactionsCount > 0"
            class="reaction-chip reaction-chip-more"
            quaternary
            round
            size="tiny"
          >
            +{{ hiddenReactionsCount }}
          </n-button>
          <n-button
            v-if="showAddButton"
            class="reaction-chip reaction-chip-add"
            quaternary
            round
            size="tiny"
          >
            +
          </n-button>
        </template>
        <template v-else>
          <n-button class="reaction-chip reaction-chip-empty" quaternary round size="tiny">
            <span class="reaction-chip-body">
              <span class="reaction-chip-emoji">😀</span>
              <span v-if="count > 0" class="reaction-chip-count">{{ count }}</span>
            </span>
          </n-button>
        </template>
      </n-flex>
    </template>
    <emoji-reaction-picker @select="emit('select', $event)" />
  </n-popover>
</template>

<script setup lang="ts">
import EmojiReactionPicker from '@/components/emoji-reaction-picker.vue';
import type { ReactionGroup } from '@/utils/reactions';
import { computed } from 'vue';

const props = withDefaults(
  defineProps<{
    reactions?: ReactionGroup[];
    count?: number;
    maxVisible?: number;
    readonly?: boolean;
    showAddButton?: boolean;
  }>(),
  {
    reactions: () => [],
    count: 0,
    maxVisible: 6,
    readonly: false,
    showAddButton: true,
  },
);

const emit = defineEmits<(e: 'select', emoji: string) => void>();

const visibleReactions = computed(() =>
  props.reactions.slice(0, props.maxVisible),
);
const hiddenReactionsCount = computed(() =>
  Math.max(props.reactions.length - visibleReactions.value.length, 0),
);
</script>

<style scoped lang="less">
.reaction-bar {
  cursor: pointer;
}

.reaction-bar-readonly {
  cursor: default;
}

.reaction-chip {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 24px;
  padding: 0 7px;
  font-size: 10px;
  --n-color-hover: color-mix(in srgb, var(--accent-soft-hover) 100%, transparent);
  --n-color-pressed: color-mix(in srgb, var(--accent-soft-hover) 100%, transparent);
  --n-color-focus: color-mix(in srgb, var(--accent-soft-hover) 100%, transparent);
  --n-text-color: var(--text-link-secondary);
  --n-text-color-hover: var(--text-link-secondary);
  --n-text-color-pressed: var(--text-link-secondary);
  --n-text-color-focus: var(--text-link-secondary);
  --n-color: color-mix(in srgb, var(--accent-soft) 100%, transparent);
  --n-ripple-color: transparent;
}

.reaction-chip :deep(.n-button__content) {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
}

.reaction-chip-body {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 3px;
  line-height: 1;
}

.reaction-chip-active {
  --n-color: color-mix(in srgb, var(--accent-soft-strong) 100%, transparent);
  --n-color-hover: color-mix(in srgb, var(--accent-soft-strong) 100%, transparent);
  --n-color-pressed: color-mix(in srgb, var(--accent-soft-strong) 100%, transparent);
  --n-color-focus: color-mix(in srgb, var(--accent-soft-strong) 100%, transparent);
  box-shadow: inset 0 0 0 1px var(--accent-soft-ring);
}

.reaction-chip-add,
.reaction-chip-more {
  font-weight: 700;
  min-width: 24px;
  padding-left: 0;
  padding-right: 0;
}

.reaction-chip-empty {
  font-weight: 600;
  min-width: 28px;
}

.reaction-chip-emoji {
  font-size: 11px;
  line-height: 1;
  font-family: var(--emoji-font-stack);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
}

.reaction-chip-count,
.reaction-chip-label {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
}

.reaction-chip-count {
  font-size: 9px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  font-family: var(--font-family);
  min-width: 0.75em;
  transform: translateY(-0.5px);
}

:global(.dark) .reaction-chip {
  --n-color: color-mix(in srgb, var(--accent-soft-dark) 100%, transparent);
  --n-color-hover: color-mix(in srgb, var(--accent-soft-hover-dark) 100%, transparent);
  --n-color-pressed: color-mix(in srgb, var(--accent-soft-hover-dark) 100%, transparent);
  --n-color-focus: color-mix(in srgb, var(--accent-soft-hover-dark) 100%, transparent);
}

:global(.dark) .reaction-chip-active {
  --n-color: color-mix(in srgb, var(--accent-soft-strong-dark) 100%, transparent);
  --n-color-hover: color-mix(in srgb, var(--accent-soft-strong-dark) 100%, transparent);
  --n-color-pressed: color-mix(in srgb, var(--accent-soft-strong-dark) 100%, transparent);
  --n-color-focus: color-mix(in srgb, var(--accent-soft-strong-dark) 100%, transparent);
  box-shadow: inset 0 0 0 1px var(--accent-soft-ring-dark);
}
</style>
