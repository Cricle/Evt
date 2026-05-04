<template>
  <div
    class="message-item"
    :class="{
      unread: isNotWhisperSender && message.is_read === 0,
      whisper: message.type === 4,
      inbound: isWhisperReceiver,
      outbound: isWhisperSender,
    }"
    @click="handleReadMessage(message)"
  >
    <n-thing content-indented>
      <template #avatar>
        <n-avatar round :size="30" :src="messageAvatar
          " />
      </template>
      <template #header>
        <div class="sender-wrap">
          <span class="nickname" v-if="(message.type != 4 && message.sender_user.id > 0) || isWhisperReceiver">
            <router-link @click.stop class="username-link" :to="{
              name: 'user',
              query: {
                s: message.sender_user.username,
              },
            }">
              {{ message.sender_user.nickname }}
            </router-link>
            <span v-if="desktopModelShow" class="username">
              @{{ message.sender_user.username }}
            </span>
          </span>
          <span class="nickname" v-else-if="isWhisperSender">
            <router-link @click.stop class="username-link" :to="{
              name: 'user',
              query: {
                s: message.receiver_user.username,
              },
            }">
              {{ message.receiver_user.nickname }}
            </router-link>
            <span v-if="desktopModelShow" class="username">
              @{{ message.receiver_user.username }}
            </span>
          </span>
          <span class="nickname" v-else> 系统 </span>
          <n-tag v-if="isWhisperSender" class="top-tag" type="info" size="small" round :bordered="false">
            发出
            <template #icon>
              <n-icon :component="CheckmarkCircle" />
            </template>
          </n-tag>
          <n-tag v-if="message.type == 4 && message.receiver_user_id == userInfo.id" class="top-tag" type="warning" size="small" round :bordered="false">
            收到
            <template #icon>
              <n-icon :component="CheckmarkCircle" />
            </template>
          </n-tag>
        </div>
      </template>
      <template #header-extra>
        <span class="timestamp">
          <n-badge v-if="isNotWhisperSender && message.is_read === 0" dot processing />
          <span class="timestamp-txt">
            {{ formatRelativeTime(message.created_on) }}
          </span>
          <n-dropdown placement="bottom-end" trigger="click" size="small" :options="actionOpts" @select="handleAction">
            <n-button quaternary circle>
              <template #icon>
                <n-icon>
                  <more-horiz-filled />
                </n-icon>
              </template>
            </n-button>
          </n-dropdown>
        </span>
      </template>
      <template #description>
          <n-alert :show-icon="false" class="brief-wrap" :type="!isNotWhisperSender || message.is_read > 0 ? 'default' : 'success'">
            <div v-if="message.type != 4" class="brief-content">
              {{ message.brief }}
              <span v-if="message.type === 1 || message.type === 2 || message.type === 3" @click.stop="viewDetail(message)" class="hash-link view-link">
                <n-icon>
                  <share-outline />
              </n-icon> 查看详情
            </span>
          </div>

          <div v-if="message.type === 4" class="whisper-content-wrap">
            <div class="whisper-bubble">
              <span class="whisper-bubble-label">{{ isWhisperSender ? '你发送给对方' : '对方发给你' }}</span>
              <p>{{ message.content }}</p>
            </div>
          </div>

          <div v-if="message.type === 5" class="requesting-friend-wrap">
            {{ message.content }}
            <span v-if="message.reply_id === 1" @click.stop="agreeAddFriend(message)" class="hash-link view-link">
              <n-icon>
                <checkmark-outline />
              </n-icon> 同意
            </span>
            <span v-if="message.reply_id === 1" @click.stop="rejectAddFriend(message)" class="hash-link view-link">
              <n-icon>
                <close-outline />
              </n-icon> 拒绝
            </span>
            <span v-if="message.reply_id === 2" class="status-info">
              <n-icon>
                <checkmark-done-outline />
              </n-icon> 已同意
            </span>
            <span v-if="message.reply_id === 3" class="status-info">
              <n-icon>
                <close-outline />
              </n-icon> 已拒绝
            </span>
          </div>
        </n-alert>
      </template>
    </n-thing>
  </div>
</template>

