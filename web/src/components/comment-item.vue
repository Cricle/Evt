<template>
    <div class="comment-item">
        <n-thing content-indented>
            <template #avatar>
                <n-avatar round :size="30" :src="comment.user.avatar || DEFAULT_USER_AVATAR" />
            </template>
            <template #header>
                <span class="nickname-wrap">
                    <router-link
                        @click.stop
                        class="username-link"
                        :to="{
                            name: 'user',
                            query: { s: comment.user.username },
                        }"
                    >
                        {{ comment.user.nickname }}
                    </router-link>
                </span>
                <span class="username-wrap">
                    @{{ comment.user.username }}
                </span>
                <n-tag
                    v-if="comment.is_essence == YesNoEnum.YES"
                    class="top-tag"
                    type="warning"
                    size="small"
                    round
                >
                    精选
                </n-tag>
            </template>
            <template #header-extra>
                <div class="opt-wrap">
                    <span class="timestamp">
                        {{  comment.ip_loc}}
                    </span>
                    <n-popconfirm
                        v-if="userInfo.id === postUserId"
                        negative-text="取消"
                        positive-text="确认"
                        @positive-click="execHightlightAction"
                    >
                        <template #trigger>
                            <n-button
                                quaternary
                                circle
                                size="tiny"
                                class="action-btn"
                            >
                                <template #icon>
                                    <n-icon v-if="comment.is_essence == YesNoEnum.NO">
                                        <ArrowBarToUp />
                                    </n-icon>
                                    <n-icon v-else>
                                        <ArrowBarDown />
                                    </n-icon>
                                </template>
                            </n-button>
                        </template>
                        {{ comment.is_essence == YesNoEnum.NO ? "是否精选这条评论" : "是否取消精选"}}
                    </n-popconfirm>
                    <n-popconfirm
                        v-if="
                            userInfo.is_admin ||
                            userInfo.id === comment.user.id
                        "
                        negative-text="取消"
                        positive-text="确认"
                        @positive-click="execDelAction"
                    >
                        <template #trigger>
                            <n-button
                                quaternary
                                circle
                                size="tiny"
                                class="action-btn"
                            >
                                <template #icon>
                                    <n-icon>
                                        <trash />
                                    </n-icon>
                                </template>
                            </n-button>
                        </template>
                        是否删除这条评论？
                    </n-popconfirm>
                </div>
            </template>
            <template #description v-if="comment.texts.length > 0">
                <span
                    v-for="content in comment.texts"
                    :key="content.id"
                    class="comment-text"
                    @click.stop="doClickText($event, comment.id)"
                    v-html="parsePostTag(content.content).content"
                ></span>
            </template>

            <template #footer>
                <post-image
                    v-if="comment.imgs.length > 0"
                    :imgs="comment.imgs" />
                  <!-- 回复编辑器 -->
                  <compose-reply
                    ref="replyComposeRef"
                    :comment="comment"
                    :at-userid="replyAtUserID"
                    :at-username="replyAtUsername"
                    @reload="reload"
                    @reset="resetReply"
                />
                <!-- 回复列表 -->
                <div class="reply-wrap">
                    <reply-item
                        v-for="reply in visibleReplies"
                        :key="reply.id"
                        :reply="reply"
                        :tweet-id="comment.post_id"
                        @focusReply="focusReply"
                        @reload="reload"
                    />
                </div>
            </template>
        </n-thing>
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useStoreMain } from '@/store/main';
import { useStoreProfile } from '@/store/profile';
import { useStoreUser } from '@/store/user';
import { useRouter } from 'vue-router';
import { parsePostTag } from '@/utils/content';
import { Trash, ArrowBarToUp, ArrowBarDown } from '@vicons/tabler';
import { deleteComment, highlightComment } from '@/api/post';
import { YesNoEnum } from '@/utils/IEnum';
import { storeToRefs } from 'pinia';
import { DEFAULT_USER_AVATAR } from '@/utils/defaults';
import { splitReplyReactions } from '@/utils/reactions';
import { buildTagSearchRoute } from '@/utils/tagRoute';

const router = useRouter();
const replyAtUserID = ref(0);
const replyAtUsername = ref('');
const replyComposeRef = ref();

const storeMain = useStoreMain();
const storeProfile = useStoreProfile();
const storeUser = useStoreUser();
const { currentSpaceSlug } = storeToRefs(storeProfile);
const { userInfo } = storeToRefs(storeUser);

const emit = defineEmits<(e: 'reload') => void>();
const props = withDefaults(
  defineProps<{
    comment: Item.CommentProps;
    postUserId: number;
  }>(),
  {},
);

const comment = computed(() => {
  const comment: Item.CommentComponentProps = Object.assign(
    {
      texts: [],
      imgs: [],
    },
    props.comment,
  );
  comment.contents.map((content) => {
    if (+content.type === 1 || +content.type === 2) {
      comment.texts.push(content);
    }
    if (+content.type === 3) {
      comment.imgs.push(content);
    }
  });
  return comment;
});

const visibleReplies = computed(() => splitReplyReactions(props.comment.replies || []).visibleReplies);

const doClickText = (e: MouseEvent, id: number | string) => {
  const target = e.target as HTMLElement | null;
  if (!target?.dataset.detail) {
    return;
  }
  const d = target.dataset.detail.split(':');
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
};

const focusReply = (reply: Item.ReplyProps) => {
  replyAtUserID.value = reply.user_id;
  replyAtUsername.value = reply.user?.username || '';
  replyComposeRef.value?.switchReply(true);
};
const reload = () => {
  emit('reload');
};
const resetReply = () => {
  replyAtUserID.value = 0;
  replyAtUsername.value = '';
};

const execDelAction = () => {
  deleteComment({
    id: comment.value.id,
  })
    .then((_res) => {
      window.$message.success('删除成功');
      reload();
    })
    .catch((_err) => {
      window.$message.error('删除评论失败');
    });
};

const execHightlightAction = () => {
  highlightComment({
    id: comment.value.id,
  })
    .then((res) => {
      comment.value.is_essence = res.highlight_status;
      window.$message.success('操作成功');
      reload();
    })
    .catch((_err) => {
      window.$message.error('更新评论亮点状态失败');
    });
};
</script>

<style lang="less" scoped>
.comment-item {
    background-color: var(--surface-base);
    width: 100%;
    padding: 16px;
    box-sizing: border-box;

    .nickname-wrap {
        font-size: 14px;
    }
    .username-wrap {
        font-size: 14px;
        opacity: 0.75;
    }
    .top-tag {
        transform: scale(0.75);
    }
    .opt-wrap {
        display: flex;
        align-items: center;
        .timestamp {
            opacity: 0.75;
            font-size: 12px;
        }
        .action-btn {
            margin-left: 4px;
        }
    }
    .comment-text {
        display: block;
        text-align: justify;
        overflow: hidden;
        white-space: pre-wrap;
        word-break: break-all;
        line-height: 1.7;
    }
    .opt-item {
        display: flex;
        align-items: center;
        opacity: 0.7;
        .opt-item-icon {
            margin-right: 10px;
        }
    }
}

.reply-wrap {
    --comment-reply-bg: var(--surface-muted);
    margin-top: 10px;
    border-radius: 5px;
    background: var(--comment-reply-bg);

    .reply-item {
        &:last-child {
            border-bottom: none;
        }
    }
}

</style>
