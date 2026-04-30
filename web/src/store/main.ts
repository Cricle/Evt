import { defineStore } from 'pinia';
import { reactive, ref } from 'vue';

const MOBILE_BREAKPOINT = 821;
const RIGHTBAR_BREAKPOINT = 1100;

function viewportWidth() {
  return (
    window.innerWidth ||
    document.documentElement.clientWidth ||
    document.body.clientWidth
  );
}

export const useStoreMain = defineStore('main', () => {
  const initialWidth = viewportWidth();
  const refresh = ref(Date.now());
  const refreshTopicFollow = ref(Date.now());
  const theme = ref(localStorage.getItem('EVT_THEME'));
  const collapsedLeft = ref(initialWidth <= MOBILE_BREAKPOINT);
  const collapsedRight = ref(initialWidth <= RIGHTBAR_BREAKPOINT);
  const drawerModelShow = ref(initialWidth <= MOBILE_BREAKPOINT);
  const desktopModelShow = ref(initialWidth > MOBILE_BREAKPOINT);
  const unreadMsgCount = ref(0);

  function doRefresh(val?: number) {
    refresh.value = val || Date.now();
  }

  function doRefreshTopicFollow() {
    refreshTopicFollow.value = Date.now();
  }

  function updateUnreadMsgCount(count: number) {
    unreadMsgCount.value = count;
  }

  function triggerTheme(t: string) {
    theme.value = t;
  }

  function triggerCollapsedLeft(status: boolean) {
    collapsedLeft.value = status;
    drawerModelShow.value = status;
    desktopModelShow.value = !status;
  }

  function triggerCollapsedRight(status: boolean) {
    collapsedRight.value = status;
  }

  function syncViewportLayout(width = viewportWidth()) {
    const isMobile = width <= MOBILE_BREAKPOINT;
    collapsedLeft.value = isMobile;
    drawerModelShow.value = isMobile;
    desktopModelShow.value = !isMobile;
    collapsedRight.value = width <= RIGHTBAR_BREAKPOINT;
  }

  return {
    refresh,
    refreshTopicFollow,
    theme,
    collapsedLeft,
    collapsedRight,
    drawerModelShow,
    desktopModelShow,
    unreadMsgCount,
    doRefresh,
    doRefreshTopicFollow,
    updateUnreadMsgCount,
    triggerTheme,
    triggerCollapsedLeft,
    triggerCollapsedRight,
    syncViewportLayout,
  };
});
