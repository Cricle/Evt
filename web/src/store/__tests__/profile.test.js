import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { useStoreProfile } from '@/store/profile';
const stubProfileEnv = () => {
    vi.stubEnv('VITE_USE_FRIENDSHIP', 'true');
    vi.stubEnv('VITE_DEFAULT_SPACE_SLUG', 'square');
    vi.stubEnv('VITE_ENABLE_TRENDS_BAR', 'true');
    vi.stubEnv('VITE_ENABLE_WALLET', 'false');
    vi.stubEnv('VITE_ALLOW_TWEET_ATTACHMENT', 'true');
    vi.stubEnv('VITE_ALLOW_TWEET_ATTACHMENT_PRICE', 'true');
    vi.stubEnv('VITE_ALLOW_TWEET_VIDEO', 'true');
    vi.stubEnv('VITE_ALLOW_USER_REGISTER', 'true');
    vi.stubEnv('VITE_ALLOW_PHONE_BIND', 'true');
    vi.stubEnv('VITE_DEFAULT_TWEET_MAX_LENGTH', '2000');
    vi.stubEnv('VITE_TWEET_WEB_ELLIPSIS_SIZE', '400');
    vi.stubEnv('VITE_TWEET_MOBILE_ELLIPSIS_SIZE', '300');
    vi.stubEnv('VITE_DEFAULT_TWEET_VISIBILITY', 'friend');
    vi.stubEnv('VITE_DEFAULT_MSG_LOOP_INTERVAL', '5000');
    vi.stubEnv('VITE_COPYRIGHT_TOP', '2026 Evt');
    vi.stubEnv('VITE_COPYRIGHT_LEFT', '');
    vi.stubEnv('VITE_COPYRIGHT_LEFT_LINK', '');
    vi.stubEnv('VITE_COPYRIGHT_RIGHT', 'Github');
    vi.stubEnv('VITE_COPYRIGHT_RIGHT_LINK', 'https://github.com/Cricle/Evt');
};
describe('profile store', () => {
    beforeEach(() => {
        setActivePinia(createPinia());
        stubProfileEnv();
    });
    it('initializes the current space from the default slug', () => {
        const store = useStoreProfile();
        store.loadDefaultSiteProfile();
        expect(store.profile.defaultSpaceSlug).toBe('public');
        expect(store.currentSpaceSlug).toBe('public');
    });
    it('keeps a valid current space when refreshing site profile data', () => {
        const store = useStoreProfile();
        store.loadDefaultSiteProfile();
        store.currentSpaceSlug = 'team-alpha';
        store.updateSiteProfile({
            default_space_slug: 'square',
        });
        expect(store.profile.defaultSpaceSlug).toBe('public');
        expect(store.currentSpaceSlug).toBe('team-alpha');
    });
    it('restores the default space when current space is empty or legacy alias', () => {
        const store = useStoreProfile();
        store.loadDefaultSiteProfile();
        store.currentSpaceSlug = 'square';
        expect(store.currentSpaceSlug).toBe('public');
        store.updateSiteProfile({
            default_space_slug: 'team-alpha',
        });
        expect(store.profile.defaultSpaceSlug).toBe('team-alpha');
        expect(store.currentSpaceSlug).toBe('public');
        store.currentSpaceSlug = '';
        expect(store.currentSpaceSlug).toBe('team-alpha');
        store.updateSiteProfile({
            default_space_slug: 'public',
        });
        expect(store.currentSpaceSlug).toBe('team-alpha');
    });
    it('normalizes direct current space assignments through the store setter', () => {
        const store = useStoreProfile();
        store.loadDefaultSiteProfile();
        store.profile.defaultSpaceSlug = 'team-alpha';
        store.currentSpaceSlug = 'square';
        expect(store.currentSpaceSlug).toBe('team-alpha');
        store.currentSpaceSlug = '  ';
        expect(store.currentSpaceSlug).toBe('team-alpha');
        store.currentSpaceSlug = ' Public-Lab ';
        expect(store.currentSpaceSlug).toBe('public-lab');
    });
});
//# sourceMappingURL=profile.test.js.map