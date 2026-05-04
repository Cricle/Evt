<template>
    <div>
        <main-nav title="消息" />

        <n-list class="main-content-wrap messages-wrap" bordered>
            <!-- 私信组件 -->
            <whisper :show="showWhisper" :user="whisperReceiver" @success="whisperSuccess" />
            <div class="message-hero">
                <div class="message-hero-copy">
                    <strong>消息中心</strong>
                    <span>集中处理私信、系统提醒、好友申请和未读动态提醒。</span>
                </div>
                <div class="message-hero-stats">
                    <div class="message-stat">
                        <span>未读</span>
                        <strong>{{ unreadMsgCount }}</strong>
                    </div>
                    <div class="message-stat">
                        <span>当前筛选</span>
                        <strong>{{ messageStyle }}</strong>
                    </div>
                </div>
            </div>
            <n-space justify="space-between" align="center" class="message-toolbar">
                <div class="title title-action">
                    <n-button text size="small" :focusable="false" @click="handleUnreadMessage">
                        <template #icon>
                            <n-icon>
                                <UnreadIcon />
                            </n-icon>
                        </template>
                        {{ unreadMsgCount }} 条未读
                    </n-button>
                    <n-divider vertical />
                    <n-button text size="small" :focusable="false" @click="handleReadAll">全标已读</n-button>
                </div>
                <div class="title title-filter">
                    <n-dropdown
                        placement="bottom-end"
                        trigger="click"
                        size="small"
                        :options="options"
                        @select="handleAction"
                    >
                        <n-button text>
                            <template #icon>
                                <n-icon>
                                    <OptionsIcon />
                                </n-icon>
                            </template>
                            {{ messageStyle }}
                        </n-button>
                    </n-dropdown>
                </div>
            </n-space>
            <div v-if="loading && list.length === 0" class="skeleton-wrap">
                <message-skeleton :num="pageSize" />
            </div>
            <div v-else>
                <div class="empty-wrap" v-if="list.length === 0">
                    <n-empty size="large" description="暂无数据" />
                </div>
                <div v-else>
                    <n-list-item v-for="m in list" :key="m.id">
                        <message-item
                            :message="m"
                            @send-whisper="onSendWhisper"
                            @sync-follow-state="syncFollowState"
                        />
                     </n-list-item>
                </div>
            </div>
        </n-list>
        <infinite-load-more
            :total-page="totalPage"
            :no-more="noMore"
            complete-text="没有更多消息了"
            @load-more="nextPage"
        />
    </div>
</template>

<script setup lang="ts">
import InfiniteLoadMore from '@/components/infinite-load-more.vue';
import { usePagination } from '@/composables/usePagination';
import { useStoreMain } from '@/store/main';
import { useStoreUser } from '@/store/user';
import { listLegacyMessages, markAllLegacyMessagesRead } from '@/utils/messageTransport';
import {
  LayersOutline as AllIcon,
  OptionsOutline as OptionsIcon,
  PersonAddOutline as RequestingIcon,
  AtOutline as SystemIcon,
  ChatbubbleEllipsesOutline as UnreadIcon,
  PaperPlaneOutline as WhisperIcon,
} from '@vicons/ionicons5';
import { type DropdownOption, NIcon } from 'naive-ui';
import { storeToRefs } from 'pinia';
import { computed, h, onMounted, ref } from 'vue';
import type { Component } from 'vue';
import { useRoute } from 'vue-router';

const storeMain = useStoreMain();
const storeUser = useStoreUser();
const { unreadMsgCount } = storeToRefs(storeMain);

const route = useRoute();
const { loading, noMore, page, pageSize, totalPage, reset: resetPagination } = usePagination(20);
// 初始化页码
page.value = +(route.query.p as string) || 1;

const list = ref<Item.MessageProps[]>([]);
const messageStyle = ref<
  '所有消息' | '系统消息' | '我的私信' | '好友申请' | '未读消息'
>('所有消息');
const messageStyleVal = ref<
  'all' | 'system' | 'whisper' | 'requesting' | 'unread'
>('all');
const showWhisper = ref(false);
const whisperReceiver = ref<Item.UserInfo>({
  id: 0,
  avatar: '',
  username: '',
  nickname: '',
  is_admin: false,
  is_friend: true,
  is_following: false,
  created_on: 0,
  follows: 0,
  followings: 0,
  status: 1,
});

const reset = () => {
  resetPagination();
  list.value = [];
};

const renderIcon = (icon: Component) => {
  return () => {
    return h(NIcon, null, {
      default: () => h(icon),
    });
  };
};

