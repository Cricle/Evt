<template>
    <n-modal
        :show="show"
        @update:show="closeModal"
        class="whisper-card"
        preset="card"
        size="small"
        title="私信"
        :mask-closable="false"
        :bordered="false"
        :style="{
            width: 'min(92vw, 420px)',
        }"
    >
        <div class="whisper-wrap">
            <div class="whisper-hero">
                <n-avatar round :size="44" :src="user.avatar || DEFAULT_USER_AVATAR" />
                <div class="whisper-hero-copy">
                    <span>发送私信给</span>
                    <strong>{{ user.nickname || user.username }}</strong>
                    <em>@{{ user.username }}</em>
                </div>
            </div>
            <n-alert :show-icon="false" type="info" class="whisper-tip">
                保持消息简洁清晰。支持换行，按 `Ctrl/Command + Enter` 可直接发送。
            </n-alert>
            <div class="whisper-line">
                <n-input
                    type="textarea"
                    placeholder="输入你想说的话…"
                    :autosize="{
                        minRows: 5,
                        maxRows: 10,
                    }"
                    v-model:value="content"
                    maxlength="200"
                    show-count
                    @keydown="handleKeydown"
                />
            </div>
            <div class="whisper-meta">
                <span>将通过消息系统发送给对方</span>
                <strong>{{ content.trim().length }}/200</strong>
            </div>
            <div class="whisper-line send-wrap">
                <n-button quaternary @click="closeModal">取消</n-button>
                <n-button
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
import { sendLegacyWhisper } from '@/utils/messageTransport';
import { ref } from 'vue';
import { DEFAULT_USER_AVATAR } from '@/utils/defaults';

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

const emit = defineEmits<(e: 'success') => void>();
const closeModal = () => {
  if (!loading.value) {
    content.value = '';
  }
  emit('success');
};
const handleKeydown = (event: KeyboardEvent) => {
  if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
    event.preventDefault();
    sendWhisper();
  }
};
const sendWhisper = () => {
  if (!content.value.trim()) {
    window.$message.warning('请输入私信内容');
    return;
  }
  loading.value = true;
  sendLegacyWhisper({
    user_id: props.user.id,
    content: content.value,
  })
    .then(() => {
      window.$message.success('发送成功');
      loading.value = false;
      content.value = '';

      closeModal();
    })
    .catch(() => {
      loading.value = false;
      window.$message.error('私信发送失败');
    });
};
</script>

<style lang="less" scoped>
.whisper-wrap {
    display: flex;
    flex-direction: column;
    gap: 14px;

    .whisper-line {
        &.send-wrap {
            display: flex;
            justify-content: flex-end;
            gap: 10px;

            .n-button:last-child {
                min-width: 124px;
            }
        }
    }
}

.whisper-hero {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px;
    border-radius: 18px;
    background:
      radial-gradient(circle at top right, color-mix(in srgb, var(--accent-soft) 75%, transparent), transparent 48%),
      color-mix(in srgb, var(--panel-bg) 88%, transparent);
    border: 1px solid color-mix(in srgb, var(--panel-border) 78%, transparent);
}

.whisper-hero-copy {
    display: grid;
    gap: 2px;
    min-width: 0;

    span,
    em {
        font-size: 12px;
        line-height: 1.5;
        opacity: 0.7;
        font-style: normal;
    }

    strong {
        font-size: 16px;
        line-height: 1.4;
    }
}

.whisper-tip {
    :deep(.n-alert-body) {
        line-height: 1.6;
    }
}

.whisper-meta {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    font-size: 12px;
    opacity: 0.7;
}

@media (max-width: 768px) {
    .whisper-wrap .whisper-line.send-wrap {
        .n-button {
            flex: 1 1 0;
            min-width: 0;
        }
    }
}
</style>
