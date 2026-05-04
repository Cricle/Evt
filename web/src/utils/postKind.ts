export const EVENT_POST_TAG = 'evt-event';

const normalizeTag = (value: string) => value.trim().toLowerCase();

export const normalizePostTags = (tags: Item.PostProps['tags'] | string[] | undefined | null): string[] => {
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

export const isEventPost = (post?: Pick<Item.PostProps, 'tags'> | null) =>
  normalizePostTags(post?.tags).includes(EVENT_POST_TAG);
