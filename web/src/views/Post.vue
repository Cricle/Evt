<template>
    <div>
        <main-nav title="动态详情" :back="true" />

        <n-list class="main-content-wrap" bordered>
            <n-list-item>
                <n-spin :show="loading">
                    <div class="detail-wrap" v-if="post.id > 0">
                        <post-detail :post="post" @reload="reloadPost" @reaction-added="appendPostReaction" />
                    </div>
                    <div class="empty-wrap" v-else>
                        <n-empty size="large" description="暂无数据" />
                    </div>
                </n-spin>
            </n-list-item>
            <div v-if="post.id > 0" class="comment-entry-wrap">
                <div v-if="reactions.length > 0" class="reaction-overview">
                    <div class="reaction-title">这条动态的表情回应</div>
                    <post-reaction-bar
                        :reactions="reactions"
                        :count="post.upvote_count"
                        :max-visible="16"
                        :show-add-button="true"
                        @select="handlePostReaction"
                    />
                </div>
                <n-list-item>
                    <compose-comment :lock="post.is_lock" :post-id="post.id" @post-success="reloadComments" />
                </n-list-item>
            </div>
            <div class="comment-opts-wrap" v-if="post.id > 0">
                <n-tabs type="bar" justify-content="end" size="small" tab-style="margin-left: -24px;" animated @update:value="commentTab">
                    <template #prefix>
                        <span class="comment-title-item">评论</span>
                    </template>
                    <n-tab-pane name="default" tab="推荐" />
                    <n-tab-pane name="hots" tab="热门" />
                    <n-tab-pane name="newest" tab="最新" />
                </n-tabs>
            </div>

            <div v-if="post.id > 0">
                    <div v-if="commentLoading && comments.length === 0" class="skeleton-wrap">
                        <post-skeleton :num="5" />
                    </div>
                    <div v-else>
                    <div class="empty-wrap" v-if="comments.length === 0">
                        <n-empty size="large" description="暂无评论，快来抢沙发" />
                    </div>

                    <n-list-item v-for="comment in comments" :key="comment.id">
                        <comment-item :comment="comment" :postUserId="post.user_id" @reload="reloadComments" />
                    </n-list-item>
                </div>
            </div>
            <n-space v-if="comments.length >= pageSize" justify="center">
                <InfiniteLoading class="load-more" :slots="{complete: '没有更多数据了', error: '加载出错'}" @infinite="loadComments">
                    <template #spinner>
                        <span v-if="defaultCommentsSort && defaultNoMore" class="load-more-spinner" ><!-- 注意一定要保留这里 --></span>
                        <span v-if="!defaultCommentsSort && hotsNoMore" class="load-more-spinner" ><!-- 注意一定要保留这里 --></span>
                        <span v-if="!defaultCommentsSort && newestNoMore" class="load-more-spinner" ><!-- 注意一定要保留这里 --></span>
                        <span v-if="defaultCommentsSort && !defaultNoMore" class="load-more-spinner" >加载评论</span>
                        <span v-if="!defaultCommentsSort && !hotsNoMore" class="load-more-spinner" >加载评论</span>
                        <span v-if="!defaultCommentsSort && !newestNoMore" class="load-more-spinner" >加载评论</span>
                    </template>
                </InfiniteLoading>
            </n-space>
        </n-list>
    </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, computed } from 'vue';
import { useRoute } from 'vue-router';
import { getPost, getPostComments, togglePostReaction } from '@/api/post';
import InfiniteLoading from 'v3-infinite-loading';
import 'v3-infinite-loading/lib/style.css';
import { splitCommentReactions } from '@/utils/reactions';
import { applyCommentPageState, createCommentPageState } from '@/views/post-comment-state';
import { useStoreUser } from '@/store/user';
import { useStoreProfile } from '@/store/profile';
import { storeToRefs } from 'pinia';
import PostReactionBar from '@/components/post-reaction-bar.vue';
import { resolveSpaceSlug } from '@/utils/spaces';

