<template>
  <div>
    <div class="compose-wrap" v-if="userInfo.id > 0">
      <div class="compose-line">
        <div class="compose-user">
          <n-avatar round :size="30" :src="userInfo.avatar || DEFAULT_USER_AVATAR" />
        </div>
        <n-mention
          type="textarea"
          size="large"
          autosize
          :bordered="false"
          :options="optionsRef"
          :prefix="['@']"
          :loading="loading"
          :value="content"
          :disabled="props.lock === 1"
          @update:value="changeContent"
          @search="handleSearch"
          @focus="focusComment"
          :placeholder="
            props.lock === 1 ? '该动态已被锁定，回复功能已关闭' : '快来评论两句吧...'
          "
        />
      </div>

      <transition name="comment-expand">
        <div v-if="showBtn" class="compose-tools">
          <div class="compose-options">
            <div class="attachment">
              <input
                ref="imageInputRef"
                class="hidden-input"
                type="file"
                accept="image/*"
                multiple
                @change="handleFilePick($event)"
              />
              <button
                type="button"
                class="tool-btn"
                :disabled="imageContents.length >= 9"
                @click="imageInputRef?.click()"
              >
                <span>🖼️</span>
                图片
              </button>

              <n-tooltip trigger="hover" placement="bottom">
                <template #trigger>
                  <n-progress
                    class="text-statistic"
                    type="circle"
                    :show-indicator="false"
                    status="success"
                    :stroke-width="10"
                    :percentage="(content.length / defaultCommentMaxLength) * 100"
                  />
                </template>
                {{ content.length }} / {{ defaultCommentMaxLength }}
              </n-tooltip>
            </div>

            <div class="submit-wrap">
              <n-button quaternary round type="tertiary" class="cancel-btn" size="small" @click="cancelComment">
                取消
              </n-button>
              <n-button :loading="submitting" @click="submitPost" type="primary" secondary size="small" round>
                发布
              </n-button>
            </div>
          </div>

          <div v-if="uploading.length > 0" class="uploading-list">
            <div v-for="item in uploading" :key="item.id" class="uploading-item">
              <span>{{ item.name }}</span>
              <n-spin size="small" />
            </div>
          </div>

          <div v-if="imageContents.length > 0" class="asset-grid">
            <div v-for="item in imageContents" :key="item.id" class="asset-card">
              <img :src="item.content" alt="" />
              <button type="button" class="asset-remove" @click="removeUpload(item.id)">×</button>
            </div>
          </div>
        </div>
      </transition>
    </div>

    <div class="compose-wrap" v-else>
      <div class="login-wrap">
        <span class="login-banner">登录后，精彩更多</span>
      </div>
      <div v-if="!allowUserRegister" class="login-only-wrap">
        <n-button strong secondary round type="primary" @click="goAuth('signin')">登录</n-button>
      </div>
      <div v-else class="login-wrap">
        <n-button strong secondary round type="primary" @click="goAuth('signin')">登录</n-button>
        <n-button strong secondary round type="primary" ghost @click="goAuth('signup')">注册</n-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { useRouter } from 'vue-router';
import { debounce } from 'lodash';
import type { MentionOption } from 'naive-ui';
import { storeToRefs } from 'pinia';
import { createComment } from '@/api/post';
import { parsePostTag } from '@/utils/content';
import { Api, request } from '@/utils/request';
import { TOKEN_KEY, useStoreUser } from '@/store/user';
import { buildApiUrl } from '@/utils/api';
import { DEFAULT_USER_AVATAR } from '@/utils/defaults';
import { goToAuth, type AuthMode } from '@/utils/authRoute';

interface UploadingItem {
  id: string;
  name: string;
}

const emit = defineEmits<{
  (e: 'post-success'): void;
}>();

const props = withDefaults(
  defineProps<{
    lock: number;
    postId: number;
  }>(),
  {
    lock: 0,
    postId: 0,
  },
);

const router = useRouter();
const storeUser = useStoreUser();
const { userInfo } = storeToRefs(storeUser);

const optionsRef = ref<MentionOption[]>([]);
const showBtn = ref(false);
const loading = ref(false);
const submitting = ref(false);
const content = ref('');
const imageContents = ref<Item.CommentItemProps[]>([]);
const uploading = ref<UploadingItem[]>([]);
const imageInputRef = ref<HTMLInputElement | null>(null);

const allowUserRegister = ref(import.meta.env.VITE_ALLOW_USER_REGISTER.toLowerCase() === 'true');
const defaultCommentMaxLength = Number(import.meta.env.VITE_DEFAULT_COMMENT_MAX_LENGTH);
const uploadGateway = buildApiUrl('/v1/attachment');

const uploadToken = computed(() => `Bearer ${localStorage.getItem(TOKEN_KEY) || ''}`);

const goAuth = (mode: AuthMode) => {
  goToAuth(router, mode, router.currentRoute.value.fullPath);
};

const loadSuggestionUsers = debounce((k) => {
  Api.v1.suggest.get.users({
    k,
  })
    .then((res) => {
      optionsRef.value = res.suggest.map((item) => ({
        label: item,
        value: item,
      }));
      loading.value = false;
    })
    .catch(() => {
      loading.value = false;
    });
}, 200);

const handleSearch = (k: string, prefix: string) => {
  if (loading.value) {
    return;
  }
  loading.value = true;
  if (prefix === '@') {
    loadSuggestionUsers(k);
  }
};

