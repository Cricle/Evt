<template>
    <div class="user-card">
        <div class="user-card-main">
            <n-avatar class="user-card-avatar" :size="50" :src="contact.avatar || DEFAULT_USER_AVATAR" />
            <div class="user-card-body">
                <div class="user-card-head">
                    <div class="user-card-title">
                        <router-link
                            @click.stop
                            class="username-link user-card-name"
                            :to="{
                                name: 'user',
                                query: { s: contact.username },
                            }"
                        >
                            {{ contact.nickname }}
                        </router-link>
                        <span class="user-card-username">@{{ contact.username }}</span>
                        <n-tag
                            v-if="showFollowingTag && contact.is_following"
                            class="top-tag"
                            type="success"
                            size="small"
                            round
                        >
                            已关注
                        </n-tag>
                    </div>
                    <div class="item-header-extra">
                        <n-dropdown
                            placement="bottom-end"
                            trigger="click"
                            size="small"
                            :options="actionOpts"
                            @select="handleAction"
                        >
                            <n-button quaternary circle>
                                <template #icon>
                                    <n-icon>
                                        <more-horiz-filled />
                                    </n-icon>
                                </template>
                            </n-button>
                        </n-dropdown>
                    </div>
                </div>
                <div class="user-info">
                    <span class="info-item">UID {{ contact.user_id }}</span>
                    <span class="info-dot">·</span>
                    <span class="info-item">{{ formatDate(contact.created_on) }} 加入</span>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import UserAction from '@/composables/useUserAction';
import { DEFAULT_USER_AVATAR } from '@/utils/defaults';
import { formatDate } from '@/utils/formatTime';
import { BodyOutline, PaperPlaneOutline, WalkOutline } from '@vicons/ionicons5';
import { MoreHorizFilled } from '@vicons/material';
import { type DropdownOption, NIcon, useDialog } from 'naive-ui';
import { computed, h } from 'vue';
import type { Component } from 'vue';

const dialog = useDialog();

const props = withDefaults(
  defineProps<{
    contact: Item.ContactItemProps;
    type?: 'contact' | 'follow';
  }>(),
  {
    type: 'contact',
  },
);

const showFollowingTag = computed(() => props.type === 'follow');
const enableFollowAction = computed(() => props.type === 'follow');

const emit = defineEmits<{
  (e: 'send-whisper', user: Item.UserInfo): void;
  (e: 'unfollow-success'): void;
}>();

const renderIcon = (icon: Component) => {
  return () => {
    return h(NIcon, null, {
      default: () => h(icon),
    });
  };
};

const handleFollowUser = () => {
  const wasFollowing = props.contact.is_following;
  UserAction.followAction(
    dialog,
    props.contact.user_id,
    props.contact.username,
    props.contact.is_following,
  )
    .then((_action) => {
      props.contact.is_following = _action;
      if (wasFollowing && !_action) {
        emit('unfollow-success');
      }
    })
    .catch(() => {
      window.$message.error('关注状态更新失败');
    });
};

const actionOpts = computed(() => {
  const options: DropdownOption[] = [
    {
      label: `私信 @${props.contact.username}`,
      key: 'whisper',
      icon: renderIcon(PaperPlaneOutline),
    },
  ];

  if (enableFollowAction.value) {
    if (props.contact.is_following) {
      options.push({
        label: `取消关注 @${props.contact.username}`,
        key: 'unfollow',
        icon: renderIcon(WalkOutline),
      });
    } else {
      options.push({
        label: `关注 @${props.contact.username}`,
        key: 'follow',
        icon: renderIcon(BodyOutline),
      });
    }
  }

  return options;
});

const handleAction = (item: 'follow' | 'unfollow' | 'whisper') => {
  switch (item) {
    case 'follow':
    case 'unfollow':
      handleFollowUser();
      break;
    case 'whisper': {
      const user: Item.UserInfo = {
        id: props.contact.user_id,
        avatar: props.contact.avatar || DEFAULT_USER_AVATAR,
        username: props.contact.username,
        nickname: props.contact.nickname,
        is_admin: false,
        is_friend: true,
        is_following: false,
        created_on: 0,
        follows: 0,
        followings: 0,
        status: 1,
      };
      emit('send-whisper', user);
      break;
    }
    default:
      break;
  }
};
</script>

<style lang="less" scoped>
.user-card {
    width: 100%;
    box-sizing: border-box;
    padding: 14px 2px;
}

.user-card-main {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    min-width: 0;
}

.user-card-avatar {
    flex-shrink: 0;
}

.user-card-body {
    min-width: 0;
    flex: 1;
}

.user-card-head {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
}

.user-card-title {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px 8px;
    min-width: 0;
}

.user-card-name {
    font-size: 15px;
    font-weight: 600;
    line-height: 1.2;
}

.user-card-username {
    font-size: 13px;
    opacity: 0.65;
    min-width: 0;
    word-break: break-all;
}

.top-tag {
    transform: translateY(-1px) scale(0.92);
    transform-origin: left center;
}

.user-info {
    margin-top: 6px;
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 6px;
}

.info-item {
    font-size: 13px;
    line-height: 1.4;
    opacity: 0.72;
}

.info-dot {
    font-size: 12px;
    opacity: 0.42;
}

.item-header-extra {
    display: flex;
    align-items: center;
    opacity: 0.75;
    flex-shrink: 0;
}

@media (max-width: 768px) {
    .user-card {
        padding: 12px 0;
    }

    .user-card-head {
        gap: 8px;
    }
}
</style>
