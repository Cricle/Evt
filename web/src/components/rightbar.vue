<template>
    <div class="rightbar-wrap" v-if="!collapsedRight">
        <div class="search-wrap">
            <n-input
                round
                clearable
                placeholder="搜一搜..."
                v-model:value="keyword"
                @keyup.enter.prevent="handleSearch"
            >
                <template #prefix>
                    <n-icon :component="Search" />
                </template>
            </n-input>
        </div>
        <n-card v-if="showFollowTopics" class="hottopic-wrap" title="关注话题" embedded :bordered="false" size="small">
            <n-spin :show="loading">
                <div class="hot-tag-item" v-for="tag in followTags" :key="tag.id">
                    <router-link class="hash-link" :to="buildTagSearchRoute(tag.tag, currentSpaceSlug)">
                        #{{ tag.tag }}
                    </router-link>

                    <div class="post-num">
                        {{ formatQuoteNum(tag.quote_num) }}
                    </div>
                </div>
            </n-spin>
        </n-card>
        <n-card class="hottopic-wrap" title="热门话题" embedded :bordered="false" size="small">
            <n-spin :show="loading">
                <div class="hot-tag-item" v-for="tag in hotTags" :key="tag.id">
                    <router-link class="hash-link" :to="buildTagSearchRoute(tag.tag, currentSpaceSlug)">
                        #{{ tag.tag }}
                    </router-link>

                    <div class="post-num">
                        {{ formatQuoteNum(tag.quote_num) }}
                    </div>
                </div>
            </n-spin>
        </n-card>
        <n-card class="copyright-wrap" embedded :bordered="false" size="small">
            <div class="copyright">&copy; {{ profile.copyrightTop }}</div>
            <div>
                <n-space>
                    <a
                        :href="profile.copyrightLeftLink"
                        target="_blank"
                        class="hash-link"
                    >
                        {{ profile.copyrightLeft }}
                    </a>
                    <a
                        :href="profile.copyrightRightLink"
                        target="_blank"
                        class="hash-link"
                    >
                        {{ profile.copyrightRight }}
                    </a>
                </n-space>
            </div>
        </n-card>
        <div class="site-info" v-if="userInfo.is_admin" ref="userInfoElement">
            <span class="site-info-item">{{ registerUserCount }} 注册用户，{{ onlineUserCount }} 人在线，最高在线 {{ historyMaxOnline }} 人，站点上线于 {{ formatRelativeTime(serverUpTime) }}</span>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { useStoreMain } from '@/store/main';
import { useRouter } from 'vue-router';
import { getTags } from '@/api/post';
import { Search } from '@vicons/ionicons5';
import { formatRelativeTime } from '@/utils/formatTime';
import { useStoreProfile } from '@/store/profile';
import { storeToRefs } from 'pinia';
import { Api } from '@/utils/request';
import { useStoreUser } from '@/store/user';
import { buildHomeRouteWithSpace, buildTagSearchRoute } from '@/utils/tagRoute';

const hotTags = ref<Item.TagProps[]>([]);
const followTags = ref<Item.TagProps[]>([]);
const loading = ref(false);
const keyword = ref('');

const storeMain = useStoreMain();
const storeUser = useStoreUser();
const storeProfile = useStoreProfile();
const { collapsedRight, refreshTopicFollow } = storeToRefs(storeMain);
const { userInfo, userLogined } = storeToRefs(storeUser);
const { profile, currentSpaceSlug } = storeToRefs(storeProfile);

const router = useRouter();
const registerUserCount = ref(0);
const onlineUserCount = ref(0);
const historyMaxOnline = ref(0);
const serverUpTime = ref(0);
const userInfoElement = ref<HTMLElement | null>(null);
const rightFollowTopicMaxSize = Number(
  import.meta.env.VITE_RIGHT_FOLLOW_TOPIC_MAX_SIZE,
);
const rightHotTopicMaxSize = Number(
  import.meta.env.VITE_RIGHT_HOT_TOPIC_MAX_SIZE,
);

