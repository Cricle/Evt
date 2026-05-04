import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import '@/assets/css/main.less';
import { normalizeInitialHashRoute } from '@/utils/navigation';

import type { MessageApiInjection } from 'naive-ui/lib/message/src/MessageProvider';

// 通用字体
import 'vfonts/Lato.css';
// 等宽字体
import 'vfonts/FiraCode.css';

const pinia = createPinia();

const shouldDeferMount =
  typeof window !== 'undefined' && normalizeInitialHashRoute(window.location);

if (!shouldDeferMount) {
  import('./router').then(({ default: router }) => {
    createApp(App).use(pinia).use(router).mount('#app');
  });
}

declare global {
  interface Window {
    $message: MessageApiInjection;
  }
}
