<template>
  <n-popover trigger="click" placement="top-start" :disabled="readonly">
    <template #trigger>
      <div class="reaction-bar" :class="{ 'reaction-bar-readonly': readonly }" @click.stop>
        <template v-if="visibleReactions.length > 0">
          <button
            v-for="reaction in visibleReactions"
            :key="reaction.emoji"
            class="reaction-chip"
            :class="{ 'reaction-chip-active': reaction.active }"
            :title="reaction.users.map((user) => user.nickname || user.username).join('、')"
            type="button"
            @click.stop="emit('select', reaction.emoji)"
          >
            <span>{{ reaction.emoji }}</span>
            <span>{{ reaction.count }}</span>
          </button>
          <div v-if="hiddenReactionsCount > 0" class="reaction-chip reaction-chip-more">
            +{{ hiddenReactionsCount }}
          </div>
          <div v-if="showAddButton" class="reaction-chip reaction-chip-add">+</div>
        </template>
        <template v-else>
          <div class="reaction-chip reaction-chip-empty">
            <span>😀</span>
            <span>表情回复</span>
            <span v-if="count > 0">{{ count }}</span>
          </div>
        </template>
      </div>
    </template>
    <emoji-reaction-picker @select="emit('select', $event)" />
  </n-popover>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { ReactionGroup } from '@/utils/reactions';
import EmojiReactionPicker from '@/components/emoji-reaction-picker.vue';

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

const emit = defineEmits<{
  (e: 'select', emoji: string): void;
}>();

const visibleReactions = computed(() => props.reactions.slice(0, props.maxVisible));
const hiddenReactionsCount = computed(() => Math.max(props.reactions.length - visibleReactions.value.length, 0));
</script>

<style scoped lang="less">
.reaction-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  cursor: pointer;
}

.reaction-bar-readonly {
  cursor: default;
}

.reaction-chip {
  --reaction-chip-bg: rgba(16, 136, 91, 0.08);
  --reaction-chip-text: rgba(18, 43, 31, 0.92);
  --reaction-chip-hover-bg: rgba(16, 136, 91, 0.14);
  --reaction-chip-hover-shadow: 0 8px 20px rgba(16, 136, 91, 0.12);
  --reaction-chip-active-bg: rgba(16, 136, 91, 0.2);
  --reaction-chip-active-shadow: inset 0 0 0 1px rgba(16, 136, 91, 0.16);
  display: inline-flex;
  align-items: center;
  gap: 5px;
  min-height: 30px;
  padding: 0 10px;
  border: 0;
  border-radius: 999px;
  background: var(--reaction-chip-bg);
  color: var(--reaction-chip-text);
  font-size: 13px;
  transition:
    transform 0.18s ease,
    background-color 0.18s ease,
    box-shadow 0.18s ease;

  &:hover {
    transform: translateY(-1px);
    background: var(--reaction-chip-hover-bg);
    box-shadow: var(--reaction-chip-hover-shadow);
  }
}

.reaction-chip-active {
  background: var(--reaction-chip-active-bg);
  box-shadow: var(--reaction-chip-active-shadow);
}

.reaction-chip-add,
.reaction-chip-more {
  font-weight: 700;
}

.reaction-chip-empty {
  font-weight: 600;
}

.reaction-chip span:first-child {
  font-size: 15px;
  line-height: 1;
}

:global(.dark) .reaction-chip {
  --reaction-chip-bg: rgba(99, 226, 183, 0.12);
  --reaction-chip-text: rgba(241, 245, 249, 0.94);
  --reaction-chip-hover-bg: rgba(99, 226, 183, 0.18);
  --reaction-chip-hover-shadow: 0 8px 20px rgba(0, 0, 0, 0.22);
  --reaction-chip-active-bg: rgba(99, 226, 183, 0.22);
  --reaction-chip-active-shadow: inset 0 0 0 1px rgba(148, 163, 184, 0.16);
}
</style>
