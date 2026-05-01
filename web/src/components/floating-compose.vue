<template>
  <transition name="float-compose">
    <button
      v-if="userLogined || hasToken"
      class="floating-compose"
      type="button"
      aria-label="发布动态"
      @click="goCompose"
    >
      <span class="floating-compose-plus">+</span>
    </button>
  </transition>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import { TOKEN_KEY, useStoreUser } from '@/store/user';
import { useStoreProfile } from '@/store/profile';
import { storeToRefs } from 'pinia';
import { resolveSpaceSlug } from '@/utils/spaces';

const router = useRouter();
const storeUser = useStoreUser();
const storeProfile = useStoreProfile();
const { userLogined } = storeToRefs(storeUser);
const { currentSpaceSlug } = storeToRefs(storeProfile);
const hasToken = typeof window !== 'undefined' && !!localStorage.getItem(TOKEN_KEY);

const goCompose = () => {
  const spaceSlug = resolveSpaceSlug(
    currentSpaceSlug.value,
    storeProfile.profile.defaultSpaceSlug,
  );
  router.push({
    name: 'compose',
    query: spaceSlug
      ? {
          space: spaceSlug,
        }
      : undefined,
  });
};
</script>

<style scoped lang="less">
.floating-compose {
  --floating-compose-bg: linear-gradient(135deg, #0f9f6e, #3dc788);
  --floating-compose-shadow:
    0 18px 40px rgba(16, 133, 90, 0.28),
    inset 0 1px 0 rgba(255, 255, 255, 0.22);
  --floating-compose-shadow-hover: 0 24px 50px rgba(16, 133, 90, 0.34);
  --floating-compose-text: #fff;
  position: fixed;
  right: 28px;
  bottom: 30px;
  z-index: 140;
  width: 64px;
  height: 64px;
  border: 0;
  border-radius: 999px;
  background: var(--floating-compose-bg);
  color: var(--floating-compose-text);
  box-shadow: var(--floating-compose-shadow);
  cursor: pointer;
  transition:
    transform 0.2s ease,
    box-shadow 0.2s ease,
    filter 0.2s ease;

  &:hover {
    transform: translateY(-3px) scale(1.03);
    box-shadow: var(--floating-compose-shadow-hover);
    filter: saturate(1.05);
  }

  &:active {
    transform: translateY(0) scale(0.98);
  }
}

.floating-compose-plus {
  display: inline-block;
  font-size: 36px;
  font-weight: 500;
  line-height: 1;
  transform: translateY(-1px);
}

.float-compose-enter-active,
.float-compose-leave-active {
  transition: all 0.24s ease;
}

.float-compose-enter-from,
.float-compose-leave-to {
  opacity: 0;
  transform: translateY(20px) scale(0.9);
}

:global(.dark) .floating-compose {
  --floating-compose-bg: linear-gradient(135deg, #17855d, #2fc08a);
  --floating-compose-shadow:
    0 18px 44px rgba(7, 15, 12, 0.52),
    inset 0 1px 0 rgba(255, 255, 255, 0.16);
  --floating-compose-shadow-hover: 0 24px 54px rgba(7, 15, 12, 0.62);
  --floating-compose-text: #fff;
}

@media screen and (max-width: 821px) {
  .floating-compose {
    right: 16px;
    bottom: 18px;
    width: 56px;
    height: 56px;
  }

  .floating-compose-plus {
    font-size: 32px;
  }
}
</style>
