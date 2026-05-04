import { describe, expect, it } from 'vitest';

import { YesNoEnum } from '@/utils/IEnum';
import {
  buildEventTextBlocks,
  resolveEventExtraDescriptionHtml,
  resolveEventNarrative,
  resolveEventSummary,
  resolveEventTitle,
  resolveTimelineNodeType,
  sortTimelineComments,
} from '@/utils/eventTimeline';

describe('event timeline helpers', () => {
  it('derives event title and summary without duplicating the first text block', () => {
    const texts = [
      { content: '<p>系统升级完成</p>' },
      { content: '<p>第一批节点已经恢复</p>' },
      { content: '<p>继续观察监控指标</p>' },
    ] as Array<{ content: string }>;

    expect(resolveEventTitle(texts)).toBe('系统升级完成');
    expect(resolveEventSummary(texts)).toBe('第一批节点已经恢复 继续观察监控指标');
    expect(resolveEventExtraDescriptionHtml(texts)).toBe('<p>第一批节点已经恢复</p><p>继续观察监控指标</p>');
  });

  it('does not create a duplicate summary when the event only has one text block', () => {
    const texts = [{ content: '<p>仅有一个事件标题</p>' }] as Array<{ content: string }>;

    expect(resolveEventTitle(texts)).toBe('仅有一个事件标题');
    expect(resolveEventSummary(texts)).toBe('');
    expect(resolveEventExtraDescriptionHtml(texts)).toBe('');
  });

  it('normalizes event text blocks to readable plain text', () => {
    expect(buildEventTextBlocks([{ content: '<p> 事件 <strong>开始</strong> </p>' }])).toEqual(['事件 开始']);
  });

  it('sorts timeline comments from oldest to latest', () => {
    const comments = [
      { id: 3, created_on: 30, is_essence: YesNoEnum.NO },
      { id: 1, created_on: 10, is_essence: YesNoEnum.NO },
      { id: 2, created_on: 20, is_essence: YesNoEnum.NO },
    ] as Item.CommentProps[];

    expect(sortTimelineComments(comments).map((item) => item.id)).toEqual([1, 2, 3]);
  });

  it('marks timeline node types with milestone taking precedence', () => {
    expect(resolveTimelineNodeType(0, 3, { is_essence: YesNoEnum.NO } as Item.CommentProps)).toBe('info');
    expect(resolveTimelineNodeType(2, 3, { is_essence: YesNoEnum.NO } as Item.CommentProps)).toBe('success');
    expect(resolveTimelineNodeType(1, 3, { is_essence: YesNoEnum.NO } as Item.CommentProps)).toBe('default');
    expect(resolveTimelineNodeType(0, 3, { is_essence: YesNoEnum.YES } as Item.CommentProps)).toBe('warning');
  });

  it('builds a consistent event narrative with or without location', () => {
    expect(resolveEventNarrative('Alice', '上海', '发起')).toBe(
      '由 Alice 在 上海 发起，后续进展会按时间顺序持续记录。',
    );
    expect(resolveEventNarrative('Alice', '', '创建')).toBe(
      '由 Alice 创建，后续进展会按时间顺序持续记录。',
    );
  });
});