const loadSiteInfo = () => {
  Api.v1.admin.get.site.status()
    .then((res) => {
      registerUserCount.value = res.register_user_count;
      onlineUserCount.value = res.online_user_count;
      historyMaxOnline.value = res.history_max_online;
      serverUpTime.value = res.server_up_time;
    })
    .catch((_err) => {
      // do nothing
    });
  observer.disconnect();
};
const loadHotTags = () => {
  loading.value = true;
  getTags({
    type: 'hot_extral',
    num: rightHotTopicMaxSize,
    extral_num: rightFollowTopicMaxSize,
    space_slug: currentSpaceSlug.value,
  })
    .then((res) => {
      hotTags.value = res.topics;
      followTags.value = res.extral_topics ?? [];
      showFollowTopics.value = true;
      loading.value = false;
    })
    .catch((_err) => {
      loading.value = false;
    });
};
const formatQuoteNum = (num: number) => {
  if (num >= 1000) {
    return (num / 1000).toFixed(1) + 'k';
  }
  return num;
};
const handleSearch = () => {
  router.push(
    buildHomeRouteWithSpace(
      {
        q: keyword.value,
      },
      currentSpaceSlug.value,
    ),
  );
};
const showFollowTopics = computed({
  get: () => {
    return userLogined.value && followTags.value.length !== 0;
  },
  set: (newVal) => {
    // do nothing
  },
});
watch(
  () => ({
    refreshTopicFollow: refreshTopicFollow.value,
    userLogined: userLogined.value,
  }),
  (to, from) => {
    if (to.refreshTopicFollow !== from.refreshTopicFollow || to.userLogined) {
      loadHotTags();
    }
    if (userInfo.value.is_admin) {
      loadSiteInfo();
    }
  },
);
const observer = new IntersectionObserver(
  (entries: IntersectionObserverEntry[]) => {
    entries.forEach((entry) => {
      if (entry.isIntersecting) {
        loadSiteInfo();
      }
    });
  },
  {
    root: null,
    rootMargin: '0px',
    threshold: 1,
  },
);
onMounted(() => {
  // 不知道为什么 store.userInfo.is_admin 在这里就是不起作用f*k，所以才用这么一种蹩脚的法子来凑合
  if (userInfoElement.value) {
    observer.observe(userInfoElement.value);
  }
  loadHotTags();
});

watch(currentSpaceSlug, () => {
  loadHotTags();
});
</script>

<style lang="less" scoped>
.rightbar-wrap::-webkit-scrollbar {
  width: 0; /* 隐藏滚动条的宽度 */
  height: 0; /* 隐藏滚动条的高度 */
}
.rightbar-wrap {
    width: var(--layout-rightbar-width);
    position: sticky;
    top: var(--layout-edge-offset);
    left: auto;
    max-height: calc(100vh - var(--layout-edge-offset) * 2);
    overflow: auto;

    .search-wrap {
        margin: 0 0 12px;
    }

    .hot-tag-item {
        line-height: 2;
        position: relative;

        .hash-link {
            width: calc(100% - 60px);
            text-overflow: ellipsis;
            white-space: nowrap;
            overflow: hidden;
            display: block;
        }

        .post-num {
            position: absolute;
            right: 0;
            top: 0;
            width: 60px;
            text-align: right;
            line-height: 2;
            opacity: 0.5;
        }
    }

    .hottopic-wrap {
        margin-bottom: 12px;
        background: var(--glass-panel-bg);
        border: var(--glass-panel-border);
        border-radius: var(--glass-panel-radius);
        box-shadow: none;
        backdrop-filter: var(--glass-panel-blur);

        :deep(.n-card-header) {
            padding-bottom: 6px;
        }

        :deep(.n-card__content) {
            padding-top: 0;
        }
    }

    .site-info {
        margin-top: 8px;
        padding-left: 6px;
        padding-right: 6px;

        .site-info-item {
            font-size: 10px;
            opacity: 0.75;
        }
    }

    .copyright-wrap {
        background: var(--glass-panel-bg);
        border: var(--glass-panel-border);
        border-radius: var(--glass-panel-radius);
        box-shadow: none;
        backdrop-filter: var(--glass-panel-blur);

        .copyright {
            font-size: 12px;
            opacity: 0.75;
        }

        .hash-link {
            font-size: 12px;
        }
    }
}
</style>
