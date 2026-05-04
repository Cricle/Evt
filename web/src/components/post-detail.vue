<template>
    <div class="detail-item">
        <n-thing content-indented>
            <template #avatar>
                <n-avatar round :size="30" :src="post.user.avatar || DEFAULT_USER_AVATAR" />
            </template>
            <template #header>
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
                    v-if="post.visibility == VisibilityEnum.PRIVATE"
                    class="top-tag"
                    type="error"
                    size="small"
                    round
                >
                    私密
                </n-tag>
                <n-tag
                    v-if="post.visibility == VisibilityEnum.FRIEND"
                    class="top-tag"
                    type="info"
                    size="small"
                    round
                >
                    好友可见
                </n-tag>
            </template>
            <template #header-extra>
                <div class="options">
                    <n-dropdown
                        placement="bottom-end"
                        trigger="click"
                        size="small"
                        :options="adminOptions"
                        @select="handlePostAction"
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

                <!-- 删除确认 -->
                <n-modal
                    v-model:show="showDelModal"
                    :mask-closable="false"
                    preset="dialog"
                    title="提示"
                    content="确定删除该动态吗？"
                    positive-text="确认"
                    negative-text="取消"
                    @positive-click="execDelAction"
                />
                <!-- 锁定确认 -->
                <n-modal
                    v-model:show="showLockModal"
                    :mask-closable="false"
                    preset="dialog"
                    title="提示"
                    :content="
                        '确定' +
                        (post.is_lock ? '解锁' : '锁定') +
                        '该动态吗？'
                    "
                    positive-text="确认"
                    negative-text="取消"
                    @positive-click="execLockAction"
                />
                <!-- 置顶确认 -->
                <n-modal
                    v-model:show="showStickModal"
                    :mask-closable="false"
                    preset="dialog"
                    title="提示"
                    :content="
                        '确定' +
                        (post.is_top ? '取消置顶' : '置顶') +
                        '该动态吗？'
                    "
                    positive-text="确认"
                    negative-text="取消"
                    @positive-click="execStickAction"
                />
                <!-- 亮点确认 -->
                <n-modal
                    v-model:show="showHighlightModal"
                    :mask-closable="false"
                    preset="dialog"
                    title="提示"
                    :content="
                        '确定将该动态' +
                        (post.is_essence ? '取消亮点' : '设为亮点') +
                        '吗？'
                    "
                    positive-text="确认"
                    negative-text="取消"
                    @positive-click="execHighlightAction"
                />
                <!-- 修改可见度确认 -->
                <n-modal
                    v-model:show="showVisibilityModal"
                    :mask-closable="false"
                    preset="dialog"
                    title="提示"
                    :content="
                        '确定将该动态可见度修改为' +
                        (tempVisibility == 0 ? '公开' : (tempVisibility == 1 ? '私密' : (tempVisibility == 2 ? '好友可见' : '关注可见'))) +
                        '吗？'
                    "
                    positive-text="确认"
                    negative-text="取消"
                    @positive-click="execVisibilityAction"
                />
                  <!-- 私信组件 -->
                <whisper :show="showWhisper" :user="whisperReceiver" @success="whisperSuccess" />
            </template>
            <div v-if="post.texts.length > 0">
                <div v-if="isEventMode" class="event-hero">
                    <div class="event-hero-main">
                        <span class="event-hero-kicker">持续更新事件</span>
                        <strong>{{ eventTitle }}</strong>
                        <p>{{ eventSummary }}</p>
                    </div>
                    <div class="event-hero-stats">
                        <div class="event-stat-card">
                            <span>节点</span>
                            <strong>{{ post.comment_count }}</strong>
                        </div>
                        <div class="event-stat-card">
                            <span>表情</span>
                            <strong>{{ post.upvote_count }}</strong>
                        </div>
                    </div>
                </div>
                <span
                    v-for="content in post.texts"
                    :key="content.id"
                    class="post-text"
                    @click.stop="doClickText($event, post.id)"
                    v-html="parsePostTag(content.content).content"
                >
                </span>
            </div>

            <template #footer>
                <post-attachment :attachments="post.attachments" />
                <post-attachment
                    :attachments="post.charge_attachments"
                    :price="post.attachment_price"
                />
                <post-image :imgs="post.imgs" />
                <post-video :videos="post.videos" :full="true" />
                <post-link :links="post.links" />
                <div class="timestamp">
                    {{ isEventMode ? '事件创建于' : '发布于' }} {{ formatPrettyTime(post.created_on) }}
                    <span v-if="post.ip_loc">
                        <n-divider vertical />
                        {{ post.ip_loc }}
                    </span>
                    <span v-if="!collapsedLeft && post.created_on != post.latest_replied_on">
                        <n-divider vertical /> 最后回复
                        {{ formatPrettyTime(post.latest_replied_on) }}
                    </span>
                </div>
            </template>
            <template #action>
                <div class="detail-actions">
                    <post-reaction-bar
                        :reactions="post.reactions || []"
                        :count="post.upvote_count"
                        :max-visible="16"
                        @select="handlePostReaction"
                    />
                    <n-button quaternary class="opt-item" @click.stop="handlePostShare">
                        <template #icon>
                            <n-icon size="18">
                                <share-social-outline />
                            </n-icon>
                        </template>
                        分享
                    </n-button>
                </div>
            </template>
        </n-thing>
    </div>
