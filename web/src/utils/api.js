const configuredHost = (import.meta.env.VITE_HOST ?? '')
    .trim()
    .replace(/\/+$/, '');
export const apiBaseUrl = configuredHost || undefined;
export function buildApiUrl(path) {
    const normalizedPath = path.startsWith('/') ? path : `/${path}`;
    return apiBaseUrl ? `${apiBaseUrl}${normalizedPath}` : normalizedPath;
}
//# sourceMappingURL=api.js.map