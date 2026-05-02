<template>
  <div class="announcement-page">
    <main-nav title="公告" />

    <div class="announcement-shell">
      <section class="announcement-hero">
        <div class="hero-copy">
          <span class="hero-badge">实例公告</span>
          <h1>当前站点能力与使用说明</h1>
          <p>
            这里展示当前实例开放的功能、默认广场以及基础使用约定，便于新用户快速了解站点状态。
          </p>
        </div>
        <div class="hero-meta">
          <n-tag round size="large" type="primary">
            默认广场：{{ displayDefaultSpace }}
          </n-tag>
          <n-space size="small" wrap>
            <n-tag
              v-for="item in enabledHighlights"
              :key="item.label"
              round
              size="small"
              type="success"
            >
              {{ item.label }}
            </n-tag>
          </n-space>
        </div>
      </section>

      <n-grid class="announcement-grid" cols="1 s:1 m:2" responsive="screen" :x-gap="16" :y-gap="16">
        <n-grid-item>
          <n-card class="announcement-card" title="功能开关" :bordered="false">
            <div class="feature-list">
              <div v-for="item in featureRows" :key="item.label" class="feature-row">
                <div class="feature-copy">
                  <span>{{ item.label }}</span>
                  <small>{{ item.description }}</small>
                </div>
                <n-tag round size="small" :type="item.enabled ? 'success' : 'default'">
                  {{ item.enabled ? '已开启' : '未开启' }}
                </n-tag>
              </div>
            </div>
          </n-card>
        </n-grid-item>

        <n-grid-item>
          <n-card class="announcement-card" title="默认规则" :bordered="false">
            <div class="rule-list">
              <div class="rule-item">
                <span>默认可见性</span>
                <strong>{{ visibilityLabel }}</strong>
              </div>
              <div class="rule-item">
                <span>动态最大字数</span>
                <strong>{{ profile.defaultTweetMaxLength }} 字</strong>
              </div>
              <div class="rule-item">
                <span>桌面端折叠阈值</span>
                <strong>{{ profile.tweetWebEllipsisSize }} 字</strong>
              </div>
              <div class="rule-item">
                <span>移动端折叠阈值</span>
                <strong>{{ profile.tweetMobileEllipsisSize }} 字</strong>
              </div>
              <div class="rule-item">
                <span>消息轮询间隔</span>
                <strong>{{ profile.defaultMsgLoopInterval / 1000 }} 秒</strong>
              </div>
            </div>
          </n-card>
        </n-grid-item>

        <n-grid-item>
          <n-card class="announcement-card" title="使用建议" :bordered="false">
            <div class="tips-list">
              <div v-for="tip in usageTips" :key="tip.title" class="tip-item">
                <strong>{{ tip.title }}</strong>
                <p>{{ tip.copy }}</p>
              </div>
            </div>
          </n-card>
        </n-grid-item>

        <n-grid-item>
          <n-card class="announcement-card" title="站点信息" :bordered="false">
            <div class="site-meta">
              <div class="site-meta-item">
                <span>顶部署名</span>
                <strong>{{ profile.copyrightTop || '未设置' }}</strong>
              </div>
              <div class="site-meta-item">
                <span>左侧链接</span>
                <a
                  v-if="profile.copyrightLeftLink"
                  class="announcement-link"
                  :href="profile.copyrightLeftLink"
                  target="_blank"
                  rel="noreferrer"
                >
                  {{ profile.copyrightLeft || profile.copyrightLeftLink }}
                </a>
                <strong v-else>{{ profile.copyrightLeft || '未设置' }}</strong>
              </div>
              <div class="site-meta-item">
                <span>右侧链接</span>
                <a
                  v-if="profile.copyrightRightLink"
                  class="announcement-link"
                  :href="profile.copyrightRightLink"
                  target="_blank"
                  rel="noreferrer"
                >
                  {{ profile.copyrightRight || profile.copyrightRightLink }}
                </a>
                <strong v-else>{{ profile.copyrightRight || '未设置' }}</strong>
              </div>
            </div>
          </n-card>
        </n-grid-item>
      </n-grid>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { storeToRefs } from 'pinia';
import { useStoreProfile } from '@/store/profile';
import { getSiteProfile } from '@/api/site';

const storeProfile = useStoreProfile();
const { profile } = storeToRefs(storeProfile);

const enabledHighlights = computed(() => {
  const entries = [
    { label: '广场', enabled: profile.value.enableSpaces },
    { label: '好友', enabled: profile.value.useFriendship },
    { label: '趋势栏', enabled: profile.value.enableTrendsBar },
    { label: '钱包', enabled: profile.value.enableWallet },
    { label: '图片附件', enabled: profile.value.allowTweetAttachment },
    { label: '视频发布', enabled: profile.value.allowTweetVideo },
    { label: '开放注册', enabled: profile.value.allowUserRegister },
  ];
  return entries.filter((item) => item.enabled);
});

