import { YesNoEnum } from '@/utils/IEnum';
import { sanitizeEditorText } from '@/components/compose-editor-content';

const stripHtml = (value: string) =>
  value
    .replace(/<[^>]+>/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();

export const normalizeEventText = (value: string) => stripHtml(value);

export const buildEventTextHtmlBlocks = (texts: Array<Pick<Item.ContentProps, 'content'>>) =>
  texts
    .map((item) => sanitizeEditorText(item.content))
    .filter(Boolean);

export const buildEventTextBlocks = (texts: Array<Pick<Item.ContentProps, 'content'>>) =>
  texts
    .map((item) => normalizeEventText(item.content))
    .filter(Boolean);

export const resolveEventTitle = (texts: Array<Pick<Item.ContentProps, 'content'>>) =>
  buildEventTextBlocks(texts)[0] || '未命名事件';

export const resolveEventSummary = (texts: Array<Pick<Item.ContentProps, 'content'>>, maxLength = 120) =>
  buildEventTextBlocks(texts)
    .slice(1)
    .join(' ')
    .slice(0, maxLength);

export const resolveEventExtraDescriptionHtml = (texts: Array<Pick<Item.ContentProps, 'content'>>) =>
  buildEventTextHtmlBlocks(texts).slice(1).join('');

export const resolveEventNarrative = (
  authorName: string,
  ipLocation?: string | null,
  verb = '发起',
) =>
  ipLocation
    ? `由 ${authorName} 在 ${ipLocation} ${verb}，后续进展会按时间顺序持续记录。`
    : `由 ${authorName} ${verb}，后续进展会按时间顺序持续记录。`;

export const sortTimelineComments = (comments: Item.CommentProps[]) =>
  [...comments].sort((left, right) => left.created_on - right.created_on);

export const resolveTimelineNodeType = (
  index: number,
  total: number,
  comment: Pick<Item.CommentProps, 'is_essence'>,
) => {
  if (comment.is_essence === YesNoEnum.YES) {
    return 'warning' as const;
  }
  if (index === total - 1) {
    return 'success' as const;
  }
  if (index === 0) {
    return 'info' as const;
  }
  return 'default' as const;
};
