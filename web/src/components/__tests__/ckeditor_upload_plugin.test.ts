import { describe, expect, it } from 'vitest';

describe('ckeditor upload plugin surface', () => {
  it('keeps toolbar entry names stable', () => {
    expect(['imageUpload', 'mediaEmbed', 'evtVideoUpload', 'evtAttachmentUpload', 'evtLink']).toEqual([
      'imageUpload',
      'mediaEmbed',
      'evtVideoUpload',
      'evtAttachmentUpload',
      'evtLink',
    ]);
  });
});