</template>

<script setup lang="ts">
import { h, ref, computed } from 'vue';
import type { Component } from 'vue';
import { NIcon, useDialog } from 'naive-ui';
import { useStoreMain } from '@/store/main';
import { useRouter } from 'vue-router';
import { formatPrettyTime } from '@/utils/formatTime';
import { parsePostTag } from '@/utils/content';
import {
  PaperPlaneOutline,
  ShareSocialOutline,
  PushOutline,
  TrashOutline,
  LockClosedOutline,
  LockOpenOutline,
  EyeOutline,
  EyeOffOutline,
  BodyOutline,
  WalkOutline,
  PersonOutline,
  FlameOutline,
} from '@vicons/ionicons5';
import { MoreHorizFilled } from '@vicons/material';
import {
  deletePost,
  lockPost,
  stickPost,
  highlightPost,
  visibilityPost,
  togglePostReaction,
} from '@/api/post';
import type { DropdownOption } from 'naive-ui';
import { VisibilityEnum } from '@/utils/IEnum';
import copy from 'copy-to-clipboard';
import { storeToRefs } from 'pinia';
import { useStoreProfile } from '@/store/profile';
import { useStoreUser } from '@/store/user';
import UserAction from '@/composables/useUserAction';
import { usePostContent } from '@/composables/usePostContent';
import PostReactionBar from '@/components/post-reaction-bar.vue';
import { DEFAULT_USER_AVATAR } from '@/utils/defaults';
import { goToAuth } from '@/utils/authRoute';
import { buildHomeRouteWithSpace, buildPostRoute, buildTagSearchRoute } from '@/utils/tagRoute';
import { isEventPost } from '@/utils/postKind';
import { resolveEventNarrative, resolveEventTitle } from '@/utils/eventTimeline';

const useFriendship =
  import.meta.env.VITE_USE_FRIENDSHIP.toLowerCase() === 'true';

const storeMain = useStoreMain();
const storeProfile = useStoreProfile();
const storeUser = useStoreUser();
const { collapsedLeft } = storeToRefs(storeMain);
const { currentSpaceSlug } = storeToRefs(storeProfile);
const { userInfo } = storeToRefs(storeUser);

const router = useRouter();
const dialog = useDialog();
const props = withDefaults(
  defineProps<{
    post: Item.PostProps;
  }>(),
  {},
);
const showDelModal = ref(false);
const showLockModal = ref(false);
const showStickModal = ref(false);
const showHighlightModal = ref(false);
const showVisibilityModal = ref(false);
const loading = ref(false);
const tempVisibility = ref<VisibilityEnum>(VisibilityEnum.PUBLIC);
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

