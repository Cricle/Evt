<template>
  <div class="compose-editor" :class="{ 'page-mode': pageMode }">
    <div v-if="userInfo.id > 0" class="editor-shell">
      <div class="editor-header">
        <n-avatar round :size="42" :src="userInfo.avatar || DEFAULT_USER_AVATAR" />
        <div class="editor-title">
          <strong>{{ userInfo.nickname || userInfo.username }}</strong>
          <span>{{ modeLabel }}</span>
        </div>
      </div>

      <div v-if="isEventMode" class="event-intro-card">
        <div class="event-intro-badge">事件</div>
        <div class="event-intro-content">
          <strong>先发布事件主题，再逐步追加时间节点</strong>
          <span>适合记录活动进展、项目里程碑、问题处理过程和持续更新的公共信息。</span>
        </div>
      </div>

      <div class="editor-card">
        <ckeditor :editor="ClassicEditor" v-model="editorHtml" :config="editorConfig" />

        <div class="editor-toolbar">
          <div class="editor-tools-left">
            <n-button
              v-if="allowTweetVisibility"
              type="success"
              secondary
              strong
              round
              size="small"
              class="tool-btn"
              @click="togglePanel('visibility')"
            >
              <span>👁️</span>
              可见性
            </n-button>
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
            <n-button type="success" secondary round :loading="submitting" @click="submitPost">
              {{ submitLabel }}
            </n-button>
          </div>
        </div>

        <transition name="editor-expand">
          <div v-if="editorPanel === 'visibility'" class="editor-extend">
            <n-radio-group v-model:value="visitType" name="visibility">
              <n-space>
                <n-radio v-for="visit in visibilities" :key="visit.value" :value="visit.value" :label="visit.label" />
              </n-space>
            </n-radio-group>
          </div>
        </transition>

        <div v-if="uploading.length > 0" class="uploading-list">
          <div v-for="item in uploading" :key="item" class="uploading-item">
            <span>{{ item }}</span>
            <n-spin size="small" />
          </div>
        </div>

        <div v-if="assetPills.length > 0" class="asset-pill-list">
          <button
            v-for="asset in assetPills"
            :key="asset.id"
            type="button"
            class="asset-pill"
            @click="removeAsset(asset)"
          >
            <span>{{ asset.icon }} {{ asset.name }}</span>
            <span class="asset-pill-remove">×</span>
          </button>
        </div>

        <div v-if="attachmentPriceVisible" class="attachment-price-wrap">
          <n-input-number v-model:value="attachmentPrice" :min="0" :max="100000" placeholder="请输入附件价格，0 为免费附件">
            <template #prefix>
              <span>附件价格￥</span>
            </template>
          </n-input-number>
        </div>
      </div>
    </div>

    <div v-else class="compose-login-card">
      <div class="compose-login-title">登录后，精彩更多</div>
      <div class="compose-login-actions">
        <n-button strong secondary round type="success" @click="goAuth('signin')">登录</n-button>
        <n-button v-if="profile.allowUserRegister" strong round type="success" ghost @click="goAuth('signup')">
          注册
        </n-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import { Ckeditor } from '@ckeditor/ckeditor5-vue';
import 'ckeditor5/ckeditor5.css';
import { storeToRefs } from 'pinia';
import {
  AutoLink,
  BlockQuote,
  Bold,
  ClassicEditor,
  Essentials,
  Heading,
  Image,
  ImageToolbar,
  ImageUpload,
  Italic,
  Link,
  List,
  MediaEmbed,
  Paragraph,
  Undo,
} from 'ckeditor5';
import { createPost } from '@/api/post';
import { useStoreProfile } from '@/store/profile';
import { useStoreUser } from '@/store/user';
import { VisibilityEnum } from '@/utils/IEnum';
import { parsePostTag } from '@/utils/content';
import { goToAuth, type AuthMode } from '@/utils/authRoute';
import { DEFAULT_USER_AVATAR } from '@/utils/defaults';
import EvtUploadPlugin, {
  EVT_UPLOAD_PLUGIN_OPTIONS,
  type UploadedAsset,
  uploadFile,
} from '@/components/ckeditor-upload-plugin';
import { EVENT_POST_TAG } from '@/utils/postKind';
import { consumePendingAttachmentSelection } from '@/utils/composeDraft';
import {
  buildComposePostContents,
  hasComposeContent,
  syncImageAssetsWithEditor,
  type ComposeAsset,
} from '@/components/compose-editor-content';

