<template>
    <div class="post-item" @click="goPostDetail(post.id)">
        <n-thing content-indented>
            <template #avatar>
                <n-avatar round :size="30" :src="post.user.avatar || DEFAULT_USER_AVATAR" />
            </template>
            <template #header>
                    <span class="nickname-wrap">
                        <router-link
                            @click.stop
                            class="username-link"
                            :to="{
                                name: 'user',
                                query: { s: post.user.username },
                            }"
                        >
                            {{ post.user.nickname }}
                        </router-link>
                    </span>
                    <span class="username-wrap"> @{{ post.user.username }} </span>
                    <n-tag
                        v-if="isEventMode"
                        class="top-tag"
                        type="success"
                        size="small"
                        round
                    >
                        事件
                    </n-tag>
                    <n-tag
                        v-if="post.is_top"
                        class="top-tag"
                        type="warning"
                        size="small"
                        round
                    >
                        置顶
                    </n-tag>
                    <n-tag
                        v-if="post.visibility == 1"
                        class="top-tag"
                        type="error"
                        size="small"
                        round
                    >
                        私密
                    </n-tag>
                    <n-tag
                        v-if="post.visibility == 2"
                        class="top-tag"
                        type="info"
                        size="small"
                        round
                    >
                        好友可见
                    </n-tag>
                    <div v-if="isMobile">
                        <span class="timestamp-mobile">
                            {{ formatPrettyDate(post.created_on) }} {{ post.ip_loc }}
                        </span>
                    </div>
            </template>
            <template #header-extra>
                <div class="item-header-extra">
                    <span v-if="!isMobile" class="timestamp">
                        {{ post.ip_loc ? post.ip_loc + ' · ' : post.ip_loc }}
                        {{ formatPrettyDate(post.created_on) }}
                    </span>
                    <n-dropdown
                        placement="bottom-end"
                        :trigger="isMobile ? 'click' : 'hover'"
                        size="small"
                        :options="tweetOptions"
                        @select="handleTweetAction"
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
            </template>
            <template #description v-if="post.texts.length > 0">
                <div v-if="isEventMode" class="event-card-preview">
                    <div class="event-card-preview-head">
                        <span class="event-card-preview-kicker">事件时间轴</span>
                        <span class="event-card-preview-time">{{ formatPrettyDate(post.created_on) }}</span>
                    </div>
                    <strong class="event-card-preview-title">{{ eventTitle }}</strong>
                    <p v-if="eventSummary" class="event-card-preview-summary">{{ eventSummary }}</p>
                    <div class="event-card-preview-stats">
                        <span>节点 {{ post.comment_count }}</span>
                        <span>表情 {{ post.upvote_count }}</span>
                    </div>
                </div>
                <div v-if="!isEventMode && isMobile" @click="goPostDetail(post.id)">
                    <span v-for="content in post.texts"
                        :key="content.id"
                        class="post-text"
                        @click.stop="doClickText($event, post.id)"
                        v-html="preparePost(content.content, '展开', '收起', profile.tweetMobileEllipsisSize, inFoldStyle)"
                    ></span>
                </div>
                <span
                    v-else-if="!isEventMode"
                    v-for="content in post.texts"
                    :key="content.id"
                    class="post-text hover"
                    @click.stop="doClickText($event, post.id)"
                    v-html="preparePost(content.content, '展开', '收起', profile.tweetWebEllipsisSize, inFoldStyle)"
                ></span>
            </template>

            <template #footer>
                <post-attachment 
                    v-if="post.attachments.length > 0"
                    :attachments="post.attachments" />
                <post-attachment
                    v-if="post.charge_attachments.length > 0"
                    :attachments="post.charge_attachments"
                    :price="post.attachment_price"
                />
                <post-image
                    v-if="post.imgs.length > 0"
                    :imgs="post.imgs" />
                <post-video
                    v-if="post.videos.length > 0"
                    :videos="post.videos" />
                <post-link
                    v-if="post.links.length > 0"
                    :links="post.links" />
            </template>
            <template #action>
                <post-reaction-bar
                    :reactions="postReactions"
                    :count="post.upvote_count"
                    :max-visible="12"
                    @select="handlePostReaction"
                />
            </template>
        </n-thing>
    </div>
</template>