const route = useRoute();
const storeUser = useStoreUser();
const storeProfile = useStoreProfile();
const { userInfo } = storeToRefs(storeUser);
const { currentSpaceSlug } = storeToRefs(storeProfile);
const post = ref<Item.PostProps>({} as Item.PostProps);
const loading = ref(false);
const commentLoading = ref(false);
const comments = ref<Item.CommentProps[]>([]);
const postId = computed(() => +(route.query.id as string));
const sortStrategy = ref<'default' | 'hots' | 'newest'>('default');
const defaultCommentsSort = ref<boolean>(true);
const pageSize = 20;
const reactions = ref<ReturnType<typeof splitCommentReactions>['reactions']>([]);

let stateHandler = {
  loading() {
    //nothing
  },
  loaded() {
    // nothing
  },
  complete() {
    // nothing
  },
  error() {
    // nothing
  },
};

let defaultCommentsState = createCommentPageState<Item.CommentProps>();
let hotsCommentsState = createCommentPageState<Item.CommentProps>();
let newestCommentsState = createCommentPageState<Item.CommentProps>();

const commentTab = (tab: 'default' | 'hots' | 'newest') => {
  sortStrategy.value = tab;
  defaultCommentsSort.value = tab === 'default';
  loadComments(stateHandler);
};

const syncSpaceFromRoute = () => {
  const routeSpace = typeof route.query.space === 'string' ? route.query.space : '';
  currentSpaceSlug.value = resolveSpaceSlug(
    routeSpace || currentSpaceSlug.value,
    storeProfile.profile.defaultSpaceSlug,
  );
};

const reloadPost = (post_id: number) => {
  getPost({
    id: post_id,
  })
    .then((res) => {
      post.value = res;
    })
    .catch((_err) => {});
};

const loadPost = () => {
  post.value = {
    id: 0,
  } as Item.PostProps;
  loading.value = true;
  getPost({
    id: postId.value,
  })
    .then((res) => {
      loading.value = false;
      post.value = res;
      reactions.value = res.reactions || [];

      // 加载评论
      loadComments(stateHandler);
    })
    .catch((err) => {
      loading.value = false;
    });
};

const defaultNoMore = ref<boolean>(false);
const defaultComments = ref<Item.CommentProps[]>([]);
const loadDefaultComments = ($state: any) => {
  if (defaultCommentsState.noMore) {
    $state?.complete?.();
    return;
  }
  const requestedPage = defaultCommentsState.page;
  getPostComments({
    id: post.value.id as number,
    style: 'default',
    page: requestedPage,
    page_size: pageSize,
  })
    .then((res) => {
      if ($state !== null) {
        stateHandler = $state;
      }
      defaultCommentsState = applyCommentPageState(defaultCommentsState, res.list, requestedPage, pageSize);
      defaultNoMore.value = defaultCommentsState.noMore;
      defaultComments.value = defaultCommentsState.items;
      const reactionView = splitCommentReactions(defaultComments.value);
      comments.value = reactionView.visibleComments;
      if (!post.value.reactions?.length) {
        reactions.value = reactionView.reactions;
      }
      $state?.loaded?.();
      commentLoading.value = false;
    })
    .catch((err) => {
      commentLoading.value = false;
      $state?.error?.();
    });
};

let hotsNoMore = ref<boolean>(false);
const hotsComments = ref<Item.CommentProps[]>([]);
const loadHotsComments = ($state: any) => {
  if (hotsCommentsState.noMore) {
    $state?.complete?.();
    return;
  }
  const requestedPage = hotsCommentsState.page;
  getPostComments({
    id: post.value.id as number,
    style: 'hots',
    page: requestedPage,
    page_size: pageSize,
  })
    .then((res) => {
      if ($state !== null) {
        stateHandler = $state;
      }
      hotsCommentsState = applyCommentPageState(hotsCommentsState, res.list, requestedPage, pageSize);
      hotsNoMore.value = hotsCommentsState.noMore;
      hotsComments.value = hotsCommentsState.items;
      const reactionView = splitCommentReactions(hotsComments.value);
      comments.value = reactionView.visibleComments;
      if (!post.value.reactions?.length) {
        reactions.value = reactionView.reactions;
      }
      $state?.loaded?.();
      commentLoading.value = false;
    })
    .catch((err) => {
      commentLoading.value = false;
      $state?.error?.();
    });
};

