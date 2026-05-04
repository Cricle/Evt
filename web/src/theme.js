export const themePresetDefinitions = {
    emerald: {
        labelKey: 'settings_theme_preset_emerald',
        light: {
            primary: '#18a058',
            primaryHover: '#15925a',
            primaryPressed: '#11794a',
            primarySuppl: '#18a058',
        },
        dark: {
            primary: '#63e2b7',
            primaryHover: '#7de7c1',
            primaryPressed: '#4bd3a7',
            primarySuppl: '#63e2b7',
        },
    },
    ocean: {
        labelKey: 'settings_theme_preset_ocean',
        light: {
            primary: '#2080f0',
            primaryHover: '#4098fc',
            primaryPressed: '#1060c9',
            primarySuppl: '#2080f0',
        },
        dark: {
            primary: '#70c0ff',
            primaryHover: '#8ac9ff',
            primaryPressed: '#54a8e8',
            primarySuppl: '#70c0ff',
        },
    },
    amber: {
        labelKey: 'settings_theme_preset_amber',
        light: {
            primary: '#f0a020',
            primaryHover: '#fcb040',
            primaryPressed: '#c97c10',
            primarySuppl: '#f0a020',
        },
        dark: {
            primary: '#f2c97d',
            primaryHover: '#f5d595',
            primaryPressed: '#d8b062',
            primarySuppl: '#f2c97d',
        },
    },
    rose: {
        labelKey: 'settings_theme_preset_rose',
        light: {
            primary: '#d03050',
            primaryHover: '#de576d',
            primaryPressed: '#ab1f3f',
            primarySuppl: '#d03050',
        },
        dark: {
            primary: '#ff8fa3',
            primaryHover: '#ffacb9',
            primaryPressed: '#e57b90',
            primarySuppl: '#ff8fa3',
        },
    },
};
const hexToRgb = (hex) => {
    const normalized = hex.replace('#', '');
    const expanded = normalized.length === 3
        ? normalized
            .split('')
            .map((char) => char + char)
            .join('')
        : normalized;
    const value = Number.parseInt(expanded, 16);
    return {
        r: (value >> 16) & 255,
        g: (value >> 8) & 255,
        b: value & 255,
    };
};
const withAlpha = (hex, alpha) => {
    const { r, g, b } = hexToRgb(hex);
    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
};
const getPalette = (preset, isDark) => isDark ? themePresetDefinitions[preset].dark : themePresetDefinitions[preset].light;
export const buildThemeOverrides = (preset, isDark) => {
    const palette = getPalette(preset, isDark);
    return {
        common: {
            primaryColor: palette.primary,
            primaryColorHover: palette.primaryHover,
            primaryColorPressed: palette.primaryPressed,
            primaryColorSuppl: palette.primarySuppl,
            infoColor: palette.primary,
            infoColorHover: palette.primaryHover,
            infoColorPressed: palette.primaryPressed,
            infoColorSuppl: palette.primarySuppl,
            successColor: palette.primary,
            successColorHover: palette.primaryHover,
            successColorPressed: palette.primaryPressed,
            successColorSuppl: palette.primarySuppl,
            borderRadius: '12px',
            borderRadiusSmall: '10px',
            fontWeightStrong: '600',
        },
        Layout: {
            color: 'transparent',
            siderColor: 'transparent',
        },
    };
};
export const buildThemeCssVars = (preset, isDark) => {
    const palette = getPalette(preset, isDark);
    const panelBorder = isDark ? 'rgba(148, 163, 184, 0.08)' : 'rgba(18, 75, 51, 0.08)';
    const panelBg = isDark ? 'rgba(18, 24, 24, 0.22)' : 'rgba(255, 255, 255, 0.14)';
    const panelShadow = isDark ? '0 6px 18px rgba(0, 0, 0, 0.1)' : '0 6px 18px rgba(20, 70, 48, 0.035)';
    const editorBg = isDark ? 'rgba(25, 33, 33, 0.92)' : 'rgba(255, 255, 255, 0.88)';
    const editorToolbarBg = isDark ? 'rgba(22, 28, 28, 0.96)' : 'rgba(245, 250, 247, 0.92)';
    const editorBorder = isDark ? 'rgba(148, 163, 184, 0.16)' : 'rgba(18, 75, 51, 0.12)';
    const editorAccentRing = withAlpha(palette.primary, isDark ? 0.26 : 0.24);
    const editorTextMain = isDark ? 'rgba(241, 245, 249, 0.94)' : 'rgba(15, 23, 42, 0.94)';
    const editorTextSubtle = isDark ? 'rgba(226, 232, 240, 0.72)' : 'rgba(15, 23, 42, 0.66)';
    const accentSoft = withAlpha(palette.primary, isDark ? 0.12 : 0.08);
    const accentSoftHover = withAlpha(palette.primary, isDark ? 0.18 : 0.14);
    const accentSoftStrong = withAlpha(palette.primary, isDark ? 0.22 : 0.2);
    const accentSoftRing = withAlpha(palette.primary, isDark ? 0.2 : 0.16);
    const accentSoftMuted = withAlpha(palette.primary, isDark ? 0.08 : 0.06);
    return {
        '--nav-bg': isDark ? 'rgba(16, 22, 22, 0.7)' : 'rgba(250, 252, 250, 0.78)',
        '--accent-link': palette.primary,
        '--accent-primary': palette.primary,
        '--accent-soft': accentSoft,
        '--accent-soft-hover': accentSoftHover,
        '--accent-soft-strong': accentSoftStrong,
        '--accent-soft-ring': accentSoftRing,
        '--accent-soft-muted': accentSoftMuted,
        '--panel-bg': panelBg,
        '--panel-border': panelBorder,
        '--panel-shadow': panelShadow,
        '--editor-bg': editorBg,
        '--editor-toolbar-bg': editorToolbarBg,
        '--editor-border': editorBorder,
        '--editor-accent-ring': editorAccentRing,
        '--editor-text-main': editorTextMain,
        '--editor-text-subtle': editorTextSubtle,
        '--shadow-accent-pop': isDark ? '0 8px 20px rgba(0, 0, 0, 0.22)' : withAlpha(palette.primary, 0.12),
        '--shadow-accent-mini': isDark ? '0 8px 18px rgba(0, 0, 0, 0.22)' : withAlpha(palette.primary, 0.14),
        '--text-link-plain': isDark ? '#eee' : '#000',
        '--text-link-secondary': isDark ? '#ddd' : '#333',
        '--status-warning': isDark ? '#fbbf24' : '#ff7600',
        '--page-hero-bg-base': isDark ? '#0d1212' : '#f3f7f2',
        '--page-hero-bg-bottom': isDark ? '#0f1717' : '#edf3ee',
        '--page-hero-bg-glow': isDark ? 'rgba(25, 33, 33, 0.28)' : 'rgba(255, 255, 255, 0.54)',
        '--page-hero-bg-accent': withAlpha(palette.primary, isDark ? 0.08 : 0.1),
    };
};
//# sourceMappingURL=theme.js.map