const changeContent = (value: string) => {
  content.value =
    value.length > defaultCommentMaxLength
      ? value.substring(0, defaultCommentMaxLength)
      : value;
};

const focusComment = () => {
  showBtn.value = true;
};

const cancelComment = () => {
  showBtn.value = false;
  content.value = '';
  imageContents.value = [];
  uploading.value = [];
};

const uploadImage = async (file: File) => {
  if (!['image/webp', 'image/png', 'image/jpg', 'image/jpeg', 'image/gif'].includes(file.type)) {
    window.$message.warning('图片仅允许 webp/png/jpg/gif 格式');
    return;
  }
  if (file.size > 10 * 1024 * 1024) {
    window.$message.warning('图片大小不能超过10MB');
    return;
  }

  const id = `${Date.now()}-${file.name}`;
  uploading.value.push({ id, name: file.name });

  try {
    const formData = new FormData();
    formData.append('type', 'public/image');
    formData.append('file', file);

    const res = await request<FormData, { content: string }>({
      method: 'post',
      url: uploadGateway,
      data: formData,
      headers: {
        Authorization: uploadToken.value,
        'Content-Type': 'multipart/form-data',
      },
    });

    imageContents.value.push({
      id: Date.now(),
      comment_id: 0,
      user_id: userInfo.value.id,
      type: 3,
      content: res.content,
      sort: imageContents.value.length + 101,
      created_on: Date.now(),
    });
  } catch (_error) {
    window.$message.error('评论图片上传失败');
  } finally {
    uploading.value = uploading.value.filter((item) => item.id !== id);
  }
};

const handleFilePick = async (event: Event) => {
  const input = event.target as HTMLInputElement;
  const files = Array.from(input.files || []);
  for (const file of files) {
    if (imageContents.value.length >= 9) {
      window.$message.warning('最多上传 9 张图片');
      break;
    }
    await uploadImage(file);
  }
  input.value = '';
};

const removeUpload = (assetId: number) => {
  imageContents.value = imageContents.value.filter((item) => item.id !== assetId);
};

const submitPost = () => {
  if (content.value.trim().length === 0) {
    window.$message.warning('请输入内容哦');
    return;
  }

  const { users } = parsePostTag(content.value);
  const contents: Partial<Item.CommentItemProps>[] = [];
  let sort = 100;

  contents.push({
    content: content.value,
    type: 2,
    sort,
  });

  imageContents.value.forEach((img) => {
    sort++;
    contents.push({
      content: img.content,
      type: 3,
      sort,
    });
  });

  submitting.value = true;
  createComment({
    contents,
    post_id: props.postId,
    users: Array.from(new Set(users)),
  })
    .then(() => {
      window.$message.success('发布成功');
      emit('post-success');
      cancelComment();
    })
    .finally(() => {
      submitting.value = false;
    });
};
</script>

<style lang="less" scoped>
.compose-wrap {
  --comment-soft-bg: var(--accent-soft);
  --comment-soft-bg-hover: var(--accent-soft-hover);
  --comment-soft-bg-muted: var(--accent-soft-muted);
  --comment-accent: var(--accent-primary);
  --comment-panel-bg: transparent;
  --comment-remove-bg: var(--overlay-strong);
  --comment-remove-text: #fff;
  width: 100%;
  padding: 16px;
  box-sizing: border-box;
  background-color: var(--comment-panel-bg);

  .compose-line {
    display: flex;
    flex-direction: row;

    .compose-user {
      width: 42px;
      height: 42px;
      display: flex;
      align-items: center;
    }
  }

  .compose-tools {
    margin-top: 8px;
    padding-left: 42px;
  }

  .compose-options {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }

  .submit-wrap {
    display: flex;
    align-items: center;

    .cancel-btn {
      margin-right: 8px;
    }
  }

  .login-only-wrap,
  .login-wrap {
    display: flex;
    justify-content: center;
    width: 100%;
  }

  .login-only-wrap button {
    width: 50%;
  }

  .login-banner {
    margin-bottom: 12px;
    opacity: 0.8;
  }

  .login-wrap button {
    margin: 0 4px;
  }
}

.attachment {
  display: flex;
  align-items: center;
  gap: 10px;
}

.tool-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  height: 36px;
  padding: 0 14px;
  border: 0;
  border-radius: 999px;
  background: var(--comment-soft-bg);
  color: var(--comment-accent);
  cursor: pointer;
  transition: transform 0.18s ease, background-color 0.18s ease;

  &:hover {
    transform: translateY(-1px);
    background: var(--comment-soft-bg-hover);
  }

  &:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }
}

.hidden-input {
  display: none;
}

.text-statistic {
  width: 18px;
  height: 18px;
  transform: rotate(180deg);
}

.uploading-list {
  margin-top: 12px;
  display: grid;
  gap: 8px;
}

.uploading-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 14px;
  background: var(--comment-soft-bg-muted);
}

.asset-grid {
  margin-top: 12px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(88px, 1fr));
  gap: 10px;
}

.asset-card {
  position: relative;
  overflow: hidden;
  aspect-ratio: 1 / 1;
  border-radius: 16px;
  animation: card-enter 0.22s ease;

  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
}

.asset-remove {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 24px;
  height: 24px;
  border: 0;
  border-radius: 999px;
  background: var(--comment-remove-bg);
  color: var(--comment-remove-text);
  cursor: pointer;
}

.comment-expand-enter-active,
.comment-expand-leave-active {
  transition: all 0.22s ease;
}

.comment-expand-enter-from,
.comment-expand-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

@keyframes card-enter {
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
