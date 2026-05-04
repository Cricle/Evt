<template>
    <div>
        <main-nav title="好友" />

        <div class="main-content-wrap contacts-page">
            <section class="contacts-panel friend-search-panel">
                <div class="friend-search-head">
                    <div class="friend-search-copy">
                        <span class="friend-search-kicker">添加好友</span>
                        <h3>搜索用户并发送申请</h3>
                        <p>按用户名查找，发送一条简短问候即可。</p>
                    </div>
                    <div class="friend-search-form">
                        <n-input
                            v-model:value="searchKeyword"
                            clearable
                            placeholder="输入用户名"
                            @keyup.enter.prevent="searchUsers"
                        />
                        <n-button
                            type="primary"
                            :loading="searching"
                            :disabled="searchKeyword.trim().length === 0"
                            @click="searchUsers"
                        >
                            搜索
                        </n-button>
                    </div>
                </div>

                <div v-if="searchTouched" class="friend-search-results">
                    <div v-if="searching" class="friend-search-loading">
                        <n-spin size="small" />
                    </div>
                    <div v-else-if="searchResults.length === 0" class="friend-search-empty">
                        <n-empty size="small" description="没有找到相关用户" />
                    </div>
                    <div v-else class="friend-search-list">
                        <div
                            v-for="item in searchResults"
                            :key="item.user_id"
                            class="friend-search-item"
                        >
                            <div class="friend-search-user">
                                <n-avatar round :size="44" :src="item.avatar || DEFAULT_USER_AVATAR" />
                                <div class="friend-search-meta">
                                    <div class="friend-search-name">{{ item.nickname }}</div>
                                    <div class="friend-search-username">@{{ item.username }}</div>
                                </div>
                            </div>
                            <div class="friend-search-actions">
                                <n-tag v-if="item.user_id === currentUserId" size="small" round>
                                    你自己
                                </n-tag>
                                <n-tag
                                    v-else-if="item.is_friend || contactUserIds.has(item.user_id)"
                                    type="success"
                                    size="small"
                                    round
                                >
                                    已是好友
                                </n-tag>
                                <n-button
                                    v-else
                                    tertiary
                                    type="primary"
                                    size="small"
                                    @click="openAddFriend(item)"
                                >
                                    添加好友
                                </n-button>
                            </div>
                        </div>
                    </div>
                </div>
            </section>

            <section class="contacts-panel contacts-list-panel">
                <div class="contacts-panel-head">
                    <div>
                        <h3>我的好友</h3>
                        <p>当前共 {{ list.length }} 位联系人</p>
                    </div>
                </div>

                <div v-if="loading && list.length === 0" class="skeleton-wrap">
                    <post-skeleton :num="pageSize" />
                </div>
                <div v-else>
                    <div class="empty-wrap" v-if="list.length === 0">
                        <n-empty size="large" description="还没有好友，先去搜索看看" />
                    </div>

                    <div v-else class="contacts-list">
                        <div class="list-item" v-for="contact in list" :key="contact.user_id">
                            <user-card type="contact" :contact="contact" @send-whisper="onSendWhisper" />
                        </div>
                    </div>
                </div>
            </section>

            <whisper :show="showWhisper" :user="whisperReceiver" @success="whisperSuccess" />
            <whisper-add-friend
                :show="showAddFriendWhisper"
                :user="selectedFriendCandidate"
                @success="handleAddFriendSuccess"
            />
        </div>

        <infinite-load-more
            :total-page="totalPage"
            :no-more="noMore"
            complete-text="没有更多好友了"
            @load-more="nextPage"
        />
    </div>
</template>

<script setup lang="ts">
import InfiniteLoadMore from '@/components/infinite-load-more.vue';
import UserCard from '@/components/user-card.vue';
import WhisperAddFriend from '@/components/whisper-add-friend.vue';
import { usePagination } from '@/composables/usePagination';
import { useStoreUser } from '@/store/user';
import { DEFAULT_USER_AVATAR } from '@/utils/defaults';
import { Api } from '@/utils/request';
import { storeToRefs } from 'pinia';
import { computed, nextTick, onMounted, ref } from 'vue';
import { useRoute } from 'vue-router';

type SearchUserItem = Item.ContactItemProps & {
  is_friend: boolean;
};

const route = useRoute();
const storeUser = useStoreUser();
const { userInfo } = storeToRefs(storeUser);
const { loading, noMore, page, pageSize, totalPage } = usePagination(20);
const list = ref<Item.ContactItemProps[]>([]);
const showWhisper = ref(false);
const showAddFriendWhisper = ref(false);
const searchKeyword = ref('');
const searching = ref(false);
const searchTouched = ref(false);
const searchResults = ref<SearchUserItem[]>([]);
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
const selectedFriendCandidate = ref<Item.UserInfo>({
  id: 0,
  avatar: DEFAULT_USER_AVATAR,
  username: '',
  nickname: '',
  is_admin: false,
  is_friend: false,
  is_following: false,
  created_on: 0,
  follows: 0,
  followings: 0,
  status: 1,
});
const currentUserId = computed(() => userInfo.value.id || 0);
const contactUserIds = computed(
  () => new Set(list.value.map((item) => item.user_id)),
);

// 初始化页码
page.value = +(route.query.p as string) || 1;

const onSendWhisper = (user: Item.UserInfo) => {
  whisperReceiver.value = user;
  showWhisper.value = true;
};

