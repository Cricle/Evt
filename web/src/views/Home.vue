<template>
  <div class="landing-page">
    <main-nav :title="t('route_landing')" />

    <section class="landing-shell">
      <div class="hero-grid">
        <div class="hero-copy">
          <n-space vertical size="large">
            <n-space align="center" :size="10">
              <n-tag round :bordered="false" type="success" size="small">Evt</n-tag>
              <span class="eyebrow">{{ t('landing_eyebrow') }}</span>
            </n-space>

            <div class="hero-text">
              <h1>{{ t('landing_title') }}</h1>
              <p>{{ t('landing_desc') }}</p>
            </div>

            <n-space wrap class="hero-actions">
              <n-button type="primary" size="large" @click="goSpace">
                {{ t('landing_cta_space') }}
              </n-button>
              <n-button v-if="userLogined" secondary size="large" @click="goCompose">
                {{ t('landing_cta_compose') }}
              </n-button>
              <n-button v-else quaternary size="large" @click="goAuth('signup')">
                {{ t('landing_cta_signup') }}
              </n-button>
            </n-space>

          <n-space wrap size="small">
              <n-tag round :bordered="false">{{ t('landing_chip_spaces') }}</n-tag>
              <n-tag round :bordered="false">{{ t('landing_chip_emoji') }}</n-tag>
              <n-tag round :bordered="false">{{ t('landing_chip_realtime') }}</n-tag>
              <n-tag round :bordered="false">{{ t('landing_chip_rust') }}</n-tag>
            </n-space>

            <div class="hero-lower">
              <div class="hero-summary">
                <div class="hero-summary-title">{{ t('landing_summary_title') }}</div>
                <div class="hero-summary-grid">
                  <div class="summary-item">
                    <span class="summary-badge">01</span>
                    <div>
                      <div class="summary-title">{{ t('landing_summary_one_title') }}</div>
                      <div class="summary-desc">{{ t('landing_summary_one_desc') }}</div>
                    </div>
                  </div>
                  <div class="summary-item">
                    <span class="summary-badge">02</span>
                    <div>
                      <div class="summary-title">{{ t('landing_summary_two_title') }}</div>
                      <div class="summary-desc">{{ t('landing_summary_two_desc') }}</div>
                    </div>
                  </div>
                  <div class="summary-item">
                    <span class="summary-badge">03</span>
                    <div>
                      <div class="summary-title">{{ t('landing_summary_three_title') }}</div>
                      <div class="summary-desc">{{ t('landing_summary_three_desc') }}</div>
                    </div>
                  </div>
                </div>
              </div>

              <div class="landing-metrics">
                <div
                  v-for="(item, index) in landingMetrics"
                  :key="item.label"
                  class="landing-metric-card"
                  :class="{ 'landing-metric-card-primary': index === 0 }"
                >
                  <div class="landing-metric-kicker">{{ item.label }}</div>
                  <div class="landing-metric-value">{{ item.value }}</div>
                  <div v-if="index === 0 && publicSpace" class="landing-metric-note">
                    {{ publicSpace.name }}
                  </div>
                  <div v-else class="landing-metric-label">{{ item.label }}</div>
                </div>
              </div>
            </div>
          </n-space>
        </div>

        <n-card class="hero-panel" :bordered="false">
          <n-space vertical size="large">
            <div class="panel-head">
              <div>
                <div class="panel-title">{{ t('landing_panel_title') }}</div>
                <div class="panel-subtitle">{{ t('landing_panel_desc') }}</div>
              </div>
              <n-tag round type="success" :bordered="false">{{ t('landing_panel_badge') }}</n-tag>
            </div>

            <div class="entry-flow">
              <div class="flow-step">
                <div class="flow-step-index">1</div>
                <div>
                  <div class="flow-step-title">{{ t('landing_flow_one_title') }}</div>
                  <div class="flow-step-desc">{{ t('landing_flow_one_desc') }}</div>
                </div>
              </div>
              <div class="flow-step">
                <div class="flow-step-index">2</div>
                <div>
                  <div class="flow-step-title">{{ t('landing_flow_two_title') }}</div>
                  <div class="flow-step-desc">{{ t('landing_flow_two_desc') }}</div>
                </div>
              </div>
              <div class="flow-step">
                <div class="flow-step-index">3</div>
                <div>
                  <div class="flow-step-title">{{ t('landing_flow_three_title') }}</div>
                  <div class="flow-step-desc">{{ t('landing_flow_three_desc') }}</div>
                </div>
              </div>
            </div>

            <div class="reaction-preview">
              <div class="reaction-preview-title">{{ t('landing_reaction_preview') }}</div>
              <div class="reaction-preview-desc">{{ t('landing_reaction_preview_desc') }}</div>
              <div class="reaction-preview-row">
                <span class="reaction-pill">😀</span>
                <span class="reaction-pill">🔥</span>
                <span class="reaction-pill">👏</span>
                <span class="reaction-pill">🤝</span>
                <span class="reaction-pill">🎉</span>
                <span class="reaction-pill">💡</span>
              </div>
            </div>

            <div class="default-space-panel" v-if="publicSpace">
              <div class="default-space-head">
                <div>
                  <div class="default-space-kicker">{{ t('landing_default_space_kicker') }}</div>
                  <div class="default-space-name">{{ publicSpace.name }}</div>
                </div>
                <n-tag round size="small" :bordered="false" type="success">
                  {{ publicSpace.visibility === 'private' ? t('landing_space_private') : t('landing_space_public') }}
                </n-tag>
              </div>
              <div class="default-space-desc">
                {{ publicSpace.description || t('landing_default_space_desc_fallback') }}
              </div>
              <div class="default-space-meta">
                <span>{{ t('landing_default_space_members') }} {{ publicSpace.members_count }}</span>
                <span>·</span>
                <span>{{ t('landing_default_space_slug') }} {{ publicSpace.slug }}</span>
              </div>
            </div>
          </n-space>
        </n-card>
      </div>

      <div class="spotlight-grid">
        <n-card :bordered="false" class="spotlight-card spotlight-card-wide">
          <div class="spotlight-head">
            <div>
              <div class="section-kicker">{{ t('landing_spotlight_spaces') }}</div>
              <div class="section-title">{{ t('landing_spotlight_spaces_desc') }}</div>
            </div>
            <n-button tertiary round @click="goSpace">
              {{ t('landing_cta_space') }}
            </n-button>
          </div>

          <div class="spotlight-space-shell" v-if="publicSpace">
            <div class="spotlight-space-main">
              <div class="spotlight-space-kicker">{{ t('landing_spotlight_default_space') }}</div>
              <div class="spotlight-space-name">{{ publicSpace.name }}</div>
              <div class="spotlight-space-desc">
                {{ publicSpace.description || t('landing_default_space_desc_fallback') }}
              </div>
            </div>

            <div class="spotlight-space-stats">
              <div class="spotlight-stat">
                <span>{{ t('landing_spotlight_members') }}</span>
                <strong>{{ publicSpace.members_count }}</strong>
              </div>
              <div class="spotlight-stat">
                <span>{{ t('landing_spotlight_visibility') }}</span>
                <strong>{{ publicSpace.visibility === 'private' ? t('landing_space_private') : t('landing_space_public') }}</strong>
              </div>
              <div class="spotlight-stat">
                <span>{{ t('landing_spotlight_mode') }}</span>
                <strong>{{ profile.enableSpaces ? t('landing_metric_enabled') : t('landing_metric_disabled') }}</strong>
              </div>
            </div>
          </div>
        </n-card>

        <n-card :bordered="false" class="spotlight-card">
          <div class="section-kicker">{{ t('landing_spotlight_presence') }}</div>
          <div class="spotlight-mini-title">{{ t('landing_spotlight_presence_desc') }}</div>
          <div class="spotlight-rule-list">
            <div class="spotlight-rule-item">
              <span class="spotlight-rule-dot">😀</span>
              <div>{{ t('landing_spotlight_rule_one') }}</div>
            </div>
            <div class="spotlight-rule-item">
              <span class="spotlight-rule-dot">💬</span>
              <div>{{ t('landing_spotlight_rule_two') }}</div>
            </div>
            <div class="spotlight-rule-item">
              <span class="spotlight-rule-dot">🏷️</span>
              <div>{{ t('landing_spotlight_rule_three') }}</div>
            </div>
          </div>
        </n-card>

        <n-card :bordered="false" class="spotlight-card">
          <div class="section-kicker">{{ t('landing_spotlight_runtime') }}</div>
          <div class="spotlight-mini-title">{{ t('landing_spotlight_runtime_desc') }}</div>
          <div class="spotlight-runtime-list">
            <div class="spotlight-runtime-item">{{ t('landing_spotlight_runtime_one') }}</div>
            <div class="spotlight-runtime-item">{{ t('landing_spotlight_runtime_two') }}</div>
            <div class="spotlight-runtime-item">{{ t('landing_spotlight_runtime_three') }}</div>
          </div>
        </n-card>
      </div>

      <div class="feature-section">
        <div class="section-head">
          <div class="section-kicker">{{ t('landing_section_capability') }}</div>
          <div class="section-title">{{ t('landing_section_capability_title') }}</div>
          <div class="section-desc">{{ t('landing_section_capability_desc') }}</div>
        </div>

        <div class="feature-grid">
          <n-card :bordered="false" class="feature-card feature-card-wide">
            <div class="feature-icon">🏠</div>
            <div class="feature-card-title">{{ t('landing_feature_space_title') }}</div>
            <div class="feature-card-desc">{{ t('landing_feature_space_desc') }}</div>
            <ul class="feature-points">
              <li>{{ t('landing_feature_space_point_one') }}</li>
              <li>{{ t('landing_feature_space_point_two') }}</li>
              <li>{{ t('landing_feature_space_point_three') }}</li>
            </ul>
          </n-card>

          <n-card :bordered="false" class="feature-card">
            <div class="feature-icon">😀</div>
            <div class="feature-card-title">{{ t('landing_feature_emoji_title') }}</div>
            <div class="feature-card-desc">{{ t('landing_feature_emoji_desc') }}</div>
            <ul class="feature-points">
              <li>{{ t('landing_feature_emoji_point_one') }}</li>
              <li>{{ t('landing_feature_emoji_point_two') }}</li>
            </ul>
          </n-card>

          <n-card :bordered="false" class="feature-card">
            <div class="feature-icon">💬</div>
            <div class="feature-card-title">{{ t('landing_feature_discussion_title') }}</div>
            <div class="feature-card-desc">{{ t('landing_feature_discussion_desc') }}</div>
            <ul class="feature-points">
              <li>{{ t('landing_feature_discussion_point_one') }}</li>
              <li>{{ t('landing_feature_discussion_point_two') }}</li>
            </ul>
          </n-card>

          <n-card :bordered="false" class="feature-card">
            <div class="feature-icon">🛠️</div>
            <div class="feature-card-title">{{ t('landing_feature_admin_title') }}</div>
            <div class="feature-card-desc">{{ t('landing_feature_admin_desc') }}</div>
            <ul class="feature-points">
              <li>{{ t('landing_feature_admin_point_one') }}</li>
              <li>{{ t('landing_feature_admin_point_two') }}</li>
            </ul>
          </n-card>

          <n-card :bordered="false" class="feature-card">
            <div class="feature-icon">🗂️</div>
            <div class="feature-card-title">{{ t('landing_feature_storage_title') }}</div>
            <div class="feature-card-desc">{{ t('landing_feature_storage_desc') }}</div>
            <ul class="feature-points">
              <li>{{ t('landing_feature_storage_point_one') }}</li>
              <li>{{ t('landing_feature_storage_point_two') }}</li>
            </ul>
          </n-card>

          <n-card :bordered="false" class="feature-card feature-card-wide">
            <div class="feature-icon">⚙️</div>
            <div class="feature-card-title">{{ t('landing_feature_arch_title') }}</div>
            <div class="feature-card-desc">{{ t('landing_feature_arch_desc') }}</div>
            <ul class="feature-points">
              <li>{{ t('landing_feature_arch_point_one') }}</li>
              <li>{{ t('landing_feature_arch_point_two') }}</li>
              <li>{{ t('landing_feature_arch_point_three') }}</li>
            </ul>
          </n-card>
        </div>
      </div>

      <div class="usecase-grid">
        <n-card :bordered="false" class="usecase-panel">
          <div class="section-kicker">{{ t('landing_section_scenario') }}</div>
          <div class="section-title">{{ t('landing_section_scenario_title') }}</div>
          <div class="section-desc">{{ t('landing_section_scenario_desc') }}</div>

          <div class="scenario-list">
            <div class="scenario-item">
              <div class="scenario-name">{{ t('landing_scenario_one_title') }}</div>
              <div class="scenario-desc">{{ t('landing_scenario_one_desc') }}</div>
            </div>
            <div class="scenario-item">
              <div class="scenario-name">{{ t('landing_scenario_two_title') }}</div>
              <div class="scenario-desc">{{ t('landing_scenario_two_desc') }}</div>
            </div>
            <div class="scenario-item">
              <div class="scenario-name">{{ t('landing_scenario_three_title') }}</div>
              <div class="scenario-desc">{{ t('landing_scenario_three_desc') }}</div>
            </div>
          </div>
        </n-card>

        <n-card :bordered="false" class="usecase-panel">
          <div class="section-kicker">{{ t('landing_section_deploy') }}</div>
          <div class="section-title">{{ t('landing_section_deploy_title') }}</div>
          <div class="section-desc">{{ t('landing_section_deploy_desc') }}</div>

          <div class="deploy-list">
            <div class="deploy-item">{{ t('landing_deploy_one') }}</div>
            <div class="deploy-item">{{ t('landing_deploy_two') }}</div>
            <div class="deploy-item">{{ t('landing_deploy_three') }}</div>
            <div class="deploy-item">{{ t('landing_deploy_four') }}</div>
          </div>
        </n-card>
      </div>

      <n-card :bordered="false" class="cta-panel">
        <div class="cta-copy">
          <div>
            <div class="section-kicker">{{ t('landing_section_ready') }}</div>
            <div class="cta-title">{{ t('landing_cta_title') }}</div>
            <div class="cta-desc">{{ t('landing_cta_desc') }}</div>
          </div>
          <n-space wrap>
            <n-button type="primary" size="large" @click="goSpace">
              {{ t('landing_cta_space') }}
            </n-button>
            <n-button v-if="!userLogined" secondary size="large" @click="goAuth('signup')">
              {{ t('landing_cta_signup') }}
            </n-button>
          </n-space>
        </div>
      </n-card>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { storeToRefs } from 'pinia';
