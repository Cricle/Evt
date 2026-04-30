<template>
  <div class="compose-editor" :class="{ 'page-mode': pageMode }">
    <div v-if="userInfo.id > 0" class="editor-shell">
      <div class="editor-header">
        <n-avatar round :size="42" :src="userInfo.avatar || DEFAULT_USER_AVATAR" />
        <div class="editor-title">
          <strong>{{ userInfo.nickname || userInfo.username }}</strong>
          <span>发布新动态</span>
        </div>
      </div>

      <div class="editor-card">
        <ckeditor :editor="ClassicEditor" v-model="editorHtml" :config="editorConfig" />

        <div class="editor-toolbar">
          <div class="editor-tools-left">
            <input ref="imageInputRef" class="hidden-input" type="file" accept="image/*" multiple @change="handleFilePick('public/image', $event)" />
            <input ref="videoInputRef" class="hidden-input" type="file" accept="video/mp4,video/quicktime" @change="handleFilePick('public/video', $event)" />
            <input ref="attachmentInputRef" class="hidden-input" type="file" accept=".zip,application/zip,application/x-zip-compressed" multiple @change="handleFilePick('attachment', $event)" />

            <button type="button" class="tool-btn" @click="imageInputRef?.click()">
              <span>🖼️</span>
              图片
            </button>
            <button
              v-if="profile.allowTweetVideo"
              type="button"
              class="tool-btn"
              @click="videoInputRef?.click()"
            >
              <span>🎬</span>
              视频
            </button>
            <button
              v-if="profile.allowTweetAttachment"
              type="button"
              class="tool-btn"
              @click="attachmentInputRef?.click()"
            >
              <span>📦</span>
              附件
            </button>
            <button type="button" class="tool-btn" @click="showLinkSet = !showLinkSet">
              <span>🔗</span>
              链接
            </button>
            <button
              v-if="allowTweetVisibility"
              type="button"
              class="tool-btn"
              @click="showVisibilitySet = !showVisibilitySet"
            >
              <span>👁️</span>
              可见性
            </button>
          </div>

          <div class="editor-tools-right">
            <n-tooltip trigger="hover" placement="bottom">
              <template #trigger>
                <n-progress
                  class="text-statistic"
                  type="circle"
                  :show-indicator="false"
                  status="success"
                  :stroke-width="10"
                  :percentage="(plainText.length / profile.defaultTweetMaxLength) * 100"
                />
              </template>
              已输入 {{ plainText.length }} 字
            </n-tooltip>
            <n-button type="primary" secondary round :loading="submitting" @click="submitPost">
              发布
            </n-button>
          </div>
        </div>

        <transition name="editor-expand">
          <div v-if="showLinkSet" class="editor-extend">
            <n-dynamic-input v-model:value="links" placeholder="请输入以 http(s):// 开头的链接" :min="0" :max="3">
              <template #create-button-default> 添加链接 </template>
            </n-dynamic-input>
          </div>
        </transition>

        <transition name="editor-expand">
          <div v-if="showVisibilitySet" class="editor-extend">
            <n-radio-group v-model:value="visitType" name="visibility">
              <n-space>
                <n-radio v-for="visit in visibilities" :key="visit.value" :value="visit.value" :label="visit.label" />
              </n-space>
            </n-radio-group>
          </div>
        </transition>

        <div v-if="uploading.length > 0" class="uploading-list">
          <div v-for="item in uploading" :key="item.id" class="uploading-item">
            <span>{{ item.name }}</span>
            <n-spin size="small" />
          </div>
        </div>

        <div v-if="imageContents.length > 0" class="asset-section">
          <div class="asset-label">图片</div>
          <div class="asset-grid">
            <div v-for="item in imageContents" :key="item.id" class="asset-card">
              <img :src="item.content" alt="" />
              <button type="button" class="asset-remove" @click="removeAsset('image', item.id)">×</button>
            </div>
          </div>
        </div>

        <div v-if="videoContents.length > 0" class="asset-section">
          <div class="asset-label">视频</div>
          <div class="asset-stack">
            <div v-for="item in videoContents" :key="item.id" class="asset-line">
              <span class="emoji">🎬</span>
              <a :href="item.content" target="_blank" rel="noreferrer">{{ item.content }}</a>
              <button type="button" class="asset-remove inline" @click="removeAsset('video', item.id)">×</button>
            </div>
          </div>
        </div>

        <div v-if="attachmentContents.length > 0" class="asset-section">
          <div class="asset-label">附件</div>
          <div class="asset-stack">
            <div v-for="item in attachmentContents" :key="item.id" class="asset-line">
              <span class="emoji">📦</span>
              <a :href="item.content" target="_blank" rel="noreferrer">{{ item.content }}</a>
              <button type="button" class="asset-remove inline" @click="removeAsset('attachment', item.id)">×</button>
            </div>
          </div>
          <div v-if="profile.allowTweetAttachmentPrice" class="attachment-price-wrap">
            <n-input-number
              v-model:value="attachmentPrice"
              :min="0"
              :max="100000"
              placeholder="请输入附件价格，0 为免费附件"
            >
              <template #prefix>
                <span>附件价格￥</span>
              </template>
            </n-input-number>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="compose-login-card">
      <div class="compose-login-title">登录后，精彩更多</div>
      <div class="compose-login-actions">
        <n-button strong secondary round type="primary" @click="goAuth('signin')">登录</n-button>
        <n-button v-if="profile.allowUserRegister" strong secondary round type="info" @click="goAuth('signup')">注册</n-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';