const emit = defineEmits<{
  (e: 'post-success', post: Item.PostProps): void;
}>();

const props = withDefaults(
  defineProps<{
    pageMode?: boolean;
    mode?: 'post' | 'event';
    quickMode?: '' | 'media';
  }>(),
  {
    pageMode: false,
    mode: 'post',
    quickMode: '',
  },
);

const router = useRouter();
const storeUser = useStoreUser();
const storeProfile = useStoreProfile();
const { userInfo } = storeToRefs(storeUser);
const { profile, currentSpaceSlug } = storeToRefs(storeProfile);

const editorHtml = ref('');
const submitting = ref(false);
const editorPanel = ref<'link' | 'visibility' | ''>('');
const imageContents = ref<ComposeAsset[]>([]);
const videoContents = ref<ComposeAsset[]>([]);
const attachmentContents = ref<ComposeAsset[]>([]);
const links = ref<ComposeAsset[]>([]);
const attachmentPrice = ref(0);
const visitType = ref<VisibilityEnum>(VisibilityEnum.PUBLIC);
const defaultVisitType = ref<VisibilityEnum>(VisibilityEnum.PUBLIC);
const uploading = ref<string[]>([]);

const allowTweetVisibility = import.meta.env.VITE_ALLOW_TWEET_VISIBILITY.toLowerCase() === 'true';
const isEventMode = computed(() => props.mode === 'event');
const modeLabel = computed(() => (isEventMode.value ? '创建事件主题，成员随后可继续追加节点' : '发布到当前广场'));
const submitLabel = computed(() => (isEventMode.value ? '创建事件' : '发布'));
const isQuickMediaMode = computed(() => props.quickMode === 'media');

const editorConfig = {
  licenseKey: 'GPL',
  plugins: [
    Essentials,
    Paragraph,
    Heading,
    Bold,
    Italic,
    BlockQuote,
    Link,
    AutoLink,
    List,
    Image,
    ImageUpload,
    ImageToolbar,
    MediaEmbed,
    Undo,
    EvtUploadPlugin,
  ],
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
    'imageUpload',
    'evtVideoUpload',
    'evtAttachmentUpload',
    'evtLink',
    'mediaEmbed',
    'undo',
    'redo',
  ],
  placeholder: isEventMode.value ? '输入事件标题、背景或当前进展...' : '说说您的新鲜事...',
  [EVT_UPLOAD_PLUGIN_OPTIONS]: {
    onStart: (fileName: string) => {
      if (!uploading.value.includes(fileName)) {
        uploading.value.push(fileName);
      }
    },
    onUploaded: (asset: UploadedAsset) => {
      handleUploadedAsset(asset);
    },
    onFinish: (fileName: string) => {
      uploading.value = uploading.value.filter((item) => item !== fileName);
    },
    onError: (error: unknown) => {
      window.$message.error('上传失败');
    },
    onLinkCreate: (url: string) => {
      addLink(url);
    },
  },
};