const options = computed(() => {
  let opts: DropdownOption[];
  switch (messageStyle.value) {
    case '所有消息':
      opts = [
        {
          label: '系统消息',
          key: 'system',
          icon: renderIcon(SystemIcon),
        },
        {
          label: '我的私信',
          key: 'whisper',
          icon: renderIcon(WhisperIcon),
        },
        {
          label: '好友申请',
          key: 'requesting',
          icon: renderIcon(RequestingIcon),
        },
        {
          label: '未读消息',
          key: 'unread',
          icon: renderIcon(UnreadIcon),
        },
      ];
      break;
    case '系统消息':
      opts = [
        {
          label: '所有消息',
          key: 'all',
          icon: renderIcon(AllIcon),
        },
        {
          label: '我的私信',
          key: 'whisper',
          icon: renderIcon(WhisperIcon),
        },
        {
          label: '好友申请',
          key: 'requesting',
          icon: renderIcon(RequestingIcon),
        },
        {
          label: '未读消息',
          key: 'unread',
          icon: renderIcon(UnreadIcon),
        },
      ];
      break;
    case '我的私信':
      opts = [
        {
          label: '所有消息',
          key: 'all',
          icon: renderIcon(AllIcon),
        },
        {
          label: '系统消息',
          key: 'system',
          icon: renderIcon(SystemIcon),
        },
        {
          label: '好友申请',
          key: 'requesting',
          icon: renderIcon(RequestingIcon),
        },
        {
          label: '未读消息',
          key: 'unread',
          icon: renderIcon(UnreadIcon),
        },
      ];
      break;
    case '好友申请':
      opts = [
        {
          label: '所有消息',
          key: 'all',
          icon: renderIcon(AllIcon),
        },
        {
          label: '系统消息',
          key: 'system',
          icon: renderIcon(SystemIcon),
        },
        {
          label: '我的私信',
          key: 'whisper',
          icon: renderIcon(WhisperIcon),
        },
        {
          label: '未读消息',
          key: 'unread',
          icon: renderIcon(UnreadIcon),
        },
      ];
      break;
    case '未读消息':
      opts = [
        {
          label: '所有消息',
          key: 'all',
          icon: renderIcon(AllIcon),
        },
        {
          label: '系统消息',
          key: 'system',
          icon: renderIcon(SystemIcon),
        },
        {
          label: '我的私信',
          key: 'whisper',
          icon: renderIcon(WhisperIcon),
        },
        {
          label: '好友申请',
          key: 'requesting',
          icon: renderIcon(RequestingIcon),
        },
      ];
      break;
    default:
      opts = [];
      break;
  }
  return opts;
});

const handleAction = (
  item: 'all' | 'system' | 'whisper' | 'requesting' | 'unread',
) => {
  switch (item) {
    case 'all':
      messageStyle.value = '所有消息';
      break;
    case 'system':
      messageStyle.value = '系统消息';
      break;
    case 'whisper':
      messageStyle.value = '我的私信';
      break;
    case 'requesting':
      messageStyle.value = '好友申请';
      break;
    case 'unread':
      messageStyle.value = '未读消息';
      break;
  }
  messageStyleVal.value = item;
  reset();
  loadMessages();
};

const handleUnreadMessage = () => {
  handleAction('unread');
};

const handleReadAll = () => {
  if (unreadMsgCount.value > 0 && list.value.length > 0) {
    markAllLegacyMessagesRead()
      .then((_res) => {
        if (messageStyleVal.value !== 'unread') {
          for (const idx in list.value) {
            list.value[idx].is_read = 1;
          }
        } else {
          list.value = [];
        }
        storeMain.updateUnreadMsgCount(0);
      })
      .catch(() => {
        window.$message.error('全部标记已读失败');
      });
  }
};

const onSendWhisper = (user: Item.UserInfo) => {
  whisperReceiver.value = user;
  showWhisper.value = true;
};

const whisperSuccess = () => {
  showWhisper.value = false;
};

const syncFollowState = (payload: { userId: number; isFollowing: boolean }) => {
  for (const message of list.value) {
    if (message.sender_user?.id === payload.userId) {
      message.sender_user.is_following = payload.isFollowing;
    }
    if (message.receiver_user?.id === payload.userId) {
      message.receiver_user.is_following = payload.isFollowing;
    }
  }
};

const loadMessages = () => {
  loading.value = true;
  listLegacyMessages({
    style: messageStyleVal.value,
    page: page.value,
    page_size: pageSize.value,
  })
    .then((res) => {
      loading.value = false;
      if (res.list.length === 0) {
        noMore.value = true;
      }
      if (page.value > 1) {
        list.value = list.value.concat(res.list);
      } else {
        list.value = res.list;
        window.scrollTo(0, 0);
      }
      totalPage.value = Math.ceil(res.pager.total_rows / pageSize.value);
    })
    .catch((_err) => {
      loading.value = false;
      if (page.value > 1) {
        page.value--;
      }
    });
};
const nextPage = () => {
  if (page.value < totalPage.value || totalPage.value === 0) {
    noMore.value = false;
    page.value++;
    loadMessages();
  } else {
    noMore.value = true;
  }
};
onMounted(() => {
  loadMessages();
});
</script>

<style lang="less" scoped>
.message-hero {
    display: flex;
    align-items: stretch;
    justify-content: space-between;
    gap: 14px;
    padding: 14px 16px 10px;
    flex-wrap: wrap;
}

.message-hero-copy {
    display: grid;
    gap: 4px;

    strong {
        font-size: 18px;
        line-height: 1.35;
    }

    span {
        font-size: 13px;
        line-height: 1.7;
        opacity: 0.72;
    }
}

.message-hero-stats {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
}

.message-stat {
    min-width: 92px;
    padding: 10px 12px;
    border-radius: 16px;
    background: color-mix(in srgb, var(--panel-bg) 84%, transparent);
    border: 1px solid color-mix(in srgb, var(--panel-border) 78%, transparent);
    display: grid;
    gap: 2px;

    span {
        font-size: 12px;
        opacity: 0.64;
    }

    strong {
        font-size: 15px;
        line-height: 1.35;
    }
}

.message-toolbar {
    padding: 4px 16px 2px;
}

.title {
    opacity: 0.9;
}

.title-action {
    display: flex;
    align-items: center;
    gap: 0;
}

.title-filter {
    margin-left: auto;
}

@media (max-width: 768px) {
    .message-hero {
        padding: 12px 12px 8px;
    }

    .message-hero-stats,
    .message-stat {
        width: 100%;
    }

    .message-toolbar {
        padding-left: 12px;
        padding-right: 12px;
    }

    .title-action {
        flex-wrap: wrap;
    }

    .title-filter {
        margin-left: 0;
    }
}
</style>
