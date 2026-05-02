import { defineConfig } from 'vite';
import path from 'path';
import vue from '@vitejs/plugin-vue';
import Components from 'unplugin-vue-components/vite';

import { NaiveUiResolver } from 'unplugin-vue-components/resolvers';
// https://vitejs.dev/config/
export default defineConfig({
  server: {
    host: '0.0.0.0',
  },
  plugins: [
    vue(),
    Components({
      resolvers: [NaiveUiResolver()],
    }),
    // esmExternalRequirePlugin({
    //   external: [/^node:/]
    // }),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, 'src'),
    },
  },
  build: {
    chunkSizeWarningLimit: 2200,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (!id.includes('node_modules')) {
            return;
          }

          if (id.includes('@ckeditor')) {
            return 'ckeditor';
          }

          if (
            id.includes('naive-ui') ||
            id.includes('@vicons') ||
            id.includes('vooks') ||
            id.includes('vueuc') ||
            id.includes('@css-render')
          ) {
            return 'ui-vendor';
          }

          if (id.includes('/vue/') || id.includes('/vue-router/') || id.includes('/pinia/')) {
            return 'vue-vendor';
          }

          return id
            .toString()
            .split('node_modules/')[1]
            .split('/')[0]
            .toString();
        },
      },
    },
  },
});
