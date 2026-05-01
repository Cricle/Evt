<template>
  <div class="auth-page">
    <main-nav :title="mode === 'signin' ? '登录' : '注册'" :back="true" />
    <div class="auth-shell">
      <section class="auth-panel">
        <div class="auth-hero">
          <div class="auth-badge">Evt</div>
          <h1>{{ mode === 'signin' ? '欢迎回来' : '创建账号' }}</h1>
          <p>{{ mode === 'signin' ? '登录后继续浏览、发帖和互动。' : '注册后即可加入社区，发布动态和参与回复。' }}</p>
        </div>

        <div class="auth-switch">
          <button
            type="button"
            class="auth-switch-btn"
            :class="{ active: mode === 'signin', subtle: mode !== 'signin' }"
            @click="switchMode('signin')"
          >
            登录
          </button>
          <button
            v-if="profile.allowUserRegister"
            type="button"
            class="auth-switch-btn"
            :class="{ active: mode === 'signup', subtle: mode !== 'signup' }"
            @click="switchMode('signup')"
          >
            注册
          </button>
        </div>

        <n-form
          v-if="mode === 'signin'"
          ref="loginRef"
          class="auth-form"
          :model="loginForm"
          :rules="loginRules"
        >
          <n-form-item label="账户" path="username">
            <n-input
              v-model:value="loginForm.username"
              placeholder="请输入用户名"
              @keyup.enter.prevent="handleLogin"
            />
          </n-form-item>
          <n-form-item label="密码" path="password">
            <n-input
              v-model:value="loginForm.password"
              type="password"
              show-password-on="mousedown"
              placeholder="请输入账户密码"
              @keyup.enter.prevent="handleLogin"
            />
          </n-form-item>
          <n-button type="primary" block strong secondary :loading="loading" @click="handleLogin">
            登录
          </n-button>
        </n-form>

        <n-form
          v-else
          ref="registerRef"
          class="auth-form"
          :model="registerForm"
          :rules="registerRules"
        >
          <n-form-item label="用户名" path="username">
            <n-input v-model:value="registerForm.username" placeholder="用户名注册后无法修改" />
          </n-form-item>
          <n-form-item label="密码" path="password">
            <n-input
              v-model:value="registerForm.password"
              type="password"
              show-password-on="mousedown"
              placeholder="密码不少于6位"
              @keyup.enter.prevent="handleRegister"
            />
          </n-form-item>
          <n-form-item label="重复密码" path="repassword">
            <n-input
              v-model:value="registerForm.repassword"
              type="password"
              show-password-on="mousedown"
              placeholder="请再次输入密码"
              @keyup.enter.prevent="handleRegister"
            />
          </n-form-item>
          <n-button
            type="primary"
            block
            strong
            :loading="loading"
            class="auth-submit-btn"
            @click="handleRegister"
          >
            注册
          </n-button>
        </n-form>
      </section>
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

const route = useRoute();
const router = useRouter();
const storeProfile = useStoreProfile();
const storeUser = useStoreUser();
const { profile } = storeToRefs(storeProfile);

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

const loginRules = {
  username: {
    required: true,
    message: '请输入账户名',
  },
  password: {
    required: true,
    message: '请输入密码',
  },
};

const registerRules = {
  username: {
    required: true,
    message: '请输入账户名',
  },
  password: [
    {
      required: true,
      message: '请输入密码',
    },
    {
      min: 6,
      message: '密码不少于6位',
      trigger: 'input',
    },
  ],
  repassword: [
    {
      required: true,
      message: '请再次输入密码',
    },
    {
      validator: (_rule: FormItemRule, value: string) => value === registerForm.password,
      message: '两次密码输入不一致',
      trigger: 'input',
    },
  ],
};

const switchMode = (nextMode: 'signin' | 'signup') => {
  router.replace({
    name: 'auth',
    query: {
      ...route.query,
      mode: nextMode,
    },
  });
};

const finishAuth = async (token: string) => {
  localStorage.setItem(TOKEN_KEY, token);
  const currentUser = await userInfo(token);
  storeUser.updateUserinfo(currentUser);
  window.$message.success('登录成功');
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
      window.$message.success('注册成功');
      await finishAuth(loginRes?.token || '');
    } finally {
      loading.value = false;
    }
  });
};
</script>

<style scoped lang="less">
.auth-page {
  --auth-bg-base: #f8fbf8;
  --auth-bg-top: rgba(24, 160, 88, 0.14);
  --auth-bg-bottom: #edf4ef;
  --auth-panel-border: rgba(15, 23, 42, 0.08);
  --auth-panel-bg: rgba(255, 255, 255, 0.9);
  --auth-panel-shadow: rgba(15, 23, 42, 0.1);
  --auth-input-bg: rgba(255, 255, 255, 0.9);
  --auth-input-border: rgba(15, 23, 42, 0.08);
  --auth-text-main: #18201b;
  --auth-text-subtle: rgba(24, 32, 27, 0.72);
  --auth-badge-bg: rgba(24, 160, 88, 0.12);
  --auth-badge-text: #12895a;
  --auth-switch-bg: rgba(15, 23, 42, 0.06);
  --auth-switch-text: rgba(24, 32, 27, 0.8);
  --auth-switch-hover-bg: rgba(24, 160, 88, 0.12);
  --auth-switch-hover-text: #12895a;
  --auth-primary-start: #0f9f6e;
  --auth-primary-end: #34bf82;
  --auth-active-shadow: rgba(22, 148, 98, 0.22);
  --auth-submit-shadow: rgba(22, 148, 98, 0.18);
  min-height: 100vh;
  background:
    radial-gradient(circle at top left, var(--auth-bg-top), transparent 28%),
    linear-gradient(180deg, var(--auth-bg-base) 0%, var(--auth-bg-bottom) 100%);
}

