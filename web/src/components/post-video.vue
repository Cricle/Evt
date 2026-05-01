<template>
    <div v-if="props.videos.length > 0">
        <div class="video-grid" :class="{ full }">
            <div v-for="video in props.videos" :key="video.id" class="video-card" @click.stop>
                <video class="video-player" :src="video.content" controls preload="metadata" playsinline />
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
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
  --post-video-card-border: rgba(18, 75, 51, 0.1);
  --post-video-card-bg: linear-gradient(180deg, rgba(20, 70, 48, 0.08), rgba(20, 70, 48, 0.02));
  overflow: hidden;
  border: 1px solid var(--post-video-card-border);
  border-radius: 18px;
  background: var(--post-video-card-bg);
}

.video-player {
  --post-video-player-bg: #06120c;
  display: block;
  width: 100%;
  max-height: 480px;
  background: var(--post-video-player-bg);
}

:global(.dark) .video-card {
  --post-video-card-border: rgba(148, 163, 184, 0.16);
  --post-video-card-bg: linear-gradient(180deg, rgba(20, 28, 28, 0.88), rgba(14, 18, 18, 0.94));
}

:global(.dark) .video-player {
  --post-video-player-bg: #040908;
}
</style>
