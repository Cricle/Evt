import { createRouter, createWebHashHistory } from 'vue-router';
import { getRouteTitleKey, translate } from '@/i18n';
import { useStoreMain } from '@/store/main';

const routes = [
  {
    path: '/',
    name: 'home',
    meta: {
      title: '首页',
    },
    component: () => import('@/views/Home.vue'),
  },
  {
    path: '/space',
    name: 'space',
    meta: {
      title: '广场',
      keepAlive: true,
    },
    component: () => import('@/views/Space.vue'),
  },
  {
    path: '/auth',
    name: 'auth',
    meta: {
      title: '登录',
    },
    component: () => import('@/views/Auth.vue'),
  },
  {
    path: '/compose',
    name: 'compose',
    meta: {
      title: '发布动态',
    },
    component: () => import('@/views/Compose.vue'),
  },
  {
    path: '/spaces/create',
    name: 'create-space',
    meta: {
      title: '新建广场',
    },
    component: () => import('@/views/CreateSpace.vue'),
  },
  {
    path: '/post',
    name: 'post',
    meta: {
      title: '动态详情',
    },
    component: () => import('@/views/Post.vue'),
  },
  {
    path: '/topic',
    name: 'topic',
    meta: {
      title: '话题',
    },
    component: () => import('@/views/Topic.vue'),
  },
  {
    path: '/announcement',
    alias: ['/anouncement'],
    name: 'announcement',
    meta: {
      title: '公告',
    },
    component: () => import('@/views/Announcement.vue'),
  },
  {
    path: '/profile',
    name: 'profile',
    meta: {
      title: '设置',
    },
    redirect: (to) => ({
      name: 'setting',
      query: to.query,
    }),
  },
  {
    path: '/u',
    name: 'user',
    meta: {
      title: '用户详情',
    },
    component: () => import('@/views/User.vue'),
  },
  {
    path: '/messages',
    name: 'messages',
    meta: {
      title: '消息',
    },
    component: () => import('@/views/Messages.vue'),
  },
  {
    path: '/contacts',
    name: 'contacts',
    meta: {
      title: '好友',
    },
    component: () => import('@/views/Contacts.vue'),
  },
  {
    path: '/following',
    name: 'following',
    meta: {
      title: '关注',
    },
    component: () => import('@/views/Following.vue'),
  },
  {
    path: '/wallet',
    name: 'wallet',
    meta: {
      title: '钱包',
    },
    component: () => import('@/views/Wallet.vue'),
  },
  {
    path: '/setting',
    name: 'setting',
    meta: {
      title: '设置',
    },
    component: () => import('@/views/Setting.vue'),
  },
  {
    path: '/admin/settings',
    name: 'admin-settings',
    meta: {
      title: '系统配置',
    },
    component: () => import('@/views/AdminSettings.vue'),
  },
  {
    path: '/404',
    name: '404',
    meta: {
      title: '404',
    },
    component: () => import('@/views/404.vue'),
  },
  {
    path: '/:pathMatch(.*)',
    redirect: '/404',
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

router.beforeEach((to, from, next) => {
  const titleKey = getRouteTitleKey(to.name);
  const locale = useStoreMain().locale;
  const pageTitle = titleKey
    ? translate(locale, titleKey, `${to.meta.title ?? ''}`)
    : `${to.meta.title ?? ''}`;
  const suffix = `${translate(locale, 'appName', 'Evt')} - ${translate(locale, 'appTagline')}`;
  document.title = `${pageTitle} | ${suffix}`;
  next();
});

export default router;
