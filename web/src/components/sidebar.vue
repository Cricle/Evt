<template>
    <div class="sidebar-wrap">
        <div class="logo-wrap">
            <n-image class="logo-img" width="36" :src="LOGO" :preview-disabled="true" @click="goHome" />
        </div>

        <n-menu :accordion="true" :icon-size="24" :options="menuOptions" :render-label="renderMenuLabel"
            :render-icon="renderMenuIcon" :value="selectedPath" @update:value="goRouter" />

        <div class="user-wrap user-wrap-bottom" v-if="userInfo.id > 0">
            <n-avatar class="user-avatar" round :size="34" :src="userInfo.avatar || DEFAULT_USER_AVATAR" />

            <div class="user-info">
                <div class="nickname">
                    <span class="nickname-txt">
                        {{ userInfo.nickname }}
                    </span>
                    <n-button class="logout" quaternary circle size="tiny" @click="handleLogout">
                        <template #icon>
                            <n-icon>
                                <log-out-outline />
                            </n-icon>
                        </template>
                    </n-button>
                </div>
                <div class="username">@{{ userInfo.username }}</div>
            </div>

            <div class="user-mini-wrap">
                <n-button class="logout" quaternary circle @click="handleLogout">
                    <template #icon>
                        <n-icon :size="24">
                            <log-out-outline />
                        </n-icon>
                    </template>
                </n-button>
            </div>
        </div>
        <div class="user-wrap" v-else>
            <div v-if="!profile.allowUserRegister" class="login-only-wrap">
                <n-button strong secondary round type="primary" @click="goAuth('signin')">
                    登录
                </n-button>
            </div>
            <div v-if="profile.allowUserRegister" class="login-wrap">
                <n-button strong secondary round type="primary" @click="goAuth('signin')">
                    登录
                </n-button>
                <n-button strong round type="primary" ghost @click="goAuth('signup')">
                    注册
                </n-button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import LOGO from '@/assets/img/logo.png';
