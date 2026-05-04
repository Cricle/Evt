export const LEGACY_DEFAULT_SPACE_SLUG = 'square';
export const PUBLIC_DEFAULT_SPACE_SLUG = 'public';
export const normalizeDefaultSpaceSlug = (slug) => {
    const normalized = `${slug || ''}`.trim().toLowerCase();
    if (!normalized || normalized === LEGACY_DEFAULT_SPACE_SLUG) {
        return PUBLIC_DEFAULT_SPACE_SLUG;
    }
    return normalized;
};
export const resolveSpaceSlug = (candidate, fallback) => {
    const normalizedCandidate = `${candidate || ''}`.trim().toLowerCase();
    if (!normalizedCandidate || normalizedCandidate === LEGACY_DEFAULT_SPACE_SLUG) {
        return normalizeDefaultSpaceSlug(fallback);
    }
    return normalizedCandidate;
};
//# sourceMappingURL=spaces.js.map