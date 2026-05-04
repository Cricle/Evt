<template>
  <div class="auth-page">
    <main-nav :title="mode === 'signin' ? t('auth_tab_signin') : t('auth_tab_signup')" :back="true" />
    <div class="auth-shell">
      <n-card class="auth-panel" :bordered="false" size="large">
        <n-space vertical :size="18">
          <div class="auth-hero">
            <div class="auth-copy">
              <h1>{{ mode === 'signin' ? '欢迎回来' : '创建账号' }}</h1>
              <p>{{ mode === 'signin' ? t('auth_signin_desc') : t('auth_signup_desc') }}</p>
            </div>
            <n-space size="small">
              <n-tag round :bordered="false">{{ t('auth_feature_spaces') }}</n-tag>
              <n-tag round :bordered="false">{{ t('auth_feature_realtime') }}</n-tag>
              <n-tag round :bordered="false">{{ t('auth_feature_theme') }}</n-tag>
            </n-space>
          </div>

          <n-tabs
            class="auth-tabs"
            type="segment"
            animated
            :value="mode"
            @update:value="switchMode"
          >
            <n-tab-pane name="signin" :tab="t('auth_tab_signin')" />
            <n-tab-pane v-if="profile.allowUserRegister" name="signup" :tab="t('auth_tab_signup')" />
          </n-tabs>

          <n-form
            v-if="mode === 'signin'"
            ref="loginRef"
            class="auth-form"
            :model="loginForm"
            :rules="loginRules"
          >
            <n-form-item :label="t('auth_account')" path="username">
              <n-input
                v-model:value="loginForm.username"
                :placeholder="t('auth_placeholder_username')"
                @keyup.enter.prevent="handleLogin"
              />
            </n-form-item>
            <n-form-item :label="t('auth_password')" path="password">
              <n-input
                v-model:value="loginForm.password"
                type="password"
                show-password-on="mousedown"
                :placeholder="t('auth_placeholder_login_password')"
                @keyup.enter.prevent="handleLogin"
              />
            </n-form-item>
            <n-button type="primary" block strong :loading="loading" @click="handleLogin">
              {{ t('auth_action_signin') }}
            </n-button>
          </n-form>

          <n-form
            v-else
            ref="registerRef"
            class="auth-form"
            :model="registerForm"
            :rules="registerRules"
          >
            <n-form-item :label="t('auth_username')" path="username">
              <n-input v-model:value="registerForm.username" :placeholder="t('auth_placeholder_register_username')" />
            </n-form-item>
            <n-form-item :label="t('auth_password')" path="password">
              <n-input
                v-model:value="registerForm.password"
                type="password"
                show-password-on="mousedown"
                :placeholder="t('auth_placeholder_register_password')"
                @keyup.enter.prevent="handleRegister"
              />
            </n-form-item>
            <n-form-item :label="t('auth_repeat_password')" path="repassword">
              <n-input
                v-model:value="registerForm.repassword"
                type="password"
                show-password-on="mousedown"
                :placeholder="t('auth_placeholder_repeat_password')"
                @keyup.enter.prevent="handleRegister"
              />
            </n-form-item>
            <n-button type="primary" block strong :loading="loading" @click="handleRegister">
              {{ t('auth_action_signup') }}
            </n-button>
          </n-form>
        </n-space>
      </n-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import type { FormInst, FormItemRule } from 'naive-ui';
import { storeToRefs } from 'pinia';
import { userInfo } from '@/api/auth';
import { Api } from '@/utils/request';
import { useStoreProfile } from '@/store/profile';
import { TOKEN_KEY, useStoreUser } from '@/store/user';
import { useI18n } from '@/i18n';

const route = useRoute();
const router = useRouter();
const storeProfile = useStoreProfile();
const storeUser = useStoreUser();
const { profile } = storeToRefs(storeProfile);
const { t } = useI18n();

const loading = ref(false);
const loginRef = ref<FormInst>();
const registerRef = ref<FormInst>();

const loginForm = reactive({
  username: '',
  password: '',
});

const registerForm = reactive({
  username: '',
  password: '',
  repassword: '',
});

const mode = computed<'signin' | 'signup'>(() => {
  const routeMode = route.query.mode === 'signup' ? 'signup' : 'signin';
  return !profile.value.allowUserRegister && routeMode === 'signup'
    ? 'signin'
    : routeMode;
});

const redirectTarget = computed(() => {
  return typeof route.query.redirect === 'string' && route.query.redirect
    ? route.query.redirect
    : '/';
});

const loginRules = computed(() => ({
  username: {
    required: true,
    message: t('auth_error_username_required'),
  },
  password: {
    required: true,
    message: t('auth_error_password_required'),
  },
}));

const registerRules = computed(() => ({
  username: {
    required: true,
    message: t('auth_error_username_required'),
  },
  password: [
    {
      required: true,
      message: t('auth_error_password_required'),
    },
    {
      min: 6,
      message: t('auth_error_password_length'),
      trigger: 'input',
    },
  ],
  repassword: [
    {
      required: true,
      message: t('auth_error_repeat_password_required'),
    },
    {
      validator: (_rule: FormItemRule, value: string) => value === registerForm.password,
      message: t('auth_error_password_mismatch'),
      trigger: 'input',
    },
  ],
}));

