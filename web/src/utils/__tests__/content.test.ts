import { describe, expect, it } from 'vitest';

import { parsePostTag, preparePost, sanitizeRichText, toPlainText } from '@/utils/content';

describe('content utils', () => {
  it('sanitizes rich text and strips unsafe tags for plain text', () => {
    expect(sanitizeRichText('<p>Hello</p><script>alert(1)</script>')).toContain('<p>Hello</p>');
    expect(sanitizeRichText('<p>Hello</p><script>alert(1)</script>')).not.toContain('script');
    expect(toPlainText('<p>Hello&nbsp;world</p>')).toBe('Hello world');
    expect(sanitizeRichText('')).toBe('');
    expect(toPlainText('')).toBe('');
  });

  it('parses tags and mentions into clickable rich text anchors', () => {
    const parsed = parsePostTag('<p>#rust @alice </p>');

    expect(parsed.tags).toEqual(['rust']);
    expect(parsed.users).toEqual(['alice']);
    expect(parsed.plainText).toBe('#rust @alice');
    expect(parsed.content).toContain('data-detail="tag:rust"');
    expect(parsed.content).toContain('data-detail="user:alice"');
  });

  it('prepares folded post previews with tag links and expand actions', () => {
    const prepared = preparePost('#rust hello world', '展开', '收起', 8, true);

    expect(prepared).toContain('data-detail="tag:rust"');
    expect(prepared).toContain('data-detail="post"');
    expect(prepared).toContain('展开');
  });

  it('prepares mention links and trims trailing mention markers when folding', () => {
    const mentionPrepared = preparePost('@alice hello world', '展开', '收起', 50, true);
    const foldedPrepared = preparePost('@alice hello world', '展开', '收起', 1, true);
    const unfoldedPrepared = preparePost('@alice hello world', '展开', '收起', 1, false);

    expect(mentionPrepared).toContain('data-detail="user:alice"');
    expect(foldedPrepared).toContain('展开');
    expect(unfoldedPrepared).toContain('收起');
    expect(unfoldedPrepared).not.toContain('...&nbsp;');
  });
});
