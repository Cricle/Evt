<template>
    <n-modal
        :show="show"
        @update:show="closeModal"
        class="whisper-card"
        preset="card"
        size="small"
        title="申请添加朋友"
        :mask-closable="false"
        :bordered="false"
        :style="{
            width: '360px',
        }"
    >
        <div class="whisper-wrap">
             <n-alert :show-icon="false">
                发送添加朋友申请给:
                <n-ellipsis style="max-width: 100%">
                    <n-gradient-text type="success">
                        {{ user.nickname }}@{{ user.username }}
                    </n-gradient-text>
                </n-ellipsis>
            </n-alert>
            <div class="whisper-line">
                <n-input
                    type="textarea"
                    placeholder="请输入真挚的问候语"
                    :autosize="{
                        minRows: 5,
                        maxRows: 10,
                    }"
                    v-model:value="content"
                    maxlength="120"
                    show-count
                />
            </div>
            <div class="whisper-line send-wrap">
                <n-button
                    strong
                    secondary
                    type="primary"
                    :loading="loading"
                    @click="sendWhisper"
                >
                    发送
                </n-button>
            </div>
        </div>
    </n-modal>
</template>

<script setup lang="ts">
import { Api } from '@/utils/request';
import { ref } from 'vue';

const props = withDefaults(
  defineProps<{
    show: boolean;
    user: Item.UserInfo;
  }>(),
  {
    show: false,
  },
);
const content = ref('');
const loading = ref(false);

const emit = defineEmits<{
  (e: 'success'): void;
}>();
const closeModal = () => {
  emit('success');
};
const sendWhisper = () => {
  if (!content.value.trim()) {
    window.$message.warning('请输入问候语');
    return;
  }
  loading.value = true;
  Api.v1.friend.post.requesting({
    user_id: props.user.id,
    greetings: content.value,
  })
    .then((res: any) => {
      window.$message.success('发送成功');
      loading.value = false;
      content.value = '';

      closeModal();
    })
    .catch(() => {
      loading.value = false;
      window.$message.error('好友申请发送失败');
    });
};
</script>

<style lang="less" scoped>
.whisper-wrap {
    --whisper-surface: transparent;
    background-color: var(--whisper-surface);

    .whisper-line {
        margin-top: 10px;

        &.send-wrap {
            .n-button {
                width: 100%;
            }
        }
    }
}

:global(.dark) .whisper-wrap {
    --whisper-surface: rgba(16, 16, 20, 0.75);
}
</style>
