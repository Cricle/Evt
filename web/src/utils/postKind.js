export const EVENT_POST_TAG = 'evt-event';
const normalizeTag = (value) => value.trim().toLowerCase();
export const normalizePostTags = (tags) => {
    if (Array.isArray(tags)) {
        return tags.map((tag) => normalizeTag(`${tag}`)).filter(Boolean);
    }
    if (!tags) {
        return [];
    }
    if (typeof tags === 'string') {
        return tags
            .split(',')
            .map((tag) => normalizeTag(tag))
            .filter(Boolean);
    }
    return Object.keys(tags)
        .map((tag) => normalizeTag(tag))
        .filter(Boolean);
};
export const isEventPost = (post) => normalizePostTags(post?.tags).includes(EVENT_POST_TAG);