import { useStoreProfile } from '@/store/profile';
import { useStoreUser } from '@/store/user';
import { buildComposeRoute, buildHomeRouteWithSpace } from '@/utils/tagRoute';
import { goToAuth, type AuthMode } from '@/utils/authRoute';
import { useI18n } from '@/i18n';

const router = useRouter();
const storeProfile = useStoreProfile();
const storeUser = useStoreUser();
const { currentSpaceSlug, profile, spaces } = storeToRefs(storeProfile);
const { userLogined, userInfo } = storeToRefs(storeUser);
const { t } = useI18n();

const publicSpace = computed(() => {
  return (
    spaces.value.find((item) => item.slug === profile.value.defaultSpaceSlug) ||
    spaces.value.find((item) => item.slug === currentSpaceSlug.value) ||
    spaces.value[0] ||
    null
  );
});

const landingMetrics = computed(() => [
  {
    value: spaces.value.length || 1,
    label: t('landing_metric_spaces'),
  },
  {
    value: profile.value.enableSpaces ? t('landing_metric_enabled') : t('landing_metric_disabled'),
    label: t('landing_metric_space_mode'),
  },
  {
    value: userLogined.value ? userInfo.value.nickname || userInfo.value.username : t('landing_metric_guest'),
    label: t('landing_metric_status'),
  },
]);

