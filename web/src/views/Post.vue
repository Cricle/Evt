<template>
    <div>
        <main-nav :title="pageTitle" :back="true" />

        <n-list :class="['main-content-wrap', { 'event-main-content-wrap': isEventMode }]" :bordered="!isEventMode">
            <template v-if="!isEventMode">
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
            </template>
            <template v-else>
                <n-spin :show="loading">
                    <div v-if="post.id > 0" class="event-page">
                        <section class="event-shell event-overview">
                            <div class="event-overview-kicker">事件时间轴</div>
                            <div class="event-overview-head">
                                <div class="event-overview-copy">
                                    <h1>{{ eventTitle }}</h1>
                                    <p>{{ eventSummary }}</p>
                                </div>
                                <div class="event-overview-stats">
                                    <div class="event-stat-card">
                                        <span>创建者</span>
                                        <strong>{{ post.user?.nickname || post.user?.username }}</strong>
                                    </div>
                                    <div class="event-stat-card">
                                        <span>时间节点</span>
                                        <strong>{{ timelineComments.length }}</strong>
                                    </div>
                                    <div class="event-stat-card">
                                        <span>表情回应</span>
                                        <strong>{{ post.upvote_count }}</strong>
                                    </div>
                                </div>
                            </div>
                            <div class="event-overview-meta">
                                <span>创建于 {{ formatTimelineTime(post.created_on) }}</span>
                                <span v-if="post.ip_loc">地点 {{ post.ip_loc }}</span>
                                <span v-if="post.latest_replied_on && post.latest_replied_on !== post.created_on">
                                    最新更新 {{ formatTimelineTime(post.latest_replied_on) }}
                                </span>
                            </div>
                            <div class="event-overview-body" v-if="eventExtraDescriptionHtml" v-html="eventExtraDescriptionHtml"></div>
                            <div class="event-overview-media">
                                <post-attachment :attachments="eventPost.attachments" />
                                <post-attachment
                                    :attachments="eventPost.charge_attachments"
                                    :price="eventPost.attachment_price"
                                />
                                <post-image :imgs="eventPost.imgs" />
                                <post-video :videos="eventPost.videos" :full="true" />
                                <post-link :links="eventPost.links" />
                            </div>
                        </section>
                    </div>
                    <div class="empty-wrap" v-else>
                        <n-empty size="large" description="暂无数据" />
                    </div>
                </n-spin>
            </template>
            <div v-if="post.id > 0 && !isEventMode" class="comment-entry-wrap">
                <n-list-item>
                    <compose-comment
                      :lock="post.is_lock"
                      :post-id="post.id"
                      :mode="isEventMode ? 'event-node' : 'comment'"
                      @post-success="reloadComments"
                    />
                </n-list-item>
            </div>
            <div v-if="post.id > 0 && isEventMode" class="event-composer-wrap">
                <section class="event-shell event-composer-shell">
                    <div class="event-composer-head">
                        <strong>追加时间节点</strong>
                        <span>继续补充进展、里程碑、图片记录或上下文说明。</span>
                    </div>
                    <compose-comment
                      :lock="post.is_lock"
                      :post-id="post.id"
                      mode="event-node"
                      @post-success="reloadComments"
                    />
                </section>
            </div>
            <div class="comment-opts-wrap" v-if="post.id > 0 && !isEventMode">
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
                    <div v-else-if="isEventMode" class="event-wrap">
                      <section class="event-shell event-timeline-shell">
                      <div class="comment-opts-wrap event-heading">
                        <span class="comment-title-item">事件时间轴</span>
                        <span class="event-heading-subtitle">按最早到最新查看全部节点</span>
                      </div>
                      <div class="empty-wrap" v-if="comments.length === 0">
                        <n-empty size="large" description="暂无时间节点，开始记录吧" />
                      </div>
                      <n-timeline v-else size="small" class="event-timeline">
                        <n-timeline-item
                          v-for="(comment, index) in timelineComments"
                          :key="comment.id"
                          :type="resolveTimelineNodeType(index, comment)"
                        >
                          <event-timeline-item
                            :comment="comment"
                            :postUserId="post.user_id"
                            :index="index"
                            :total="timelineComments.length"
                            :is-first="index === 0"
                            :is-latest="index === timelineComments.length - 1"
                            :is-milestone="comment.is_essence === YesNoEnum.YES"
                            @reload="reloadComments"
                          />
                        </n-timeline-item>
                      </n-timeline>
                      </section>
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
            <n-space v-if="comments.length >= pageSize && !isEventMode" justify="center">
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
import { getPost, getPostComments } from '@/api/post';
import InfiniteLoading from 'v3-infinite-loading';
import 'v3-infinite-loading/lib/style.css';
import { splitCommentReactions } from '@/utils/reactions';
import { applyCommentPageState, createCommentPageState } from '@/views/post-comment-state';
import { useStoreProfile } from '@/store/profile';
import { storeToRefs } from 'pinia';
import { resolveSpaceSlug } from '@/utils/spaces';
import { isEventPost } from '@/utils/postKind';
import { formatPrettyTime } from '@/utils/formatTime';
import EventTimelineItem from '@/components/event-timeline-item.vue';
import { YesNoEnum } from '@/utils/IEnum';
import {
  resolveEventExtraDescriptionHtml,
  resolveEventNarrative,
  resolveEventTitle,
  resolveTimelineNodeType as resolveEventTimelineNodeType,
  sortTimelineComments,
} from '@/utils/eventTimeline';