let newestNoMore = ref<boolean>(false);
const newestComments = ref<Item.CommentProps[]>([]);
const loadNewestComments = ($state: any) => {
  if (newestCommentsState.noMore) {
    $state?.complete?.();
    return;
  }
  const requestedPage = newestCommentsState.page;
  getPostComments({
    id: post.value.id as number,
    style: 'newest',
    page: requestedPage,
    page_size: pageSize,
  })
    .then((res) => {
      if ($state !== null) {
        stateHandler = $state;
      }
      newestCommentsState = applyCommentPageState(newestCommentsState, res.list, requestedPage, pageSize);
      newestNoMore.value = newestCommentsState.noMore;
      newestComments.value = newestCommentsState.items;
      const reactionView = splitCommentReactions(newestComments.value);
      comments.value = reactionView.visibleComments;
      if (!post.value.reactions?.length) {
        reactions.value = reactionView.reactions;
      }
      $state?.loaded?.();
      commentLoading.value = false;
    })
    .catch((err) => {
      commentLoading.value = false;
      $state?.error?.();
    });
};

const loadComments = ($state: any) => {
  if (postId.value < 1) {
    return;
  }
  if (comments.value.length === 0) {
    commentLoading.value = true;
  }
  if (sortStrategy.value === 'default') {
    comments.value = defaultComments.value;
    loadDefaultComments($state);
  } else if (sortStrategy.value === 'hots') {
    comments.value = hotsComments.value;
    loadHotsComments($state);
  } else {
    comments.value = newestComments.value;
    loadNewestComments($state);
  }
};

const reloadComments = () => {
  defaultCommentsState = createCommentPageState();
  hotsCommentsState = createCommentPageState();
  newestCommentsState = createCommentPageState();
  defaultComments.value = [];
  hotsComments.value = [];
  newestComments.value = [];
  comments.value = [];
  defaultNoMore.value = defaultCommentsState.noMore;
  hotsNoMore.value = hotsCommentsState.noMore;
  newestNoMore.value = newestCommentsState.noMore;
  loadComments(stateHandler);
};

const appendPostReaction = (payload: { reactions: Item.ReactionGroup[]; commentCount: number }) => {
  reactions.value = payload.reactions;
  post.value = {
    ...post.value,
    reactions: payload.reactions,
    upvote_count: payload.reactions.reduce((sum, item) => sum + item.count, 0),
    comment_count: payload.commentCount,
  };
};

const handlePostReaction = (emoji: string) => {
  if (userInfo.value.id < 1) {
    return;
  }
  togglePostReaction(post.value.id, emoji)
    .then((res) => {
      appendPostReaction({
        reactions: res.reactions || [],
        commentCount: res.comment_count,
      });
    })
    .catch(() => {
      window.$message.error('表情回复失败');
    });
};

onMounted(() => {
  syncSpaceFromRoute();
  loadPost();
});

watch(postId, () => {
  if (postId.value > 0 && route.name === 'post') {
    loadPost();
  }
});

watch(
  () => route.query.space,
  () => {
    syncSpaceFromRoute();
  },
);
</script>

<style lang="less" scoped>
.detail-wrap {
    min-height: 100px;
}

.comment-opts-wrap {
    padding-top: 2px;
    padding-left: 16px;
    padding-right: 16px;
    opacity: 0.75;

    .comment-title-item {
        padding-top: 4px;
        font-size: 16px;
        text-align: center;
    }
}

.comment-entry-wrap {
    padding: 10px 16px 0;
}

.reaction-overview {
    padding: 0 0 12px;
}

.reaction-title {
    margin-bottom: 8px;
    font-size: 12px;
    font-weight: 700;
    opacity: 0.68;
}

.main-content-wrap {
    --post-view-surface: transparent;
    background-color: var(--post-view-surface);

    .load-more {
        margin-bottom: 8px;
        .load-more-spinner {
            font-size: 14px;
            opacity: 0.65;
        }
    }
}

.skeleton-wrap {
    background-color: var(--post-view-surface);
}

:global(.dark) .main-content-wrap,
:global(.dark) .skeleton-wrap {
    --post-view-surface: rgba(16, 16, 20, 0.75);
}

@keyframes reaction-pop {
    from {
        opacity: 0;
        transform: scale(0.94);
    }
    to {
        opacity: 1;
        transform: scale(1);
    }
}
</style>