const onSendWhisper = (user: Item.UserInfo) => {
  whisperReceiver.value = user;
  showWhisper.value = true;
};

const whisperSuccess = () => {
  showWhisper.value = false;
};

const emit = defineEmits<{
  (e: 'reload', post_id: number): void;
  (e: 'reaction-added', payload: { reactions: Item.ReactionGroup[]; commentCount: number }): void;
}>();

// 使用 usePostContent composable (包含额外字段)
const post = usePostContent(props.post, true);
const isEventMode = computed(() => isEventPost(post.value));
const eventTitle = computed(() => resolveEventTitle(post.value.texts || []));
const eventSummary = computed(() =>
  resolveEventNarrative(post.value.user.nickname || post.value.user.username, post.value.ip_loc, '创建'),
);

const renderIcon = (icon: Component) => {
  return () => {
    return h(NIcon, null, {
      default: () => h(icon),
    });
  };
};

const adminOptions = computed(() => {
  let options: DropdownOption[] = [];
  if (
    !userInfo.value.is_admin &&
    userInfo.value.id != props.post.user.id
  ) {
    options.push({
      label: '私信 @' + props.post.user.username,
      key: 'whisper',
      icon: renderIcon(PaperPlaneOutline),
    });
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
    return options;
  }
  options.push({
    label: '删除',
    key: 'delete',
    icon: renderIcon(TrashOutline),
  });
  if (post.value.is_lock === 0) {
    options.push({
      label: '锁定',
      key: 'lock',
      icon: renderIcon(LockClosedOutline),
    });
  } else {
    options.push({
      label: '解锁',
      key: 'unlock',
      icon: renderIcon(LockOpenOutline),
    });
  }
  if (userInfo.value.is_admin) {
    if (post.value.is_top === 0) {
      options.push({
        label: '置顶',
        key: 'stick',
        icon: renderIcon(PushOutline),
      });
    } else {
      options.push({
        label: '取消置顶',
        key: 'unstick',
        icon: renderIcon(PushOutline),
      });
    }
  }
  if (post.value.is_essence === 0) {
    options.push({
      label: '设为亮点',
      key: 'highlight',
      icon: renderIcon(FlameOutline),
    });
  } else {
    options.push({
      label: '取消亮点',
      key: 'unhighlight',
      icon: renderIcon(FlameOutline),
    });
  }
  let visitMenu: DropdownOption;
  if (post.value.visibility === VisibilityEnum.PUBLIC) {
    visitMenu = {
      label: '公开',
      key: 'vpublic',
      icon: renderIcon(EyeOutline),
      children: [
        { label: '私密', key: 'vprivate', icon: renderIcon(EyeOffOutline) },
        { label: '关注可见', key: 'vfollowing', icon: renderIcon(BodyOutline) },
      ],
    };
  } else if (post.value.visibility === VisibilityEnum.PRIVATE) {
    visitMenu = {
      label: '私密',
      key: 'vprivate',
      icon: renderIcon(EyeOffOutline),
      children: [
        { label: '公开', key: 'vpublic', icon: renderIcon(EyeOutline) },
        { label: '关注可见', key: 'vfollowing', icon: renderIcon(BodyOutline) },
      ],
    };
  } else if (useFriendship && post.value.visibility === VisibilityEnum.FRIEND) {
    visitMenu = {
      label: '好友可见',
      key: 'vfriend',
      icon: renderIcon(PersonOutline),
      children: [
        { label: '公开', key: 'vpublic', icon: renderIcon(EyeOutline) },
        { label: '私密', key: 'vprivate', icon: renderIcon(EyeOffOutline) },
        { label: '关注可见', key: 'vfollowing', icon: renderIcon(BodyOutline) },
      ],
    };
  } else {
    visitMenu = {
      label: '关注可见',
      key: 'vfollowing',
      icon: renderIcon(BodyOutline),
      children: [
        { label: '公开', key: 'vpublic', icon: renderIcon(EyeOutline) },
        { label: '私密', key: 'vprivate', icon: renderIcon(EyeOffOutline) },
      ],
    };
  }
  if (useFriendship && post.value.visibility !== VisibilityEnum.FRIEND) {
    visitMenu.children?.push({
      label: '好友可见',
      key: 'vfriend',
      icon: renderIcon(PersonOutline),
    });
  }
  options.push(visitMenu);
  return options;
});