<script setup lang="ts">
import { h, ref, computed, watch } from 'vue';
import { useStoreMain } from '@/store/main';
import { useRouter } from 'vue-router';
import { NIcon, useDialog } from 'naive-ui';
import type { Component } from 'vue';
import type { DropdownOption } from 'naive-ui';
import { formatPrettyDate } from '@/utils/formatTime';
import { preparePost } from '@/utils/content';
import { createComment, togglePostReaction } from '@/api/post';
import {
  PaperPlaneOutline,
  ShareSocialOutline,
  PersonAddOutline,
  PersonRemoveOutline,
  BodyOutline,
  WalkOutline,
} from '@vicons/ionicons5';
import { MoreHorizFilled } from '@vicons/material';
import copy from 'copy-to-clipboard';
import { useStoreProfile } from '@/store/profile';
import { storeToRefs } from 'pinia';
import UserAction from '@/composables/useUserAction';
import { usePostContent } from '@/composables/usePostContent';
import { DEFAULT_USER_AVATAR } from '@/utils/defaults';
import { useStoreUser } from '@/store/user';
import { goToAuth } from '@/utils/authRoute';
import { buildPostRoute, buildTagSearchRoute } from '@/utils/tagRoute';
import PostReactionBar from '@/components/post-reaction-bar.vue';
import type { ReactionGroup } from '@/utils/reactions';
import { splitCommentReactions } from '@/utils/reactions';
import { isEventPost } from '@/utils/postKind';
import { resolveEventSummary, resolveEventTitle } from '@/utils/eventTimeline';

const router = useRouter();

const storeMain = useStoreMain();
const storeUser = useStoreUser();
const storeProfile = useStoreProfile();
const { profile, currentSpaceSlug } = storeToRefs(storeProfile);
const { userInfo } = storeToRefs(storeUser);

const dialog = useDialog();

const inFoldStyle = ref<boolean>(true);
const props = withDefaults(defineProps<{
    post: Item.PostProps;
    isOwner: boolean;
    addFriendAction?: boolean;
    addFollowAction?: boolean;
    isMobile?: boolean;
}>(), {
	addFollowAction: false,
	addFriendAction: false,
    isMobile: false,
});

const emit = defineEmits<{
  (e: 'send-whisper', user: Item.UserInfo): void;
  (e: 'handle-follow-action', user: Item.PostProps): void;
  (e: 'handle-friend-action', user: Item.PostProps): void;
  (e: 'post-follow-action', user_id: number, is_following: boolean): void;
}>();
const isEventMode = computed(() => isEventPost(props.post));
const eventTitle = computed(() => resolveEventTitle(props.post.texts || []));
const eventSummary = computed(() => resolveEventSummary(props.post.texts || []));

const renderIcon = (icon: Component) => {
  return () => {
    return h(NIcon, null, {
      default: () => h(icon),
    });
  };
};

const tweetOptions = computed(() => {
  let options: DropdownOption[] = [];
  if (!props.isOwner) {
    options.push({
      label: '私信 @' + props.post.user.username,
      key: 'whisper',
      icon: renderIcon(PaperPlaneOutline),
    });
  }
  if (!props.isOwner && props.addFollowAction) {
    if (props.post.user.is_following) {
      options.push({
        label: '取消关注 @' + props.post.user.username,
        key: 'unfollow',
        icon: renderIcon(WalkOutline),
      });
    } else {
      options.push({
        label: '关注 @' + props.post.user.username,
        key: 'follow',
        icon: renderIcon(BodyOutline),
      });
    }
  }
  if (!props.isOwner && props.addFriendAction) {
    if (props.post.user.is_friend) {
      options.push({
        label: '删除好友 @' + props.post.user.username,
        key: 'delete',
        icon: renderIcon(PersonRemoveOutline),
      });
    } else {
      options.push({
        label: '添加朋友 @' + props.post.user.username,
        key: 'requesting',
        icon: renderIcon(PersonAddOutline),
      });
    }
  }
  options.push({
    label: '复制链接',
    key: 'copyTweetLink',
    icon: renderIcon(ShareSocialOutline),
  });
  return options;
});

const handleTweetAction = async (
  item:
    | 'copyTweetLink'
    | 'whisper'
    | 'follow'
    | 'unfollow'
    | 'delete'
    | 'requesting',
) => {
  switch (item) {
    case 'copyTweetLink':
      copy(
        `${window.location.origin}/#/post?id=${post.value.id}&share=copy_link&t=${new Date().getTime()}`,
      );
      window.$message.success('链接已复制到剪贴板');
      break;
    case 'whisper':
      emit('send-whisper', props.post.user);
      break;
    case 'delete':
    case 'requesting':
      emit('handle-friend-action', props.post);
      break;
    case 'follow':
    case 'unfollow':
      UserAction.followAction(dialog, props.post.user.id, props.post.user.username, props.post.user.is_following)
        .then(_action => {
          emit('post-follow-action', props.post.user.id, _action);
        })
		  emit('handle-follow-action', props.post);
      break;
    default:
    	break;
  }
};

