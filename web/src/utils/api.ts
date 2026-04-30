const configuredHost = (import.meta.env.VITE_HOST ?? '')
  .trim()
  .replace(/\/+$/, '');

export const apiBaseUrl = configuredHost || undefined;

export function buildApiUrl(path: string): string {
  const normalizedPath = path.startsWith('/') ? path : `/${path}`;
  return apiBaseUrl ? `${apiBaseUrl}${normalizedPath}` : normalizedPath;
}
