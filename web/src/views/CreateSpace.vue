<template>
  <div class="create-space-page">
    <main-nav title="新建广场" :back="true" />
    <div class="create-space-shell">
      <section class="create-space-panel">
        <n-card :bordered="false" class="create-space-card">
          <div class="create-space-copy">
            <h1>创建你的广场</h1>
            <p>为固定成员、兴趣小组或团队讨论创建独立空间。</p>
          </div>

          <n-form
            ref="formRef"
            :model="formValue"
            :rules="rules"
            label-placement="top"
            size="large"
            class="create-space-form"
          >
            <n-form-item label="广场名称" path="name">
              <n-input
                v-model:value="formValue.name"
                placeholder="例如：设计协作组"
                maxlength="32"
                show-count
              />
            </n-form-item>

            <n-form-item label="广场标识" path="slug">
              <n-input
                v-model:value="formValue.slug"
                placeholder="例如：design-team"
                maxlength="32"
                show-count
              />
            </n-form-item>

            <n-form-item label="可见性" path="visibility">
              <n-radio-group v-model:value="formValue.visibility">
                <n-space vertical>
                  <n-radio value="public">公开广场</n-radio>
                  <n-radio value="private">私密广场</n-radio>
                </n-space>
              </n-radio-group>
            </n-form-item>

            <n-form-item label="简介" path="description">
              <n-input
                v-model:value="formValue.description"
                type="textarea"
                placeholder="简单介绍这个广场的用途"
                :autosize="{ minRows: 4, maxRows: 8 }"
                maxlength="200"
                show-count
              />
            </n-form-item>

            <div class="create-space-actions">
              <n-button tertiary round @click="router.back()">取消</n-button>
              <n-button
                type="primary"
                round
                :loading="submitting"
                @click="handleSubmit"
              >
                创建广场
              </n-button>
            </div>
          </n-form>
        </n-card>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue';
import type { FormInst, FormRules } from 'naive-ui';
import { useRoute, useRouter } from 'vue-router';
import { Api } from '@/utils/request';
import { useStoreProfile } from '@/store/profile';
import { storeToRefs } from 'pinia';
import { buildHomeRouteWithSpace } from '@/utils/tagRoute';
import { resolveSpaceSlug } from '@/utils/spaces';

const route = useRoute();
const router = useRouter();
const storeProfile = useStoreProfile();
const { currentSpaceSlug, spaces } = storeToRefs(storeProfile);

const formRef = ref<FormInst | null>(null);
const submitting = ref(false);
const formValue = reactive({
  name: '',
  slug: '',
  description: '',
  visibility: 'public' as 'public' | 'private',
});

const rules: FormRules = {
  name: [
    {
      required: true,
      message: '请输入广场名称',
      trigger: ['input', 'blur'],
    },
  ],
  slug: [
    {
      required: true,
      message: '请输入广场标识',
      trigger: ['input', 'blur'],
    },
    {
      validator: (_rule, value: string) => /^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(value.trim()),
      message: '仅支持小写字母、数字和中划线',
      trigger: ['input', 'blur'],
    },
  ],
};

onMounted(() => {
  const routeSpace = typeof route.query.space === 'string' ? route.query.space : '';
  currentSpaceSlug.value = resolveSpaceSlug(
    routeSpace || currentSpaceSlug.value,
    storeProfile.profile.defaultSpaceSlug,
  );
});

const handleSubmit = async () => {
  await formRef.value?.validate();
  submitting.value = true;

  try {
    const created = await Api.v1.spaces.post._self({
      slug: formValue.slug.trim(),
      name: formValue.name.trim(),
      description: formValue.description.trim(),
      visibility: formValue.visibility,
    });

    const nextSpaces = await Api.v1.spaces.get._self({
      limit: 100,
    });
    spaces.value = nextSpaces || [];
    currentSpaceSlug.value = created.slug;
    window.$message.success('广场创建成功');

    router.replace(buildHomeRouteWithSpace({}, created.slug));
  } finally {
    submitting.value = false;
  }
};
</script>

<style scoped lang="less">
.create-space-page {
  min-height: 100vh;
  background:
    radial-gradient(circle at left top, var(--page-hero-bg-glow), transparent 34%),
    radial-gradient(circle at top right, var(--page-hero-bg-accent), transparent 26%),
    linear-gradient(180deg, var(--page-hero-bg-base) 0%, var(--page-hero-bg-bottom) 100%);
}

.create-space-shell {
  padding: 24px 18px 80px;
}

.create-space-panel {
  width: 100%;
  max-width: 760px;
  margin: 0 auto;
}

.create-space-card {
  border-radius: var(--page-card-radius);
  box-shadow: var(--panel-shadow);
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
}

.create-space-copy {
  margin-bottom: 24px;

  h1 {
    margin: 0;
    font-size: 28px;
    line-height: 1.1;
  }

  p {
    margin: 10px 0 0;
    opacity: 0.72;
    font-size: 14px;
    line-height: 1.7;
  }
}

.create-space-form {
  display: grid;
  gap: 4px;
}

.create-space-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 12px;
}

@media screen and (max-width: 821px) {
  .create-space-shell {
    padding: 12px 10px 80px;
  }

  .create-space-copy h1 {
    font-size: 24px;
  }

  .create-space-actions {
    flex-direction: column-reverse;
  }
}

.create-space-page :deep(.n-card__content) {
  padding: var(--page-card-padding);
}
</style>
