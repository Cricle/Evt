<template>
    <div>
        <main-nav title="404" />

        <n-list class="main-content-wrap wrap404" bordered>
            <n-result
                status="404"
                title="404 资源不存在"
                description="再看看其他的吧"
            >
                <template #footer>
                    <n-button @click="goHome">回主页</n-button>
                </template>
            </n-result>
        </n-list>
    </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useStoreProfile } from '@/store/profile';
import { storeToRefs } from 'pinia';
import { buildHomeRouteWithSpace } from '@/utils/tagRoute';

const router = useRouter();
const storeProfile = useStoreProfile();
const { currentSpaceSlug } = storeToRefs(storeProfile);
const goHome = () => {
  router.push(buildHomeRouteWithSpace({}, currentSpaceSlug.value));
};
</script>

<style lang="less" scoped>
.wrap404 {
    min-height: 500px;
    display: flex;
    align-items: center;
    justify-content: center;
}
:global(.dark) .main-content-wrap {
    background-color: rgba(16, 16, 20, 0.75);
}
</style>