const route = useRoute();
const storeProfile = useStoreProfile();
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
const isEventMode = computed(() => isEventPost(post.value));
const pageTitle = computed(() => (isEventMode.value ? '事件详情' : '动态详情'));
const eventPost = computed(() => {
  const source = post.value as Item.PostProps;
  const model: Item.PostComponentProps = Object.assign(
    {
      texts: [],
      imgs: [],
      videos: [],
      links: [],
      attachments: [],
      charge_attachments: [],
    },
    source,
  );
  (source.contents || []).forEach((content) => {
    if (+content.type === 1 || +content.type === 2) {
      model.texts.push(content);
    }
    if (+content.type === 3) {
      model.imgs.push(content);
    }
    if (+content.type === 4) {
      model.videos.push(content);
    }
    if (+content.type === 6) {
      model.links.push(content);
    }
    if (+content.type === 7) {
      model.attachments.push(content);
    }
    if (+content.type === 8) {
      model.charge_attachments.push(content);
    }
  });
  return model;
});
const eventTitle = computed(() => resolveEventTitle(eventPost.value.texts || []));
const eventSummary = computed(() =>
  resolveEventNarrative(post.value.user?.nickname || post.value.user?.username || '未知用户', post.value.ip_loc, '发起'),
);
const eventExtraDescriptionHtml = computed(() => resolveEventExtraDescriptionHtml(eventPost.value.texts || []));
const timelineComments = computed(() => sortTimelineComments(comments.value));

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

const formatTimelineTime = (value: number) => formatPrettyTime(value);
const resolveTimelineNodeType = (index: number, comment: Item.CommentProps) =>
  resolveEventTimelineNodeType(index, timelineComments.value.length, comment);

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
      reloadComments();
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
  sortStrategy.value = isEventMode.value ? 'newest' : sortStrategy.value;
  defaultCommentsSort.value = sortStrategy.value === 'default';
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
    padding: 4px 16px 0;
    opacity: 0.75;

    .comment-title-item {
        padding-top: 2px;
        font-size: 15px;
        font-weight: 600;
    }
}

.comment-entry-wrap {
    padding: 6px 18px 0;
}

.event-wrap {
    padding: 0 18px 18px;
}

.event-composer-wrap {
    padding: 14px 18px 0;
}

.event-page {
    padding: 18px 18px 0;
}

.event-shell {
    border: 1px solid color-mix(in srgb, var(--panel-border) 82%, transparent);
    background:
      radial-gradient(circle at top right, color-mix(in srgb, var(--accent-soft) 70%, transparent), transparent 38%),
      color-mix(in srgb, var(--panel-bg) 88%, transparent);
    border-radius: 24px;
    overflow: hidden;
}

