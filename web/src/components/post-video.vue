<template>
    <div v-if="props.videos.length > 0">
        <div class="video-grid" :class="{ full }">
            <div v-for="video in props.videos" :key="video.id" class="video-card" @click.stop>
                <video class="video-player" :src="toVideoUrl(video.content)" controls preload="metadata" playsinline />
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { buildApiUrl } from '@/utils/api';

const props = withDefaults(
  defineProps<{
    videos: Item.PostItemProps[];
    full?: boolean;
  }>(),
  {
    videos: () => [],
    full: false,
  },
);

const toVideoUrl = (value: string) => {
  const normalized = (value || '').trim();
  if (!normalized) {
    return '';
  }
  if (/^(https?:)?\/\//i.test(normalized) || normalized.startsWith('blob:') || normalized.startsWith('data:')) {
    return normalized;
  }
  return buildApiUrl(normalized.startsWith('/') ? normalized : `/${normalized}`);
};
</script>

<style scoped lang="less">
.video-grid {
  display: grid;
  gap: 12px;
  grid-template-columns: repeat(1, minmax(0, 1fr));
}

.video-grid:not(.full) {
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
}

.video-card {
  --post-video-card-border: var(--border-subtle);
  --post-video-card-bg: linear-gradient(180deg, var(--accent-soft), transparent);
  overflow: hidden;
  border: 1px solid var(--post-video-card-border);
  border-radius: 18px;
  background: var(--post-video-card-bg);
}

.video-player {
  --post-video-player-bg: var(--surface-subtle);
  display: block;
  width: 100%;
  max-height: 480px;
  background: var(--post-video-player-bg);
}
</style>