<script setup lang="ts">
import { h, computed } from 'vue';
import type { Component } from 'vue';
import { NIcon, useDialog, DropdownOption } from 'naive-ui';
import { useStoreMain } from '@/store/main';
import { useStoreUser } from '@/store/user';
import { useRouter } from 'vue-router';
import {
  ShareOutline,
  CheckmarkOutline,
  CloseOutline,
  CheckmarkDoneOutline,
  PaperPlaneOutline,
  CheckmarkCircle,
  BodyOutline,
  WalkOutline,
} from '@vicons/ionicons5';
import { formatRelativeTime } from '@/utils/formatTime';
import { MoreHorizFilled } from '@vicons/material';
import { storeToRefs } from 'pinia';
import { Api } from '@/utils/request';
import UserAction from '@/composables/useUserAction';
import { DEFAULT_USER_AVATAR } from '@/utils/defaults';
import { useStoreProfile } from '@/store/profile';
import { buildPostRoute } from '@/utils/tagRoute';
import { markLegacyMessageRead } from '@/utils/messageTransport';

const router = useRouter();

const storeMain = useStoreMain();
const storeUser = useStoreUser();
const storeProfile = useStoreProfile();
const { desktopModelShow } = storeToRefs(storeMain);
const { userInfo } = storeToRefs(storeUser);
const { currentSpaceSlug } = storeToRefs(storeProfile);

const dialog = useDialog();
const props = withDefaults(
  defineProps<{
    message: Item.MessageProps;
  }>(),
  {},
);

const renderIcon = (icon: Component) => {
  return () => {
    return h(NIcon, null, {
      default: () => h(icon),
    });
  };
};

const actionOpts = computed(() => {
  const user =
    props.message.type === 4 &&
      props.message.sender_user_id === userInfo.value.id
      ? props.message.receiver_user
      : props.message.sender_user;
  const options: DropdownOption[] = [
    {
      label: `私信 @${user.username}`,
      key: 'whisper',
      icon: renderIcon(PaperPlaneOutline),
    },
  ];
  if (userInfo.value.id !== user.id) {
    if (user.is_following) {
      options.push({
        label: `取消关注 @${user.username}`,
        key: 'unfollow',
        icon: renderIcon(WalkOutline),
      });
    } else {
      options.push({
        label: `关注 @${user.username}`,
        key: 'follow',
        icon: renderIcon(BodyOutline),
      });
    }
  }
  return options;
});

const emit = defineEmits<{
  (e: 'send-whisper', user: Item.UserInfo): void;
  (e: 'sync-follow-state', payload: { userId: number; isFollowing: boolean }): void;
}>();

const messageAvatar = computed(() => {
  if (props.message.type === 4 && props.message.sender_user_id === userInfo.value.id) {
    return props.message.receiver_user.avatar || DEFAULT_USER_AVATAR;
  }

  if (props.message.sender_user.id > 0) {
    return props.message.sender_user.avatar || DEFAULT_USER_AVATAR;
  }

  return DEFAULT_USER_AVATAR;
});

const onHandleFollowAction = (message: Item.MessageProps) => {
  const user =
    message.type === 4 && message.sender_user_id === userInfo.value.id
      ? message.receiver_user
      : message.sender_user;
  UserAction.followAction(dialog, user.id, user.username, user.is_following)
    .then((_action) => {
      user.is_following = _action;
      emit('sync-follow-state', {
        userId: user.id,
        isFollowing: _action,
      });
    })
    .catch(() => {});
};

const handleAction = (item: 'whisper' | 'follow' | 'unfollow') => {
  switch (item) {
    case 'whisper': {
      const message = props.message;
      if (message.type !== 99) {
        const user =
          message.type === 4 && message.sender_user_id === userInfo.value.id
            ? message.receiver_user
            : message.sender_user;
        emit('send-whisper', user);
      }
      break;
    }
    case 'follow':
    case 'unfollow':
      onHandleFollowAction(props.message);
      break;
    default:
      break;
  }
};

const isNotWhisperSender = computed(() => {
  return (
    props.message.type !== 4 ||
    props.message.sender_user_id !== userInfo.value.id
  );
});

const isWhisperReceiver = computed(() => {
  return (
    props.message.type === 4 &&
    props.message.receiver_user_id === userInfo.value.id
  );
});

const isWhisperSender = computed(() => {
  return (
    props.message.type === 4 &&
    props.message.sender_user_id === userInfo.value.id
  );
});

const viewDetail = (message: Item.MessageProps) => {
  handleReadMessage(message);
  if (message.type === 1 || message.type === 2 || message.type === 3) {
    if (message.post && message.post.id > 0) {
      router.push(buildPostRoute(message.post_id, currentSpaceSlug.value));
    } else {
      window.$message.error('该动态已被删除');
    }
  }
};

