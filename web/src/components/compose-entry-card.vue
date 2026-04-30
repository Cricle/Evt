<template>
  <button type="button" class="compose-entry-card" @click="goCompose">
    <div class="compose-entry-left">
      <n-avatar round :size="42" :src="userInfo.avatar || DEFAULT_USER_AVATAR" />
      <div class="compose-entry-copy">
        <strong>发布动态</strong>
        <span>点击进入编辑页，支持图文、视频、附件和富文本。</span>
      </div>
    </div>
    <div class="compose-entry-action">
      <span>+</span>
    </div>
  </button>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useStoreUser } from '@/store/user';
import { storeToRefs } from 'pinia';
import { DEFAULT_USER_AVATAR } from '@/utils/defaults';

const router = useRouter();
const storeUser = useStoreUser();
const { userInfo } = storeToRefs(storeUser);

const goCompose = () => {
  router.push({
    name: 'compose',
  });
};
</script>

<style scoped lang="less">
.compose-entry-card {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18px;
  padding: 18px 20px;
  border: 1px solid rgba(18, 75, 51, 0.08);
  border-radius: 24px;
  background:
    radial-gradient(circle at top right, rgba(53, 194, 129, 0.16), transparent 32%),
    rgba(255, 255, 255, 0.9);
  cursor: pointer;
  transition:
    transform 0.2s ease,
    box-shadow 0.2s ease,
    border-color 0.2s ease;

  &:hover {
    transform: translateY(-2px);
    border-color: rgba(24, 160, 88, 0.18);
    box-shadow: 0 18px 36px rgba(33, 92, 64, 0.1);
  }
}

.compose-entry-left {
  display: flex;
  align-items: center;
  gap: 14px;
  text-align: left;
}

.compose-entry-copy {
  display: grid;
  gap: 4px;

  span {
    font-size: 13px;
    opacity: 0.72;
  }
}

.compose-entry-action {
  width: 42px;
  height: 42px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  background: linear-gradient(135deg, #0f9f6e, #34bf82);
  color: #fff;
  font-size: 26px;
  line-height: 1;
}
</style>