import { Ckeditor } from '@ckeditor/ckeditor5-vue';
import ClassicEditor from '@ckeditor/ckeditor5-build-classic';
import DOMPurify from 'dompurify';
import { storeToRefs } from 'pinia';
import { createPost } from '@/api/post';
import { useStoreProfile } from '@/store/profile';
import { TOKEN_KEY, useStoreUser } from '@/store/user';
import { VisibilityEnum, PostItemTypeEnum } from '@/utils/IEnum';
import { parsePostTag } from '@/utils/content';
import { isZipFile } from '@/utils/isZipFile';
import { buildApiUrl } from '@/utils/api';
import { request } from '@/utils/request';
import { goToAuth, type AuthMode } from '@/utils/authRoute';
import { DEFAULT_USER_AVATAR } from '@/utils/defaults';

type UploadKind = 'public/image' | 'public/video' | 'attachment';
type AssetKind = 'image' | 'video' | 'attachment';

interface UploadingItem {
  id: string;
  name: string;
}

const emit = defineEmits<{
  (e: 'post-success', post: Item.PostProps): void;
}>();

const props = withDefaults(
  defineProps<{
    pageMode?: boolean;
  }>(),
  {
    pageMode: false,
  },
);

const router = useRouter();
const storeUser = useStoreUser();
const storeProfile = useStoreProfile();
const { userInfo } = storeToRefs(storeUser);
const { profile } = storeToRefs(storeProfile);

const editorHtml = ref('');
const submitting = ref(false);
const showLinkSet = ref(false);
const showVisibilitySet = ref(false);
const links = ref<string[]>([]);
const imageContents = ref<Item.PostItemProps[]>([]);
const videoContents = ref<Item.PostItemProps[]>([]);
const attachmentContents = ref<Item.PostItemProps[]>([]);
const uploading = ref<UploadingItem[]>([]);
const attachmentPrice = ref(0);
const visitType = ref<VisibilityEnum>(VisibilityEnum.PUBLIC);
const defaultVisitType = ref<VisibilityEnum>(VisibilityEnum.PUBLIC);

const imageInputRef = ref<HTMLInputElement | null>(null);
const videoInputRef = ref<HTMLInputElement | null>(null);
const attachmentInputRef = ref<HTMLInputElement | null>(null);

const allowTweetVisibility = import.meta.env.VITE_ALLOW_TWEET_VISIBILITY.toLowerCase() === 'true';
const uploadGateway = buildApiUrl('/v1/attachment');

const editorConfig = {
  licenseKey: 'GPL',
  toolbar: [
    'heading',
    '|',
    'bold',
    'italic',
    'link',
    'bulletedList',
    'numberedList',
    'blockQuote',
    '|',
    'undo',
    'redo',
  ],
  placeholder: '说说您的新鲜事...',
};

const plainText = computed(() => {
  const sanitized = DOMPurify.sanitize(editorHtml.value, {
    ALLOWED_TAGS: [],
    ALLOWED_ATTR: [],
  });
  return sanitized.replace(/\s+/g, ' ').trim();
});

const visibilities = computed(() => {
  const result = [
    { value: VisibilityEnum.PUBLIC, label: '公开' },
    { value: VisibilityEnum.PRIVATE, label: '私密' },
    { value: VisibilityEnum.Following, label: '关注可见' },
  ];
  if (profile.value.useFriendship) {
    result.push({ value: VisibilityEnum.FRIEND, label: '好友可见' });
  }
  return result;
});

const goAuth = (mode: AuthMode) => {
  goToAuth(router, mode, router.currentRoute.value.fullPath);
};