.event-overview {
    padding: 22px 22px 18px;
    display: grid;
    gap: 16px;
}

.event-overview-kicker {
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.06em;
    color: var(--accent-primary);
}

.event-overview-head {
    display: flex;
    justify-content: space-between;
    gap: 18px;
    align-items: flex-start;
    flex-wrap: wrap;
}

.event-overview-copy {
    display: grid;
    gap: 8px;
    min-width: min(100%, 320px);

    h1 {
        margin: 0;
        font-size: 28px;
        line-height: 1.2;
        font-weight: 800;
    }

    p {
        margin: 0;
        font-size: 14px;
        line-height: 1.8;
        color: var(--editor-text-subtle);
    }
}

.event-overview-stats {
    display: grid;
    grid-template-columns: repeat(3, minmax(110px, 1fr));
    gap: 10px;
    min-width: min(100%, 360px);
}

.event-stat-card {
    padding: 12px 14px;
    border-radius: 18px;
    background: color-mix(in srgb, var(--panel-bg) 92%, transparent);
    border: 1px solid color-mix(in srgb, var(--panel-border) 84%, transparent);
    display: grid;
    gap: 4px;

    span {
        font-size: 12px;
        opacity: 0.66;
    }

    strong {
        font-size: 16px;
        line-height: 1.35;
    }
}

.event-overview-meta {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;

    span {
        padding: 7px 11px;
        border-radius: 999px;
        background: color-mix(in srgb, var(--accent-soft-muted) 86%, transparent);
        font-size: 12px;
        line-height: 1.4;
    }
}

.event-overview-body {
    font-size: 15px;
    line-height: 1.82;
    color: var(--editor-text-main);
    padding-top: 2px;
    border-top: 1px solid color-mix(in srgb, var(--panel-border) 72%, transparent);

    :deep(p) {
        margin: 14px 0 0;
    }
}

.event-overview-media {
    display: grid;
    gap: 12px;
}

.event-composer-shell {
    padding: 10px 0 0;
}

.event-composer-head {
    padding: 18px 20px 0;
    display: grid;
    gap: 4px;

    strong {
        font-size: 16px;
        line-height: 1.4;
    }

    span {
        font-size: 13px;
        line-height: 1.7;
        color: var(--editor-text-subtle);
    }
}

.event-timeline-shell {
    padding: 18px 18px 10px;
}

.event-heading {
    padding: 0 0 14px;
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
}

.event-heading-subtitle {
    font-size: 12px;
    opacity: 0.64;
}

.event-timeline {
    padding: 0 4px 8px;
}

:deep(.event-timeline .n-timeline-item-content) {
    padding-bottom: 20px;
}

:deep(.event-timeline .n-timeline-item-timeline__circle) {
    transform: scale(1.08);
    box-shadow: 0 0 0 6px color-mix(in srgb, var(--accent-soft-muted) 42%, transparent);
}

:deep(.event-timeline .n-timeline-item-timeline__line) {
    background: color-mix(in srgb, var(--accent-primary) 28%, var(--panel-border));
}

.event-main-content-wrap {
    background: transparent;
    backdrop-filter: none;
}

.main-content-wrap {
    background-color: color-mix(in srgb, var(--panel-bg) 44%, transparent);
    backdrop-filter: blur(10px);

    .load-more {
        margin-bottom: 8px;
        .load-more-spinner {
            font-size: 14px;
            opacity: 0.65;
        }
    }
}

.skeleton-wrap {
    background-color: transparent;
}

@media screen and (max-width: 821px) {
    .event-page {
        padding: 12px 10px 0;
    }

    .event-composer-wrap,
    .event-wrap {
        padding-left: 10px;
        padding-right: 10px;
    }

    .event-overview {
        padding: 16px 16px 14px;
        border-radius: 20px;
    }

    .event-overview-copy h1 {
        font-size: 22px;
    }

    .event-overview-stats {
        grid-template-columns: 1fr;
        min-width: 100%;
    }

    .event-timeline-shell {
        padding: 16px 14px 8px;
    }
}
</style>