const onHandleFollowAction = (post: Item.PostProps) => {
	UserAction.followAction(dialog, post.user.id, post.user.username, post.user.is_following)
		.then(_action => {
			post.user.is_following = _action;
		})
};

const doClickText = (e: MouseEvent, id: number) => {
  if ((e.target as any).dataset.detail) {
    const d = (e.target as any).dataset.detail.split(':');
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
      return;
    }
  }
  router.push(buildPostRoute(id, currentSpaceSlug.value));
};
const handlePostAction = (
  item:
    | 'whisper'
    | 'follow'
    | 'unfollow'
    | 'delete'
    | 'lock'
    | 'unlock'
    | 'stick'
    | 'unstick'
    | 'highlight'
    | 'unhighlight'
    | 'vpublic'
    | 'vprivate'
    | 'vfriend'
    | 'vfollowing',
) => {
  switch (item) {
    case 'whisper':
      onSendWhisper(props.post.user);
      break;
    case 'follow':
    case 'unfollow':
      onHandleFollowAction(props.post);
      break;
    case 'delete':
      showDelModal.value = true;
      break;
    case 'lock':
    case 'unlock':
      showLockModal.value = true;
      break;
    case 'stick':
    case 'unstick':
      showStickModal.value = true;
      break;
    case 'highlight':
    case 'unhighlight':
      showHighlightModal.value = true;
      break;
    case 'vpublic':
      tempVisibility.value = 0;
      showVisibilityModal.value = true;
      break;
    case 'vprivate':
      tempVisibility.value = 1;
      showVisibilityModal.value = true;
      break;
    case 'vfriend':
      tempVisibility.value = 2;
      showVisibilityModal.value = true;
      break;
    case 'vfollowing':
      tempVisibility.value = 3;
      showVisibilityModal.value = true;
      break;
    default:
      break;
  }
};
const execDelAction = () => {
  deletePost({
    id: post.value.id,
  })
    .then((_res) => {
      window.$message.success('删除成功');
      router.replace(buildHomeRouteWithSpace({}, currentSpaceSlug.value));
      storeMain.doRefresh();
    })
    .catch((_err) => {
      loading.value = false;
      window.$message.error('删除动态失败');
    });
};
const execLockAction = () => {
  lockPost({
    id: post.value.id,
  })
    .then((res) => {
      emit('reload', post.value.id);
      if (res.lock_status === 1) {
        window.$message.success('锁定成功');
      } else {
        window.$message.success('解锁成功');
      }
    })
    .catch((_err) => {
      loading.value = false;
      window.$message.error('更新锁定状态失败');
    });
};
const execStickAction = () => {
  stickPost({
    id: post.value.id,
  })
    .then((res) => {
      emit('reload', post.value.id);
      if (res.top_status === 1) {
        window.$message.success('置顶成功');
      } else {
        window.$message.success('取消置顶成功');
      }
    })
    .catch((_err) => {
      loading.value = false;
      window.$message.error('更新置顶状态失败');
    });
};
const execHighlightAction = () => {
  highlightPost({
    id: post.value.id,
  })
    .then((res) => {
      post.value = {
        ...post.value,
        is_essence: res.highlight_status,
      };
      if (res.highlight_status === 1) {
        window.$message.success('设为亮点成功');
      } else {
        window.$message.success('取消亮点成功');
      }
    })
    .catch((_err) => {
      loading.value = false;
      window.$message.error('更新亮点状态失败');
    });
};
const execVisibilityAction = () => {
  visibilityPost({
    id: post.value.id,
    visibility: tempVisibility.value,
  })
    .then((_res) => {
      emit('reload', post.value.id);
      window.$message.success('修改可见性成功');
    })
    .catch((_err) => {
      loading.value = false;
      window.$message.error('修改可见性失败');
    });
};
const handlePostReaction = (emoji: string) => {
  if (userInfo.value.id < 1) {
    goToAuth(router, 'signin', router.currentRoute.value.fullPath);
    return;
  }

  togglePostReaction(post.value.id, emoji)
    .then((res) => {
      post.value = {
        ...post.value,
        reactions: res.reactions || [],
        upvote_count: (res.reactions || []).reduce((sum, item) => sum + item.count, 0),
        comment_count: res.comment_count,
      };
      emit('reaction-added', {
        reactions: res.reactions || [],
        commentCount: res.comment_count,
      });
    })
    .catch(() => {
      window.$message.error('表情回复失败');
    });
};
const handlePostShare = () => {
  copy(
    `${window.location.origin}/#/post?id=${post.value.id}&share=copy_link&t=${new Date().getTime()}`,
  );
  window.$message.success(isEventMode.value ? '事件链接已复制到剪贴板' : '链接已复制到剪贴板');
};
</script>

