import { describe, expect, it, vi } from 'vitest';

vi.mock('@ckeditor/ckeditor5-core/src/plugin', () => ({
  default: class Plugin {
    editor: any;
  },
}));

vi.mock('@ckeditor/ckeditor5-ui/src/button/filedialogbuttonview', () => ({
  default: class FileDialogButtonView {},
}));

vi.mock('@ckeditor/ckeditor5-ui/src/button/buttonview', () => ({
  default: class ButtonView {},
}));

vi.mock('@ckeditor/ckeditor5-upload/src/filerepository', () => ({
  default: class FileRepository {},
}));

import {
  EvtImageUploadAdapter,
  normalizeUploadedAssetUrl,
  normalizeLinkInput,
  resolveUploadAuthHeader,
  uploadFilesSequentially,
} from '@/components/ckeditor-upload-plugin';

describe('ckeditor upload plugin', () => {
  it('normalizes link input without changing inner url content', () => {
    expect(normalizeLinkInput('  https://evt.example/path?q=1  ')).toBe('https://evt.example/path?q=1');
  });

  it('normalizes relative uploaded asset urls to api urls', () => {
    expect(normalizeUploadedAssetUrl('/v1/media/42')).toBe('/v1/media/42');
    expect(normalizeUploadedAssetUrl('v1/media/42')).toBe('/v1/media/42');
    expect(normalizeUploadedAssetUrl('https://cdn.evt.example/v1/media/42')).toBe(
      'https://cdn.evt.example/v1/media/42',
    );
  });

  it('builds the auth header from local storage token', () => {
    localStorage.setItem('EVT_TOKEN', 'token-123');
    expect(resolveUploadAuthHeader()).toBe('Bearer token-123');
  });

  it('uploads non-image assets sequentially and keeps lifecycle balanced on partial failure', async () => {
    const onStart = vi.fn();
    const onUploaded = vi.fn();
    const onFinish = vi.fn();
    const onError = vi.fn();
    const upload = vi
      .fn()
      .mockResolvedValueOnce({
        kind: 'attachment',
        content: '/v1/attachment/1',
        name: 'a.txt',
      })
      .mockRejectedValueOnce(new Error('boom'));

    const files = [new File(['a'], 'a.txt'), new File(['b'], 'b.txt')];

    await uploadFilesSequentially(files, 'attachment', { onStart, onUploaded, onFinish, onError }, upload as never);

    expect(upload).toHaveBeenNthCalledWith(1, files[0], 'attachment');
    expect(upload).toHaveBeenNthCalledWith(2, files[1], 'attachment');
    expect(onStart).toHaveBeenCalledTimes(2);
    expect(onUploaded).toHaveBeenCalledTimes(1);
    expect(onError).toHaveBeenCalledTimes(1);
    expect(onFinish).toHaveBeenCalledTimes(2);
  });

  it('uploads editor images and returns the ckeditor default payload', async () => {
    const onStart = vi.fn();
    const onUploaded = vi.fn();
    const onFinish = vi.fn();
    const onError = vi.fn();
    const file = new File(['image'], 'cover.png', { type: 'image/png' });
    const adapter = new EvtImageUploadAdapter(
      { file: Promise.resolve(file) },
      { onStart, onUploaded, onFinish, onError },
      vi.fn().mockResolvedValue({
        kind: 'public/image',
        content: '/v1/attachment/cover.png',
        name: 'cover.png',
      }) as never,
    );

    await expect(adapter.upload()).resolves.toEqual({
      default: '/v1/attachment/cover.png',
    });
    expect(onStart).toHaveBeenCalledWith('cover.png', 'public/image');
    expect(onUploaded).toHaveBeenCalledWith({
      kind: 'public/image',
      content: '/v1/attachment/cover.png',
      name: 'cover.png',
    });
    expect(onError).not.toHaveBeenCalled();
    expect(onFinish).toHaveBeenCalledWith('cover.png', 'public/image');
  });

  it('propagates image upload failures after notifying plugin hooks', async () => {
    const onError = vi.fn();
    const adapter = new EvtImageUploadAdapter(
      { file: Promise.resolve(new File(['x'], 'broken.png', { type: 'image/png' })) },
      { onError },
      vi.fn().mockRejectedValue(new Error('upload failed')) as never,
    );

    await expect(adapter.upload()).rejects.toThrow('upload failed');
    expect(onError).toHaveBeenCalledTimes(1);
  });
});
