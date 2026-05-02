<template>
    <div>
        <main-nav title="话题" />

        <n-list class="main-content-wrap tags-wrap" bordered>
            <n-tabs type="line" animated @update:value="changeTab">
                <n-tab-pane name="hot" tab="热门" />
                <n-tab-pane name="new" tab="最新" />
                <n-tab-pane name="follow" tab="关注" v-if="userLogined" />
                <n-tab-pane name="pin" tab="钉住" v-if="userLogined" />
                <template v-if="userLogined" #suffix>
                    <n-tag v-model:checked="tagsChecked" checkable>
                        {{tagsEditText}}
                    </n-tag>
                </template>
            </n-tabs>
            <n-spin :show="loading">
                <n-space>
                    <tag-item
                        v-for="tag in tags"
                        :tag="tag"
                        :showAction="userLogined && tagsChecked"
                        :checkFollowing="inFollowTab"
                        :checkPin="inPinTab"
                    >
                    </tag-item>
                </n-space>
                <div class="empty-wrap" v-if="tags.length === 0">
                    <n-empty size="large" description="暂无数据" />
                </div>
            </n-spin>
        </n-list>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { useRoute } from 'vue-router';
import { getTags } from '@/api/post';
import { useStoreMain } from '@/store/main';
import { useStoreUser } from '@/store/user';
import { useStoreProfile } from '@/store/profile';
import { storeToRefs } from 'pinia';
import { resolveSpaceSlug } from '@/utils/spaces';

const route = useRoute();
const storeMain = useStoreMain();
const storeUser = useStoreUser();
const storeProfile = useStoreProfile();
const { userLogined } = storeToRefs(storeUser);
const { currentSpaceSlug, spaces } = storeToRefs(storeProfile);

const tags = ref<Item.TagProps[]>([]);
const tagType = ref<'hot' | 'new' | 'follow' | 'pin'>('hot');
const loading = ref(false);
const tagsChecked = ref(false);
const inFollowTab = ref(false);
const inPinTab = ref(false);

const syncSpaceFromRoute = () => {
  const routeSpace = typeof route.query.space === 'string' ? route.query.space : '';
  currentSpaceSlug.value = resolveSpaceSlug(
    routeSpace || currentSpaceSlug.value,
    storeProfile.profile.defaultSpaceSlug,
  );
};

watch(tagsChecked, () => {
  if (!tagsChecked.value) {
    window.$message.success('保存成功');
    storeMain.doRefreshTopicFollow();
  }
});
const tagsEditText = computed({
  get: () => {
    let text = '编辑';
    if (tagsChecked.value) {
      text = '保存';
    }
    return text;
  },
  set: (newVal) => {
    // do nothing
  },
});
const loadTags = () => {
  loading.value = true;
  getTags({
    type: tagType.value,
    num: 50,
    space_slug: currentSpaceSlug.value,
  })
    .then((res) => {
      tags.value = res.topics;
      loading.value = false;
    })
    .catch(() => {
      tags.value = [];
      loading.value = false;
    });
};
const changeTab = (tab: 'hot' | 'new' | 'follow' | 'pin') => {
  tagType.value = tab;
  inFollowTab.value = tab === 'follow';
  inPinTab.value = tab === 'pin';
  loadTags();
};
onMounted(() => {
  syncSpaceFromRoute();
  loadTags();
});

watch(
  () => route.query.space,
  () => {
    syncSpaceFromRoute();
  },
);

watch(currentSpaceSlug, () => {
  loadTags();
});
</script>

<style lang="less" scoped>
.tags-wrap {
    --topic-surface-bg: transparent;
    padding: 20px;
    background-color: var(--topic-surface-bg);
}

.empty-wrap {
    background-color: var(--topic-surface-bg);
}
:global(.dark) .tags-wrap,
:global(.dark) .empty-wrap {
    --topic-surface-bg: rgba(16, 16, 20, 0.75);
}
</style>