.auth-shell {
  padding: 48px 20px 72px;
}

.auth-panel {
  max-width: 460px;
  margin: 0 auto;
  padding: 28px;
  border: 1px solid var(--auth-panel-border);
  border-radius: 28px;
  background: var(--auth-panel-bg);
  box-shadow: 0 28px 60px var(--auth-panel-shadow);
  backdrop-filter: blur(10px);
  animation: auth-rise 0.28s ease;
}

.auth-hero {
  margin-bottom: 20px;

  h1 {
    margin: 14px 0 8px;
    font-size: 34px;
    line-height: 1.05;
  }

  p {
    margin: 0;
    color: var(--auth-text-subtle);
  }
}

.auth-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 30px;
  padding: 0 14px;
  border-radius: 999px;
  background: var(--auth-badge-bg);
  color: var(--auth-badge-text);
  font-weight: 700;
}

.auth-switch {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
}

.auth-switch-btn {
  --auth-switch-btn-text: #fff;
  flex: 1;
  height: 44px;
  border: 0;
  border-radius: 14px;
  background: var(--auth-switch-bg);
  color: var(--auth-switch-text);
  cursor: pointer;
  transition: all 0.2s ease;

  &.active {
    background: linear-gradient(135deg, var(--auth-primary-start), var(--auth-primary-end));
    color: var(--auth-switch-btn-text);
    box-shadow: 0 12px 26px var(--auth-active-shadow);
  }

  &.subtle:hover {
    background: var(--auth-switch-hover-bg);
    color: var(--auth-switch-hover-text);
  }
}

.auth-form {
  display: grid;
  gap: 8px;
}

.auth-submit-btn {
  background: linear-gradient(135deg, var(--auth-primary-start), var(--auth-primary-end));
  border-color: transparent;
  box-shadow: 0 14px 30px var(--auth-submit-shadow);
}

.auth-page :deep(.n-form-item-label__text),
.auth-page :deep(.n-input__placeholder),
.auth-page :deep(.n-input__input-el),
.auth-page :deep(.n-button__content) {
  color: var(--auth-text-main);
}

.auth-page :deep(.n-input) {
  --n-border: var(--auth-input-border);
  --n-border-hover: var(--auth-primary-start);
  --n-border-focus: var(--auth-primary-start);
  --n-border-disabled: var(--auth-input-border);
  --n-box-shadow-focus: 0 0 0 2px color-mix(in srgb, var(--auth-primary-start) 18%, transparent);
  --n-color: var(--auth-input-bg);
  --n-color-focus: var(--auth-input-bg);
  --n-text-color: var(--auth-text-main);
  --n-caret-color: var(--auth-primary-start);
  --n-placeholder-color: var(--auth-text-subtle);
}

.auth-page :deep(.n-input .n-input__input-el),
.auth-page :deep(.n-input .n-input__textarea-el) {
  color-scheme: light;
}

.auth-page :deep(.n-button--primary-type) {
  --n-color: transparent;
  --n-color-hover: transparent;
  --n-color-pressed: transparent;
  --n-color-focus: transparent;
  --n-border: transparent;
  --n-border-hover: transparent;
  --n-border-pressed: transparent;
  --n-border-focus: transparent;
  --n-ripple-color: rgba(255, 255, 255, 0.18);
  color: var(--auth-switch-btn-text);
}

:global(.dark) .auth-page {
  --auth-bg-base: #0d1212;
  --auth-bg-top: rgba(99, 226, 183, 0.12);
  --auth-bg-bottom: #111717;
  --auth-panel-border: rgba(148, 163, 184, 0.12);
  --auth-panel-bg: rgba(18, 24, 24, 0.9);
  --auth-panel-shadow: rgba(0, 0, 0, 0.42);
  --auth-input-bg: rgba(25, 33, 33, 0.92);
  --auth-input-border: rgba(148, 163, 184, 0.16);
  --auth-text-main: rgba(241, 245, 249, 0.94);
  --auth-text-subtle: rgba(226, 232, 240, 0.72);
  --auth-badge-bg: rgba(99, 226, 183, 0.14);
  --auth-badge-text: #63e2b7;
  --auth-switch-bg: rgba(148, 163, 184, 0.12);
  --auth-switch-text: rgba(241, 245, 249, 0.86);
  --auth-switch-hover-bg: rgba(99, 226, 183, 0.16);
  --auth-switch-hover-text: #63e2b7;
  --auth-primary-start: #21c58a;
  --auth-primary-end: #63e2b7;
  --auth-active-shadow: rgba(54, 199, 132, 0.28);
  --auth-submit-shadow: rgba(54, 199, 132, 0.22);
}

:global(.dark) .auth-hero h1 {
  color: var(--auth-text-main);
}

:global(.dark) .auth-page :deep(.n-input .n-input__input-el),
:global(.dark) .auth-page :deep(.n-input .n-input__textarea-el) {
  color-scheme: dark;
}

@keyframes auth-rise {
  from {
    opacity: 0;
    transform: translateY(16px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media screen and (max-width: 821px) {
  .auth-shell {
    padding: 24px 14px 56px;
  }

  .auth-panel {
    padding: 20px;
    border-radius: 22px;
  }

  .auth-hero h1 {
    font-size: 28px;
  }
}
</style>
