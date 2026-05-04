<template>
  <article class="event-timeline-item">
    <div class="event-timeline-flags">
      <span v-if="isFirst" class="event-flag">起点</span>
      <span v-if="isLatest" class="event-flag event-flag-latest">最新</span>
      <span v-if="isMilestone" class="event-flag event-flag-milestone">里程碑</span>
      <span class="event-flag event-flag-index">#{{ index + 1 }}</span>
    </div>
    <div class="event-timeline-item-head">
      <div class="event-timeline-user">
        <n-avatar round :size="34" :src="comment.user.avatar || DEFAULT_USER_AVATAR" />
        <div class="event-timeline-user-copy">
          <strong>{{ comment.user.nickname || comment.user.username }}</strong>
          <span>@{{ comment.user.username }}</span>
        </div>
      </div>
      <div class="event-timeline-meta">
        <span>{{ exactTimeLabel }}</span>
        <span class="event-timeline-relative">{{ timeLabel }}</span>
        <span v-if="comment.ip_loc">{{ comment.ip_loc }}</span>
      </div>
    </div>

    <div class="event-timeline-body">
      <div v-if="texts.length > 0" class="event-timeline-texts">
        <span
          v-for="content in texts"
          :key="content.id"
          class="event-timeline-text"
          @click.stop="doClickText($event)"
          v-html="parsePostTag(content.content).content"
        ></span>
      </div>
      <post-image v-if="images.length > 0" :imgs="images" />
    </div>

    <div class="event-timeline-actions">
      <span v-if="comment.is_essence == YesNoEnum.YES" class="event-timeline-badge">精选节点</span>
      <div class="event-timeline-actions-right">
        <n-button
          v-if="userInfo.id === postUserId"
          quaternary
          circle
          size="tiny"
          @click="execHighlightAction"
        >
          <template #icon>
            <n-icon v-if="comment.is_essence == YesNoEnum.NO">
              <ArrowBarToUp />
            </n-icon>
            <n-icon v-else>
              <ArrowBarDown />
            </n-icon>
          </template>
        </n-button>
        <n-popconfirm
          v-if="userInfo.is_admin || userInfo.id === comment.user.id"
          negative-text="取消"
          positive-text="确认"
          @positive-click="execDelAction"
        >
          <template #trigger>
            <n-button quaternary circle size="tiny">
              <template #icon>
                <n-icon>
                  <Trash />
                </n-icon>
              </template>
            </n-button>
          </template>
          是否删除这条时间节点？
        </n-popconfirm>
      </div>
    </div>
  </article>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { storeToRefs } from 'pinia';
import { Trash, ArrowBarToUp, ArrowBarDown } from '@vicons/tabler';
import { deleteComment, highlightComment } from '@/api/post';
import { parsePostTag } from '@/utils/content';
import { buildTagSearchRoute } from '@/utils/tagRoute';
import { useStoreMain } from '@/store/main';
import { useStoreProfile } from '@/store/profile';
import { useStoreUser } from '@/store/user';
import { DEFAULT_USER_AVATAR } from '@/utils/defaults';
import { YesNoEnum } from '@/utils/IEnum';
import { formatPrettyTime, formatTime } from '@/utils/formatTime';

const props = defineProps<{
  comment: Item.CommentProps;
  postUserId: number;
  index: number;
  total: number;
  isFirst: boolean;
  isLatest: boolean;
  isMilestone: boolean;
}>();

const emit = defineEmits<(e: 'reload') => void>();

const router = useRouter();
const storeMain = useStoreMain();
const storeProfile = useStoreProfile();
const storeUser = useStoreUser();
const { currentSpaceSlug } = storeToRefs(storeProfile);
const { userInfo } = storeToRefs(storeUser);

const texts = computed(() =>
  props.comment.contents.filter((content) => +content.type === 1 || +content.type === 2),
);
const images = computed(() =>
  props.comment.contents.filter((content) => +content.type === 3),
);
const timeLabel = computed(() => formatPrettyTime(props.comment.created_on));
const exactTimeLabel = computed(() => formatTime(props.comment.created_on));

const doClickText = (event: MouseEvent) => {
  const target = event.target as HTMLElement | null;
  if (!target?.dataset.detail) {
    return;
  }

  const detail = target.dataset.detail.split(':');
  if (detail.length !== 2) {
    return;
  }

  storeMain.doRefresh();
  if (detail[0] === 'tag') {
    router.push(buildTagSearchRoute(detail[1], currentSpaceSlug.value));
    return;
  }

  router.push({
    name: 'user',
    query: {
      s: detail[1],
    },
  });
};

const execDelAction = () => {
  deleteComment({ id: props.comment.id })
    .then(() => {
      window.$message.success('删除成功');
      emit('reload');
    })
    .catch(() => {
      window.$message.error('删除时间节点失败');
    });
};

const execHighlightAction = () => {
  highlightComment({ id: props.comment.id })
    .then(() => {
      window.$message.success('节点状态已更新');
      emit('reload');
    })
    .catch(() => {
      window.$message.error('更新节点状态失败');
    });
};
</script>

<style scoped lang="less">
.event-timeline-item {
  display: grid;
  gap: 12px;
  padding: 14px 16px;
  border-radius: 18px;
  border: 1px solid color-mix(in srgb, var(--panel-border) 82%, transparent);
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--panel-bg) 90%, transparent), color-mix(in srgb, var(--surface-subtle) 92%, transparent));
}

.event-timeline-flags {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.event-flag {
  display: inline-flex;
  align-items: center;
  min-height: 24px;
  padding: 0 9px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  background: color-mix(in srgb, var(--accent-soft-muted) 78%, transparent);
  color: var(--editor-text-subtle);
}

.event-flag-latest {
  background: color-mix(in srgb, var(--accent-soft) 84%, transparent);
  color: var(--accent-primary);
}

.event-flag-milestone {
  background: color-mix(in srgb, #f6c453 26%, var(--panel-bg));
  color: #a45c00;
}

.event-flag-index {
  background: transparent;
  border: 1px solid color-mix(in srgb, var(--panel-border) 82%, transparent);
}

.event-timeline-item-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 14px;
  flex-wrap: wrap;
}

.event-timeline-user {
  display: flex;
  align-items: center;
  gap: 10px;
}

.event-timeline-user-copy {
  display: grid;
  gap: 2px;

  strong {
    font-size: 14px;
    line-height: 1.4;
  }

  span {
    font-size: 12px;
    opacity: 0.7;
  }
}

.event-timeline-meta {
  display: flex;
  gap: 10px;
  align-items: center;
  flex-wrap: wrap;

  span {
    font-size: 12px;
    opacity: 0.68;
  }
}

.event-timeline-relative {
  opacity: 0.54;
}

.event-timeline-body {
  display: grid;
  gap: 10px;
}

.event-timeline-texts {
  display: grid;
  gap: 8px;
}

.event-timeline-text {
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.82;
  color: var(--editor-text-main);
}

.event-timeline-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.event-timeline-actions-right {
  display: flex;
  align-items: center;
  gap: 6px;
}

.event-timeline-badge {
  display: inline-flex;
  align-items: center;
  min-height: 28px;
  padding: 0 10px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent-soft) 88%, transparent);
  color: var(--accent-primary);
  font-size: 12px;
  font-weight: 700;
}

@media screen and (max-width: 821px) {
  .event-timeline-item {
    padding: 12px 14px;
    border-radius: 16px;
  }
}
</style>