const switchMode = (nextMode: string) => {
  router.replace({
    name: 'auth',
    query: {
      ...route.query,
      mode: nextMode === 'signup' ? 'signup' : 'signin',
    },
  });
};

const finishAuth = async (token: string) => {
  localStorage.setItem(TOKEN_KEY, token);
  const currentUser = await userInfo(token);
  storeUser.updateUserinfo(currentUser);
  window.$message.success(t('auth_success_signin'));
  await router.replace(redirectTarget.value);
};

const handleLogin = () => {
  loginRef.value?.validate(async (errors) => {
    if (errors) return;
    loading.value = true;
    try {
      const res = await Api.v1.auth.post.login({
        username: loginForm.username,
        password: loginForm.password,
      });
      await finishAuth(res?.token || '');
    } finally {
      loading.value = false;
    }
  });
};

const handleRegister = () => {
  registerRef.value?.validate(async (errors) => {
    if (errors) return;
    loading.value = true;
    try {
      await Api.v1.auth.post.register({
        username: registerForm.username,
        password: registerForm.password,
      });
      const loginRes = await Api.v1.auth.post.login({
        username: registerForm.username,
        password: registerForm.password,
      });
      window.$message.success(t('auth_success_signup'));
      await finishAuth(loginRes?.token || '');
    } finally {
      loading.value = false;
    }
  });
};
</script>

<style scoped lang="less">
.auth-page {
  min-height: 100dvh;
  display: flex;
  flex-direction: column;
  background:
    radial-gradient(circle at top left, var(--page-hero-bg-accent), transparent 28%),
    radial-gradient(circle at 82% 16%, color-mix(in srgb, var(--accent-link) 12%, transparent), transparent 22%),
    linear-gradient(180deg, var(--page-hero-bg-base) 0%, var(--page-hero-bg-bottom) 100%);
}

.auth-shell {
  display: grid;
  place-items: center;
  flex: 1 1 auto;
  min-height: calc(100dvh - 58px);
  box-sizing: border-box;
  padding: 10px 16px 14px;
}

.auth-panel {
  width: min(100%, 460px);
  max-width: 460px;
  background: color-mix(in srgb, var(--panel-bg) 76%, transparent);
  border: var(--glass-panel-border);
  border-radius: var(--page-card-radius);
  box-shadow: var(--panel-shadow);
  backdrop-filter: blur(14px);
}

.auth-hero {
  display: grid;
  gap: 12px;
}

.auth-copy {
  display: grid;
  gap: 8px;
}

.auth-hero h1 {
  margin: 0;
  font-size: 32px;
  line-height: 1.08;
}

.auth-hero p {
  margin: 0;
  color: var(--editor-text-subtle);
}

.auth-tabs {
  margin-top: 2px;
}

.auth-tabs :deep(.n-tabs-pane-wrapper) {
  display: none;
}

.auth-form {
  display: grid;
  gap: 8px;
}

.auth-page :deep(.n-form-item-label__text),
.auth-page :deep(.n-input__placeholder),
.auth-page :deep(.n-input__input-el),
.auth-page :deep(.n-button__content) {
  color: var(--editor-text-main);
}

.auth-page :deep(.n-input) {
  --n-border: var(--panel-border);
  --n-border-hover: var(--accent-primary);
  --n-border-focus: var(--accent-primary);
  --n-border-disabled: var(--panel-border);
  --n-box-shadow-focus: 0 0 0 2px color-mix(in srgb, var(--accent-primary) 18%, transparent);
  --n-color: color-mix(in srgb, var(--surface-subtle) 84%, transparent);
  --n-color-focus: color-mix(in srgb, var(--surface-subtle) 92%, transparent);
  --n-text-color: var(--editor-text-main);
  --n-caret-color: var(--accent-primary);
  --n-placeholder-color: var(--editor-text-subtle);
}

.auth-page :deep(.n-card__content) {
  padding: 24px;
}

.auth-page :deep(.n-base-selection),
.auth-page :deep(.n-tabs-nav-scroll-content),
.auth-page :deep(.n-input .n-input__input-el),
.auth-page :deep(.n-input .n-input__textarea-el) {
  color-scheme: light;
}

:global(.dark) .auth-page :deep(.n-base-selection),
:global(.dark) .auth-page :deep(.n-tabs-nav-scroll-content),
:global(.dark) .auth-page :deep(.n-input .n-input__input-el),
:global(.dark) .auth-page :deep(.n-input .n-input__textarea-el) {
  color-scheme: dark;
}

@media screen and (max-width: 821px) {
  .auth-shell {
    min-height: calc(100dvh - 54px);
    padding: 8px 10px 10px;
  }

  .auth-page :deep(.n-card__content) {
    padding: 16px;
  }

  .auth-copy h1 {
    font-size: 28px;
  }
}
</style>