import { useI18n } from '@/i18n';
import { useStoreMain } from '@/store/main';
import { useStoreProfile } from '@/store/profile';
import { useStoreUser } from '@/store/user';
import { type AuthMode, goToAuth } from '@/utils/authRoute';
import { DEFAULT_USER_AVATAR } from '@/utils/defaults';
import { unreadLegacyMessageCount } from '@/utils/messageTransport';
import { buildHomeRouteWithSpace } from '@/utils/tagRoute';
import {
  ChatbubblesOutline,
  CompassOutline,
  PeopleOutline,
  ConstructOutline,
  HomeOutline,
  LogOutOutline,
  MegaphoneOutline,
  SettingsOutline,
  WalletOutline,
} from '@vicons/ionicons5';
import { Hash } from '@vicons/tabler';
import { NBadge, NIcon, useMessage } from 'naive-ui';
import { storeToRefs } from 'pinia';
import { computed, h, onBeforeUnmount, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';

const storeMain = useStoreMain();
const storeUser = useStoreUser();
const storeProfile = useStoreProfile();
const { unreadMsgCount } = storeToRefs(storeMain);
const { userInfo } = storeToRefs(storeUser);
const { profile, currentSpaceSlug } = storeToRefs(storeProfile);

const route = useRoute();
const router = useRouter();
const { t } = useI18n();
const hasUnreadMsg = ref(false);
const selectedPath = ref<string>((route.name as string) || '');
const msgLoop = ref<ReturnType<typeof setInterval>>();

const enableAnnouncement =
  (import.meta.env.VITE_ENABLE_ANNOUNCEMENT ??
    import.meta.env.VITE_ENABLE_ANOUNCEMENT).toLowerCase() === 'true';

watch(route, () => {
  selectedPath.value = route.name;
});
const syncUnreadMessages = () => {
  unreadLegacyMessageCount()
    .then((res) => {
      hasUnreadMsg.value = res.count > 0;
      storeMain.updateUnreadMsgCount(res.count);
    })
    .catch(() => {});
};

const clearMsgLoop = () => {
  if (msgLoop.value) {
    clearInterval(msgLoop.value);
    msgLoop.value = undefined;
  }
};

watch(
  [() => unreadMsgCount.value, () => userInfo.value.id, () => profile.value.defaultMsgLoopInterval],
  () => {
    hasUnreadMsg.value = unreadMsgCount.value > 0;

    if (userInfo.value.id <= 0) {
      clearMsgLoop();
      return;
    }

    clearMsgLoop();
    syncUnreadMessages();
    msgLoop.value = setInterval(syncUnreadMessages, profile.value.defaultMsgLoopInterval);
  },
  { immediate: true },
);

onBeforeUnmount(() => {
  clearMsgLoop();
});
const menuOptions = computed(() => {
  const options = [
    {
      label: t('route_landing'),
      key: 'home',
      icon: () => h(CompassOutline),
      href: '/',
    },
    {
      label: t('nav_home'),
      key: 'space',
      icon: () => h(HomeOutline),
      href: '/space',
    },
    {
      label: t('nav_topic'),
      key: 'topic',
      icon: () => h(Hash),
      href: '/topic',
    },
  ];
  if (enableAnnouncement) {
    options.push({
      label: t('nav_announcement'),
      key: 'announcement',
      icon: () => h(MegaphoneOutline),
      href: '/announcement',
    });
  }
  options.push({
    label: t('nav_messages'),
    key: 'messages',
    icon: () => h(ChatbubblesOutline),
    href: '/messages',
  });
  if (profile.value.useFriendship) {
    options.push({
      label: t('nav_contacts'),
      key: 'contacts',
      icon: () => h(PeopleOutline),
      href: '/contacts',
    });
  }
  if (profile.value.enableWallet) {
    options.push({
      label: t('nav_wallet'),
      key: 'wallet',
      icon: () => h(WalletOutline),
      href: '/wallet',
    });
  }
  options.push({
    label: t('nav_settings'),
    key: 'setting',
    icon: () => h(SettingsOutline),
    href: '/setting',
  });
  if (userInfo.value.is_admin) {
    options.push({
      label: t('nav_admin_settings'),
      key: 'admin-settings',
      icon: () => h(ConstructOutline),
      href: '/admin/settings',
    });
  }

  return userInfo.value.id > 0
    ? options
    : [
        {
          label: t('route_landing'),
          key: 'home',
          icon: () => h(CompassOutline),
          href: '/',
        },
        {
          label: t('nav_home'),
          key: 'space',
          icon: () => h(HomeOutline),
          href: '/space',
        },
        {
          label: t('nav_topic'),
          key: 'topic',
          icon: () => h(Hash),
          href: '/topic',
        },
      ];
});

const renderMenuLabel = (option: AnyObject) => {
  if ('href' in option) {
    return h('div', {}, option.label);
  }
  return option.label;
};
const renderMenuIcon = (option: AnyObject) => {
  if (option.key === 'messages') {
    return h(
      NBadge,
      {
        dot: true,
        show: hasUnreadMsg.value,
        processing: true,
      },
      {
        default: () =>
          h(
            NIcon,
            {
              color:
                option.key === selectedPath.value
                  ? 'var(--n-item-icon-color-active)'
                  : 'var(--n-item-icon-color)',
            },
            { default: option.icon },
          ),
      },
    );
  }
  return h(NIcon, null, { default: option.icon });
};

const goRouter = (name: string, item: AnyObject = {}) => {
  selectedPath.value = name;
  const keepSpace =
    name === 'space' || name === 'topic' || name === 'compose' || name === 'create-space';
  if (name === 'home') {
    router.push({
      name: 'home',
    });
    return;
  }
  if (name === 'space') {
    router.push(
      buildHomeRouteWithSpace(
        {
          t: new Date().getTime(),
        },
        currentSpaceSlug.value,
      ),
    );
    return;
  }
  router.push({
    name,
    query: {
      t: new Date().getTime(),
      ...(keepSpace && currentSpaceSlug.value
        ? {
            space: currentSpaceSlug.value,
          }
        : {}),
    },
  });
};
const goHome = () => {
  if (route.name === 'home') {
    storeMain.doRefresh();
  }
  goRouter('home');
};
const goAuth = (mode: AuthMode) => {
  goToAuth(router, mode, router.currentRoute.value.fullPath);
};
const handleLogout = () => {
  storeUser.userLogout();
  storeMain.doRefresh();
  goHome();
};
window.$message = useMessage();
</script>

<style lang="less">
.sidebar-wrap::-webkit-scrollbar {
    width: 0;
    /* 隐藏滚动条的宽度 */
    height: 0;
    /* 隐藏滚动条的高度 */
}

.sidebar-wrap {
    z-index: 99;
    width: var(--layout-sidebar-width);
    height: 100vh;
    position: sticky;
    top: 0;
    right: auto;
    padding: var(--layout-edge-offset) 0 18px;
    box-sizing: border-box;
    max-height: 100vh;
    overflow: auto;
    display: flex;
    flex-direction: column;
    border-radius: 0;
    border: 0;
    background: transparent;
    box-shadow: none;
    backdrop-filter: none;

    .n-menu .n-menu-item-content::before {
        border-radius: 21px;
    }

    .logo-wrap {
        display: flex;
        justify-content: flex-start;
        margin-bottom: 10px;

        .logo-img {
            margin-left: 24px;
            border-radius: 0;

            &:hover {
                cursor: pointer;
            }
        }
    }

    .user-wrap {
        display: flex;
        align-items: center;
        position: static;
        margin-top: auto;
        padding: 0;
        border-radius: 0;
        background: transparent;
        box-shadow: none;

        .user-mini-wrap {
            display: none;
        }

        .user-avatar {
            margin-right: 8px;
            flex: 0 0 auto;
        }

        .user-info {
            display: flex;
            flex-direction: column;
            min-width: 0;

            .nickname {
                font-size: 15px;
                font-weight: 600;
                line-height: 16px;
                height: 16px;
                margin-bottom: 2px;
                display: flex;
                align-items: center;

                .nickname-txt {
                    max-width: 90px;
                    text-overflow: ellipsis;
                    overflow: hidden;
                    white-space: nowrap;
                }

                .logout {
                    margin-left: 6px;
                }
            }

            .username {
                font-size: 14px;
                line-height: 16px;
                height: 16px;
                max-width: 120px;
                text-overflow: ellipsis;
                overflow: hidden;
                white-space: nowrap;
                opacity: 0.75;
            }
        }

        .login-only-wrap {
            display: flex;
            justify-content: center;
            width: 100%;

            button {
                margin: 0 4px;
                width: 80%;
            }
        }

        .login-wrap {
            display: flex;
            justify-content: center;
            width: 100%;

            button {
                margin: 0 4px;
            }
        }
    }
}

.auth-card {
    .n-card-header {
        z-index: 999;
    }
}

.mobile .sidebar-wrap {
    width: 100%;
    max-width: 100%;
    right: auto;
    left: 0;
    position: relative;
    top: 0;
    height: auto;
    min-height: 100%;
    max-height: none;
    padding: 10px 0 16px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border-radius: 0;
    border: 0;
    background: transparent;
    box-shadow: none;
    backdrop-filter: none;
}

.mobile .logo-wrap {
    .logo-img {
        margin-left: 10px !important;
    }
}

.mobile .user-wrap {
    position: static;
    margin-top: auto;
    padding: 0 10px;

    .user-avatar,
    .user-info,
    .login-only-wrap,
    .login-wrap {
        margin-bottom: 0;
    }
}

.mobile .user-wrap-bottom {
    display: none !important;
}

@media screen and (max-width: 821px) {
    .sidebar-wrap {
        width: 100%;
        max-width: 100%;
        right: auto;
        left: 0;
        position: relative;
        top: 0;
        height: auto;
        min-height: 100%;
        max-height: none;
        padding: 10px 0 16px;
        display: flex;
        flex-direction: column;
        overflow: hidden;
        border-radius: 0;
        border: 0;
        background: transparent;
        box-shadow: none;
        backdrop-filter: none;
    }

    .logo-wrap {
        .logo-img {
            margin-left: 10px !important;
        }
    }

    .user-wrap {
        position: static;
        margin-top: auto;
        padding: 0 10px;

        .user-avatar,
        .user-info,
        .login-only-wrap,
        .login-wrap {
            margin-bottom: 0;
        }

    }
    .user-wrap-bottom {
        display: none !important;
    }
}
</style>