const featureRows = computed(() => [
  {
    label: '广场系统',
    description: '支持默认公共广场和自定义广场切换',
    enabled: profile.value.enableSpaces,
  },
  {
    label: '好友关系',
    description: '启用后可添加好友并使用好友可见能力',
    enabled: profile.value.useFriendship,
  },
  {
    label: '趋势栏',
    description: '在首页右侧展示热门信息和附加导航',
    enabled: profile.value.enableTrendsBar,
  },
  {
    label: '钱包功能',
    description: '允许查看余额、账单与充值能力',
    enabled: profile.value.enableWallet,
  },
  {
    label: '图片与附件',
    description: '发布动态时可携带图片和文件',
    enabled: profile.value.allowTweetAttachment,
  },
  {
    label: '视频发布',
    description: '发布动态时可携带视频内容',
    enabled: profile.value.allowTweetVideo,
  },
  {
    label: '用户注册',
    description: '关闭时仅允许已有账号登录',
    enabled: profile.value.allowUserRegister,
  },
  {
    label: '手机号绑定',
    description: '个人设置页支持手机号能力',
    enabled: profile.value.allowPhoneBind,
  },
]);

const visibilityLabel = computed(() => {
  switch ((profile.value.defaultTweetVisibility || '').toLowerCase()) {
    case 'public':
      return '公开';
    case 'following':
      return '关注可见';
    case 'friend':
      return '好友可见';
    case 'private':
      return '仅自己可见';
    default:
      return profile.value.defaultTweetVisibility || '未设置';
  }
});

const displayDefaultSpace = computed(() => {
  if (!profile.value.defaultSpaceSlug || profile.value.defaultSpaceSlug === 'public') {
    return '公共广场';
  }
  return profile.value.defaultSpaceSlug;
});

const usageTips = computed(() => [
  {
    title: '优先在默认广场开始',
    copy: `当前实例默认把新内容发布到“${displayDefaultSpace.value}”，便于所有用户快速参与。`,
  },
  {
    title: '表情回复是核心互动',
    copy: '动态下方支持聚合表情回应，适合快速反馈；文字评论更适合补充完整上下文。',
  },
  {
    title: '遵循当前实例能力发布内容',
    copy: '如果附件、视频或钱包能力未开启，对应入口会自动受限，请以本页配置为准。',
  },
]);

onMounted(async () => {
  try {
    const latestProfile = await getSiteProfile();
    storeProfile.updateSiteProfile(latestProfile);
  } catch (_err) {
    // Ignore refresh failures and keep the locally cached site profile.
  }
});
</script>

<style lang="less" scoped>
.announcement-page {
  --announcement-page-bg:
    radial-gradient(circle at top left, var(--accent-soft), transparent 28%),
    radial-gradient(circle at top right, var(--accent-soft-muted), transparent 24%),
    linear-gradient(180deg, transparent 0%, transparent 100%);
  min-height: 100vh;
  background: var(--announcement-page-bg);
}

.announcement-shell {
  max-width: 980px;
  margin: 0 auto;
  padding: 20px 16px 72px;
}

.announcement-hero {
  display: grid;
  gap: 16px;
  margin-bottom: 18px;
  padding: 24px;
  border: 1px solid var(--panel-border);
  border-radius: 28px;
  background: var(--panel-bg);
  box-shadow: var(--panel-shadow);
}

.hero-copy {
  display: grid;
  gap: 10px;

  h1 {
    margin: 0;
    font-size: 30px;
    line-height: 1.1;
  }

  p {
    margin: 0;
    max-width: 700px;
    opacity: 0.78;
    line-height: 1.7;
  }
}

.hero-badge {
  display: inline-flex;
  width: fit-content;
  padding: 6px 12px;
  border-radius: 999px;
  background: var(--accent-soft);
  color: var(--accent-primary);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.08em;
}

.hero-meta {
  display: grid;
  gap: 12px;
}

.announcement-card {
  height: 100%;
  border-radius: 24px;
  background: var(--panel-bg);
  box-shadow: var(--panel-shadow);
}

.feature-list,
.rule-list,
.tips-list,
.site-meta {
  display: grid;
  gap: 12px;
}

.feature-row,
.rule-item,
.site-meta-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 0;
  border-bottom: 1px solid var(--border-subtle);
}

.feature-row:last-child,
.rule-item:last-child,
.site-meta-item:last-child {
  border-bottom: 0;
  padding-bottom: 0;
}

.feature-copy {
  display: grid;
  gap: 4px;

  small {
    opacity: 0.72;
    line-height: 1.5;
  }
}

.rule-item strong,
.site-meta-item strong {
  text-align: right;
}

.tip-item {
  padding: 14px 16px;
  border-radius: 18px;
  background: var(--surface-elevated);

  strong {
    display: block;
    margin-bottom: 6px;
  }

  p {
    margin: 0;
    opacity: 0.76;
    line-height: 1.7;
  }
}

.announcement-link {
  color: var(--accent-link);
  text-decoration: none;

  &:hover {
    opacity: 0.82;
  }
}

@media screen and (max-width: 821px) {
  .announcement-shell {
    padding: 12px 10px 80px;
  }

  .announcement-hero {
    padding: 18px;
    border-radius: 22px;
  }

  .hero-copy h1 {
    font-size: 24px;
  }

  .feature-row,
  .rule-item,
  .site-meta-item {
    align-items: flex-start;
    flex-direction: column;
  }

  .rule-item strong,
  .site-meta-item strong {
    text-align: left;
  }
}
</style>
