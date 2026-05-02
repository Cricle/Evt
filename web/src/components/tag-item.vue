<template>
    <div v-if="(!checkFollowing && !checkPin) || (checkFollowing && tag.is_following === 1) || (checkPin && tag.is_following === 1 && tag.is_pin === 1)" class="tag-item">
        <n-thing>
            <template #header>
                <n-tag
                        type="success"
                        size="large"
                        round
                        :key="tag.id"
                    >
                        <router-link
                            class="hash-link"
                            :to="buildTagSearchRoute(tag.tag, currentSpaceSlug)"
                        >
                            #{{ tag.tag }}
                        </router-link>
                        <span v-if="!showAction" class="tag-quote">({{ tag.quote_num }})</span>
                        <span v-if="showAction" class="tag-quote tag-follow">({{ tag.quote_num }})</span>
                        <template #avatar>
                            <n-avatar :src="tagUserAvatar" />
                        </template>
                    </n-tag>
            </template>
            <template #header-extra>
                <div 
                    v-if="showAction" 
                    class="options">
                    <n-dropdown
                        placement="bottom-end"
                        trigger="click"
                        size="small"
                        :options="tagOptions"
                        @select="handleTagAction"
                    >
                        <n-button type="success" quaternary circle block>
                            <template #icon>
                                <n-icon>
                                    <more-vert-outlined />
                                </n-icon>
                            </template>
                        </n-button>
                    </n-dropdown>
                </div>
            </template>
        </n-thing>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { MoreVertOutlined } from '@vicons/material';
import type { DropdownOption } from 'naive-ui';
import { storeToRefs } from 'pinia';
import { pinTopic, stickTopic, followTopic, unfollowTopic } from '@/api/post';
import { DEFAULT_USER_AVATAR } from '@/utils/defaults';
import { useStoreProfile } from '@/store/profile';
import { buildTagSearchRoute } from '@/utils/tagRoute';

const hasFollowing = ref(false);
const storeProfile = useStoreProfile();
const { currentSpaceSlug } = storeToRefs(storeProfile);
const props = withDefaults(
  defineProps<{
    tag: Item.TagProps;
    showAction: boolean;
    checkFollowing: boolean;
    checkPin: boolean;
  }>(),
  {},
);

const tagUserAvatar = computed(() => {
  if (props.tag.user) {
    return props.tag.user.avatar || DEFAULT_USER_AVATAR;
  } else {
    return DEFAULT_USER_AVATAR;
  }
});

const tagOptions = computed(() => {
  let options: DropdownOption[] = [];
  if (props.tag.is_following === 0) {
    options.push({
      label: '关注',
      key: 'follow',
    });
  } else {
    if (props.tag.is_pin === 0) {
      options.push({
        label: '钉住',
        key: 'pin',
      });
    } else {
      options.push({
        label: '取消钉住',
        key: 'unpin',
      });
    }
    if (props.tag.is_top === 0) {
      options.push({
        label: '置顶',
        key: 'stick',
      });
    } else {
      options.push({
        label: '取消置顶',
        key: 'unstick',
      });
    }
    options.push({
      label: '取消关注',
      key: 'unfollow',
    });
  }
  return options;
});

const handleTagAction = (
  item: 'follow' | 'unfollow' | 'pin' | 'unpin' | 'stick' | 'unstick',
) => {
  switch (item) {
    case 'follow':
      followTopic({
        topic_id: props.tag.id,
        space_slug: currentSpaceSlug.value,
      })
        .then((_res) => {
          props.tag.is_following = 1;
          window.$message.success(`关注成功`);
        })
        .catch(() => {
          window.$message.error('关注话题失败');
        });
      break;
    case 'unfollow':
      unfollowTopic({
        topic_id: props.tag.id,
        space_slug: currentSpaceSlug.value,
      })
        .then((_res) => {
          props.tag.is_following = 0;
          window.$message.success(`取消关注`);
        })
        .catch(() => {
          window.$message.error('取消关注话题失败');
        });
      break;
    case 'pin':
      pinTopic({
        topic_id: props.tag.id,
        space_slug: currentSpaceSlug.value,
      })
        .then((_res) => {
          props.tag.is_pin = 1;
          window.$message.success(`钉住成功`);
        })
        .catch(() => {
          window.$message.error('钉住话题失败');
        });
      break;
    case 'unpin':
      pinTopic({
        topic_id: props.tag.id,
        space_slug: currentSpaceSlug.value,
      })
        .then((_res) => {
          props.tag.is_pin = 0;
          window.$message.success(`取消钉住`);
        })
        .catch(() => {
          window.$message.error('取消钉住话题失败');
        });
      break;
    case 'stick':
      stickTopic({
        topic_id: props.tag.id,
        space_slug: currentSpaceSlug.value,
      })
        .then((res) => {
          props.tag.is_top = res.top_status;
          window.$message.success(`置顶成功`);
        })
        .catch(() => {
          window.$message.error('置顶话题失败');
        });
      break;
    case 'unstick':
      stickTopic({
        topic_id: props.tag.id,
        space_slug: currentSpaceSlug.value,
      })
        .then((res) => {
          props.tag.is_top = res.top_status;
          window.$message.success(`取消置顶`);
        })
        .catch(() => {
          window.$message.error('取消置顶话题失败');
        });
      break;
    default:
      break;
  }
};

onMounted(() => {
  hasFollowing.value = false;
});
</script>

<style lang="less">
.tag-item {
    .tag-quote {
        margin-left: 12px;
        font-size: 14px;
        opacity: 0.75;
    }
    .tag-follow {
        margin-right: 22px;
    }
    .options {
        margin-left: -32px;
        margin-bottom: 4px;
        opacity: 0.55;
    }
    .n-thing {
        .n-thing-header {
            margin-bottom: 0px;
        }
        .n-thing-avatar-header-wrapper {
            align-items: center;
        }
    }
}
</style>
