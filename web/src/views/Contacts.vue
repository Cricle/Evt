<template>
    <div>
        <main-nav title="好友" />

        <n-list class="main-content-wrap" bordered>
            <n-list-item class="friend-search-panel">
                <div class="friend-search-head">
                    <div class="friend-search-copy">
                        <h3>搜索用户并添加好友</h3>
                        <p>按用户名搜索，发送好友申请。</p>
                    </div>
                    <div class="friend-search-form">
                        <n-input
                            v-model:value="searchKeyword"
                            clearable
                            round
                            placeholder="输入用户名"
                            @keyup.enter.prevent="searchUsers"
                        />
                        <n-button
                            type="primary"
                            secondary
                            round
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
                        <n-list embedded :bordered="false">
                            <n-list-item
                                v-for="item in searchResults"
                                :key="item.user_id"
                                class="friend-search-item"
                            >
                                <div class="friend-search-user">
                                    <n-avatar round :size="44" :src="item.avatar" />
                                    <div class="friend-search-meta">
                                        <div class="friend-search-name">
                                            {{ item.nickname }}
                                        </div>
                                        <div class="friend-search-username">
                                            @{{ item.username }}
                                        </div>
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
                                        round
                                        size="small"
                                        @click="openAddFriend(item)"
                                    >
                                        添加好友
                                    </n-button>
                                </div>
                            </n-list-item>
                        </n-list>
                    </div>
                </div>
            </n-list-item>

            <div v-if="loading && list.length === 0" class="skeleton-wrap">
                <post-skeleton :num="pageSize" />
            </div>
            <div v-else>
                <div class="empty-wrap" v-if="list.length === 0">
                    <n-empty size="large" description="暂无数据" />
                </div>

                <n-list-item class="list-item" v-for="contact in list" :key="contact.user_id">
                     <user-card type="contact" :contact="contact" @send-whisper="onSendWhisper" />
                </n-list-item>
            </div>
            <!-- 私信组件 -->
            <whisper :show="showWhisper" :user="whisperReceiver" @success="whisperSuccess" />
            <whisper-add-friend
                :show="showAddFriendWhisper"
                :user="selectedFriendCandidate"
                @success="handleAddFriendSuccess"
            />
        </n-list>

        <infinite-load-more
            :total-page="totalPage"
            :no-more="noMore"
            complete-text="没有更多好友了"
            @load-more="nextPage"
        />
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useRoute } from 'vue-router';
import { Api } from '@/utils/request';
import { usePagination } from '@/composables/usePagination';
import InfiniteLoadMore from '@/components/infinite-load-more.vue';
import UserCard from '@/components/user-card.vue';
import WhisperAddFriend from '@/components/whisper-add-friend.vue';
import { DEFAULT_USER_AVATAR } from '@/utils/defaults';
import { useStoreUser } from '@/store/user';
import { storeToRefs } from 'pinia';

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
const contactUserIds = computed(() => new Set(list.value.map((item) => item.user_id)));

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
  Api.v1.suggest.users({ k: keyword })
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
  if (page.value < totalPage.value || totalPage.value == 0) {
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

const loadContacts = (scrollToBottom: boolean = false) => {
  if (list.value.length === 0) {
    loading.value = true;
  }
  Api.v1.user.get.contacts({
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
          setTimeout(() => {
            window.scrollTo(0, 99999);
          }, 50);
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
.friend-search-panel {
    display: block;
}

.friend-search-head {
    display: flex;
    flex-wrap: wrap;
    align-items: end;
    justify-content: space-between;
    gap: 16px;
}

.friend-search-copy {
    h3 {
        margin: 0;
        font-size: 18px;
    }

    p {
        margin: 6px 0 0;
        opacity: 0.7;
        font-size: 13px;
    }
}

.friend-search-form {
    display: flex;
    gap: 10px;
    width: min(100%, 420px);

    .n-input {
        flex: 1;
    }
}

.friend-search-results {
    margin-top: 16px;
    border-top: 1px solid var(--n-border-color);
    padding-top: 16px;
}

.friend-search-loading {
    display: flex;
    justify-content: center;
    padding: 12px 0;
}

.friend-search-list {
    :deep(.n-list) {
        background: transparent;
    }
}

.friend-search-item {
    padding-left: 0;
    padding-right: 0;
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
}

.friend-search-username {
    opacity: 0.65;
    font-size: 13px;
}

.friend-search-actions {
    display: flex;
    align-items: center;
    gap: 8px;
}

.main-content-wrap,
.empty-wrap,
.skeleton-wrap {
    --contacts-surface-bg: transparent;
    background-color: var(--contacts-surface-bg);
}

:global(.dark) .main-content-wrap,
:global(.dark) .empty-wrap,
:global(.dark) .skeleton-wrap {
    --contacts-surface-bg: rgba(16, 16, 20, 0.75);
}

@media (max-width: 768px) {
    .friend-search-form {
        width: 100%;
    }

    .friend-search-item {
        :deep(.n-list-item__main) {
            gap: 12px;
        }
    }
}
</style>