// 使用 usePostContent composable
const post = usePostContent(props.post);
const postReactions = ref<ReactionGroup[]>([]);

watch(
  () => props.post,
  (value) => {
    const withComments = value as Item.PostProps & { comments?: Item.CommentProps[] };
    postReactions.value = value.reactions || splitCommentReactions(withComments.comments || []).reactions;
  },
  { immediate: true },
);

const handlePostReaction = (emoji: string) => {
  if (userInfo.value.id < 1) {
    goToAuth(router, 'signin', router.currentRoute.value.fullPath);
    return;
  }

  togglePostReaction(post.value.id, emoji)
    .then((res) => {
      postReactions.value = res.reactions || [];
      post.value = {
        ...post.value,
        reactions: res.reactions || [],
        upvote_count: (res.reactions || []).reduce((sum, item) => sum + item.count, 0),
        comment_count: res.comment_count,
      };
    })
    .catch(() => {
      window.$message.error('表情回复失败');
    });
};
const goPostDetail = (id: number) => {
  router.push(buildPostRoute(id, currentSpaceSlug.value));
};
const doClickText = (e: MouseEvent, id: number) => {
  const detail = (e.target as any).dataset.detail;
  if (detail && detail !== 'post') {
    const d = detail.split(':');
    if (d.length === 2) {
      storeMain.doRefresh();
      if (d[0] === 'tag') {
        router.push(buildTagSearchRoute(d[1], currentSpaceSlug.value));
      } else {
        router.push({
          name: 'user',
          query: {
            s: d[1],
          },
        });
      }
    }
  } else if (detail && detail === 'post') {
    inFoldStyle.value = !inFoldStyle.value;
  } else {
    goPostDetail(id);
  }
};
</script>

<style lang="less">
.post-item {
    width: 100%;
    padding: 16px 18px 14px;
    box-sizing: border-box;
    border-bottom: 1px solid var(--panel-border);
    background: transparent;
    transition: background-color 0.18s ease;

    .nickname-wrap {
        font-size: 15px;
        font-weight: 600;
    }
    .username-wrap {
        font-size: 13px;
        opacity: 0.75;
    }

    .top-tag {
        transform: scale(0.75);
    }
    .timestamp-mobile {
        margin-top: 2px;
        opacity: 0.75;
        font-size: 11px;
    }
    .item-header-extra {
        display: flex;
        align-items: center;
        opacity: 0.75;
        .timestamp {
            font-size: 12px;
        }
    }
    .post-text {
        display: block;
        text-align: justify;
        overflow: hidden;
        white-space: pre-wrap;
        word-break: break-all;
        line-height: 1.78;
        color: var(--editor-text-main);
    }

    .event-card-preview {
        display: grid;
        gap: 8px;
        margin-bottom: 12px;
        padding: 14px 16px;
        border: 1px solid color-mix(in srgb, var(--panel-border) 84%, transparent);
        border-radius: 18px;
        background:
          radial-gradient(circle at top right, color-mix(in srgb, var(--accent-soft) 62%, transparent), transparent 40%),
          color-mix(in srgb, var(--surface-subtle) 92%, transparent);
    }

    .event-card-preview-head {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 10px;
        flex-wrap: wrap;
    }

    .event-card-preview-kicker {
        font-size: 11px;
        font-weight: 700;
        letter-spacing: 0.06em;
        color: var(--accent-primary);
    }

    .event-card-preview-time {
        font-size: 12px;
        opacity: 0.62;
    }

    .event-card-preview-title {
        font-size: 17px;
        line-height: 1.45;
        color: var(--editor-text-main);
    }

    .event-card-preview-summary {
        margin: 0;
        font-size: 13px;
        line-height: 1.72;
        color: var(--editor-text-subtle);
    }

    .event-card-preview-stats {
        display: flex;
        align-items: center;
        gap: 12px;
        flex-wrap: wrap;

        span {
            font-size: 12px;
            color: var(--editor-text-subtle);
        }
    }

    .opt-item {
        display: flex;
        align-items: center;
        opacity: 0.7;
        transition: opacity 0.18s ease;
        .opt-item-icon {
            margin-right: 10px;
        }

        &:hover {
            opacity: 1;
        }
    }
    
    &:hover {
        background: color-mix(in srgb, var(--surface-subtle) 82%, transparent);
    }
    
    &.hover {
        cursor: pointer;
    }

    .n-thing-avatar {
        margin-top: 0;
    }
    .n-thing-header {
        line-height: 16px;
        margin-bottom: 8px !important;
    }
}

.post-item:last-child {
    border-bottom: 0;
}

</style>
