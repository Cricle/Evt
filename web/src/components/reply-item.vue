<template>
    <div class="reply-item">
        <div class="header-wrap">
            <div class="username">
                <router-link class="user-link" :to="{
                    name: 'user',
                    query: { s: props.reply.user.username },
                }">
                    {{ props.reply.user.username }}
                </router-link>
                <span class="reply-name">
                    {{ props.reply.at_user_id > 0 ? '回复' : ':' }}
                </span>

                <router-link class="user-link" :to="{
                    name: 'user',
                    query: { s: props.reply.at_user.username },
                }" v-if="props.reply.at_user_id > 0">
                    {{ props.reply.at_user.username }}
                </router-link>
            </div>
            <div class="timestamp">
                {{ props.reply.ip_loc }}
                <n-popconfirm v-if="
                    userInfo.is_admin ||
                    userInfo.id === props.reply.user.id
                " negative-text="取消" positive-text="确认" @positive-click="execDelAction">
                    <template #trigger>
                        <n-button quaternary circle size="tiny" class="del-btn">
                            <template #icon>
                                <n-icon>
                                    <trash />
                                </n-icon>
                            </template>
                        </n-button>
                    </template>
                    是否删除这条回复？
                </n-popconfirm>
            </div>
        </div>

        <div class="base-wrap">
            <div class="content">
                <n-ellipsis expand-trigger="click" line-clamp="5" :tooltip="false">
                    {{ props.reply.content }}
                </n-ellipsis>
            </div>
            <div class="reply-switch">
                <span class="time-item">
                    {{ formatPrettyTime(props.reply.created_on) }}
                </span>

                <div class="actions">
                    <n-popover trigger="click" placement="top" v-if="userLogined">
                        <template #trigger>
                            <span class="show opacity-item reply-btn"> 表情回应 </span>
                        </template>
                        <emoji-reaction-picker @select="handleReaction" />
                    </n-popover>
                    <span v-if="userLogined" class="show opacity-item reply-btn" @click="focusReply"> 回复 </span>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { Trash } from '@vicons/tabler';
import { formatPrettyTime } from '@/utils/formatTime';
import { createCommentReply, deleteCommentReply } from '@/api/post';
import { useStoreUser } from '@/store/user';
import { storeToRefs } from 'pinia';
import EmojiReactionPicker from '@/components/emoji-reaction-picker.vue';

const props = withDefaults(
  defineProps<{
    tweetId: number;
    reply: Item.ReplyProps;
  }>(),
  {},
);

const storeUser = useStoreUser();
const { userInfo, userLogined } = storeToRefs(storeUser);

const emit = defineEmits<{
  (e: 'focusReply', reply: Item.ReplyProps): void;
  (e: 'reload'): void;
}>();

const focusReply = () => {
  emit('focusReply', props.reply);
};
const handleReaction = (emoji: string) => {
  createCommentReply({
    comment_id: props.reply.comment_id,
    at_user_id: props.reply.user_id,
    content: emoji,
  })
    .then(() => {
      window.$message.success(`已添加表情 ${emoji}`);
      emit('reload');
    })
    .catch((err) => {
      console.log(err);
    });
};
const execDelAction = () => {
  deleteCommentReply({
    id: props.reply.id,
  })
    .then((res) => {
      window.$message.success('删除成功');

      setTimeout(() => {
        emit('reload');
      }, 50);
    })
    .catch((err) => {
      console.log(err);
    });
};
</script>


<style lang="less" scoped>
.reply-item {
    display: flex;
    flex-direction: column;
    font-size: 12px;
    padding: 8px;
    border-bottom: 1px solid #f3f3f3;

    .header-wrap {
        display: flex;
        align-items: center;
        justify-content: space-between;

        .username {
            max-width: 50%;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;

            .reply-name {
                margin: 0 3px;
                opacity: 0.75;
            }
        }

        .timestamp {
            opacity: 0.75;
            text-align: right;
            max-width: 50%;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
        }
    }

    .base-wrap {
        display: block;

        .content {
            width: calc(100%);
            margin-top: 4px;
            font-size: 12px;
            text-align: justify;
            line-height: 2;
        }

        .reply-switch {
            display: flex;
            align-items: center;
            justify-content: space-between;
            font-size: 12px;

            .actions {
                display: flex;
                align-items: center;
                text-align: right;
                font-size: 12px;
                margin: 10px 0;
            }

            .time-item {
                font-size: 12px;
                opacity: 0.75;
                margin-right: 18px;
            }

            .action-item {
                display: flex;
                align-items: center;
                margin-left: 18px;
                opacity: 0.65;

                .upvote-count {
                    margin-left: 4px;
                    font-size: 12px;
                }

                &.hover {
                    cursor: pointer;
                }
            }

            .opacity-item {
                opacity: 0.75;
            }

            .reply-btn {
                margin-left: 18px;
             }

            .show {
                color: #18a058;
                cursor: pointer;
                transition: transform 0.18s ease, opacity 0.18s ease;
                &:hover {
                    transform: translateY(-1px);
                }
            }

            .hide {
                opacity: 0.75;
                cursor: pointer;
            }
        }
    }
}

.dark {
    .reply-item {
        border-bottom: 1px solid #262628;
        background-color: rgba(16, 16, 20, 0.75);

        .base-wrap {
            .reply-switch {
                .show {
                    color: #63e2b7;
                }
            }
        }
    }
}
</style>