const goSpace = () => {
  router.push(buildHomeRouteWithSpace({}, currentSpaceSlug.value));
};

const goCompose = () => {
  router.push(buildComposeRoute(currentSpaceSlug.value));
};

const goAuth = (mode: AuthMode) => {
  goToAuth(router, mode, router.currentRoute.value.fullPath);
};
</script>

<style scoped lang="less">
.landing-page {
  min-height: 100dvh;
  background:
    radial-gradient(circle at 0% 0%, color-mix(in srgb, var(--page-hero-bg-glow) 100%, transparent), transparent 24%),
    radial-gradient(circle at 100% 12%, color-mix(in srgb, var(--page-hero-bg-accent) 100%, transparent), transparent 26%),
    radial-gradient(circle at 50% 100%, color-mix(in srgb, var(--accent-soft-muted) 70%, transparent), transparent 28%),
    linear-gradient(180deg, var(--page-hero-bg-base) 0%, var(--page-hero-bg-bottom) 100%);
}

.landing-shell {
  max-width: 1180px;
  margin: 0 auto;
  padding: 20px 18px 64px;
  display: grid;
  gap: 18px;
}

.hero-grid,
.usecase-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.12fr) minmax(320px, 0.88fr);
  gap: 20px;
}

.hero-copy,
.hero-panel,
.feature-card,
.usecase-panel,
.cta-panel {
  background: color-mix(in srgb, var(--panel-bg) 90%, transparent);
  border: var(--glass-panel-border);
  box-shadow:
    0 16px 40px color-mix(in srgb, var(--shadow-color, #0f172a) 8%, transparent),
    inset 0 1px 0 color-mix(in srgb, #ffffff 18%, transparent);
  backdrop-filter: blur(18px) saturate(128%);
}

.hero-copy {
  padding: 18px 8px 10px;
  position: relative;
}

.eyebrow,
.section-kicker {
  font-size: 12px;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  opacity: 0.62;
}

.hero-text {
  display: grid;
  gap: 14px;
}

.hero-text h1 {
  margin: 0;
  max-width: 760px;
  font-size: clamp(36px, 5.8vw, 64px);
  line-height: 1.02;
  letter-spacing: -0.03em;
}

.hero-text p {
  margin: 0;
  max-width: 680px;
  font-size: 15px;
  line-height: 1.9;
  opacity: 0.78;
}

.hero-lower {
  display: grid;
  grid-template-columns: minmax(0, 1.4fr) minmax(250px, 0.9fr);
  gap: 14px;
  align-items: stretch;
}

.hero-summary {
  padding: 16px;
  border-radius: 20px;
  background:
    linear-gradient(135deg, color-mix(in srgb, var(--accent-soft) 32%, transparent), transparent 70%),
    color-mix(in srgb, var(--accent-soft-muted) 84%, transparent);
  border: 1px solid color-mix(in srgb, var(--panel-border) 80%, transparent);
}

.landing-metrics {
  display: grid;
  grid-template-columns: repeat(1, minmax(0, 1fr));
  gap: 10px;
}

.landing-metric-card {
  padding: 14px 16px 15px;
  border-radius: 18px;
  background:
    linear-gradient(135deg, color-mix(in srgb, var(--accent-soft) 12%, transparent), transparent 88%),
    color-mix(in srgb, var(--panel-bg) 78%, transparent);
  border: 1px solid color-mix(in srgb, var(--panel-border) 70%, transparent);
  backdrop-filter: blur(10px);
}

.landing-metric-card-primary {
  background:
    linear-gradient(135deg, color-mix(in srgb, var(--accent-soft) 42%, transparent), transparent 80%),
    color-mix(in srgb, var(--panel-bg) 82%, transparent);
}

.landing-metric-kicker {
  font-size: 11px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  opacity: 0.56;
}

.landing-metric-value {
  margin-top: 8px;
  font-size: 24px;
  font-weight: 700;
  line-height: 1.1;
}

.landing-metric-note,
.landing-metric-label {
  margin-top: 6px;
  font-size: 12px;
  opacity: 0.7;
}

.hero-summary-title,
.reaction-preview-title {
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  opacity: 0.7;
}

.hero-summary-grid,
.entry-flow,
.scenario-list,
.deploy-list {
  margin-top: 12px;
  display: grid;
  gap: 12px;
}

.summary-item,
.flow-step,
.scenario-item {
  display: grid;
  grid-template-columns: 36px minmax(0, 1fr);
  gap: 12px;
  align-items: start;
}

.summary-badge,
.flow-step-index {
  width: 36px;
  height: 36px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
  background: color-mix(in srgb, var(--accent-soft) 100%, transparent);
  font-size: 13px;
  font-weight: 700;
}

.summary-title,
.flow-step-title,
.feature-card-title,
.scenario-name,
.panel-title,
.section-title,
.cta-title {
  font-weight: 700;
}

.summary-desc,
.flow-step-desc,
.panel-subtitle,
.feature-card-desc,
.section-desc,
.scenario-desc,
.cta-desc,
.reaction-preview-desc {
  margin-top: 6px;
  font-size: 13px;
  line-height: 1.72;
  opacity: 0.72;
}

.reaction-preview {
  padding: 14px;
  border-radius: 18px;
  background:
    linear-gradient(135deg, color-mix(in srgb, var(--accent-soft) 24%, transparent), transparent 72%),
    color-mix(in srgb, var(--accent-soft-muted) 92%, transparent);
}

.default-space-panel {
  padding: 16px;
  border-radius: 20px;
  border: 1px solid color-mix(in srgb, var(--panel-border) 80%, transparent);
  background:
    linear-gradient(135deg, color-mix(in srgb, var(--accent-soft) 48%, transparent), transparent 74%),
    color-mix(in srgb, var(--panel-bg) 84%, transparent);
}

.default-space-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.default-space-kicker {
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  opacity: 0.64;
}

.default-space-name {
  margin-top: 6px;
  font-size: 24px;
  font-weight: 700;
  line-height: 1.1;
}

.default-space-desc,
.default-space-meta {
  margin-top: 10px;
  font-size: 13px;
  line-height: 1.72;
  opacity: 0.76;
}

.default-space-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.reaction-preview-row {
  margin-top: 12px;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.reaction-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 32px;
  padding: 0 10px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent-soft) 100%, transparent);
  font-size: 15px;
  font-family: var(--emoji-font-stack);
}

.feature-section {
  display: grid;
  gap: 18px;
}

.section-head {
  max-width: 760px;
  display: grid;
  gap: 8px;
}

.feature-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 14px;
}

