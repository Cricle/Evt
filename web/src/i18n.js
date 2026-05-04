import { computed } from 'vue';
import { storeToRefs } from 'pinia';
import { enUS, zhCN, dateEnUS, dateZhCN, } from 'naive-ui';
import moment from 'moment';
import 'moment/dist/locale/zh-cn';
import { useStoreMain, EVT_LOCALE_KEY } from '@/store/main';
import { safeLocalStorageSet } from '@/utils/storage';
const messages = {
    'zh-CN': {
        appName: 'Evt',
        appTagline: '一个清新文艺的微社区',
        route_home: '广场',
        route_landing: '首页',
        route_auth_signin: '登录',
        route_auth_signup: '注册',
        route_compose: '发布动态',
        route_create_space: '新建广场',
        route_post: '动态详情',
        route_topic: '话题',
        route_announcement: '公告',
        route_user: '用户详情',
        route_messages: '消息',
        route_contacts: '好友',
        route_following: '关注',
        route_wallet: '钱包',
        route_setting: '设置',
        route_admin_settings: '系统配置',
        route_not_found: '404',
        landing_title: '广场驱动的社区协作',
        landing_desc: '用独立广场组织成员、话题和动态，结合 Emoji 互动与评论，让讨论更轻、更快、更聚焦。',
        landing_cta_space: '进入公共广场',
        landing_cta_compose: '发布动态',
        landing_cta_signup: '注册体验',
        landing_chip_spaces: '多广场隔离',
        landing_chip_emoji: 'Emoji 互动',
        landing_chip_realtime: '实时消息',
        landing_chip_rust: 'Rust API',
        landing_panel_title: '默认入口',
        landing_panel_desc: '所有成员都会先进入公共广场',
        landing_feature_space_title: '广场是独立单元',
        landing_feature_space_desc: '成员、动态、标签、权限都按广场组织。',
        landing_feature_emoji_title: '互动以 Emoji 为核心',
        landing_feature_emoji_desc: '更适合快速反馈、轻讨论和团队氛围营造。',
        landing_feature_admin_title: '管理员集中控制',
        landing_feature_admin_desc: '系统配置、成员能力和运行参数统一管理。',
        landing_value_one_title: '空间化内容',
        landing_value_one_desc: '不同团队、兴趣组和项目都能各自运营，不相互打扰。',
        landing_value_two_title: '轻交互体验',
        landing_value_two_desc: '表情反馈优先，减少低质量回复，让互动更自然。',
        landing_value_three_title: '统一后台治理',
        landing_value_three_desc: '站点能力、搜索、存储和运行时参数都可以集中配置。',
        nav_login: '登录',
        nav_signup: '注册',
        nav_settings: '设置',
        nav_admin_settings: '系统配置',
        nav_messages: '消息',
        nav_contacts: '好友',
        nav_wallet: '钱包',
        nav_home: '广场',
        nav_topic: '话题',
        nav_announcement: '公告',
        auth_welcome_back: '欢迎回来',
        auth_create_account: '创建账号',
        auth_signin_desc: '登录后继续浏览、发帖和互动。',
        auth_signup_desc: '注册后即可加入社区，发布动态和参与互动。',
        auth_feature_spaces: '多广场切换',
        auth_feature_realtime: '实时互动',
        auth_feature_theme: '轻量主题',
        auth_tab_signin: '登录',
        auth_tab_signup: '注册',
        auth_account: '账户',
        auth_username: '用户名',
        auth_password: '密码',
        auth_repeat_password: '重复密码',
        auth_placeholder_username: '请输入用户名',
        auth_placeholder_login_password: '请输入账户密码',
        auth_placeholder_register_username: '用户名注册后无法修改',
        auth_placeholder_register_password: '密码不少于6位',
        auth_placeholder_repeat_password: '请再次输入密码',
        auth_action_signin: '登录',
        auth_action_signup: '注册',
        auth_success_signin: '登录成功',
        auth_success_signup: '注册成功',
        auth_error_username_required: '请输入账户名',
        auth_error_password_required: '请输入密码',
        auth_error_password_length: '密码不少于6位',
        auth_error_repeat_password_required: '请再次输入密码',
        auth_error_password_mismatch: '两次密码输入不一致',
        settings_language: '语言',
        settings_language_help: '切换界面文案、组件提示和时间展示语言。',
        settings_language_zh: '简体中文',
        settings_language_en: 'English',
        settings_language_updated: '语言已切换',
        settings_theme: '主题',
        settings_theme_help: '主题模式和主题方案统一由 Naive UI 驱动，并同步到全局界面。',
        settings_theme_mode: '主题模式',
        settings_theme_mode_system: '跟随系统',
        settings_theme_mode_light: '浅色',
        settings_theme_mode_dark: '深色',
        settings_theme_preset: '主题方案',
        settings_theme_preset_help: '切换主色、交互强调色和相关表面色。',
        settings_theme_updated: '主题已切换',
        settings_theme_preset_emerald: '翠绿',
        settings_theme_preset_ocean: '海蓝',
        settings_theme_preset_amber: '琥珀',
        settings_theme_preset_rose: '蔷薇',
    },
    'en-US': {
        appName: 'Evt',
        appTagline: 'A fresh and artful micro community',
        route_home: 'Spaces',
        route_landing: 'Home',
        route_auth_signin: 'Sign In',
        route_auth_signup: 'Sign Up',
        route_compose: 'Compose',
        route_create_space: 'Create Space',
        route_post: 'Post',
        route_topic: 'Topics',
        route_announcement: 'Announcements',
        route_user: 'User',
        route_messages: 'Messages',
        route_contacts: 'Contacts',
        route_following: 'Following',
        route_wallet: 'Wallet',
        route_setting: 'Settings',
        route_admin_settings: 'Admin Settings',
        route_not_found: '404',
        landing_title: 'Space-first community collaboration',
        landing_desc: 'Organize members, topics, and posts with independent spaces, then keep discussions fast and focused with emoji reactions and comments.',
        landing_cta_space: 'Enter Public Space',
        landing_cta_compose: 'Create Post',
        landing_cta_signup: 'Try Sign Up',
        landing_chip_spaces: 'Multi-space',
        landing_chip_emoji: 'Emoji-first',
        landing_chip_realtime: 'Realtime messaging',
        landing_chip_rust: 'Rust API',
        landing_panel_title: 'Default entry',
        landing_panel_desc: 'Everyone starts from the public space.',
        landing_feature_space_title: 'Spaces are isolated units',
        landing_feature_space_desc: 'Members, posts, tags, and permissions are organized per space.',
        landing_feature_emoji_title: 'Emoji-first interaction',
        landing_feature_emoji_desc: 'Ideal for quick feedback, light discussion, and team atmosphere.',
        landing_feature_admin_title: 'Centralized admin control',
        landing_feature_admin_desc: 'System settings, member capabilities, and runtime parameters are managed in one place.',
        landing_value_one_title: 'Structured by space',
        landing_value_one_desc: 'Teams, interest groups, and projects can all operate independently without interference.',
        landing_value_two_title: 'Lightweight interaction',
        landing_value_two_desc: 'Emoji feedback comes first, reducing low-signal replies and keeping communication natural.',
        landing_value_three_title: 'Unified governance',
        landing_value_three_desc: 'Site capabilities, search, storage, and runtime settings can be managed centrally.',
        nav_login: 'Sign In',
        nav_signup: 'Sign Up',
        nav_settings: 'Settings',
        nav_admin_settings: 'Admin Settings',
        nav_messages: 'Messages',
        nav_contacts: 'Contacts',
        nav_wallet: 'Wallet',
        nav_home: 'Spaces',
        nav_topic: 'Topics',
        nav_announcement: 'Announcements',
        auth_welcome_back: 'Welcome back',
        auth_create_account: 'Create an account',
        auth_signin_desc: 'Sign in to keep browsing, posting, and reacting.',
        auth_signup_desc: 'Create an account to join the community, post updates, and interact.',
        auth_feature_spaces: 'Multiple spaces',
        auth_feature_realtime: 'Realtime interaction',
        auth_feature_theme: 'Lightweight themes',
        auth_tab_signin: 'Sign In',
        auth_tab_signup: 'Sign Up',
        auth_account: 'Account',
        auth_username: 'Username',
        auth_password: 'Password',
        auth_repeat_password: 'Repeat Password',
        auth_placeholder_username: 'Enter your username',
        auth_placeholder_login_password: 'Enter your password',
        auth_placeholder_register_username: 'Usernames cannot be changed later',
        auth_placeholder_register_password: 'At least 6 characters',
        auth_placeholder_repeat_password: 'Enter the password again',
        auth_action_signin: 'Sign In',
        auth_action_signup: 'Sign Up',
        auth_success_signin: 'Signed in successfully',
        auth_success_signup: 'Signed up successfully',
        auth_error_username_required: 'Please enter your username',
        auth_error_password_required: 'Please enter your password',
        auth_error_password_length: 'Password must be at least 6 characters',
        auth_error_repeat_password_required: 'Please repeat your password',
        auth_error_password_mismatch: 'The two passwords do not match',
        settings_language: 'Language',
        settings_language_help: 'Switch UI copy, component prompts, and time formats.',
        settings_language_zh: '简体中文',
        settings_language_en: 'English',
        settings_language_updated: 'Language switched',
        settings_theme: 'Theme',
        settings_theme_help: 'Theme mode and theme palette are driven by Naive UI and synced across the app.',
        settings_theme_mode: 'Theme Mode',
        settings_theme_mode_system: 'System',
        settings_theme_mode_light: 'Light',
        settings_theme_mode_dark: 'Dark',
        settings_theme_preset: 'Theme Palette',
        settings_theme_preset_help: 'Switch the primary color, emphasis colors, and related surfaces.',
        settings_theme_updated: 'Theme updated',
        settings_theme_preset_emerald: 'Emerald',
        settings_theme_preset_ocean: 'Ocean',
        settings_theme_preset_amber: 'Amber',
        settings_theme_preset_rose: 'Rose',
    },
};
const naiveLocales = {
    'zh-CN': {
        locale: zhCN,
        dateLocale: dateZhCN,
    },
    'en-US': {
        locale: enUS,
        dateLocale: dateEnUS,
    },
};
const momentLocales = {
    'zh-CN': 'zh-cn',
    'en-US': 'en',
};
export const setMomentLocale = (locale) => {
    moment.locale(momentLocales[locale]);
};
export const persistLocale = (locale) => {
    safeLocalStorageSet(EVT_LOCALE_KEY, locale);
};
export const getRouteTitleKey = (routeName) => {
    switch (routeName) {
        case 'home':
            return 'route_landing';
        case 'space':
            return 'route_home';
        case 'auth':
            return 'route_auth_signin';
        case 'compose':
            return 'route_compose';
        case 'create-space':
            return 'route_create_space';
        case 'post':
            return 'route_post';
        case 'topic':
            return 'route_topic';
        case 'announcement':
            return 'route_announcement';
        case 'user':
            return 'route_user';
        case 'messages':
            return 'route_messages';
        case 'contacts':
            return 'route_contacts';
        case 'following':
            return 'route_following';
        case 'wallet':
            return 'route_wallet';
        case 'setting':
            return 'route_setting';
        case 'admin-settings':
            return 'route_admin_settings';
        case '404':
            return 'route_not_found';
        default:
            return null;
    }
};
export const translate = (locale, key, fallback) => messages[locale][key] ?? fallback ?? key;
export const useI18n = () => {
    const storeMain = useStoreMain();
    const { locale } = storeToRefs(storeMain);
    const naiveLocale = computed(() => naiveLocales[locale.value].locale);
    const naiveDateLocale = computed(() => naiveLocales[locale.value].dateLocale);
    const t = (key, fallback) => translate(locale.value, key, fallback);
    return {
        locale,
        naiveLocale,
        naiveDateLocale,
        t,
    };
};
//# sourceMappingURL=i18n.js.map