<style lang="less">
.detail-item {
    width: 100%;
    padding: 18px 18px 14px;
    box-sizing: border-box;
    background: transparent;
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
    .options {
        opacity: 0.75;
    }
    .post-text {
        font-size: 16px;
        text-align: justify;
        overflow: hidden;
        white-space: pre-wrap;
        word-break: break-all;
        line-height: 1.84;
    }
    .n-thing {
        .n-thing-avatar-header-wrapper {
            align-items: center;
        }
    }
    .timestamp {
        opacity: 0.75;
        font-size: 12px;
        margin-top: 14px;
    }
    .event-hero {
        margin-bottom: 18px;
        display: flex;
        align-items: stretch;
        justify-content: space-between;
        gap: 14px;
        flex-wrap: wrap;
        padding: 16px;
        border-radius: 20px;
        border: 1px solid var(--panel-border);
        background:
          radial-gradient(circle at top right, color-mix(in srgb, var(--accent-soft) 72%, transparent), transparent 42%),
          var(--accent-soft-muted);
    }
    .event-hero-main {
        display: grid;
        gap: 6px;
        min-width: min(100%, 320px);

        strong {
            font-size: 20px;
            line-height: 1.4;
        }

        p {
            margin: 0;
            font-size: 13px;
            line-height: 1.7;
            color: var(--editor-text-subtle);
        }
    }
    .event-hero-kicker {
        font-size: 12px;
        font-weight: 700;
        letter-spacing: 0.04em;
        color: var(--accent-primary);
    }
    .event-hero-stats {
        display: flex;
        align-items: stretch;
        gap: 10px;
        flex-wrap: wrap;
    }
    .event-stat-card {
        min-width: 92px;
        padding: 12px 14px;
        border-radius: 16px;
        background: var(--panel-bg);
        border: 1px solid var(--panel-border);
        display: grid;
        gap: 4px;

        span {
            font-size: 12px;
            opacity: 0.68;
        }

        strong {
            font-size: 20px;
            line-height: 1.2;
        }
    }
    .detail-actions {
        margin-top: 18px;
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
    }
    .opt-item {
        min-width: 72px;
        justify-content: center;
    }
}

@media screen and (max-width: 821px) {
    .detail-item {
        .event-hero {
            padding: 14px;
            border-radius: 18px;
        }

        .event-hero-main strong {
            font-size: 18px;
        }
    }
}
</style>