const plainText = computed(() => {
  return buildComposePostContents({
    textHtml: editorHtml.value,
    images: imageContents.value,
    videos: videoContents.value,
    attachments: attachmentContents.value,
    links: links.value,
    attachmentPrice: attachmentPrice.value * 100,
  }).plainText;
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

const attachmentPriceVisible = computed(() => profile.value.allowTweetAttachmentPrice);
const assetPills = computed(() => {
  return [
    ...videoContents.value.map((asset) => ({ ...asset, icon: '🎬' })),
    ...attachmentContents.value.map((asset) => ({ ...asset, icon: '📎' })),
    ...links.value.map((asset) => ({ ...asset, icon: '🔗' })),
  ];
});

const goAuth = (mode: AuthMode) => {
  goToAuth(router, mode, router.currentRoute.value.fullPath);
};

const toComposeAsset = (kind: ComposeAsset['kind'], content: string, name: string) => {
  return {
    id: `${kind}-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`,
    kind,
    name,
    content,
  } satisfies ComposeAsset;
};

const addLink = (url: string) => {
  const value = url.trim();
  if (!value) {
    return;
  }

  if (!/^https?:\/\//i.test(value)) {
    window.$message.warning('请输入以 http(s):// 开头的链接');
    return;
  }

  if (links.value.some((item) => item.content === value)) {
    return;
  }

  if (links.value.length >= 3) {
    window.$message.warning('最多添加 3 个链接');
    return;
  }

  links.value = [...links.value, toComposeAsset('link', value, value)];
};

const togglePanel = (panel: 'visibility') => {
  editorPanel.value = editorPanel.value === panel ? '' : panel;
};

const syncEditorImages = () => {
  imageContents.value = syncImageAssetsWithEditor(editorHtml.value, imageContents.value);
};

const handleUploadedAsset = (asset: UploadedAsset) => {
  if (asset.kind === 'public/image') {
    imageContents.value = [
      ...imageContents.value,
      toComposeAsset('image', asset.content, asset.name || `图片 ${imageContents.value.length + 1}`),
    ];
    return;
  }

  if (asset.kind === 'public/video') {
    if (!profile.value.allowTweetVideo) {
      window.$message.warning('当前站点未开启视频发布');
      return;
    }
    videoContents.value = [...videoContents.value, toComposeAsset('video', asset.content, asset.name)];
    window.$message.success(`视频 ${asset.name} 上传成功`);
    return;
  }

  if (!profile.value.allowTweetAttachment) {
    window.$message.warning('当前站点未开启附件发布');
    return;
  }
  attachmentContents.value = [...attachmentContents.value, toComposeAsset('attachment', asset.content, asset.name)];
  window.$message.success(`附件 ${asset.name} 上传成功`);
};

const removeAsset = (asset: ComposeAsset) => {
  if (asset.kind === 'video') {
    videoContents.value = videoContents.value.filter((item) => item.id !== asset.id);
    return;
  }
  if (asset.kind === 'attachment') {
    attachmentContents.value = attachmentContents.value.filter((item) => item.id !== asset.id);
    return;
  }
  if (asset.kind === 'link') {
    links.value = links.value.filter((item) => item.id !== asset.id);
  }
};

const resetEditor = () => {
  editorHtml.value = '';
  imageContents.value = [];
  videoContents.value = [];
  attachmentContents.value = [];
  links.value = [];
  attachmentPrice.value = 0;
  editorPanel.value = '';
  visitType.value = defaultVisitType.value;
  uploading.value = [];
};

const appendQuickMediaAssets = async () => {
  const pending = consumePendingAttachmentSelection();
  if (!pending || pending.files.length === 0) {
    return;
  }

  for (const file of pending.files) {
    const uploadKind = file.type.startsWith('video/') ? 'public/video' : 'public/image';
    if (uploadKind === 'public/video' && !profile.value.allowTweetVideo) {
      continue;
    }
    try {
      uploading.value = [...uploading.value, file.name];
      const asset = await uploadFile(file, uploadKind);
      handleUploadedAsset(asset);
    } catch {
      window.$message.error(`${file.name} 导入失败`);
    } finally {
      uploading.value = uploading.value.filter((item) => item !== file.name);
    }
  }
};

const submitPost = async () => {
  syncEditorImages();
  if (
    !hasComposeContent(editorHtml.value, {
      images: imageContents.value,
      videos: videoContents.value,
      attachments: attachmentContents.value,
      links: links.value,
    })
  ) {
    window.$message.warning('请输入内容哦');
    return;
  }
  if (plainText.value.length > profile.value.defaultTweetMaxLength) {
    window.$message.warning(`内容不能超过 ${profile.value.defaultTweetMaxLength} 字`);
    return;
  }

  const { contents, plainText: normalizedText, textContent } = buildComposePostContents({
    textHtml: editorHtml.value,
    images: imageContents.value,
    videos: videoContents.value,
    attachments: attachmentContents.value,
    links: links.value,
    attachmentPrice: attachmentPrice.value * 100,
  });
  const { tags, users } = parsePostTag(`${normalizedText || textContent} `);
  const normalizedTags = Array.from(
    new Set(isEventMode.value ? [...tags, EVENT_POST_TAG] : tags),
  );

  submitting.value = true;
  try {
    const post = await createPost({
      space_slug: currentSpaceSlug.value,
      contents,
      tags: normalizedTags,
      users: Array.from(new Set(users)),
      attachment_price: attachmentPrice.value * 100,
      visibility: visitType.value,
    });
    window.$message.success(isEventMode.value ? '事件已创建' : '发布成功');
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
  if (isQuickMediaMode.value) {
    void appendQuickMediaAssets();
  }
});

watch(editorHtml, () => {
  syncEditorImages();
});
</script>

<style scoped lang="less">
.compose-editor {
  --compose-panel-bg: var(--panel-bg);
  --compose-panel-border: var(--panel-border);
  --compose-panel-shadow: var(--panel-shadow);
  --compose-soft-bg: var(--accent-soft-muted);
  --compose-soft-bg-strong: var(--accent-soft);
  --compose-text-main: var(--editor-text-main);
  --compose-text-subtle: var(--editor-text-subtle);
  --compose-editor-bg: var(--editor-bg);
  --compose-editor-toolbar-bg: var(--editor-toolbar-bg);
  --compose-editor-border: var(--editor-border);
  --compose-accent: var(--accent-primary);
  --compose-accent-ring: var(--editor-accent-ring);
  width: 100%;
  color: var(--compose-text-main);
}

.compose-editor :deep(.ck) {
  --ck-color-base-background: var(--compose-editor-bg);
  --ck-color-toolbar-background: var(--compose-editor-toolbar-bg);
  --ck-color-toolbar-border: var(--compose-editor-border);
  --ck-color-base-border: var(--compose-editor-border);
  --ck-color-panel-background: var(--compose-editor-bg);
  --ck-color-panel-border: var(--compose-editor-border);
  --ck-color-input-background: var(--compose-editor-bg);
  --ck-color-input-border: var(--compose-editor-border);
  --ck-color-input-text: var(--compose-text-main);
  --ck-color-text: var(--compose-text-main);
  --ck-color-button-default-hover-background: var(--compose-soft-bg);
  --ck-color-button-default-active-background: var(--compose-soft-bg-strong);
  --ck-color-button-on-background: var(--compose-soft-bg-strong);
  --ck-color-button-on-color: var(--compose-accent);
  --ck-color-focus-border: var(--compose-accent);
  --ck-color-shadow-drop: transparent;
  --ck-focus-ring: 0 0 0 3px var(--compose-accent-ring);
  color: var(--compose-text-main);
}

.compose-editor :deep(.ck.ck-editor) {
  display: block;
}

.compose-editor :deep(.ck.ck-editor__main > .ck-editor__editable) {
  min-height: 260px;
  color: var(--compose-text-main);
  background: var(--compose-editor-bg);
}

.compose-editor :deep(.ck.ck-toolbar),
.compose-editor :deep(.ck.ck-editor__main > .ck-editor__editable),
.compose-editor :deep(.ck.ck-balloon-panel),
.compose-editor :deep(.ck.ck-dropdown__panel) {
  border-color: var(--compose-editor-border);
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
  border-radius: 24px;
  overflow: hidden;
  border: 1px solid var(--compose-panel-border);
  background: var(--compose-panel-bg);
  box-shadow: var(--compose-panel-shadow);
}

.event-intro-card {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px 16px;
  border-radius: 18px;
  border: 1px solid var(--compose-panel-border);
  background:
    linear-gradient(135deg, color-mix(in srgb, var(--accent-soft) 72%, transparent), transparent 62%),
    var(--compose-panel-bg);
  box-shadow: var(--compose-panel-shadow);
}

.event-intro-badge {
  flex: none;
  min-width: 44px;
  height: 28px;
  padding: 0 10px;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: var(--compose-soft-bg-strong);
  color: var(--compose-accent);
  font-size: 12px;
  font-weight: 700;
}

.event-intro-content {
  display: grid;
  gap: 4px;

  strong {
    font-size: 14px;
    line-height: 1.45;
  }

  span {
    font-size: 12px;
    line-height: 1.6;
    color: var(--compose-text-subtle);
  }
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
  border-radius: 999px;
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
.asset-pill-list {
  margin-top: 16px;
  display: grid;
  gap: 10px;
}

.uploading-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 14px;
  background: var(--compose-soft-bg);
  overflow-wrap: anywhere;
}

.asset-pill-list {
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
}

.asset-pill {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  width: 100%;
  padding: 10px 12px;
  border: 0;
  border-radius: 14px;
  background: var(--compose-soft-bg-strong);
  color: inherit;
  text-align: left;
  cursor: pointer;
}

.asset-pill-remove {
  font-size: 18px;
  line-height: 1;
  opacity: 0.65;
}

.compose-login-card {
  padding: 24px;
  border-radius: 22px;
  border: 1px solid var(--compose-panel-border);
  background: var(--compose-panel-bg);
  box-shadow: var(--compose-panel-shadow);
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

.compose-editor :deep(.ck.ck-editor) {
  color: var(--compose-text-main);
}

.compose-editor :deep(.ck.ck-toolbar) {
  border-color: var(--compose-editor-border);
  background: var(--compose-editor-toolbar-bg);
}

.compose-editor :deep(.ck.ck-toolbar .ck-button),
.compose-editor :deep(.ck.ck-toolbar .ck-button .ck-button__label),
.compose-editor :deep(.ck.ck-toolbar .ck-icon) {
  color: var(--compose-text-main);
}

.compose-editor :deep(.ck.ck-editor__main > .ck-editor__editable) {
  min-height: 240px;
  border-color: var(--compose-editor-border);
  background: var(--compose-editor-bg);
  color: var(--compose-text-main);
}

.compose-editor :deep(.ck.ck-editor__main > .ck-editor__editable.ck-focused) {
  border-color: var(--compose-accent);
  box-shadow: 0 0 0 1px var(--compose-accent-ring);
}

.compose-editor :deep(.ck.ck-editor__editable_inline) {
  color-scheme: light;
}

.compose-editor :deep(.ck.ck-toolbar .ck-button:hover),
.compose-editor :deep(.ck.ck-toolbar .ck-button.ck-on) {
  background: var(--accent-soft-hover);
}

.compose-editor :deep(.n-input-number) {
  --n-color: var(--compose-editor-bg);
  --n-color-focus: var(--compose-editor-bg);
  --n-border: var(--compose-editor-border);
  --n-border-hover: var(--compose-accent);
  --n-border-focus: var(--compose-accent);
  --n-text-color: var(--compose-text-main);
  --n-placeholder-color: var(--compose-text-subtle);
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

:global(.dark) .compose-editor :deep(.ck.ck-editor__editable_inline) {
  color-scheme: dark;
}
</style>
