<template>
  <transition name="float-compose">
    <button
      v-if="userLogined"
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
import { useStoreUser } from '@/store/user';
import { storeToRefs } from 'pinia';

const router = useRouter();
const storeUser = useStoreUser();
const { userLogined } = storeToRefs(storeUser);

const goCompose = () => {
  router.push({
    name: 'compose',
  });
};
</script>

<style scoped lang="less">
.floating-compose {
  position: fixed;
  right: 24px;
  bottom: 28px;
  z-index: 140;
  width: 60px;
  height: 60px;
  border: 0;
  border-radius: 999px;
  background: linear-gradient(135deg, #0f9f6e, #3dc788);
  color: #fff;
  box-shadow: 0 18px 40px rgba(16, 133, 90, 0.28);
  cursor: pointer;
  transition:
    transform 0.2s ease,
    box-shadow 0.2s ease,
    filter 0.2s ease;

  &:hover {
    transform: translateY(-3px) scale(1.03);
    box-shadow: 0 24px 50px rgba(16, 133, 90, 0.34);
    filter: saturate(1.05);
  }

  &:active {
    transform: translateY(0) scale(0.98);
  }
}

.floating-compose-plus {
  display: inline-block;
  font-size: 34px;
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

@media screen and (max-width: 821px) {
  .floating-compose {
    right: 16px;
    bottom: 20px;
    width: 56px;
    height: 56px;
  }
}
</style>