const agreeAddFriend = (message: Item.MessageProps) => {
  handleReadMessage(message);
  Api.v1.friend.post.add({
    user_id: message.sender_user_id,
  })
    .then((res) => {
      message.reply_id = 2;
      window.$message.success('已同意添加好友');
    })
    .catch(() => {
      window.$message.error('同意好友申请失败');
    });
};

const rejectAddFriend = (message: Item.MessageProps) => {
  handleReadMessage(message);
  Api.v1.friend.post.reject({
    user_id: message.sender_user_id,
  })
    .then((res) => {
      message.reply_id = 3;
      window.$message.success('已拒绝添加好友');
    })
    .catch(() => {
      window.$message.error('拒绝好友申请失败');
    });
};

const handleReadMessage = (message: Item.MessageProps) => {
  if (props.message.receiver_user_id !== userInfo.value.id) {
    return;
  }
  if (message.is_read === 0) {
    markLegacyMessageRead(message.id)
      .then((_res) => {
        message.is_read = 1;
      })
      .catch(() => {});
  }
};
</script>

<style lang="less" scoped>
.message-item {
  --message-item-bg: var(--surface-base);
  --message-item-unread-bg: var(--surface-unread);
  --message-item-brief-bg: color-mix(in srgb, var(--surface-subtle) 78%, transparent);
  --message-item-brief-unread-bg: color-mix(in srgb, var(--accent-soft) 92%, transparent);
  padding: 16px;
  background-color: var(--message-item-bg);

  &.unread {
    background: var(--message-item-unread-bg);
  }

  .sender-wrap {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;

    .top-tag {
      transform: none;
    }

    .username {
      opacity: 0.75;
      font-size: 14px;
    }
  }

  .timestamp {
    opacity: 0.75;
    font-size: 12px;
    display: flex;
    align-items: center;

    .timestamp-txt {
      margin-left: 6px;
    }
  }

  .brief-wrap {
    margin-top: 10px;
    background-color: var(--message-item-brief-bg);
    border: 1px solid color-mix(in srgb, var(--panel-border) 88%, transparent);
    color: var(--editor-text-main);

    :deep(.n-alert-body) {
      color: inherit;
    }

    .brief-content {
      display: flex;
      width: 100%;
      flex-wrap: wrap;
      gap: 8px;
    }

    .whisper-content-wrap {
      display: flex;
      width: 100%;
      flex-wrap: wrap;
    }

    .requesting-friend-wrap {
      display: flex;
      width: 100%;
      flex-wrap: wrap;
      gap: 8px;
    }
  }

  .view-link {
    display: flex;
    align-items: center;
  }

  .status-info {
    margin-left: 8px;
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .whisper-bubble {
    width: min(100%, 540px);
    display: grid;
    gap: 6px;
    padding: 12px 14px;
    border-radius: 18px;
    border: 1px solid color-mix(in srgb, var(--panel-border) 82%, transparent);
    background: color-mix(in srgb, var(--surface-subtle) 82%, transparent);

    p {
      margin: 0;
      white-space: pre-wrap;
      word-break: break-word;
      line-height: 1.7;
    }
  }

  .whisper-bubble-label {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.02em;
    opacity: 0.62;
  }
}

.message-item.whisper {
  .brief-wrap {
    border: 0;
    background: transparent;
    padding: 0;

    :deep(.n-alert-body) {
      padding: 0;
    }
  }
}

.message-item.whisper.inbound {
  .whisper-content-wrap {
    justify-content: flex-start;
  }

  .whisper-bubble {
    background:
      linear-gradient(135deg, color-mix(in srgb, var(--accent-soft) 72%, transparent), transparent 72%),
      color-mix(in srgb, var(--surface-subtle) 86%, transparent);
  }
}

.message-item.whisper.outbound {
  .whisper-content-wrap {
    justify-content: flex-end;
  }

  .whisper-bubble {
    background:
      linear-gradient(135deg, color-mix(in srgb, var(--accent-soft) 86%, transparent), transparent 82%),
      color-mix(in srgb, var(--panel-bg) 88%, transparent);
  }
}

.message-item.unread .brief-wrap {
  background-color: var(--message-item-brief-unread-bg);
}

@media (max-width: 768px) {
  .message-item {
    padding: 14px 12px;
  }

  .message-item .timestamp {
    gap: 4px;
  }
}
</style>