const sanitizeHtml = (html: string) => {
  return DOMPurify.sanitize(html, {
    ALLOWED_TAGS: ['p', 'br', 'strong', 'em', 'ul', 'ol', 'li', 'blockquote', 'a'],
    ALLOWED_ATTR: ['href', 'target', 'rel'],
  });
};

const nextAssetId = () => Date.now() + Math.floor(Math.random() * 10_000);

const validateUpload = async (kind: UploadKind, file: File) => {
  if (kind === 'public/image') {
    if (!['image/webp', 'image/png', 'image/jpg', 'image/jpeg', 'image/gif'].includes(file.type)) {
      return '图片仅允许 webp/png/jpg/gif 格式';
    }
    if (file.size > 10 * 1024 * 1024) {
      return '图片大小不能超过10MB';
    }
    return '';
  }

  if (kind === 'public/video') {
    if (!['video/mp4', 'video/quicktime'].includes(file.type)) {
      return '视频仅允许 mp4/mov 格式';
    }
    if (file.size > 100 * 1024 * 1024) {
      return '视频大小不能超过100MB';
    }
    return '';
  }

  if (!(await isZipFile(file))) {
    return '附件仅允许 zip 格式';
  }
  if (file.size > 100 * 1024 * 1024) {
    return '附件大小不能超过100MB';
  }
  return '';
};

const uploadSingleFile = async (kind: UploadKind, file: File) => {
  const errorMessage = await validateUpload(kind, file);
  if (errorMessage) {
    window.$message.warning(errorMessage);
    return;
  }

  const id = `${Date.now()}-${file.name}`;
  uploading.value.push({
    id,
    name: file.name,
  });

  try {
    const form = new FormData();
    form.append('type', kind);
    form.append('file', file);

    const res = await request<FormData, {
      user_id: number;
      file_size: number;
      img_width: number;
      img_height: number;
      type: number;
      content: string;
    }>({
      method: 'post',
      url: uploadGateway,
      data: form,
      headers: {
        Authorization: `Bearer ${localStorage.getItem(TOKEN_KEY) || ''}`,
        'Content-Type': 'multipart/form-data',
      },
    });

    const asset = {
      id: nextAssetId(),
      content: res.content,
      type:
        kind === 'public/image'
          ? PostItemTypeEnum.IMAGEURL
          : kind === 'public/video'
            ? PostItemTypeEnum.VIDEOURL
            : PostItemTypeEnum.ATTACHMENT,
      sort: 0,
      post_id: 0,
      created_on: Date.now(),
    } as Item.PostItemProps;

    if (kind === 'public/image') {
      imageContents.value.push(asset);
    } else if (kind === 'public/video') {
      videoContents.value.push(asset);
    } else {
      attachmentContents.value.push(asset);
    }
  } finally {
    uploading.value = uploading.value.filter((item) => item.id !== id);
  }
};

const handleFilePick = async (kind: UploadKind, event: Event) => {
  const input = event.target as HTMLInputElement;
  const files = Array.from(input.files || []);
  for (const file of files) {
    await uploadSingleFile(kind, file);
  }
  input.value = '';
};

const removeAsset = (kind: AssetKind, assetId: number) => {
  if (kind === 'image') {
    imageContents.value = imageContents.value.filter((item) => item.id !== assetId);
  } else if (kind === 'video') {
    videoContents.value = videoContents.value.filter((item) => item.id !== assetId);
  } else {
    attachmentContents.value = attachmentContents.value.filter((item) => item.id !== assetId);
  }
};

const resetEditor = () => {
  editorHtml.value = '';
  links.value = [];
  imageContents.value = [];
  videoContents.value = [];
  attachmentContents.value = [];
  attachmentPrice.value = 0;
  showLinkSet.value = false;
  showVisibilitySet.value = false;
  visitType.value = defaultVisitType.value;
};

const submitPost = async () => {
  const normalizedText = plainText.value;
  if (!normalizedText) {
    window.$message.warning('请输入内容哦');
    return;
  }
  if (normalizedText.length > profile.value.defaultTweetMaxLength) {
    window.$message.warning(`内容不能超过 ${profile.value.defaultTweetMaxLength} 字`);
    return;
  }

  const html = sanitizeHtml(editorHtml.value);
  const { tags, users } = parsePostTag(`${normalizedText} `);
  const contents: Partial<Item.PostItemProps>[] = [];
  let sort = 100;

  contents.push({
    content: html,
    type: PostItemTypeEnum.TEXT,
    sort,
  });

  for (const image of imageContents.value) {
    sort++;
    contents.push({
      content: image.content,
      type: PostItemTypeEnum.IMAGEURL,
      sort,
    });
  }
  for (const video of videoContents.value) {
    sort++;
    contents.push({
      content: video.content,
      type: PostItemTypeEnum.VIDEOURL,
      sort,
    });
  }
  for (const attachment of attachmentContents.value) {
    sort++;
    contents.push({
      content: attachment.content,
      type: PostItemTypeEnum.ATTACHMENT,
      sort,
    });
  }
  for (const link of links.value.filter(Boolean)) {
    sort++;
    contents.push({
      content: link,
      type: PostItemTypeEnum.LINKURL,
      sort,
    });
  }

  submitting.value = true;
  try {
    const post = await createPost({
      contents,
      tags: Array.from(new Set(tags)),
      users: Array.from(new Set(users)),
      attachment_price: attachmentPrice.value * 100,
      visibility: visitType.value,
    });
    window.$message.success('发布成功');
    resetEditor();
    emit('post-success', post);
  } finally {
    submitting.value = false;
  }
};