.feature-card {
  min-height: 100%;
  border-radius: 20px;
  position: relative;
  overflow: hidden;
}

.feature-card::before {
  content: '';
  position: absolute;
  inset: 0 0 auto;
  height: 72px;
  background: linear-gradient(180deg, color-mix(in srgb, var(--accent-soft) 24%, transparent), transparent);
  pointer-events: none;
}

.feature-card-wide {
  grid-column: span 2;
}

.feature-icon {
  width: 42px;
  height: 42px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 14px;
  background: color-mix(in srgb, var(--accent-soft) 100%, transparent);
  font-size: 21px;
}

.feature-card-desc {
  margin-bottom: 10px;
}

.feature-points {
  margin: 0;
  padding-left: 18px;
  display: grid;
  gap: 8px;
  font-size: 13px;
  line-height: 1.72;
  opacity: 0.84;
}

.section-title,
.cta-title {
  font-size: 28px;
  line-height: 1.1;
}

.deploy-list {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.deploy-item {
  padding: 12px;
  border-radius: 16px;
  background:
    linear-gradient(135deg, color-mix(in srgb, var(--accent-soft) 18%, transparent), transparent 82%),
    color-mix(in srgb, var(--accent-soft-muted) 90%, transparent);
  font-size: 13px;
  line-height: 1.7;
}

.cta-copy {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 22px;
  padding: 20px 22px;
  border-radius: 22px;
  background:
    linear-gradient(135deg, color-mix(in srgb, var(--accent-soft) 92%, transparent), transparent 60%),
    color-mix(in srgb, var(--panel-bg) 92%, transparent);
}

@media screen and (max-width: 980px) {
  .hero-grid,
  .usecase-grid,
  .feature-grid,
  .hero-lower {
    grid-template-columns: 1fr;
  }

  .feature-card-wide {
    grid-column: span 1;
  }
}

@media screen and (max-width: 821px) {
  .landing-shell {
    padding: 16px 10px 56px;
    gap: 14px;
  }

  .hero-text h1 {
    font-size: 34px;
    line-height: 1.02;
  }

  .hero-text p {
    font-size: 14px;
  }

  .deploy-list {
    grid-template-columns: 1fr;
  }

  .cta-copy {
    flex-direction: column;
  }
}
</style>