const whisperSuccess = () => {
  showWhisper.value = false;
};

const openAddFriend = (contact: SearchUserItem) => {
  selectedFriendCandidate.value = {
    id: contact.user_id,
    avatar: contact.avatar || DEFAULT_USER_AVATAR,
    username: contact.username,
    nickname: contact.nickname,
    is_admin: false,
    is_friend: false,
    is_following: contact.is_following,
    created_on: contact.created_on,
    follows: 0,
    followings: 0,
    status: 1,
  };
  showAddFriendWhisper.value = true;
};

const handleAddFriendSuccess = () => {
  showAddFriendWhisper.value = false;
};

const searchUsers = () => {
  const keyword = searchKeyword.value.trim();
  searchTouched.value = true;
  if (!keyword) {
    searchResults.value = [];
    return;
  }

  searching.value = true;
  Api.v1.suggest
    .users({ k: keyword })
    .then(async (res) => {
      const usernames = Array.from(new Set(res.suggest || []));
      const profiles = await Promise.all(
        usernames.map((username) =>
          Api.v1.user.get
            .profile({ username })
            .then((profile) => ({
              user_id: profile.id,
              username: profile.username,
              nickname: profile.nickname || profile.username,
              avatar: profile.avatar || DEFAULT_USER_AVATAR,
              is_friend: profile.is_friend,
              is_following: profile.is_following,
              created_on: profile.created_on,
            }))
            .catch(() => null),
        ),
      );
      searchResults.value = profiles.filter(Boolean) as SearchUserItem[];
      searching.value = false;
    })
    .catch(() => {
      searching.value = false;
      searchResults.value = [];
    });
};

const nextPage = () => {
  if (page.value < totalPage.value || totalPage.value === 0) {
    noMore.value = false;
    page.value++;
    loadContacts();
  } else {
    noMore.value = true;
  }
};

onMounted(() => {
  loadContacts();
});

const loadContacts = (scrollToBottom = false) => {
  if (list.value.length === 0) {
    loading.value = true;
  }
  Api.v1.user.get
    .contacts({
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
        if (scrollToBottom) {
          void nextTick(() => {
            window.scrollTo(0, document.body.scrollHeight);
          });
        }
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
</script>

<style lang="less" scoped>
.contacts-page {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px 0 0;
}

.contacts-panel {
    border: 1px solid color-mix(in srgb, var(--panel-border) 94%, transparent);
    background: color-mix(in srgb, var(--panel-bg) 96%, transparent);
    box-shadow: var(--panel-shadow);
    padding: 18px 18px 12px;
}

.friend-search-panel {
    display: block;
}

.friend-search-head {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
    justify-content: space-between;
    gap: 14px;
}

.friend-search-copy {
    display: flex;
    flex-direction: column;
    gap: 4px;

    .friend-search-kicker {
        font-size: 12px;
        font-weight: 600;
        color: var(--accent-primary);
        letter-spacing: 0.04em;
    }

    h3 {
        margin: 0;
        font-size: 18px;
        line-height: 1.2;
    }

    p {
        margin: 0;
        opacity: 0.7;
        font-size: 13px;
    }
}

.friend-search-form {
    display: flex;
    align-items: center;
    gap: 10px;
    width: min(100%, 420px);

    .n-input {
        flex: 1;
    }

    :deep(.n-button) {
        min-width: 84px;
    }
}

.friend-search-results {
    margin-top: 14px;
    border-top: 1px solid color-mix(in srgb, var(--panel-border) 92%, transparent);
    padding-top: 14px;
}

.friend-search-loading {
    display: flex;
    justify-content: center;
    padding: 16px 0;
}

.friend-search-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
}

.friend-search-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px 0;
    border-bottom: 1px solid color-mix(in srgb, var(--panel-border) 84%, transparent);

    &:last-child {
        border-bottom: 0;
        padding-bottom: 0;
    }

    &:first-child {
        padding-top: 0;
    }
}

.friend-search-user {
    display: flex;
    align-items: center;
    gap: 12px;
}

.friend-search-meta {
    min-width: 0;
}

.friend-search-name {
    font-size: 15px;
    font-weight: 600;
    line-height: 1.2;
}

.friend-search-username {
    margin-top: 4px;
    opacity: 0.65;
    font-size: 13px;
}

.friend-search-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
}

.contacts-list-panel {
    padding-bottom: 6px;
}

.contacts-panel-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding-bottom: 10px;

    h3 {
        margin: 0;
        font-size: 18px;
        line-height: 1.2;
    }

    p {
        margin: 4px 0 0;
        font-size: 13px;
        opacity: 0.68;
    }
}

.contacts-list {
    display: flex;
    flex-direction: column;
}

.list-item {
    border-top: 1px solid color-mix(in srgb, var(--panel-border) 88%, transparent);

    &:first-child {
        border-top: 0;
    }
}

@media (max-width: 768px) {
    .contacts-page {
        gap: 10px;
        padding-top: 10px;
    }

    .contacts-panel {
        padding: 16px 14px 10px;
    }

    .friend-search-form {
        width: 100%;
    }

    .friend-search-form,
    .friend-search-actions {
        flex-wrap: wrap;
    }

    .friend-search-item {
        align-items: flex-start;
        flex-direction: column;
    }

    .contacts-panel-head {
        padding-bottom: 8px;
    }
}
</style>