onMounted(() => {
  const defaultVisibility = profile.value.defaultTweetVisibility;
  if (profile.value.useFriendship && defaultVisibility === 'friend') {
    defaultVisitType.value = VisibilityEnum.FRIEND;
  } else if (defaultVisibility === 'following') {
    defaultVisitType.value = VisibilityEnum.Following;
  } else if (defaultVisibility === 'public') {
    defaultVisitType.value = VisibilityEnum.PUBLIC;
  } else {
    defaultVisitType.value = VisibilityEnum.PRIVATE;
  }
  visitType.value = defaultVisitType.value;
});
</script>

<style scoped lang="less">
.compose-editor {
  width: 100%;
}

.editor-shell {
  display: grid;
  gap: 14px;
}

.editor-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.editor-title {
  display: grid;
  gap: 2px;

  span {
    font-size: 13px;
    opacity: 0.72;
  }
}

.editor-card {
  padding: 18px;
  border: 1px solid rgba(18, 75, 51, 0.08);
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.9);
  box-shadow: 0 22px 48px rgba(38, 80, 60, 0.08);
}

:deep(.ck-editor__editable_inline) {
  min-height: 240px;
  max-height: 420px;
  border-radius: 0 0 16px 16px;
}

:deep(.ck-toolbar) {
  border-radius: 16px 16px 0 0;
}

.editor-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  margin-top: 16px;
  flex-wrap: wrap;
}

.editor-tools-left,
.editor-tools-right {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.tool-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  height: 38px;
  padding: 0 14px;
  border: 0;
  border-radius: 999px;
  background: rgba(16, 136, 91, 0.08);
  color: #12724d;
  cursor: pointer;
  transition: transform 0.2s ease, background-color 0.2s ease;

  &:hover {
    transform: translateY(-1px);
    background: rgba(16, 136, 91, 0.14);
  }
}

.hidden-input {
  display: none;
}

.text-statistic {
  width: 22px;
  height: 22px;
  transform: rotate(180deg);
}

.editor-extend {
  margin-top: 16px;
}

.uploading-list,
.asset-section {
  margin-top: 16px;
}

.uploading-item,
.asset-line {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 14px;
  background: rgba(16, 136, 91, 0.06);
}

.asset-label {
  margin-bottom: 10px;
  font-size: 13px;
  font-weight: 700;
  opacity: 0.78;
}

.asset-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 12px;
}

.asset-card {
  position: relative;
  aspect-ratio: 1 / 1;
  overflow: hidden;
  border-radius: 18px;
  background: rgba(0, 0, 0, 0.05);

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
  width: 28px;
  height: 28px;
  border: 0;
  border-radius: 999px;
  background: rgba(0, 0, 0, 0.58);
  color: #fff;
  font-size: 18px;
  cursor: pointer;

  &.inline {
    position: static;
    background: rgba(0, 0, 0, 0.08);
    color: #333;
  }
}

.asset-stack {
  display: grid;
  gap: 10px;
}

.emoji {
  font-size: 18px;
}

.attachment-price-wrap {
  margin-top: 12px;
}

.compose-login-card {
  padding: 24px;
  border-radius: 22px;
  background: rgba(255, 255, 255, 0.84);
  border: 1px solid rgba(18, 75, 51, 0.08);
}

.compose-login-title {
  margin-bottom: 14px;
  font-size: 16px;
  font-weight: 700;
}

.compose-login-actions {
  display: flex;
  gap: 12px;
}

.editor-expand-enter-active,
.editor-expand-leave-active {
  transition: all 0.22s ease;
}

.editor-expand-enter-from,
.editor-expand-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

@media screen and (max-width: 821px) {
  .editor-card {
    padding: 14px;
    border-radius: 20px;
  }

  :deep(.ck-editor__editable_inline) {
    min-height: 180px;
  }
}
</style>
